//! X64CodeBuilder 与 PeAssembler writer 的集成
//!
//! 提供从 Gaia 指令到 PE 文件的完整编译流程

use crate::{
    assembler::{ExportTable, ImportTable as AssemblerImportTable},
    types::{DataDirectory, DosHeader, NtHeader, OptionalHeader, PeHeader, PeProgram, PeSection, SubsystemType},
    writer::PeWriter,
};
use gaia_types::{helpers::Architecture, GaiaError, GaiaFunction, GaiaInstruction, GaiaProgram};
use pe_coff::types::CoffHeader;
use std::{collections::HashMap, io::Cursor};

use super::{
    code_builder::X64CodeBuilder,
    context::{FunctionCall, Label, RelocationInfo, RelocationType, X64Context, X64Register},
};

type Result<T> = std::result::Result<T, GaiaError>;

/// PE 文件生成器，集成 X64CodeBuilder 和 PeAssembler
#[derive(Debug)]
pub struct PeGenerator {
    /// x64 代码构建器
    code_builder: X64CodeBuilder,
    /// 导入函数映射：函数名 -> DLL名
    import_functions: HashMap<String, String>,
}

impl PeGenerator {
    /// 创建新的 PE 生成器
    pub fn new() -> Self {
        let mut generator = Self { code_builder: X64CodeBuilder::new(), import_functions: HashMap::new() };
        generator.add_system_imports();
        generator
    }

    /// 添加系统导入函数
    fn add_system_imports(&mut self) {
        self.import_functions.insert("ExitProcess".to_string(), "kernel32.dll".to_string());
        self.import_functions.insert("printf".to_string(), "msvcrt.dll".to_string());
        self.import_functions.insert("MessageBoxA".to_string(), "user32.dll".to_string());
    }

    /// 从 Gaia 程序生成 PE 文件
    pub fn generate_from_gaia(&mut self, program: &GaiaProgram) -> Result<Vec<u8>> {
        // 1. 编译 Gaia 指令到 x64 机器码
        self.compile_gaia_program(program)?;

        // 2. 获取代码生成上下文 - 创建新的 code_builder 来避免所有权问题
        let mut temp_builder = std::mem::replace(&mut self.code_builder, X64CodeBuilder::new());
        let context = temp_builder.finish();

        // 3. 构建 PE 结构
        let pe_program = self.build_pe_program(context)?;

        // 4. 写入 PE 文件
        let mut buffer = Cursor::new(Vec::new());
        let mut pe_writer = PeWriter::new(&mut buffer);
        pe_writer.write_program(&pe_program)?;

        Ok(buffer.into_inner())
    }

    /// 编译 Gaia 程序到 x64 机器码
    fn compile_gaia_program(&mut self, program: &GaiaProgram) -> Result<()> {
        // 生成程序入口点
        self.code_builder.function_prologue();

        // 编译主函数
        if let Some(main_func) = program.functions.iter().find(|f| f.name == "main") {
            self.compile_function(main_func)?;
        }

        // 生成程序退出
        self.code_builder.exit_program(0);

        Ok(())
    }

    /// 编译单个函数
    fn compile_function(&mut self, function: &GaiaFunction) -> Result<()> {
        // 定义函数标签
        self.code_builder.context_mut().define_label(&function.name);

        // 编译函数指令
        for instruction in &function.instructions {
            self.compile_instruction(instruction)?;
        }

        Ok(())
    }

    /// 编译单个指令
    fn compile_instruction(&mut self, instruction: &GaiaInstruction) -> Result<()> {
        match instruction {
            GaiaInstruction::LoadConstant(constant) => match constant {
                gaia_types::GaiaConstant::Integer32(value) => {
                    self.compile_load_immediate(*value as i64);
                }
                gaia_types::GaiaConstant::Integer64(value) => {
                    self.compile_load_immediate(*value);
                }
                _ => {
                    return Err(GaiaError::syntax_error(
                        &format!("Unsupported constant type: {:?}", constant),
                        gaia_types::SourceLocation::default(),
                    ));
                }
            },
            GaiaInstruction::Call(function_name) => {
                self.compile_function_call(function_name);
            }
            GaiaInstruction::Add => {
                self.compile_add_operation();
            }
            GaiaInstruction::StoreLocal(local_index) => {
                self.compile_store_local(*local_index as i32);
            }
            GaiaInstruction::LoadLocal(local_index) => {
                self.compile_load_local(*local_index as i32);
            }
            GaiaInstruction::BranchIfTrue(label) => {
                self.compile_conditional_jump(label);
            }
            GaiaInstruction::Branch(label) => {
                self.compile_unconditional_jump(label);
            }
            GaiaInstruction::Label(name) => {
                self.code_builder.context_mut().define_label(name);
            }
            _ => {
                return Err(GaiaError::syntax_error(
                    &format!("Unsupported instruction: {:?}", instruction),
                    gaia_types::SourceLocation::default(),
                ));
            }
        }

        Ok(())
    }

