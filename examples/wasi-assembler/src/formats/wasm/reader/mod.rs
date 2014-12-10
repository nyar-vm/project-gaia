#![doc = include_str!("readme.md")]

use crate::{
    formats::wasm::WasmReadConfig,
    program::{WasiProgram, WasmInfo},
};
use byteorder::LittleEndian;
use gaia_types::{BinaryReader, GaiaDiagnostics, GaiaError};
use std::{
    cell::{OnceCell, RefCell},
    io::{Read, Seek},
};

/// WASM LEB128 reader helper
fn read_u32_leb128<R: Read>(reader: &mut R) -> std::io::Result<u32> {
    let mut result = 0u32;
    let mut shift = 0;
    loop {
        let mut byte = [0u8; 1];
        reader.read_exact(&mut byte)?;
        let b = byte[0];
        result |= ((b & 0x7F) as u32) << shift;
        if b & 0x80 == 0 {
            break;
        }
        shift += 7;
        if shift >= 32 {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "LEB128 value too large for u32"));
        }
    }
    Ok(result)
}

fn read_i32_leb128<R: Read>(reader: &mut R) -> std::io::Result<i32> {
    let mut result = 0i32;
    let mut shift = 0;
    let mut byte;
    loop {
        let mut buf = [0u8; 1];
        reader.read_exact(&mut buf)?;
        byte = buf[0];
        result |= ((byte & 0x7F) as i32) << shift;
        shift += 7;
        if byte & 0x80 == 0 {
            break;
        }
    }
    if shift < 32 && (byte & 0x40) != 0 {
        result |= !0 << shift;
    }
    Ok(result)
}

fn read_i64_leb128<R: Read>(reader: &mut R) -> std::io::Result<i64> {
    let mut result = 0i64;
    let mut shift = 0;
    let mut byte;
    loop {
        let mut buf = [0u8; 1];
        reader.read_exact(&mut buf)?;
        byte = buf[0];
        result |= ((byte & 0x7F) as i64) << shift;
        shift += 7;
        if byte & 0x80 == 0 {
            break;
        }
    }
    if shift < 64 && (byte & 0x40) != 0 {
        result |= !0 << shift;
    }
    Ok(result)
}

/// wasm lazy reader
#[derive(Debug)]
pub struct WasmReader<'config, R> {
    _config: &'config WasmReadConfig,
    reader: RefCell<BinaryReader<R, LittleEndian>>,
    view: OnceCell<WasmInfo>,
    program: OnceCell<WasiProgram>,
}

impl WasmReadConfig {
    pub fn as_reader<R: Read + Seek>(&self, reader: R) -> WasmReader<'_, R> {
        WasmReader::new(reader, self)
    }
}

impl<'config, R> WasmReader<'config, R> {
    pub fn new(reader: R, config: &'config WasmReadConfig) -> Self {
        Self {
            reader: RefCell::new(BinaryReader::new(reader)),
            view: Default::default(),
            program: Default::default(),
            _config: config,
        }
    }
    pub fn finish(mut self) -> GaiaDiagnostics<WasiProgram>
    where
        R: Read + Seek,
    {
        match self.get_program() {
            Ok(_) => {
                let errors = vec![]; // self.reader.borrow_mut().take_errors();
                GaiaDiagnostics {
                    result: self.program.take().ok_or(GaiaError::custom_error("unreachable")),
                    diagnostics: errors,
                }
            }
            Err(e) => {
                let errors = vec![]; // self.reader.borrow_mut().take_errors();
                GaiaDiagnostics { result: Err(e), diagnostics: errors }
            }
        }
    }
}

