//! 退出代码 PE 文件生成器
//!
//! 此模块提供生成简单退出代码 PE 文件的功能，使用正确的 PE 汇编器结构。

use gaia_types::{helpers::Architecture, GaiaError};
use pe_assembler::{
    types::{DataDirectory, DosHeader, NtHeader, OptionalHeader, PeHeader, PeSection, SubsystemType},
    writer::{PeBuilder, PeWriter},
};
use pe_coff::types::CoffHeader;

/// 生成 x86 架构的退出代码 PE 文件
pub fn generate_x86_exit_code(exit_code: u32) -> Result<Vec<u8>, GaiaError> {
    // 创建 DOS 头
    let dos_header = DosHeader {
        e_magic: 0x5A4D, // "MZ"
        e_cblp: 0x90,
        e_cp: 0x03,
        e_crlc: 0,
        e_cparhdr: 4,
        e_minalloc: 0,
        e_maxalloc: 0xFFFF,
        e_ss: 0,
        e_sp: 0xB8,
        e_csum: 0,
        e_ip: 0,
        e_cs: 0,
        e_lfarlc: 0x40,
        e_ovno: 0,
        e_res: [0; 4],
        e_oemid: 0,
        e_oeminfo: 0,
        e_res2: [0; 10],
        e_lfanew: 0x80, // PE 头偏移
    };

    // 创建 NT 头
    let nt_header = NtHeader {
        signature: 0x00004550, // "PE\0\0"
    };

    // 创建 COFF 头
    let coff_header = CoffHeader::new(0x014C, 1) // IMAGE_FILE_MACHINE_I386
        .with_time_date_stamp(0)
        .with_pointer_to_symbol_table(0)
        .with_number_of_symbols(0)
        .with_size_of_optional_header(224)
        .with_characteristics(0x0102); // IMAGE_FILE_EXECUTABLE_IMAGE | IMAGE_FILE_32BIT_MACHINE

    // 创建可选头 (PE32)
    let optional_header = OptionalHeader::new_for_architecture(
        &Architecture::X86,
        0x1000,   // entry_point
        0x400000, // image_base
        0x1000,   // size_of_code
        0x200,    // size_of_headers
        0x2000,   // size_of_image
        SubsystemType::Console,
    );

    // 创建 PE 头
    let pe_header = PeHeader { dos_header, nt_header, coff_header, optional_header };

    // 创建 .text 节的机器码
    let mut code = Vec::new();

    // Windows x86 退出代码
    // 简化的实现，直接设置退出代码并返回
    // mov eax, exit_code
    code.push(0xB8);
    code.extend_from_slice(&exit_code.to_le_bytes());
    // ret (返回)
    code.push(0xC3);

    // 创建 .text 节
    let text_section = PeSection {
        name: ".text".to_string(),
        virtual_size: 0x1000,
        virtual_address: 0x1000,
        size_of_raw_data: 0x200,
        pointer_to_raw_data: 0x200,
        pointer_to_relocations: 0,
        pointer_to_line_numbers: 0,
        number_of_relocations: 0,
        number_of_line_numbers: 0,
        characteristics: 0x60000020, // IMAGE_SCN_CNT_CODE | IMAGE_SCN_MEM_EXECUTE | IMAGE_SCN_MEM_READ
        data: code,
    };

    // 创建空的导入表和导出表
    let import_table = ImportTable { dll_name: String::new(), functions: Vec::new() };

    let export_table = ExportTable { name: String::new(), functions: Vec::new() };

    // 构建 PE 程序
    let pe_program = PeBuilder::new().with_header(pe_header).add_section(text_section).build()?;

    let mut final_program = pe_program;
    final_program.imports = import_table;
    final_program.exports = export_table;

    // 使用 PeAssembler 写入二进制数据
    PeWriter::write_program(&final_program)
}

/// 生成 x64 架构的退出代码 PE 文件
pub fn generate_x64_exit_code(exit_code: u32) -> Result<Vec<u8>, GaiaError> {
    // 创建 DOS 头
    let dos_header = DosHeader {
        e_magic: 0x5A4D, // "MZ"
        e_cblp: 0x90,
        e_cp: 0x03,
        e_crlc: 0,
        e_cparhdr: 4,
        e_minalloc: 0,
        e_maxalloc: 0xFFFF,
        e_ss: 0,
        e_sp: 0xB8,
        e_csum: 0,
        e_ip: 0,
        e_cs: 0,
        e_lfarlc: 0x40,
        e_ovno: 0,
        e_res: [0; 4],
        e_oemid: 0,
        e_oeminfo: 0,
        e_res2: [0; 10],
        e_lfanew: 0x80, // PE 头偏移
    };

    // 创建 NT 头
    let nt_header = NtHeader {
        signature: 0x00004550, // "PE\0\0"
    };

    // 创建 COFF 头
    let coff_header = CoffHeader::new(0x8664, 1) // IMAGE_FILE_MACHINE_AMD64
        .with_time_date_stamp(0)
        .with_pointer_to_symbol_table(0)
        .with_number_of_symbols(0)
        .with_size_of_optional_header(240)
        .with_characteristics(0x0102); // IMAGE_FILE_EXECUTABLE_IMAGE | IMAGE_FILE_32BIT_MACHINE

    // 创建可选头 (PE32+)
    let optional_header = OptionalHeader::new_for_architecture(
        &Architecture::X86_64,
        0x1000,      // entry_point
        0x140000000, // image_base
        0x1000,      // size_of_code
        0x200,       // size_of_headers
        0x2000,      // size_of_image
        SubsystemType::Console,
    );

    // 创建 PE 头
    let pe_header = PeHeader { dos_header, nt_header, coff_header, optional_header };

    // 创建 .text 节的机器码
    let mut code = Vec::new();

    // Windows x64 程序入口点
    // 我们需要正确设置退出代码并返回

    // mov eax, exit_code (设置返回值)
    code.push(0xB8); // MOV EAX, imm32
    code.extend_from_slice(&exit_code.to_le_bytes());

    // ret (返回到操作系统)
    code.push(0xC3);

    // 创建 .text 节
    let text_section = PeSection {
        name: ".text".to_string(),
        virtual_size: 0x1000,
        virtual_address: 0x1000,
        size_of_raw_data: 0x200,
        pointer_to_raw_data: 0x200,
        pointer_to_relocations: 0,
        pointer_to_line_numbers: 0,
        number_of_relocations: 0,
        number_of_line_numbers: 0,
        characteristics: 0x60000020, // IMAGE_SCN_CNT_CODE | IMAGE_SCN_MEM_EXECUTE | IMAGE_SCN_MEM_READ
        data: code,
    };

    // 创建空的导入表和导出表
    let import_table = ImportTable { dll_name: String::new(), functions: Vec::new() };

    let export_table = ExportTable { name: String::new(), functions: Vec::new() };

    // 构建 PE 程序
    let pe_program = PeBuilder::new().with_header(pe_header).add_section(text_section).build()?;

    let mut final_program = pe_program;
    final_program.imports = import_table;
    final_program.exports = export_table;

    // 使用 PeAssembler 写入二进制数据
    PeWriter::write_program(&final_program)
}