    /// 编译加载立即数指令
    fn compile_load_immediate(&mut self, value: i64) {
        // mov rax, value
        if value >= i32::MIN as i64 && value <= i32::MAX as i64 {
            self.code_builder.context_mut().emit_bytes(&[0xB8]);
            self.code_builder.context_mut().emit_bytes(&(value as i32).to_le_bytes());
        }
        else {
            self.code_builder.context_mut().emit_bytes(&[0x48, 0xB8]);
            self.code_builder.context_mut().emit_bytes(&value.to_le_bytes());
        }
    }

    /// 编译函数调用指令
    fn compile_function_call(&mut self, function_name: &str) {
        // 检查是否为导入函数
        if self.import_functions.contains_key(function_name) {
            self.code_builder.context_mut().add_function_call(function_name, true);
            // call [function_name] - RIP相对寻址
            self.code_builder.context_mut().emit_bytes(&[0xFF, 0x15, 0x00, 0x00, 0x00, 0x00]);
        }
        else {
            // 内部函数调用
            self.code_builder.context_mut().add_function_call(function_name, false);
            // call function_name - 相对调用
            self.code_builder.context_mut().emit_bytes(&[0xE8]);
            let _offset = self.code_builder.context_mut().reference_label(function_name);
            self.code_builder.context_mut().emit_bytes(&[0x00, 0x00, 0x00, 0x00]);
            // 占位符
        }
    }

    /// 编译加法操作
    fn compile_add_operation(&mut self) {
        // pop rbx; pop rax; add rax, rbx; push rax
        self.code_builder.context_mut().emit_bytes(&[
            0x5B, // pop rbx
            0x58, // pop rax
            0x48, 0x01, 0xD8, // add rax, rbx
            0x50, // push rax
        ]);
    }

    /// 编译存储本地变量
    fn compile_store_local(&mut self, local_index: i32) {
        let offset = self.code_builder.context_mut().allocate_stack(8);
        // mov [rbp + offset], rax
        if offset >= -128 && offset <= 127 {
            self.code_builder.context_mut().emit_bytes(&[0x48, 0x89, 0x45]);
            self.code_builder.context_mut().emit_bytes(&[offset as u8]);
        }
        else {
            self.code_builder.context_mut().emit_bytes(&[0x48, 0x89, 0x85]);
            self.code_builder.context_mut().emit_bytes(&offset.to_le_bytes());
        }
    }

    /// 编译加载本地变量
    fn compile_load_local(&mut self, local_index: i32) {
        let offset = -(local_index * 8); // 简化的偏移计算
                                         // mov rax, [rbp + offset]
        if offset >= -128 && offset <= 127 {
            self.code_builder.context_mut().emit_bytes(&[0x48, 0x8B, 0x45]);
            self.code_builder.context_mut().emit_bytes(&[offset as u8]);
        }
        else {
            self.code_builder.context_mut().emit_bytes(&[0x48, 0x8B, 0x85]);
            self.code_builder.context_mut().emit_bytes(&offset.to_le_bytes());
        }
    }

    /// 编译条件跳转
    fn compile_conditional_jump(&mut self, label: &str) {
        // test rax, rax; jz label
        self.code_builder.context_mut().emit_bytes(&[0x48, 0x85, 0xC0]); // test rax, rax
        self.code_builder.context_mut().emit_bytes(&[0x0F, 0x84]); // jz
        let _offset = self.code_builder.context_mut().reference_label(label);
        self.code_builder.context_mut().emit_bytes(&[0x00, 0x00, 0x00, 0x00]); // 占位符
    }

    /// 编译无条件跳转
    fn compile_unconditional_jump(&mut self, label: &str) {
        // jmp label
        self.code_builder.context_mut().emit_bytes(&[0xE9]); // jmp
        let _offset = self.code_builder.context_mut().reference_label(label);
        self.code_builder.context_mut().emit_bytes(&[0x00, 0x00, 0x00, 0x00]); // 占位符
    }