impl<'config, R: Read + Seek> WasmReader<'config, R> {
    pub fn get_program(&self) -> Result<&WasiProgram, GaiaError> {
        self.program.get_or_try_init(|| self.read_program())
    }
    fn read_program(&self) -> Result<WasiProgram, GaiaError> {
        let mut reader = self.reader.borrow_mut();

        // 重新定位到文件开头
        reader.seek(std::io::SeekFrom::Start(0))?;

        // 验证 WASM 文件头
        self.validate_wasm_header(&mut reader)?;

        // 创建核心模块程序
        let mut program = crate::program::WasiProgram::new_core_module();

        // 解析各个段
        self.parse_sections(&mut reader, &mut program)?;

        Ok(program)
    }

    fn validate_wasm_header(&self, reader: &mut BinaryReader<R, LittleEndian>) -> Result<(), GaiaError> {
        // 读取并验证 WASM 文件头
        let magic = reader.read_u32()?;
        if magic != 0x6D736100 {
            // "\0asm" in little-endian
            return Err(GaiaError::invalid_data("Invalid WASM file magic number"));
        }

        let version = reader.read_u32()?;
        if version != 1 {
            return Err(GaiaError::invalid_data(&format!("Unsupported WASM version: {}", version)));
        }

        Ok(())
    }

    fn parse_sections(
        &self,
        reader: &mut BinaryReader<R, LittleEndian>,
        program: &mut crate::program::WasiProgram,
    ) -> Result<(), GaiaError> {
        while let Ok(section_id) = reader.read_u8() {
            let section_size = read_u32_leb128(reader)?;

            match section_id {
                0 => self.read_custom_section(reader, program, section_size)?,
                1 => self.read_type_section(reader, program)?,
                2 => self.read_import_section(reader, program)?,
                3 => self.read_function_section(reader, program)?,
                4 => self.read_table_section(reader, program)?,
                5 => self.read_memory_section(reader, program)?,
                6 => self.read_global_section(reader, program)?,
                7 => self.read_export_section(reader, program)?,
                8 => self.read_start_section(reader, program)?,
                9 => self.skip_section(reader, section_size)?, // Element section
                10 => self.read_code_section(reader, program)?,
                11 => self.skip_section(reader, section_size)?, // Data section
                _ => self.skip_section(reader, section_size)?,  // Unknown section
            }
        }
        Ok(())
    }

    fn read_custom_section(
        &self,
        reader: &mut BinaryReader<R, LittleEndian>,
        program: &mut crate::program::WasiProgram,
        section_size: u32,
    ) -> Result<(), GaiaError> {
        let name_len = read_u32_leb128(reader)?;
        let mut name_bytes = vec![0u8; name_len as usize];
        reader.read_exact(&mut name_bytes)?;
        let name = String::from_utf8_lossy(&name_bytes).to_string();

        let data_len = section_size - name_len - gaia_types::BinaryReader::<R, LittleEndian>::leb128_size(name_len);
        let mut data = vec![0u8; data_len as usize];
        reader.read_exact(&mut data)?;

        program.custom_sections.push(crate::program::WasiCustomSection { name, data });
        Ok(())
    }

    fn read_start_section(
        &self,
        reader: &mut BinaryReader<R, LittleEndian>,
        program: &mut crate::program::WasiProgram,
    ) -> Result<(), GaiaError> {
        let start_func = read_u32_leb128(reader)?;
        program.start_function = Some(start_func);
        Ok(())
    }

    pub fn get_view(&self) -> Result<&WasmInfo, GaiaError> {
        self.view.get_or_try_init(|| self.read_view())
    }

    fn read_view(&self) -> Result<WasmInfo, GaiaError> {
        let mut reader = self.reader.borrow_mut();

        // 重新定位到文件开头
        reader.seek(std::io::SeekFrom::Start(0))?;

        // 读取并验证 WASM 文件头
        let magic = reader.read_u32()?;
        if magic != 0x6D736100 {
            // "\0asm" in little-endian
            return Err(GaiaError::invalid_data("Invalid WASM file magic number"));
        }

        let _version = reader.read_u32()?;

        Ok(WasmInfo {
            magic_head: [0x00, 0x61, 0x73, 0x6D], // "\0asm"
        })
    }

