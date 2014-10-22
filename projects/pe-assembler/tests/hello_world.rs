//! Hello World PE 文件生成器
//!
//! 此模块提供生成简单控制台输出 PE 文件的功能。
//! 使用新的人体工程学友好的汇编器API。

use gaia_types::helpers::Architecture;
use pe_assembler::{
    assembler::{x86::X86CodeBuilder, PeAssemblerBuilder},
    types::SubsystemType,
};

/// 生成 Hello World PE 文件 - 使用新的汇编器API
pub fn generate_hello_world_pe() -> Vec<u8> {
    // 使用X86代码生成器创建Hello World程序代码
    let code = X86CodeBuilder::hello_world_program();

    // 使用PeAssemblerBuilder构建PE文件
    PeAssemblerBuilder::new()
        .architecture(Architecture::X86)
        .subsystem(SubsystemType::Console)
        .entry_point(0x1000)
        .image_base(0x400000)
        .import_functions("kernel32.dll", &["GetStdHandle", "WriteConsoleA", "ExitProcess"])
        .code(code)
        .data(b"Hello World!\n\0".to_vec())
        .generate()
        .expect("Failed to generate PE bytes")
}

/// 生成简单的退出程序PE文件
pub fn generate_exit_pe(exit_code: i32) -> Vec<u8> {
    let code = X86CodeBuilder::exit_program(exit_code);

    PeAssemblerBuilder::new()
        .architecture(Architecture::X86)
        .subsystem(SubsystemType::Console)
        .entry_point(0x1000)
        .image_base(0x400000)
        .import_function("kernel32.dll", "ExitProcess")
        .code(code)
        .generate()
        .expect("Failed to generate PE bytes")
}

/// 生成消息框程序PE文件
pub fn generate_message_box_pe(message: &str, title: &str) -> Vec<u8> {
    let code = X86CodeBuilder::message_box_program();

    let mut data = Vec::new();
    data.extend_from_slice(message.as_bytes());
    data.push(0); // null terminator

    // 对齐到合适的位置
    while data.len() % 4 != 0 {
        data.push(0);
    }

    let _title_offset = data.len();
    data.extend_from_slice(title.as_bytes());
    data.push(0); // null terminator

    PeAssemblerBuilder::new()
        .architecture(Architecture::X86)
        .subsystem(SubsystemType::Windows)
        .entry_point(0x1000)
        .image_base(0x400000)
        .import_function("user32.dll", "MessageBoxA")
        .import_function("kernel32.dll", "ExitProcess")
        .code(code)
        .data(data)
        .generate()
        .expect("Failed to generate PE bytes")
}

