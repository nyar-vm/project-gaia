use crate::program::{ClrInstruction, ClrMethod, ClrOpcode, ClrProgram};
use gaia_types::{helpers::Url, GaiaDiagnostics, GaiaError};
use pe_assembler::{
    exe_write_path,
    helpers::PeWriter,
    types::{
        tables::{ExportTable, ImportTable},
        CoffHeader, DosHeader, NtHeader, OptionalHeader, PeHeader, PeProgram, PeSection, SubsystemType,
    },
};
use std::{
    io::{Cursor, Seek, Write},
    path::Path,
};

#[derive(Debug)]
pub struct DllWriter<W> {
    writer: W,
}

impl<W> DllWriter<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }
}

impl<W: Write + Seek> DllWriter<W> {
    pub fn write(mut self, clr: &ClrProgram) -> GaiaDiagnostics<W> {
        match self.build_pe_program(clr) {
            Ok(pe_program) => match self.write_pe_program(&pe_program) {
                Ok(_) => GaiaDiagnostics::success(self.writer),
                Err(e) => GaiaDiagnostics::failure(e),
            },
            Err(e) => GaiaDiagnostics::failure(e),
        }
    }

    /// 写入 CLR 程序到指定路径
    pub fn write_to_path(clr: &ClrProgram, path: &Path) -> Result<Url, GaiaError> {
        let pe_program = Self::build_pe_program_static(clr)?;
        exe_write_path(&pe_program, path)
    }

    /// 静态方法构建 PE 程序
    fn build_pe_program_static(clr: &ClrProgram) -> Result<PeProgram, GaiaError> {
        let instance = DllWriter::new(Cursor::new(Vec::new()));
        instance.build_pe_program(clr)
    }

    fn build_pe_program(&self, clr: &ClrProgram) -> Result<PeProgram, GaiaError> {
        // 构建 CLR 数据
        let clr_data = self.build_clr_data(clr)?;

        // 创建 .text 节（包含 CLR 头、元数据和代码）
        let text_section = PeSection {
            name: ".text".to_string(),
            virtual_size: clr_data.len() as u32,
            virtual_address: 0x2000,
            size_of_raw_data: align_to(clr_data.len() as u32, 0x200),
            pointer_to_raw_data: 0x400,
            pointer_to_relocations: 0,
            pointer_to_line_numbers: 0,
            number_of_relocations: 0,
            number_of_line_numbers: 0,
            characteristics: 0x60000020, // IMAGE_SCN_CNT_CODE | IMAGE_SCN_MEM_EXECUTE | IMAGE_SCN_MEM_READ
            data: clr_data,
        };

        // 创建 PE 头
        let pe_header = self.build_pe_header(&text_section)?;

        // 直接创建 PE 程序
        let pe_program = PeProgram {
            header: pe_header,
            sections: vec![text_section],
            imports: ImportTable::new(),
            exports: ExportTable::new(),
        };

        Ok(pe_program)
    }

    fn write_pe_program(&mut self, pe_program: &PeProgram) -> Result<(), GaiaError> {
        use pe_assembler::formats::dll::writer::DllWriter;
        let mut pe_writer = DllWriter::new(&mut self.writer);
        pe_writer.write_program(pe_program)?;
        Ok(())
    }

    fn write_dos_header(&mut self, dos_header: &DosHeader) -> Result<(), GaiaError> {
        self.writer.write_all(b"MZ")?;
        self.writer.write_all(&[0; 58])?; // DOS header padding
        self.writer.write_all(&dos_header.e_lfanew.to_le_bytes())?;
        Ok(())
    }

    fn write_nt_header(&mut self, nt_header: &NtHeader) -> Result<(), GaiaError> {
        self.writer.write_all(&nt_header.signature.to_le_bytes())?;
        Ok(())
    }

    fn write_coff_header(&mut self, coff_header: &CoffHeader) -> Result<(), GaiaError> {
        self.writer.write_all(&coff_header.machine.to_le_bytes())?;
        self.writer.write_all(&coff_header.number_of_sections.to_le_bytes())?;
        self.writer.write_all(&coff_header.time_date_stamp.to_le_bytes())?;
        self.writer.write_all(&coff_header.pointer_to_symbol_table.to_le_bytes())?;
        self.writer.write_all(&coff_header.number_of_symbols.to_le_bytes())?;
        self.writer.write_all(&coff_header.size_of_optional_header.to_le_bytes())?;
        self.writer.write_all(&coff_header.characteristics.to_le_bytes())?;
        Ok(())
    }