    // Helper methods for reading WASM format
    fn skip_section(&self, reader: &mut BinaryReader<R, LittleEndian>, size: u32) -> Result<(), GaiaError> {
        let mut buffer = vec![0u8; size as usize];
        reader.read_exact(&mut buffer)?;
        Ok(())
    }

    fn read_type_section(
        &self,
        reader: &mut BinaryReader<R, LittleEndian>,
        program: &mut crate::program::WasiProgram,
    ) -> Result<(), GaiaError> {
        let count = read_u32_leb128(reader)?;

        for _ in 0..count {
            let form = reader.read_u8()?;
            if form != 0x60 {
                // Function type
                return Err(GaiaError::invalid_data("Unsupported type form"));
            }

            let param_count = read_u32_leb128(reader)?;
            let mut params = Vec::new();
            for _ in 0..param_count {
                let param_type = self.read_value_type(reader)?;
                params.push(param_type);
            }

            let result_count = read_u32_leb128(reader)?;
            let mut results = Vec::new();
            for _ in 0..result_count {
                let result_type = self.read_value_type(reader)?;
                results.push(result_type);
            }

            program.function_types.push(crate::program::WasiFunctionType { params, results });
        }

        Ok(())
    }

    fn read_type(&self, reader: &mut BinaryReader<R, LittleEndian>) -> Result<crate::program::WasiType, GaiaError> {
        let tag = reader.read_u8()?;
        match tag {
            0x7F | 0x7E | 0x7D | 0x7C | 0x7B | 0x7A | 0x79 | 0x78 | 0x77 | 0x76 | 0x75 | 0x74 | 0x73 => {
                let prim = match tag {
                    0x7F => crate::program::WasiPrimitiveType::Bool,
                    0x7E => crate::program::WasiPrimitiveType::S8,
                    0x7D => crate::program::WasiPrimitiveType::U8,
                    0x7C => crate::program::WasiPrimitiveType::S16,
                    0x7B => crate::program::WasiPrimitiveType::U16,
                    0x7A => crate::program::WasiPrimitiveType::S32,
                    0x79 => crate::program::WasiPrimitiveType::U32,
                    0x78 => crate::program::WasiPrimitiveType::S64,
                    0x77 => crate::program::WasiPrimitiveType::U64,
                    0x76 => crate::program::WasiPrimitiveType::F32,
                    0x75 => crate::program::WasiPrimitiveType::F64,
                    0x74 => crate::program::WasiPrimitiveType::Char,
                    0x73 => crate::program::WasiPrimitiveType::String,
                    _ => unreachable!(),
                };
                Ok(crate::program::WasiType::Primitive(prim))
            }
            0x72 => {
                // record
                let count = read_u32_leb128(reader)?;
                let mut fields = Vec::new();
                for _ in 0..count {
                    let name_len = read_u32_leb128(reader)?;
                    let mut name_bytes = vec![0u8; name_len as usize];
                    reader.read_exact(&mut name_bytes)?;
                    let name = String::from_utf8_lossy(&name_bytes).to_string();
                    let field_type = self.read_type(reader)?;
                    fields.push(crate::program::WasiRecordField { name, field_type });
                }
                Ok(crate::program::WasiType::Record(fields))
            }
            0x69 => {
                // future (WASIp3)
                let has_type = reader.read_u8()?;
                let inner = if has_type == 0x01 { Some(Box::new(self.read_type(reader)?)) } else { None };
                Ok(crate::program::WasiType::Future(inner))
            }
            0x68 => {
                // stream (WASIp3)
                let has_type = reader.read_u8()?;
                let inner = if has_type == 0x01 { Some(Box::new(self.read_type(reader)?)) } else { None };
                Ok(crate::program::WasiType::Stream(inner))
            }
            _ => Err(GaiaError::invalid_data(&format!("Unsupported type tag: 0x{:02X}", tag))),
        }
    }

