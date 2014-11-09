#![doc = include_str!("readme.md")]

use crate::program::*;
use byteorder::{LittleEndian, WriteBytesExt};
use gaia_types::{BinaryAssembler, GaiaDiagnostics, GaiaError, Result};
use leb128::write::{signed as write_sleb128, unsigned as write_uleb128};
use std::io::Write;

/// WASM 二进制写入器
#[derive(Debug)]
pub struct WasmWriter<W> {
    writer: BinaryAssembler<W, LittleEndian>,
}

impl<W> WasmWriter<W> {
    /// 创建新的 WASM 写入器
    pub fn new(assembler: W) -> Self {
        Self { writer: BinaryAssembler::new(assembler) }
    }

    /// 完成写入并返回底层写入器
    pub fn finish(self) -> W {
        self.writer.finish()
    }
}
impl<W: Write> WasmWriter<W> {
    /// 将 WasiProgram 写入为二进制 WASM 格式
    pub fn write(&mut self, program: WasiProgram) -> GaiaDiagnostics<Vec<u8>> {
        match self.write_program(program) {
            Ok(bytes) => GaiaDiagnostics::success(bytes),
            Err(error) => GaiaDiagnostics::failure(error),
        }
    }

    /// 内部写入程序的实现
    fn write_program(&mut self, program: WasiProgram) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();

        // 根据程序类型选择不同的写入策略
        match program.program_type {
            WasiProgramType::Component => {
                self.write_component(&mut buffer, program)?;
            }
            WasiProgramType::CoreModule => {
                self.write_core_module(&mut buffer, program)?;
            }
        }