    fn write_optional_header(&mut self, optional_header: &OptionalHeader) -> Result<(), GaiaError> {
        self.writer.write_all(&0x10bu16.to_le_bytes())?; // PE32 magic
        self.writer.write_all(&[1, 0])?; // linker version
        self.writer.write_all(&optional_header.size_of_code.to_le_bytes())?;
        self.writer.write_all(&0u32.to_le_bytes())?; // size of initialized data
        self.writer.write_all(&0u32.to_le_bytes())?; // size of uninitialized data
        self.writer.write_all(&optional_header.address_of_entry_point.to_le_bytes())?;
        self.writer.write_all(&0x2000u32.to_le_bytes())?; // base of code
        self.writer.write_all(&0x4000u32.to_le_bytes())?; // base of data
        self.writer.write_all(&optional_header.image_base.to_le_bytes())?;
        self.writer.write_all(&0x2000u32.to_le_bytes())?; // section alignment
        self.writer.write_all(&0x200u32.to_le_bytes())?; // file alignment
        self.writer.write_all(&[4, 0, 0, 0])?; // OS version
        self.writer.write_all(&[0, 0, 0, 0])?; // image version
        self.writer.write_all(&[4, 0, 0, 0])?; // subsystem version
        self.writer.write_all(&[0, 0, 0, 0])?; // win32 version
        self.writer.write_all(&optional_header.size_of_image.to_le_bytes())?;
        self.writer.write_all(&optional_header.size_of_headers.to_le_bytes())?;
        self.writer.write_all(&0u32.to_le_bytes())?; // checksum
        self.writer.write_all(&(optional_header.subsystem as u16).to_le_bytes())?;
        self.writer.write_all(&0u16.to_le_bytes())?; // dll characteristics
                                                     // Stack and heap sizes
        self.writer.write_all(&[0x00, 0x00, 0x10, 0x00])?; // stack reserve
        self.writer.write_all(&[0x00, 0x10, 0x00, 0x00])?; // stack commit
        self.writer.write_all(&[0x00, 0x00, 0x10, 0x00])?; // heap reserve
        self.writer.write_all(&[0x00, 0x10, 0x00, 0x00])?; // heap commit
        self.writer.write_all(&0u32.to_le_bytes())?; // loader flags
        self.writer.write_all(&16u32.to_le_bytes())?; // number of rva and sizes

        // Data directories (16 entries, 8 bytes each)
        for _ in 0..16 {
            self.writer.write_all(&[0; 8])?;
        }

        Ok(())
    }

    fn write_section_header(&mut self, section: &PeSection) -> Result<(), GaiaError> {
        let mut name_bytes = [0u8; 8];
        let name_bytes_slice = section.name.as_bytes();
        let copy_len = name_bytes_slice.len().min(8);
        name_bytes[..copy_len].copy_from_slice(&name_bytes_slice[..copy_len]);
        self.writer.write_all(&name_bytes)?;

        self.writer.write_all(&section.virtual_size.to_le_bytes())?;
        self.writer.write_all(&section.virtual_address.to_le_bytes())?;
        self.writer.write_all(&section.size_of_raw_data.to_le_bytes())?;
        self.writer.write_all(&section.pointer_to_raw_data.to_le_bytes())?;
        self.writer.write_all(&section.pointer_to_relocations.to_le_bytes())?;
        self.writer.write_all(&section.pointer_to_line_numbers.to_le_bytes())?;
        self.writer.write_all(&section.number_of_relocations.to_le_bytes())?;
        self.writer.write_all(&section.number_of_line_numbers.to_le_bytes())?;
        self.writer.write_all(&section.characteristics.to_le_bytes())?;

        Ok(())
    }
    fn build_pe_header(&self, text_section: &PeSection) -> Result<PeHeader, GaiaError> {
        let dos_header = DosHeader::new(0x80); // PE 头偏移

        let nt_header = NtHeader {
            signature: 0x00004550, // "PE\0\0"
        };

        let coff_header = CoffHeader::new(0x014C, 1) // IMAGE_FILE_MACHINE_I386
            .with_timestamp(0)
            .with_symbol_table(0, 0)
            .with_optional_header_size(224) // PE32 可选头大小
            .with_characteristics(0x0102); // IMAGE_FILE_EXECUTABLE_IMAGE | IMAGE_FILE_32BIT_MACHINE

        let optional_header = OptionalHeader::new(
            0x2000,                        // entry_point
            0x400000,                      // image_base
            text_section.size_of_raw_data, // size_of_code
            0x400,                         // size_of_headers
            0x4000,                        // size_of_image
            SubsystemType::Console,        // subsystem
        );

        Ok(PeHeader { dos_header, nt_header, coff_header, optional_header })
    }