    fn read_canon_options(
        &self,
        reader: &mut BinaryReader<R, LittleEndian>,
    ) -> Result<Vec<crate::program::WasiCanonOption>, GaiaError> {
        let count = read_u32_leb128(reader)?;
        let mut options = Vec::new();
        for _ in 0..count {
            let tag = reader.read_u8()?;
            match tag {
                0x00 => {
                    let enc_tag = reader.read_u8()?;
                    let enc = match enc_tag {
                        0x00 => "utf8",
                        0x01 => "utf16",
                        0x02 => "compact-utf16",
                        _ => "utf8",
                    };
                    options.push(crate::program::WasiCanonOption::StringEncoding(enc.to_string()));
                }
                0x01 => {
                    let _mem_index = read_u32_leb128(reader)?;
                    options.push(crate::program::WasiCanonOption::Memory("0".to_string()));
                }
                0x02 => {
                    let _realloc_index = read_u32_leb128(reader)?;
                    options.push(crate::program::WasiCanonOption::Realloc("0".to_string()));
                }
                0x04 => {
                    // async (WASIp3)
                    options.push(crate::program::WasiCanonOption::Async);
                }
                0x05 => {
                    // callback (WASIp3)
                    let _callback_index = read_u32_leb128(reader)?;
                    options.push(crate::program::WasiCanonOption::Callback("0".to_string()));
                }
                _ => return Err(GaiaError::invalid_data(&format!("Unsupported canon option tag: 0x{:02X}", tag))),
            }
        }
        Ok(options)
    }

    fn read_value_type(&self, reader: &mut BinaryReader<R, LittleEndian>) -> Result<crate::program::WasmValueType, GaiaError> {
        let type_byte = reader.read_u8()?;
        crate::program::WasmValueType::try_from(type_byte)
    }

    fn read_import_section(
        &self,
        reader: &mut BinaryReader<R, LittleEndian>,
        program: &mut crate::program::WasiProgram,
    ) -> Result<(), GaiaError> {
        let count = read_u32_leb128(reader)?;

        for _ in 0..count {
            // 读取模块名
            let module_len = read_u32_leb128(reader)?;
            let mut module_bytes = vec![0u8; module_len as usize];
            reader.read_exact(&mut module_bytes)?;
            let module = String::from_utf8_lossy(&module_bytes).to_string();

            // 读取字段名
            let field_len = read_u32_leb128(reader)?;
            let mut field_bytes = vec![0u8; field_len as usize];
            reader.read_exact(&mut field_bytes)?;
            let field = String::from_utf8_lossy(&field_bytes).to_string();

            let import_type = self.read_import_type(reader)?;

            program.imports.push(crate::program::WasiImport { module, field, import_type });
        }

        Ok(())
    }

    fn read_import_type(
        &self,
        reader: &mut BinaryReader<R, LittleEndian>,
    ) -> Result<crate::program::WasmImportType, GaiaError> {
        let kind = reader.read_u8()?;
        match kind {
            0x00 => {
                // Function
                let type_index = read_u32_leb128(reader)?;
                Ok(crate::program::WasmImportType::Function { type_index })
            }
            0x01 => {
                // Table
                let table_type = self.read_table_type(reader)?;
                Ok(crate::program::WasmImportType::Table { table_type })
            }
            0x02 => {
                // Memory
                let memory_type = self.read_memory_type(reader)?;
                Ok(crate::program::WasmImportType::Memory { memory_type })
            }
            0x03 => {
                // Global
                let global_type = self.read_global_type(reader)?;
                Ok(crate::program::WasmImportType::Global { global_type })
            }
            _ => Err(GaiaError::invalid_data(&format!("Unknown import kind: {}", kind))),
        }
    }

    fn read_function_section(
        &self,
        reader: &mut BinaryReader<R, LittleEndian>,
        program: &mut crate::program::WasiProgram,
    ) -> Result<(), GaiaError> {
        let count = read_u32_leb128(reader)?;

        for _ in 0..count {
            let type_index = read_u32_leb128(reader)?;
            // 创建空的函数，稍后在 code section 中填充
            program.functions.push(crate::program::WasiFunction { type_index, locals: Vec::new(), body: Vec::new() });
        }

        Ok(())
    }

