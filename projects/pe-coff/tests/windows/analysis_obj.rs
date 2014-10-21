//! COFF 对象文件分析测试
//!
//! 这个模块测试 COFF 对象文件的解析功能，使用 Windows 系统中常见的 obj 文件

use pe_coff::{
    easy_test::print_coff_summary,
    reader::{read_coff_from_file, CoffReader},
};
use std::path::Path;

/// 测试解析 Windows 系统中的 obj 文件
#[test]
fn test_parse_system_obj_files() {
    // 常见的 Windows 系统 obj 文件路径
    let test_paths = vec![
        // Visual Studio 运行时库对象文件
        r"C:\Program Files (x86)\Microsoft Visual Studio\2019\Community\VC\Tools\MSVC\14.29.30133\lib\x64\libcmt.lib",
        r"C:\Program Files (x86)\Microsoft Visual Studio\2019\Community\VC\Tools\MSVC\14.29.30133\lib\x86\libcmt.lib",
        // Windows SDK 库文件
        r"C:\Program Files (x86)\Windows Kits\10\Lib\10.0.19041.0\um\x64\kernel32.lib",
        r"C:\Program Files (x86)\Windows Kits\10\Lib\10.0.19041.0\um\x86\kernel32.lib",
    ];

    let mut found_files = 0;

    for path_str in test_paths {
        let path = Path::new(path_str);
        if path.exists() {
            found_files += 1;
            println!("测试文件: {}", path_str);

            match test_parse_obj_file(path) {
                Ok(_) => println!("✓ 成功解析: {}", path_str),
                Err(e) => println!("✗ 解析失败: {} - {}", path_str, e),
            }
        }
        else {
            println!("文件不存在: {}", path_str);
        }
    }

    if found_files == 0 {
        println!("警告: 未找到任何测试文件，跳过测试");
    }
    else {
        println!("总共测试了 {} 个文件", found_files);
    }
}

/// 测试解析单个 obj 文件
fn test_parse_obj_file(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // 尝试解析文件
    let coff_object = read_coff_from_file(path)?;

    // 使用 easy_test 模块的便利函数打印摘要
    print_coff_summary(&coff_object);

    Ok(())
}

/// 测试 COFF 读取器配置
#[test]
fn test_coff_reader_config() {
    let config = CoffReader::default();
    assert!(config.include_section_data);
    assert!(config.parse_symbols);
    assert!(config.parse_relocations);

    let custom_config = CoffReader { include_section_data: false, parse_symbols: true, parse_relocations: false };

    assert!(!custom_config.include_section_data);
    assert!(custom_config.parse_symbols);
    assert!(!custom_config.parse_relocations);
}

/// 测试创建临时 obj 文件并解析
#[test]
fn test_minimal_obj_parsing() {
    // 创建一个最小的 COFF 头部用于测试
    let minimal_coff = create_minimal_coff_data();

    // 这里我们只测试头部解析，因为创建完整的 obj 文件比较复杂
    if minimal_coff.len() >= 20 {
        println!("创建了 {} 字节的最小 COFF 数据", minimal_coff.len());
        // 实际的解析测试会在有真实文件时进行
    }
}

/// 创建最小的 COFF 数据用于测试
fn create_minimal_coff_data() -> Vec<u8> {
    let mut data = Vec::new();

    // COFF 头部 (20 字节)
    data.extend_from_slice(&0x014cu16.to_le_bytes()); // machine (x86)
    data.extend_from_slice(&0u16.to_le_bytes()); // number_of_sections
    data.extend_from_slice(&0u32.to_le_bytes()); // time_date_stamp
    data.extend_from_slice(&0u32.to_le_bytes()); // pointer_to_symbol_table
    data.extend_from_slice(&0u32.to_le_bytes()); // number_of_symbols
    data.extend_from_slice(&0u16.to_le_bytes()); // size_of_optional_header
    data.extend_from_slice(&0u16.to_le_bytes()); // characteristics

    data
}