    fn build_clr_data(&self, clr: &ClrProgram) -> Result<Vec<u8>, GaiaError> {
        let mut data = Vec::new();

        // 预留 CLR 头的空间 (72 字节)
        let clr_header_size = 72;
        data.resize(clr_header_size, 0);

        // 计算各部分的偏移量
        let metadata_offset = clr_header_size;
        let metadata_start = data.len();

        // 写入元数据
        self.write_metadata_to_buffer(&mut data, clr)?;
        let metadata_size = data.len() - metadata_start;

        // 对齐到 4 字节边界
        while data.len() % 4 != 0 {
            data.push(0);
        }

        let code_offset = data.len();

        // 写入代码
        self.write_code_to_buffer(&mut data, clr)?;

        // 现在回填 CLR 头，使用正确的 RVA 和大小
        let base_rva = 0x2000; // .text 节的虚拟地址
        let metadata_rva = base_rva + metadata_offset as u32;
        let code_rva = base_rva + code_offset as u32;

        self.write_clr_header_with_offsets(&mut data, clr, metadata_rva, metadata_size as u32)?;

        Ok(data)
    }

    fn write_clr_header_with_offsets(
        &self,
        buffer: &mut Vec<u8>,
        clr: &ClrProgram,
        metadata_rva: u32,
        metadata_size: u32,
    ) -> Result<(), GaiaError> {
        let clr_header = ClrHeader::new(
            clr.version.major as u16,
            clr.version.minor as u16,
            metadata_rva,
            metadata_size,
            0, // 入口点方法的 token
        );

        // 将 CLR 头写入到缓冲区的开始位置
        let mut cursor = Cursor::new(&mut buffer[0..72]);

        // 写入 CLR 头的字节
        cursor.write_all(&clr_header.cb.to_le_bytes())?;
        cursor.write_all(&clr_header.major_runtime_version.to_le_bytes())?;
        cursor.write_all(&clr_header.minor_runtime_version.to_le_bytes())?;
        cursor.write_all(&clr_header.metadata_rva.to_le_bytes())?;
        cursor.write_all(&clr_header.metadata_size.to_le_bytes())?;
        cursor.write_all(&clr_header.flags.to_le_bytes())?;
        cursor.write_all(&clr_header.entry_point_token.to_le_bytes())?;
        cursor.write_all(&clr_header.resources_rva.to_le_bytes())?;
        cursor.write_all(&clr_header.resources_size.to_le_bytes())?;
        cursor.write_all(&clr_header.strong_name_signature_rva.to_le_bytes())?;
        cursor.write_all(&clr_header.strong_name_signature_size.to_le_bytes())?;
        cursor.write_all(&clr_header.code_manager_table_rva.to_le_bytes())?;
        cursor.write_all(&clr_header.code_manager_table_size.to_le_bytes())?;
        cursor.write_all(&clr_header.vtable_fixups_rva.to_le_bytes())?;
        cursor.write_all(&clr_header.vtable_fixups_size.to_le_bytes())?;
        cursor.write_all(&clr_header.export_address_table_jumps_rva.to_le_bytes())?;
        cursor.write_all(&clr_header.export_address_table_jumps_size.to_le_bytes())?;
        cursor.write_all(&clr_header.managed_native_header_rva.to_le_bytes())?;
        cursor.write_all(&clr_header.managed_native_header_size.to_le_bytes())?;

        Ok(())
    }