    fn read_table_section(
        &self,
        reader: &mut BinaryReader<R, LittleEndian>,
        program: &mut crate::program::WasiProgram,
    ) -> Result<(), GaiaError> {
        let count = read_u32_leb128(reader)?;

        for _ in 0..count {
            let table_type = self.read_table_type(reader)?;
            program.tables.push(crate::program::WasiTable { table_type });
        }

        Ok(())
    }

    fn read_table_type(&self, reader: &mut BinaryReader<R, LittleEndian>) -> Result<crate::program::WasmTableType, GaiaError> {
        let element_type = reader.read_u8()?;
        let element_type = match element_type {
            0x70 => crate::program::WasmReferenceType::FuncRef,
            0x6F => crate::program::WasmReferenceType::ExternRef,
            _ => return Err(GaiaError::invalid_data(&format!("Unknown reference type: 0x{:02X}", element_type))),
        };

        let limits = self.read_limits(reader)?;

        Ok(crate::program::WasmTableType { element_type, min: limits.0, max: limits.1 })
    }

    fn read_memory_section(
        &self,
        reader: &mut BinaryReader<R, LittleEndian>,
        program: &mut crate::program::WasiProgram,
    ) -> Result<(), GaiaError> {
        let count = read_u32_leb128(reader)?;

        for _ in 0..count {
            let memory_type = self.read_memory_type(reader)?;
            program.memories.push(crate::program::WasiMemory { memory_type });
        }

        Ok(())
    }

    fn read_memory_type(
        &self,
        reader: &mut BinaryReader<R, LittleEndian>,
    ) -> Result<crate::program::WasmMemoryType, GaiaError> {
        let limits = self.read_limits(reader)?;
        Ok(crate::program::WasmMemoryType { min: limits.0, max: limits.1 })
    }

    fn read_limits(&self, reader: &mut BinaryReader<R, LittleEndian>) -> Result<(u32, Option<u32>), GaiaError> {
        let flags = reader.read_u8()?;
        let min = read_u32_leb128(reader)?;
        let max = if flags & 0x01 != 0 { Some(read_u32_leb128(reader)?) } else { None };
        Ok((min, max))
    }

    fn read_global_section(
        &self,
        reader: &mut BinaryReader<R, LittleEndian>,
        program: &mut crate::program::WasiProgram,
    ) -> Result<(), GaiaError> {
        let count = read_u32_leb128(reader)?;

        for _ in 0..count {
            let global_type = self.read_global_type(reader)?;
            let init_expr = self.read_init_expr(reader)?;
            program.globals.push(crate::program::WasiGlobal { global_type, init_expr });
        }

        Ok(())
    }

    fn read_global_type(
        &self,
        reader: &mut BinaryReader<R, LittleEndian>,
    ) -> Result<crate::program::WasmGlobalType, GaiaError> {
        let value_type = self.read_value_type(reader)?;
        let mutability = reader.read_u8()?;
        let mutable = mutability != 0;

        Ok(crate::program::WasmGlobalType { value_type, mutable })
    }

    fn read_init_expr(
        &self,
        reader: &mut BinaryReader<R, LittleEndian>,
    ) -> Result<Vec<crate::program::WasiInstruction>, GaiaError> {
        let mut instructions = Vec::new();

        loop {
            let opcode = reader.read_u8()?;
            match opcode {
                0x41 => {
                    // i32.const
                    let value = read_i32_leb128(reader)?;
                    instructions.push(crate::program::WasiInstruction::I32Const { value });
                }
                0x42 => {
                    // i64.const
                    let value = read_i64_leb128(reader)?;
                    instructions.push(crate::program::WasiInstruction::I64Const { value });
                }
                0x43 => {
                    // f32.const
                    let value = reader.read_f32()?;
                    instructions.push(crate::program::WasiInstruction::F32Const { value });
                }
                0x44 => {
                    // f64.const
                    let value = reader.read_f64()?;
                    instructions.push(crate::program::WasiInstruction::F64Const { value });
                }
                0x0B => {
                    // end
                    instructions.push(crate::program::WasiInstruction::End);
                    break;
                }
                _ => {
                    return Err(GaiaError::invalid_data(&format!("Unsupported opcode in init expression: 0x{:02X}", opcode)));
                }
            }
        }

        Ok(instructions)
    }