/// 传统的手动构建方式 - 保留作为参考
#[allow(dead_code)]
pub fn generate_hello_world_pe_manual() -> Vec<u8> {
    use pe_assembler::types::{DataDirectory, DosHeader, NtHeader, OptionalHeader, PeHeader, PeSection};
    use pe_coff::types::CoffHeader;

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
    let coff_header = CoffHeader {
        machine: 0x014C, // IMAGE_FILE_MACHINE_I386
        number_of_sections: 2,
        time_date_stamp: 0,
        pointer_to_symbol_table: 0,
        number_of_symbols: 0,
        size_of_optional_header: 224, // PE32 可选头大小
        characteristics: 0x0102,      // IMAGE_FILE_EXECUTABLE_IMAGE | IMAGE_FILE_32BIT_MACHINE
    };

    // 创建可选头
    let optional_header = OptionalHeader {
        magic: 0x010B, // PE32
        major_linker_version: 14,
        minor_linker_version: 0,
        size_of_code: 0x200,
        size_of_initialized_data: 0x200,
        size_of_uninitialized_data: 0,
        address_of_entry_point: 0x1000,
        base_of_code: 0x1000,
        base_of_data: Some(0x2000),
        image_base: 0x400000,
        section_alignment: 0x1000,
        file_alignment: 0x200,
        major_operating_system_version: 6,
        minor_operating_system_version: 0,
        major_image_version: 0,
        minor_image_version: 0,
        major_subsystem_version: 6,
        minor_subsystem_version: 0,
        win32_version_value: 0,
        size_of_image: 0x3000,
        size_of_headers: 0x200,
        checksum: 0,
        subsystem: SubsystemType::Console,
        dll_characteristics: 0,
        size_of_stack_reserve: 0x100000,
        size_of_stack_commit: 0x1000,
        size_of_heap_reserve: 0x100000,
        size_of_heap_commit: 0x1000,
        loader_flags: 0,
        number_of_rva_and_sizes: 16,
        data_directories: vec![
            DataDirectory { virtual_address: 0, size: 0 },       // Export Table
            DataDirectory { virtual_address: 0x2040, size: 40 }, // Import Table
            DataDirectory { virtual_address: 0, size: 0 },       // Resource Table
            DataDirectory { virtual_address: 0, size: 0 },       // Exception Table
            DataDirectory { virtual_address: 0, size: 0 },       // Certificate Table
            DataDirectory { virtual_address: 0, size: 0 },       // Base Relocation Table
            DataDirectory { virtual_address: 0, size: 0 },       // Debug
            DataDirectory { virtual_address: 0, size: 0 },       // Architecture
            DataDirectory { virtual_address: 0, size: 0 },       // Global Ptr
            DataDirectory { virtual_address: 0, size: 0 },       // TLS Table
            DataDirectory { virtual_address: 0, size: 0 },       // Load Config Table
            DataDirectory { virtual_address: 0, size: 0 },       // Bound Import
            DataDirectory { virtual_address: 0x2010, size: 16 }, // IAT
            DataDirectory { virtual_address: 0, size: 0 },       // Delay Import Descriptor
            DataDirectory { virtual_address: 0, size: 0 },       // COM+ Runtime Header
            DataDirectory { virtual_address: 0, size: 0 },       // Reserved
        ],
    };

    // 创建 PE 头
    let pe_header = PeHeader { dos_header, nt_header, coff_header, optional_header };

    // 生成Hello World程序的机器码
    let mut code = vec![
        // 程序入口点 - Hello World
        // 调用 GetStdHandle(STD_OUTPUT_HANDLE)
        0x6A, 0xF5, // push -11 (STD_OUTPUT_HANDLE)
        0xFF, 0x15, 0x10, 0x20, 0x40, 0x00, // call dword ptr [0x402010] (GetStdHandle)
        0x89, 0xC1, // mov ecx, eax (保存句柄到ecx)
        // 调用 WriteConsoleA(handle, text, length, written, reserved)
        0x6A, 0x00, // push 0 (reserved)
        0x6A, 0x00, // push 0 (written - 可以为NULL)
        0x6A, 0x0D, // push 13 (length of "Hello World!\n")
        0x68, 0x00, 0x20, 0x40, 0x00, // push 0x402000 (address of "Hello World!\n")
        0x51, // push ecx (handle)
        0xFF, 0x15, 0x14, 0x20, 0x40, 0x00, // call dword ptr [0x402014] (WriteConsoleA)
        // 调用 ExitProcess(0)
        0x6A, 0x00, // push 0 (exit code)
        0xFF, 0x15, 0x18, 0x20, 0x40, 0x00, // call dword ptr [0x402018] (ExitProcess)
        // 不应该到达这里
        0xCC, // int 3
    ];

    // 填充到 512 字节
    while code.len() < 0x200 {
        code.push(0);
    }

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

    // 创建数据节，包含导入地址表和字符串
    let mut data = Vec::new();

    // Hello World 字符串 (0x2000)
    data.extend_from_slice(b"Hello World!\n\0");

    // 导入地址表 (IAT) - 从 0x2010 开始
    data.extend_from_slice(&[0x00; 0x10 - data.len()]); // 填充到 0x2010

    // IAT 条目 (0x2010-0x201F)
    data.extend_from_slice(&0x2080u32.to_le_bytes()); // GetStdHandle 函数名地址
    data.extend_from_slice(&0x2090u32.to_le_bytes()); // WriteConsoleA 函数名地址
    data.extend_from_slice(&0x20A0u32.to_le_bytes()); // ExitProcess 函数名地址
    data.extend_from_slice(&[0x00; 4]); // IAT 结束标记

    // 填充到导入表区域 (0x2040)
    while data.len() < 0x40 {
        data.push(0);
    }

    // 导入表描述符 (0x2040)
    data.extend_from_slice(&0x2060u32.to_le_bytes()); // OriginalFirstThunk (函数名指针表)
    data.extend_from_slice(&[0x00; 8]); // TimeDateStamp + ForwarderChain
    data.extend_from_slice(&0x20B0u32.to_le_bytes()); // Name (DLL名称地址)
    data.extend_from_slice(&0x2010u32.to_le_bytes()); // FirstThunk (IAT地址)

    // 导入表结束标记 (0x2054)
    data.extend_from_slice(&[0x00; 20]);

    // 函数名指针表 (0x2060)
    data.extend_from_slice(&0x2080u32.to_le_bytes()); // GetStdHandle
    data.extend_from_slice(&0x2090u32.to_le_bytes()); // WriteConsoleA
    data.extend_from_slice(&0x20A0u32.to_le_bytes()); // ExitProcess
    data.extend_from_slice(&[0x00; 4]); // 结束标记

    // 填充到函数名区域 (0x2080)
    while data.len() < 0x80 {
        data.push(0);
    }

    // 函数名 (0x2080开始)
    // GetStdHandle (0x2080)
    data.extend_from_slice(&[0x00, 0x00]); // hint
    data.extend_from_slice(b"GetStdHandle\0");
    // WriteConsoleA (0x2090)
    data.extend_from_slice(&[0x00, 0x00]); // hint
    data.extend_from_slice(b"WriteConsoleA\0");
    // ExitProcess (0x20A0)
    data.extend_from_slice(&[0x00, 0x00]); // hint
    data.extend_from_slice(b"ExitProcess\0");

    // DLL名称 (0x20B0)
    data.extend_from_slice(b"kernel32.dll\0");

    // 填充到合理的大小
    while data.len() < 0x200 {
        data.push(0);
    }

    // 创建 .data 节
    let data_section = PeSection {
        name: ".data".to_string(),
        virtual_size: 0x1000,
        virtual_address: 0x2000,
        size_of_raw_data: 0x200,
        pointer_to_raw_data: 0x400,
        pointer_to_relocations: 0,
        pointer_to_line_numbers: 0,
        number_of_relocations: 0,
        number_of_line_numbers: 0,
        characteristics: 0xC0000040, // IMAGE_SCN_CNT_INITIALIZED_DATA | IMAGE_SCN_MEM_READ | IMAGE_SCN_MEM_WRITE
        data,
    };

    // 手动构建PE文件
    let mut pe_data = Vec::new();

    // 写入DOS头
    pe_data.extend_from_slice(&pe_header.dos_header.e_magic.to_le_bytes());
    pe_data.extend_from_slice(&pe_header.dos_header.e_cblp.to_le_bytes());
    pe_data.extend_from_slice(&pe_header.dos_header.e_cp.to_le_bytes());
    pe_data.extend_from_slice(&pe_header.dos_header.e_crlc.to_le_bytes());
    pe_data.extend_from_slice(&pe_header.dos_header.e_cparhdr.to_le_bytes());
    pe_data.extend_from_slice(&pe_header.dos_header.e_minalloc.to_le_bytes());
    pe_data.extend_from_slice(&pe_header.dos_header.e_maxalloc.to_le_bytes());
    pe_data.extend_from_slice(&pe_header.dos_header.e_ss.to_le_bytes());
    pe_data.extend_from_slice(&pe_header.dos_header.e_sp.to_le_bytes());
    pe_data.extend_from_slice(&pe_header.dos_header.e_csum.to_le_bytes());
    pe_data.extend_from_slice(&pe_header.dos_header.e_ip.to_le_bytes());
    pe_data.extend_from_slice(&pe_header.dos_header.e_cs.to_le_bytes());
    pe_data.extend_from_slice(&pe_header.dos_header.e_lfarlc.to_le_bytes());
    pe_data.extend_from_slice(&pe_header.dos_header.e_ovno.to_le_bytes());
    for res in &pe_header.dos_header.e_res {
        pe_data.extend_from_slice(&res.to_le_bytes());
    }
    pe_data.extend_from_slice(&pe_header.dos_header.e_oemid.to_le_bytes());
    pe_data.extend_from_slice(&pe_header.dos_header.e_oeminfo.to_le_bytes());
    for res in &pe_header.dos_header.e_res2 {
        pe_data.extend_from_slice(&res.to_le_bytes());
    }
    pe_data.extend_from_slice(&pe_header.dos_header.e_lfanew.to_le_bytes());

    // 填充到PE头位置
    while pe_data.len() < 0x80 {
        pe_data.push(0);
    }

    // 写入NT头
    pe_data.extend_from_slice(&pe_header.nt_header.signature.to_le_bytes());

    // 写入COFF头
    pe_data.extend_from_slice(&pe_header.coff_header.machine.to_le_bytes());
    pe_data.extend_from_slice(&pe_header.coff_header.number_of_sections.to_le_bytes());
    pe_data.extend_from_slice(&pe_header.coff_header.time_date_stamp.to_le_bytes());
    pe_data.extend_from_slice(&pe_header.coff_header.pointer_to_symbol_table.to_le_bytes());
    pe_data.extend_from_slice(&pe_header.coff_header.number_of_symbols.to_le_bytes());
    pe_data.extend_from_slice(&pe_header.coff_header.size_of_optional_header.to_le_bytes());
    pe_data.extend_from_slice(&pe_header.coff_header.characteristics.to_le_bytes());

    // 写入可选头
    pe_data.extend_from_slice(&pe_header.optional_header.magic.to_le_bytes());
    pe_data.extend_from_slice(&pe_header.optional_header.major_linker_version.to_le_bytes());
    pe_data.extend_from_slice(&pe_header.optional_header.minor_linker_version.to_le_bytes());
    pe_data.extend_from_slice(&pe_header.optional_header.size_of_code.to_le_bytes());
    pe_data.extend_from_slice(&pe_header.optional_header.size_of_initialized_data.to_le_bytes());
    pe_data.extend_from_slice(&pe_header.optional_header.size_of_uninitialized_data.to_le_bytes());
    pe_data.extend_from_slice(&pe_header.optional_header.address_of_entry_point.to_le_bytes());
    pe_data.extend_from_slice(&pe_header.optional_header.base_of_code.to_le_bytes());
    if let Some(base_of_data) = pe_header.optional_header.base_of_data {
        pe_data.extend_from_slice(&base_of_data.to_le_bytes());
    }
    pe_data.extend_from_slice(&pe_header.optional_header.image_base.to_le_bytes());
    pe_data.extend_from_slice(&pe_header.optional_header.section_alignment.to_le_bytes());
    pe_data.extend_from_slice(&pe_header.optional_header.file_alignment.to_le_bytes());
    pe_data.extend_from_slice(&pe_header.optional_header.major_operating_system_version.to_le_bytes());
    pe_data.extend_from_slice(&pe_header.optional_header.minor_operating_system_version.to_le_bytes());
    pe_data.extend_from_slice(&pe_header.optional_header.major_image_version.to_le_bytes());
    pe_data.extend_from_slice(&pe_header.optional_header.minor_image_version.to_le_bytes());
    pe_data.extend_from_slice(&pe_header.optional_header.major_subsystem_version.to_le_bytes());
    pe_data.extend_from_slice(&pe_header.optional_header.minor_subsystem_version.to_le_bytes());
    pe_data.extend_from_slice(&pe_header.optional_header.win32_version_value.to_le_bytes());
    pe_data.extend_from_slice(&pe_header.optional_header.size_of_image.to_le_bytes());
    pe_data.extend_from_slice(&pe_header.optional_header.size_of_headers.to_le_bytes());
    pe_data.extend_from_slice(&pe_header.optional_header.checksum.to_le_bytes());

    // 子系统类型
    let subsystem_value = match pe_header.optional_header.subsystem {
        SubsystemType::Console => 3u16,
        SubsystemType::Windows => 2u16,
        _ => 3u16,
    };
    pe_data.extend_from_slice(&subsystem_value.to_le_bytes());

    pe_data.extend_from_slice(&pe_header.optional_header.dll_characteristics.to_le_bytes());
    pe_data.extend_from_slice(&pe_header.optional_header.size_of_stack_reserve.to_le_bytes());
    pe_data.extend_from_slice(&pe_header.optional_header.size_of_stack_commit.to_le_bytes());
    pe_data.extend_from_slice(&pe_header.optional_header.size_of_heap_reserve.to_le_bytes());
    pe_data.extend_from_slice(&pe_header.optional_header.size_of_heap_commit.to_le_bytes());
    pe_data.extend_from_slice(&pe_header.optional_header.loader_flags.to_le_bytes());
    pe_data.extend_from_slice(&pe_header.optional_header.number_of_rva_and_sizes.to_le_bytes());

    // 数据目录
    for dir in &pe_header.optional_header.data_directories {
        pe_data.extend_from_slice(&dir.virtual_address.to_le_bytes());
        pe_data.extend_from_slice(&dir.size.to_le_bytes());
    }

    // 写入节头
    // .text 节头
    let mut text_name = [0u8; 8];
    text_name[..6].copy_from_slice(b".text\0");
    pe_data.extend_from_slice(&text_name);
    pe_data.extend_from_slice(&text_section.virtual_size.to_le_bytes());
    pe_data.extend_from_slice(&text_section.virtual_address.to_le_bytes());
    pe_data.extend_from_slice(&text_section.size_of_raw_data.to_le_bytes());
    pe_data.extend_from_slice(&text_section.pointer_to_raw_data.to_le_bytes());
    pe_data.extend_from_slice(&text_section.pointer_to_relocations.to_le_bytes());
    pe_data.extend_from_slice(&text_section.pointer_to_line_numbers.to_le_bytes());
    pe_data.extend_from_slice(&text_section.number_of_relocations.to_le_bytes());
    pe_data.extend_from_slice(&text_section.number_of_line_numbers.to_le_bytes());
    pe_data.extend_from_slice(&text_section.characteristics.to_le_bytes());

    // .data 节头
    let mut data_name = [0u8; 8];
    data_name[..6].copy_from_slice(b".data\0");
    pe_data.extend_from_slice(&data_name);
    pe_data.extend_from_slice(&data_section.virtual_size.to_le_bytes());
    pe_data.extend_from_slice(&data_section.virtual_address.to_le_bytes());
    pe_data.extend_from_slice(&data_section.size_of_raw_data.to_le_bytes());
    pe_data.extend_from_slice(&data_section.pointer_to_raw_data.to_le_bytes());
    pe_data.extend_from_slice(&data_section.pointer_to_relocations.to_le_bytes());
    pe_data.extend_from_slice(&data_section.pointer_to_line_numbers.to_le_bytes());
    pe_data.extend_from_slice(&data_section.number_of_relocations.to_le_bytes());
    pe_data.extend_from_slice(&data_section.number_of_line_numbers.to_le_bytes());
    pe_data.extend_from_slice(&data_section.characteristics.to_le_bytes());

    // 填充到文件对齐边界
    while pe_data.len() < 0x200 {
        pe_data.push(0);
    }

    // 写入节数据
    pe_data.extend_from_slice(&text_section.data);
    pe_data.extend_from_slice(&data_section.data);

    pe_data
}