    fn write_metadata_to_buffer(&self, buffer: &mut Vec<u8>, clr: &ClrProgram) -> Result<(), GaiaError> {
        // .NET 元数据根结构
        // 元数据头签名
        buffer.extend_from_slice(b"BSJB");

        // 版本信息
        buffer.extend_from_slice(&1u16.to_le_bytes()); // major version
        buffer.extend_from_slice(&1u16.to_le_bytes()); // minor version
        buffer.extend_from_slice(&0u32.to_le_bytes()); // reserved

        // 版本字符串长度和内容
        let version_str = "v4.0.30319";
        let version_len = ((version_str.len() + 3) / 4) * 4; // 对齐到 4 字节
        buffer.extend_from_slice(&(version_len as u32).to_le_bytes());
        buffer.extend_from_slice(version_str.as_bytes());

        // 填充到 4 字节对齐
        while buffer.len() % 4 != 0 {
            buffer.push(0);
        }

        // 标志和流数量
        buffer.extend_from_slice(&0u16.to_le_bytes()); // flags
        buffer.extend_from_slice(&5u16.to_le_bytes()); // 5 个流

        // 写入流头信息
        self.write_stream_headers(buffer)?;

        // 写入各个流的数据
        self.write_metadata_streams(buffer, clr)?;

        Ok(())
    }

    fn write_stream_headers(&self, buffer: &mut Vec<u8>) -> Result<(), GaiaError> {
        // #~ 流 (压缩元数据表)
        buffer.extend_from_slice(&0u32.to_le_bytes()); // offset (稍后填充)
        buffer.extend_from_slice(&0u32.to_le_bytes()); // size (稍后填充)
        buffer.extend_from_slice(b"#~\0\0"); // name

        // #Strings 流
        buffer.extend_from_slice(&0u32.to_le_bytes()); // offset
        buffer.extend_from_slice(&0u32.to_le_bytes()); // size
        buffer.extend_from_slice(b"#Strings\0\0\0\0"); // name (对齐到 4 字节)

        // #US 流 (用户字符串)
        buffer.extend_from_slice(&0u32.to_le_bytes()); // offset
        buffer.extend_from_slice(&0u32.to_le_bytes()); // size
        buffer.extend_from_slice(b"#US\0"); // name

        // #GUID 流
        buffer.extend_from_slice(&0u32.to_le_bytes()); // offset
        buffer.extend_from_slice(&0u32.to_le_bytes()); // size
        buffer.extend_from_slice(b"#GUID\0\0\0"); // name

        // #Blob 流
        buffer.extend_from_slice(&0u32.to_le_bytes()); // offset
        buffer.extend_from_slice(&0u32.to_le_bytes()); // size
        buffer.extend_from_slice(b"#Blob\0\0\0"); // name

        Ok(())
    }

    fn write_metadata_streams(&self, buffer: &mut Vec<u8>, clr: &ClrProgram) -> Result<(), GaiaError> {
        // 简化实现：写入最小的元数据流

        // #~ 流 (元数据表)
        self.write_metadata_tables_stream(buffer, clr)?;

        // #Strings 流
        self.write_strings_stream(buffer, clr)?;

        // #US 流 (用户字符串)
        self.write_user_strings_stream(buffer)?;

        // #GUID 流
        self.write_guid_stream(buffer)?;

        // #Blob 流
        self.write_blob_stream(buffer)?;

        Ok(())
    }

    fn write_metadata_tables_stream(&self, buffer: &mut Vec<u8>, clr: &ClrProgram) -> Result<(), GaiaError> {
        // 元数据表流头
        buffer.extend_from_slice(&0u32.to_le_bytes()); // reserved
        buffer.extend_from_slice(&2u8.to_le_bytes()); // major version
        buffer.extend_from_slice(&0u8.to_le_bytes()); // minor version
        buffer.extend_from_slice(&0u8.to_le_bytes()); // heap sizes
        buffer.extend_from_slice(&1u8.to_le_bytes()); // reserved

        // 有效表的位掩码 (简化：只包含 Module 表)
        buffer.extend_from_slice(&0x01u64.to_le_bytes()); // valid tables
        buffer.extend_from_slice(&0x01u64.to_le_bytes()); // sorted tables

        // 表行数
        buffer.extend_from_slice(&1u32.to_le_bytes()); // Module 表有 1 行

        // Module 表数据 (简化)
        buffer.extend_from_slice(&0u16.to_le_bytes()); // Generation
        buffer.extend_from_slice(&1u16.to_le_bytes()); // Name (字符串索引)
        buffer.extend_from_slice(&1u16.to_le_bytes()); // Mvid (GUID 索引)
        buffer.extend_from_slice(&0u16.to_le_bytes()); // EncId
        buffer.extend_from_slice(&0u16.to_le_bytes()); // EncBaseId

        Ok(())
    }