    fn read_export_section(
        &self,
        reader: &mut BinaryReader<R, LittleEndian>,
        program: &mut crate::program::WasiProgram,
    ) -> Result<(), GaiaError> {
        let count = read_u32_leb128(reader)?;

        for _ in 0..count {
            // 读取导出名称
            let name_len = read_u32_leb128(reader)?;
            let mut name_bytes = vec![0u8; name_len as usize];
            reader.read_exact(&mut name_bytes)?;
            let name = String::from_utf8_lossy(&name_bytes).to_string();

            // 读取导出类型和索引
            let kind = reader.read_u8()?;
            let index = read_u32_leb128(reader)?;

            let export_type = match kind {
                0x00 => crate::program::WasmExportType::Function { function_index: index },
                0x01 => crate::program::WasmExportType::Table { table_index: index },
                0x02 => crate::program::WasmExportType::Memory { memory_index: index },
                0x03 => crate::program::WasmExportType::Global { global_index: index },
                _ => return Err(GaiaError::invalid_data(&format!("Unknown export kind: {}", kind))),
            };

            program.exports.push(crate::program::WasiExport { name, export_type });
        }

        Ok(())
    }

    fn read_code_section(
        &self,
        reader: &mut BinaryReader<R, LittleEndian>,
        program: &mut crate::program::WasiProgram,
    ) -> Result<(), GaiaError> {
        let count = read_u32_leb128(reader)?;

        for i in 0..count {
            // 读取函数体大小
            let body_size = read_u32_leb128(reader)?;
            let start_pos = reader.get_position();

            // 读取局部变量
            let local_count = read_u32_leb128(reader)?;
            let mut locals = Vec::new();
            for _ in 0..local_count {
                let count = read_u32_leb128(reader)?;
                let value_type = self.read_value_type(reader)?;
                locals.push(crate::program::WasmLocal { count, value_type });
            }

            // For now, just skip the function body
            // In a full implementation, you would parse the instructions here
            let remaining_size = body_size - (reader.get_position() - start_pos) as u32;
            let mut body_bytes = vec![0u8; remaining_size as usize];
            reader.read_exact(&mut body_bytes)?;

            // Update the function with locals (body remains empty for now)
            program.functions[i as usize].locals = locals;

            // 解析指令
            let mut body_reader = BinaryReader::new(std::io::Cursor::new(body_bytes));
            program.functions[i as usize].body = self.read_instructions(&mut body_reader)?;
        }

        Ok(())
    }