        Ok(buffer)
    }

    /// 写入组件格式
    fn write_component(&mut self, buffer: &mut Vec<u8>, program: WasiProgram) -> Result<()> {
        // WebAssembly Component Model 魔数和版本
        buffer.extend_from_slice(&[0x00, 0x61, 0x73, 0x6D]); // "\0asm"
        buffer.extend_from_slice(&[0x0A, 0x00, 0x01, 0x00]); // version 1.10 (component model)

        // 写入组件段
        if !program.core_modules.is_empty() {
            self.write_core_module_section(buffer, &program.core_modules)?;
        }

        if !program.component_items.is_empty() {
            self.write_component_items_section(buffer, &program.component_items)?;
        }

        if !program.instances.is_empty() {
            self.write_instance_section(buffer, &program.instances)?;
        }

        if !program.aliases.is_empty() {
            self.write_alias_section(buffer, &program.aliases)?;
        }

        if !program.exports.is_empty() {
            self.write_export_section(buffer, &program.exports)?;
        }

        Ok(())
    }

    /// 写入核心模块格式
    fn write_core_module(&mut self, buffer: &mut Vec<u8>, program: WasiProgram) -> Result<()> {
        // 写入 WASM 魔数和版本
        self.write_header(buffer)?;

        // 写入各个段
        if !program.function_types.is_empty() {
            self.write_type_section(buffer, &program.function_types)?;
        }

        if !program.imports.is_empty() {
            self.write_import_section(buffer, &program.imports)?;
        }

        if !program.functions.is_empty() {
            self.write_function_section(buffer, &program.functions)?;
        }

        if !program.memories.is_empty() {
            self.write_memory_section(buffer, &program.memories)?;
        }

        if !program.exports.is_empty() {
            self.write_export_section(buffer, &program.exports)?;
        }

        if let Some(start_func) = program.start_function {
            self.write_start_section(buffer, start_func)?;
        }

        if !program.functions.is_empty() {
            self.write_code_section(buffer, &program.functions)?;
        }

        for custom_section in &program.custom_sections {
            self.write_custom_section(buffer, custom_section)?;
        }

        Ok(())
    }

    /// 写入 WASM 文件头
    fn write_header(&mut self, buffer: &mut Vec<u8>) -> Result<()> {
        // WASM 魔数: 0x00 0x61 0x73 0x6D
        buffer.extend_from_slice(&[0x00, 0x61, 0x73, 0x6D]);
        // WASM 版本: 0x01 0x00 0x00 0x00
        buffer.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]);
        Ok(())
    }

    /// 写入类型段
    fn write_type_section(&mut self, buffer: &mut Vec<u8>, types: &[WasiFunctionType]) -> Result<()> {
        let mut section_data = Vec::new();

        // 写入类型数量
        write_uleb128(&mut section_data, types.len() as u64)?;

        for func_type in types {
            // 函数类型标识符 (0x60)
            section_data.push(0x60);

            // 写入参数类型
            write_uleb128(&mut section_data, func_type.params.len() as u64)?;
            for param_type in &func_type.params {
                section_data.push(self.encode_value_type(*param_type));
            }

            // 写入返回值类型
            write_uleb128(&mut section_data, func_type.results.len() as u64)?;
            for result_type in &func_type.results {
                section_data.push(self.encode_value_type(*result_type));
            }
        }

        self.write_section(buffer, 1, &section_data)?;
        Ok(())
    }

    /// 写入导入段
    fn write_import_section(&mut self, buffer: &mut Vec<u8>, imports: &[WasiImport]) -> Result<()> {
        let mut section_data = Vec::new();

        write_uleb128(&mut section_data, imports.len() as u64)?;

        for import in imports {
            // 写入模块名
            self.write_string(&mut section_data, &import.module)?;
            // 写入导入名
            self.write_string(&mut section_data, &import.field)?;

            // 写入导入类型
            match &import.import_type {
                WasmImportType::Function { type_index } => {
                    section_data.push(0x00); // 函数导入
                    write_uleb128(&mut section_data, *type_index as u64)?;
                }
                WasmImportType::Memory { memory_type } => {
                    section_data.push(0x02); // 内存导入
                    self.write_memory_type(&mut section_data, memory_type)?;
                }
                WasmImportType::Global { global_type } => {
                    section_data.push(0x03); // 全局变量导入
                    section_data.push(self.encode_value_type(global_type.value_type));
                    section_data.push(if global_type.mutable { 0x01 } else { 0x00 });
                }
                WasmImportType::Table { table_type } => {
                    section_data.push(0x01); // 表导入
                    self.write_table_type(&mut section_data, table_type)?;
                }
            }
        }

        self.write_section(buffer, 2, &section_data)?;
        Ok(())
    }

    /// 写入函数段
    fn write_function_section(&mut self, buffer: &mut Vec<u8>, functions: &[WasiFunction]) -> Result<()> {
        let mut section_data = Vec::new();

        write_uleb128(&mut section_data, functions.len() as u64)?;

        for function in functions {
            write_uleb128(&mut section_data, function.type_index as u64)?;
        }

        self.write_section(buffer, 3, &section_data)?;
        Ok(())
    }

    /// 写入内存段
    fn write_memory_section(&mut self, buffer: &mut Vec<u8>, memories: &[WasiMemory]) -> Result<()> {
        let mut section_data = Vec::new();

        write_uleb128(&mut section_data, memories.len() as u64)?;

        for memory in memories {
            self.write_memory_type(&mut section_data, &memory.memory_type)?;
        }

        self.write_section(buffer, 5, &section_data)?;
        Ok(())
    }

    /// 写入导出段
    fn write_export_section(&mut self, buffer: &mut Vec<u8>, exports: &[WasiExport]) -> Result<()> {
        let mut section_data = Vec::new();

        write_uleb128(&mut section_data, exports.len() as u64)?;

        for export in exports {
            // 写入导出名
            self.write_string(&mut section_data, &export.name)?;

            // 写入导出类型和索引
            match &export.export_type {
                WasmExportType::Function { function_index } => {
                    section_data.push(0x00); // 函数导出
                    write_uleb128(&mut section_data, *function_index as u64)?;
                }
                WasmExportType::Memory { memory_index } => {
                    section_data.push(0x02); // 内存导出
                    write_uleb128(&mut section_data, *memory_index as u64)?;
                }
                WasmExportType::Global { global_index } => {
                    section_data.push(0x03); // 全局变量导出
                    write_uleb128(&mut section_data, *global_index as u64)?;
                }
                WasmExportType::Table { table_index } => {
                    section_data.push(0x01); // 表导出
                    write_uleb128(&mut section_data, *table_index as u64)?;
                }
            }
        }

        self.write_section(buffer, 7, &section_data)?;
        Ok(())
    }

    /// 写入起始段
    fn write_start_section(&mut self, buffer: &mut Vec<u8>, start_func: u32) -> Result<()> {
        let mut section_data = Vec::new();
        write_uleb128(&mut section_data, start_func as u64)?;
        self.write_section(buffer, 8, &section_data)?;
        Ok(())
    }

    /// 写入代码段
    fn write_code_section(&mut self, buffer: &mut Vec<u8>, functions: &[WasiFunction]) -> Result<()> {
        let mut section_data = Vec::new();

        write_uleb128(&mut section_data, functions.len() as u64)?;

        for function in functions {
            let mut func_data = Vec::new();

            // 写入局部变量
            write_uleb128(&mut func_data, function.locals.len() as u64)?;
            for local in &function.locals {
                write_uleb128(&mut func_data, local.count as u64)?;
                func_data.push(self.encode_value_type(local.value_type));
            }

            // 写入指令
            for instruction in &function.body {
                self.write_instruction(&mut func_data, instruction)?;
            }

            // 函数结束标记
            func_data.push(0x0B);

            // 写入函数大小和数据
            write_uleb128(&mut section_data, func_data.len() as u64)?;
            section_data.extend_from_slice(&func_data);
        }

        self.write_section(buffer, 10, &section_data)?;
        Ok(())
    }

    /// 写入自定义段
    fn write_custom_section(&mut self, buffer: &mut Vec<u8>, custom: &WasiCustomSection) -> Result<()> {
        let mut section_data = Vec::new();

        // 写入段名称
        self.write_string(&mut section_data, &custom.name)?;
        // 写入段数据
        section_data.extend_from_slice(&custom.data);

        self.write_section(buffer, 0, &section_data)?;
        Ok(())
    }

    /// 写入段
    fn write_section(&mut self, buffer: &mut Vec<u8>, section_id: u8, data: &[u8]) -> Result<()> {
        buffer.push(section_id);
        write_uleb128(buffer, data.len() as u64)?;
        buffer.extend_from_slice(data);
        Ok(())
    }

    /// 写入字符串
    fn write_string(&mut self, buffer: &mut Vec<u8>, s: &str) -> Result<()> {
        let bytes = s.as_bytes();
        write_uleb128(buffer, bytes.len() as u64)?;
        buffer.extend_from_slice(bytes);
        Ok(())
    }

    /// 写入内存类型
    fn write_memory_type(&mut self, buffer: &mut Vec<u8>, memory_type: &WasmMemoryType) -> Result<()> {
        if let Some(max) = memory_type.max {
            buffer.push(0x01); // 有最大值
            write_uleb128(buffer, memory_type.min as u64)?;
            write_uleb128(buffer, max as u64)?;
        }
        else {
            buffer.push(0x00); // 无最大值
            write_uleb128(buffer, memory_type.min as u64)?;
        }
        Ok(())
    }

    /// 写入表类型
    fn write_table_type(&mut self, buffer: &mut Vec<u8>, table_type: &WasmTableType) -> Result<()> {
        // 写入元素类型
        match table_type.element_type {
            WasmReferenceType::FuncRef => buffer.push(0x70),
            WasmReferenceType::ExternRef => buffer.push(0x6F),
        }

        // 写入限制
        if let Some(max) = table_type.max {
            buffer.push(0x01); // 有最大值
            write_uleb128(buffer, table_type.min as u64)?;
            write_uleb128(buffer, max as u64)?;
        }
        else {
            buffer.push(0x00); // 无最大值
            write_uleb128(buffer, table_type.min as u64)?;
        }
        Ok(())
    }

    /// 写入指令
    fn write_instruction(&mut self, buffer: &mut Vec<u8>, instruction: &WasmInstruction) -> Result<()> {
        match instruction {
            WasmInstruction::Nop => buffer.push(0x01),
            WasmInstruction::Unreachable => buffer.push(0x00),
            WasmInstruction::Block { .. } => buffer.push(0x02),
            WasmInstruction::Loop { .. } => buffer.push(0x03),
            WasmInstruction::If { .. } => buffer.push(0x04),
            WasmInstruction::Else => buffer.push(0x05),
            WasmInstruction::End => buffer.push(0x0B),
            WasmInstruction::Br { label_index } => {
                buffer.push(0x0C);
                write_uleb128(buffer, *label_index as u64)?;
            }
            WasmInstruction::BrIf { label_index } => {
                buffer.push(0x0D);
                write_uleb128(buffer, *label_index as u64)?;
            }
            WasmInstruction::Return => buffer.push(0x0F),
            WasmInstruction::Call { function_index } => {
                buffer.push(0x10);
                write_uleb128(buffer, *function_index as u64)?;
            }
            WasmInstruction::Drop => buffer.push(0x1A),
            WasmInstruction::Select => buffer.push(0x1B),
            WasmInstruction::LocalGet { local_index } => {
                buffer.push(0x20);
                write_uleb128(buffer, *local_index as u64)?;
            }
            WasmInstruction::LocalSet { local_index } => {
                buffer.push(0x21);
                write_uleb128(buffer, *local_index as u64)?;
            }
            WasmInstruction::I32Const { value } => {
                buffer.push(0x41);
                write_sleb128(buffer, *value as i64)?;
            }
            WasmInstruction::I64Const { value } => {
                buffer.push(0x42);
                write_sleb128(buffer, *value)?;
            }
            WasmInstruction::F32Const { value } => {
                buffer.push(0x43);
                buffer.write_f32::<LittleEndian>(*value)?;
            }
            WasmInstruction::F64Const { value } => {
                buffer.push(0x44);
                buffer.write_f64::<LittleEndian>(*value)?;
            }
            WasmInstruction::I32Add => buffer.push(0x6A),
            WasmInstruction::I32Sub => buffer.push(0x6B),
            WasmInstruction::I32Mul => buffer.push(0x6C),
            WasmInstruction::I32DivS => buffer.push(0x6D),
            WasmInstruction::I32DivU => buffer.push(0x6E),
            WasmInstruction::I32RemS => buffer.push(0x6F),
            WasmInstruction::I32RemU => buffer.push(0x70),
            WasmInstruction::I32And => buffer.push(0x71),
            WasmInstruction::I32Or => buffer.push(0x72),
            WasmInstruction::I32Xor => buffer.push(0x73),
            WasmInstruction::I32Shl => buffer.push(0x74),
            WasmInstruction::I32ShrS => buffer.push(0x75),
            WasmInstruction::I32ShrU => buffer.push(0x76),
            WasmInstruction::I32Rotl => buffer.push(0x77),
            WasmInstruction::I32Rotr => buffer.push(0x78),
            WasmInstruction::I32Eqz => buffer.push(0x45),
            WasmInstruction::I32Eq => buffer.push(0x46),
            WasmInstruction::I32Ne => buffer.push(0x47),
            WasmInstruction::I32LtS => buffer.push(0x48),
            _ => {
                return Err(GaiaError::not_implemented(format!("Instruction {:?} not yet supported", instruction)));
            }
        }
        Ok(())
    }

    /// 编码值类型
    fn encode_value_type(&self, value_type: WasmValueType) -> u8 {
        match value_type {
            WasmValueType::I32 => 0x7F,
            WasmValueType::I64 => 0x7E,
            WasmValueType::F32 => 0x7D,
            WasmValueType::F64 => 0x7C,
        }
    }

    // 组件模型相关的写入方法
    fn write_core_module_section(&mut self, buffer: &mut Vec<u8>, core_modules: &[WasiCoreModule]) -> Result<()> {
        let mut section_data = Vec::new();
        write_uleb128(&mut section_data, core_modules.len() as u64)?;

        for core_module in core_modules {
            // 写入核心模块的二进制数据
            let mut module_data = Vec::new();
            self.write_core_module_binary(&mut module_data, core_module)?;
            write_uleb128(&mut section_data, module_data.len() as u64)?;
            section_data.extend_from_slice(&module_data);
        }

        self.write_section(buffer, 1, &section_data) // Core module section ID
    }

    fn write_core_module_binary(&mut self, buffer: &mut Vec<u8>, core_module: &WasiCoreModule) -> Result<()> {
        // 写入 WASM 魔数和版本
        buffer.extend_from_slice(&[0x00, 0x61, 0x73, 0x6D]); // "\0asm"
        buffer.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]); // version 1

        // 写入核心模块的各个段
        if !core_module.function_types.is_empty() {
            self.write_type_section(buffer, &core_module.function_types)?;
        }

        if !core_module.imports.is_empty() {
            self.write_import_section(buffer, &core_module.imports)?;
        }

        if !core_module.functions.is_empty() {
            self.write_function_section(buffer, &core_module.functions)?;
        }

        if !core_module.memories.is_empty() {
            self.write_memory_section(buffer, &core_module.memories)?;
        }

        if !core_module.exports.is_empty() {
            self.write_export_section(buffer, &core_module.exports)?;
        }

        if let Some(start_func) = core_module.start_function {
            self.write_start_section(buffer, start_func)?;
        }

        if !core_module.functions.is_empty() {
            self.write_code_section(buffer, &core_module.functions)?;
        }

        if !core_module.data_segments.is_empty() {
            self.write_data_section(buffer, &core_module.data_segments)?;
        }

        Ok(())
    }

    fn write_component_items_section(&mut self, buffer: &mut Vec<u8>, items: &[WasiComponentItem]) -> Result<()> {
        let mut section_data = Vec::new();
        write_uleb128(&mut section_data, items.len() as u64)?;

        for item in items {
            match item {
                WasiComponentItem::Type(type_def) => {
                    section_data.push(0x00); // Type item tag
                    self.write_type_definition(&mut section_data, type_def)?;
                }
                WasiComponentItem::Alias(alias) => {
                    section_data.push(0x01); // Alias item tag
                    self.write_alias(&mut section_data, alias)?;
                }
                WasiComponentItem::Instance(instance) => {
                    section_data.push(0x02); // Instance item tag
                    self.write_instance(&mut section_data, instance)?;
                }
                WasiComponentItem::CoreFunc(core_func) => {
                    section_data.push(0x03); // Core func item tag
                    self.write_core_func(&mut section_data, core_func)?;
                }
                WasiComponentItem::CoreInstance(core_instance) => {
                    section_data.push(0x04); // Core instance item tag
                    self.write_core_instance(&mut section_data, core_instance)?;
                }
            }
        }

        self.write_section(buffer, 2, &section_data) // Component items section ID
    }

    fn write_instance_section(&mut self, buffer: &mut Vec<u8>, instances: &[WasiInstance]) -> Result<()> {
        let mut section_data = Vec::new();
        write_uleb128(&mut section_data, instances.len() as u64)?;

        for instance in instances {
            self.write_instance(&mut section_data, instance)?;
        }

        self.write_section(buffer, 3, &section_data) // Instance section ID
    }

    fn write_alias_section(&mut self, buffer: &mut Vec<u8>, aliases: &[WasiAlias]) -> Result<()> {
        let mut section_data = Vec::new();
        write_uleb128(&mut section_data, aliases.len() as u64)?;

        for alias in aliases {
            self.write_alias(&mut section_data, alias)?;
        }

        self.write_section(buffer, 4, &section_data) // Alias section ID
    }

    fn write_data_section(&mut self, buffer: &mut Vec<u8>, data_segments: &[WasiDataSegment]) -> Result<()> {
        let mut section_data = Vec::new();
        write_uleb128(&mut section_data, data_segments.len() as u64)?;

        for data_segment in data_segments {
            // 写入数据段模式（0 = active）
            section_data.push(0x00);

            // 写入偏移表达式
            for instruction in &data_segment.offset {
                self.write_instruction(&mut section_data, instruction)?;
            }
            section_data.push(0x0B); // end

            // 写入数据
            write_uleb128(&mut section_data, data_segment.data.len() as u64)?;
            section_data.extend_from_slice(&data_segment.data);
        }

        self.write_section(buffer, 11, &section_data) // Data section ID
    }

    // 占位符实现，需要根据具体规范完善
    fn write_type_definition(&mut self, buffer: &mut Vec<u8>, _type_def: &WasiTypeDefinition) -> Result<()> {
        // 简化实现
        buffer.push(0x60); // func type
        buffer.push(0x00); // no params
        buffer.push(0x00); // no results
        Ok(())
    }

    fn write_alias(&mut self, buffer: &mut Vec<u8>, _alias: &WasiAlias) -> Result<()> {
        // 简化实现
        buffer.push(0x00); // outer alias
        write_uleb128(buffer, 0)?; // outer index
        write_uleb128(buffer, 0)?; // item index
        Ok(())
    }

    fn write_instance(&mut self, buffer: &mut Vec<u8>, _instance: &WasiInstance) -> Result<()> {
        // 简化实现
        buffer.push(0x00); // instantiate module
        write_uleb128(buffer, 0)?; // module index
        write_uleb128(buffer, 0)?; // arg count
        Ok(())
    }

    fn write_core_func(&mut self, buffer: &mut Vec<u8>, _core_func: &WasiCoreFunc) -> Result<()> {
        // 简化实现
        buffer.push(0x00); // canon lower
        write_uleb128(buffer, 0)?; // func index
        write_uleb128(buffer, 0)?; // option count
        Ok(())
    }

    fn write_core_instance(&mut self, buffer: &mut Vec<u8>, _core_instance: &WasiCoreInstance) -> Result<()> {
        // 简化实现
        buffer.push(0x00); // instantiate
        write_uleb128(buffer, 0)?; // module index
        write_uleb128(buffer, 0)?; // arg count
        Ok(())
    }
}