    /// 构建 PE 程序结构
    fn build_pe_program(&self, context: X64Context) -> Result<PeProgram> {
        // 先构建导入表，避免借用冲突
        let import_table = self.build_import_table(&context)?;

        // 创建 .text 节
        let text_section = PeSection {
            name: ".text".to_string(),
            virtual_size: context.code.len() as u32,
            virtual_address: 0x1000,
            size_of_raw_data: align_to(context.code.len() as u32, 0x200),
            pointer_to_raw_data: 0x400,
            pointer_to_relocations: 0,
            pointer_to_line_numbers: 0,
            number_of_relocations: 0,
            number_of_line_numbers: 0,
            characteristics: 0x60000020, // CODE | EXECUTE | READ
            data: context.code,          // 现在可以安全地移动 context.code
        };

        // 创建 PE 头
        let header = self.build_pe_header(&text_section)?;

        Ok(PeProgram {
            header,
            sections: vec![text_section],
            imports: import_table,
            exports: crate::assembler::ExportTable { name: "main".to_string(), functions: Vec::new() },
        })
    }

    /// 构建导入表
    fn build_import_table(&self, context: &X64Context) -> Result<AssemblerImportTable> {
        let mut dll_functions: HashMap<String, Vec<String>> = HashMap::new();

        // 收集所有导入函数
        for func_call in &context.function_calls {
            if func_call.is_import {
                if let Some(dll_name) = self.import_functions.get(&func_call.name) {
                    dll_functions.entry(dll_name.clone()).or_insert_with(Vec::new).push(func_call.name.clone());
                }
            }
        }

        // 简化处理：只处理第一个 DLL
        if let Some((dll_name, functions)) = dll_functions.into_iter().next() {
            Ok(AssemblerImportTable { dll_name, functions })
        }
        else {
            Ok(AssemblerImportTable { dll_name: String::new(), functions: Vec::new() })
        }
    }

    /// 构建 PE 头
    fn build_pe_header(&self, text_section: &PeSection) -> Result<PeHeader> {
        let dos_header = DosHeader::default();
        let nt_header = NtHeader { signature: 0x00004550 };

        let optional_header = OptionalHeader {
            magic: 0x020B, // PE32+
            major_linker_version: 14,
            minor_linker_version: 0,
            size_of_code: text_section.size_of_raw_data,
            size_of_initialized_data: 0,
            size_of_uninitialized_data: 0,
            address_of_entry_point: 0x1000,
            base_of_code: 0x1000,
            base_of_data: None,      // PE32+ 没有这个字段
            image_base: 0x140000000, // 64位默认基址
            section_alignment: 0x1000,
            file_alignment: 0x200,
            major_operating_system_version: 6,
            minor_operating_system_version: 0,
            major_image_version: 0,
            minor_image_version: 0,
            major_subsystem_version: 6,
            minor_subsystem_version: 0,
            win32_version_value: 0,
            size_of_image: 0x2000,
            size_of_headers: 0x400,
            checksum: 0,
            subsystem: SubsystemType::Console,
            dll_characteristics: 0x8160,
            size_of_stack_reserve: 0x100000,
            size_of_stack_commit: 0x1000,
            size_of_heap_reserve: 0x100000,
            size_of_heap_commit: 0x1000,
            loader_flags: 0,
            number_of_rva_and_sizes: 16,
            data_directories: vec![Default::default(); 16],
        };

        Ok(PeHeader {
            dos_header,
            nt_header,
            coff_header: pe_coff::types::CoffHeader {
                machine: 0x8664, // AMD64
                number_of_sections: 1,
                time_date_stamp: 0,
                pointer_to_symbol_table: 0,
                number_of_symbols: 0,
                size_of_optional_header: 240,
                characteristics: 0x0102,
            },
            optional_header,
        })
    }
}

impl Default for PeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// 对齐到指定边界
fn align_to(value: u32, alignment: u32) -> u32 {
    (value + alignment - 1) & !(alignment - 1)
}

/// 导入表项
#[derive(Debug, Clone)]
pub struct ImportEntry {
    pub function_name: String,
    pub ordinal: Option<u16>,
    pub iat_offset: u32,
}

/// 导入表
#[derive(Debug, Clone)]
pub struct ImportTable {
    pub dll_name: String,
    pub functions: Vec<ImportEntry>,
}