    fn read_instructions<RR: Read + Seek>(
        &self,
        reader: &mut BinaryReader<RR, LittleEndian>,
    ) -> Result<Vec<crate::program::WasiInstruction>, GaiaError> {
        let mut instructions = Vec::new();
        while let Ok(opcode) = reader.read_u8() {
            match opcode {
                0x00 => instructions.push(crate::program::WasiInstruction::Unreachable),
                0x01 => instructions.push(crate::program::WasiInstruction::Nop),
                0x02 => {
                    let _block_type = reader.read_u8()?; // Simplified
                    instructions.push(crate::program::WasiInstruction::Block { block_type: None });
                }
                0x03 => {
                    let _block_type = reader.read_u8()?; // Simplified
                    instructions.push(crate::program::WasiInstruction::Loop { block_type: None });
                }
                0x04 => {
                    let _block_type = reader.read_u8()?; // Simplified
                    instructions.push(crate::program::WasiInstruction::If { block_type: None });
                }
                0x05 => instructions.push(crate::program::WasiInstruction::Else),
                0x0B => {
                    instructions.push(crate::program::WasiInstruction::End);
                    // 在 Wasm 中，函数结束由 body_size 控制，但指令流通常以 0x0B 结尾
                }
                0x0C => {
                    let label_index = read_u32_leb128(reader)?;
                    instructions.push(crate::program::WasiInstruction::Br { label_index });
                }
                0x0D => {
                    let label_index = read_u32_leb128(reader)?;
                    instructions.push(crate::program::WasiInstruction::BrIf { label_index });
                }
                0x0F => instructions.push(crate::program::WasiInstruction::Return),
                0x10 => {
                    let function_index = read_u32_leb128(reader)?;
                    instructions.push(crate::program::WasiInstruction::Call { function_index });
                }
                0x1A => instructions.push(crate::program::WasiInstruction::Drop),
                0x1B => instructions.push(crate::program::WasiInstruction::Select),
                0x20 => {
                    let local_index = read_u32_leb128(reader)?;
                    instructions.push(crate::program::WasiInstruction::LocalGet { local_index });
                }
                0x21 => {
                    let local_index = read_u32_leb128(reader)?;
                    instructions.push(crate::program::WasiInstruction::LocalSet { local_index });
                }
                0x41 => {
                    let value = read_i32_leb128(reader)?;
                    instructions.push(crate::program::WasiInstruction::I32Const { value });
                }
                0x42 => {
                    let value = read_i64_leb128(reader)?;
                    instructions.push(crate::program::WasiInstruction::I64Const { value });
                }
                0x6A => instructions.push(crate::program::WasiInstruction::I32Add),
                0x6B => instructions.push(crate::program::WasiInstruction::I32Sub),
                0x6C => instructions.push(crate::program::WasiInstruction::I32Mul),
                0x45 => instructions.push(crate::program::WasiInstruction::I32Eqz),
                0x46 => instructions.push(crate::program::WasiInstruction::I32Eq),
                0x47 => instructions.push(crate::program::WasiInstruction::I32Ne),
                0x48 => instructions.push(crate::program::WasiInstruction::I32LtS),
                0x49 => instructions.push(crate::program::WasiInstruction::I32LtU),
                0x4A => instructions.push(crate::program::WasiInstruction::I32GtS),
                0x4B => instructions.push(crate::program::WasiInstruction::I32GtU),
                0x4C => instructions.push(crate::program::WasiInstruction::I32LeS),
                0x4D => instructions.push(crate::program::WasiInstruction::I32LeU),
                0x4E => instructions.push(crate::program::WasiInstruction::I32GeS),
                0x4F => instructions.push(crate::program::WasiInstruction::I32GeU),

                // WASIp3 扩展指令 (0x5F prefix)
                0x5F => {
                    let sub_opcode = reader.read_u8()?;
                    match sub_opcode {
                        0x00 => instructions.push(crate::program::WasiInstruction::TaskBackpressure),
                        0x01 => instructions.push(crate::program::WasiInstruction::TaskReturn),
                        0x02 => instructions.push(crate::program::WasiInstruction::TaskWait),
                        0x03 => instructions.push(crate::program::WasiInstruction::TaskPoll),
                        0x04 => instructions.push(crate::program::WasiInstruction::TaskYield),
                        0x05 => instructions.push(crate::program::WasiInstruction::ErrorContextNew),
                        0x06 => instructions.push(crate::program::WasiInstruction::ErrorContextDebugMessage),
                        _ => {
                            return Err(GaiaError::invalid_data(&format!("Unknown sub-opcode for 0x5F: 0x{:02X}", sub_opcode)))
                        }
                    }
                }
                _ => {
                    // 对于未知的指令，我们可以选择跳过或者报错
                    // 这里为了简单起见，我们只解析已知的
                }
            }
        }
        Ok(instructions)
    }
}