    fn write_strings_stream(&self, buffer: &mut Vec<u8>, clr: &ClrProgram) -> Result<(), GaiaError> {
        // 字符串流以空字节开始
        buffer.push(0);

        // 添加模块名称
        let module_name = clr.name.as_bytes();
        buffer.extend_from_slice(module_name);
        buffer.push(0); // null terminator

        // 对齐到 4 字节边界
        while buffer.len() % 4 != 0 {
            buffer.push(0);
        }

        Ok(())
    }

    fn write_user_strings_stream(&self, buffer: &mut Vec<u8>) -> Result<(), GaiaError> {
        // 用户字符串流以空字节开始
        buffer.push(0);

        // 对齐到 4 字节边界
        while buffer.len() % 4 != 0 {
            buffer.push(0);
        }

        Ok(())
    }

    fn write_guid_stream(&self, buffer: &mut Vec<u8>) -> Result<(), GaiaError> {
        // GUID 流包含一个模块 GUID (16 字节)
        let module_guid = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, 0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0];
        buffer.extend_from_slice(&module_guid);

        Ok(())
    }

    fn write_blob_stream(&self, buffer: &mut Vec<u8>) -> Result<(), GaiaError> {
        // Blob 流以空字节开始
        buffer.push(0);

        // 对齐到 4 字节边界
        while buffer.len() % 4 != 0 {
            buffer.push(0);
        }

        Ok(())
    }

    fn write_code_to_buffer(&self, buffer: &mut Vec<u8>, clr: &ClrProgram) -> Result<(), GaiaError> {
        // 写入方法代码
        for clr_type in &clr.types {
            for method in &clr_type.methods {
                self.write_method_code_to_buffer(buffer, method)?;
            }
        }

        // 写入全局方法
        for method in &clr.global_methods {
            self.write_method_code_to_buffer(buffer, method)?;
        }
        Ok(())
    }

    fn write_method_code_to_buffer(&self, buffer: &mut Vec<u8>, method: &ClrMethod) -> Result<(), GaiaError> {
        // 计算代码大小
        let mut code_size = 0u32;
        for instruction in &method.instructions {
            code_size += self.calculate_instruction_size(instruction)?;
        }

        // 选择方法头格式
        if code_size < 64 && method.max_stack <= 8 && method.locals.is_empty() {
            // 使用 Tiny 格式
            let header = (code_size << 2) | 0x02; // CorILMethod_TinyFormat
            buffer.push(header as u8);
        }
        else {
            // 使用 Fat 格式
            let flags: u16 = 0x03; // CorILMethod_FatFormat | CorILMethod_InitLocals
            buffer.extend_from_slice(&flags.to_le_bytes());
            buffer.push(0x30); // 头大小 (12 字节)
            buffer.extend_from_slice(&method.max_stack.to_le_bytes());
            buffer.extend_from_slice(&code_size.to_le_bytes());
            buffer.extend_from_slice(&0u32.to_le_bytes()); // 局部变量签名 token
        }

        // 写入指令
        for instruction in &method.instructions {
            self.write_instruction_to_buffer(buffer, instruction)?;
        }

        // 对齐到 4 字节边界
        while buffer.len() % 4 != 0 {
            buffer.push(0);
        }

        Ok(())
    }

    fn calculate_instruction_size(&self, instruction: &ClrInstruction) -> Result<u32, GaiaError> {
        match instruction {
            ClrInstruction::Simple { opcode } => {
                match opcode {
                    ClrOpcode::Nop | ClrOpcode::Ret => Ok(1),
                    ClrOpcode::Ldstr | ClrOpcode::Call => Ok(5), // opcode + 4 字节 token
                    _ => Ok(1),                                  // 默认单字节指令
                }
            }
            ClrInstruction::WithImmediate { opcode, .. } => {
                match opcode {
                    ClrOpcode::LdcI4 => Ok(5), // opcode + 4 字节立即数
                    _ => Ok(5),
                }
            }
            ClrInstruction::WithString { .. } => Ok(5), // opcode + 4 字节 token
            ClrInstruction::WithMethod { .. } => Ok(5), // opcode + 4 字节 token
            _ => Ok(1),
        }
    }

    fn write_instruction_to_buffer(&self, buffer: &mut Vec<u8>, instruction: &ClrInstruction) -> Result<(), GaiaError> {
        match instruction {
            ClrInstruction::Simple { opcode } => match opcode {
                ClrOpcode::Nop => buffer.push(0x00),
                ClrOpcode::Ldstr => buffer.push(0x72),
                ClrOpcode::Call => buffer.push(0x28),
                ClrOpcode::Ret => buffer.push(0x2A),
                _ => return Err(GaiaError::not_implemented("Unsupported opcode")),
            },
            ClrInstruction::WithImmediate { opcode, value } => match opcode {
                ClrOpcode::LdcI4 => {
                    buffer.push(0x20);
                    buffer.extend_from_slice(&value.to_le_bytes());
                }
                _ => return Err(GaiaError::not_implemented("Unsupported opcode with immediate")),
            },
            ClrInstruction::WithString { opcode, value } => {
                match opcode {
                    ClrOpcode::Ldstr => {
                        buffer.push(0x72);
                        // 这里应该写入字符串表的索引，暂时写入占位符
                        buffer.extend_from_slice(&[0x01, 0x00, 0x00, 0x70]);
                    }
                    _ => return Err(GaiaError::not_implemented("Unsupported opcode with string")),
                }
            }
            ClrInstruction::WithMethod { opcode, method_ref } => {
                match opcode {
                    ClrOpcode::Call => {
                        buffer.push(0x28);
                        // 这里应该写入方法表的索引，暂时写入占位符
                        buffer.extend_from_slice(&[0x01, 0x00, 0x00, 0x0A]);
                    }
                    _ => return Err(GaiaError::not_implemented("Unsupported opcode with method")),
                }
            }
            _ => return Err(GaiaError::not_implemented("Unsupported instruction type")),
        }
        Ok(())
    }
}

// 对齐到指定边界的辅助函数
fn align_to(value: u32, alignment: u32) -> u32 {
    (value + alignment - 1) & !(alignment - 1)
}

// CLR 头结构
#[derive(Debug, Copy, Clone)]
pub struct ClrHeader {
    cb: u32,
    major_runtime_version: u16,
    minor_runtime_version: u16,
    metadata_rva: u32,
    metadata_size: u32,
    flags: u32,
    entry_point_token: u32,
    resources_rva: u32,
    resources_size: u32,
    strong_name_signature_rva: u32,
    strong_name_signature_size: u32,
    code_manager_table_rva: u32,
    code_manager_table_size: u32,
    vtable_fixups_rva: u32,
    vtable_fixups_size: u32,
    export_address_table_jumps_rva: u32,
    export_address_table_jumps_size: u32,
    managed_native_header_rva: u32,
    managed_native_header_size: u32,
}

impl ClrHeader {
    /// 创建一个标准的 CLR 头
    pub fn new(
        major_runtime_version: u16,
        minor_runtime_version: u16,
        metadata_rva: u32,
        metadata_size: u32,
        entry_point_token: u32,
    ) -> Self {
        Self {
            cb: 72, // CLR 头大小
            major_runtime_version,
            minor_runtime_version,
            metadata_rva,
            metadata_size,
            flags: 0x01, // COMIMAGE_FLAGS_ILONLY
            entry_point_token,
            resources_rva: 0,
            resources_size: 0,
            strong_name_signature_rva: 0,
            strong_name_signature_size: 0,
            code_manager_table_rva: 0,
            code_manager_table_size: 0,
            vtable_fixups_rva: 0,
            vtable_fixups_size: 0,
            export_address_table_jumps_rva: 0,
            export_address_table_jumps_size: 0,
            managed_native_header_rva: 0,
            managed_native_header_size: 0,
        }
    }
}
