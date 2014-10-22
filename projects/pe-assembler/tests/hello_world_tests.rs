//! Hello World PE 文件生成测试

use gaia_types::helpers::Architecture;
use pe_assembler::generator::easy_console_log;
use std::fs;

#[test]
fn test_hello_world_x86() {
    let pe_data = easy_console_log(Architecture::X86, "Hello World!").unwrap();

    // 验证 DOS 头
    assert_eq!(pe_data[0], 0x4D); // 'M'
    assert_eq!(pe_data[1], 0x5A); // 'Z'

    // 验证 PE 签名位置
    let pe_offset = u32::from_le_bytes([pe_data[60], pe_data[61], pe_data[62], pe_data[63]]) as usize;
    assert_eq!(&pe_data[pe_offset..pe_offset + 4], b"PE\0\0");

    // 验证机器类型 (x86)
    let machine_type = u16::from_le_bytes([pe_data[pe_offset + 4], pe_data[pe_offset + 5]]);
    assert_eq!(machine_type, 0x014C); // IMAGE_FILE_MACHINE_I386

    println!("✅ x86 Hello World PE file generated successfully ({} bytes)", pe_data.len());
}

#[test]
fn test_hello_world_x64() {
    let pe_data = easy_console_log(Architecture::X86_64, "Hello World!").unwrap();

    // 验证 DOS 头
    assert_eq!(pe_data[0], 0x4D); // 'M'
    assert_eq!(pe_data[1], 0x5A); // 'Z'

    // 验证 PE 签名位置
    let pe_offset = u32::from_le_bytes([pe_data[60], pe_data[61], pe_data[62], pe_data[63]]) as usize;
    assert_eq!(&pe_data[pe_offset..pe_offset + 4], b"PE\0\0");

    // 验证机器类型 (x64)
    let machine_type = u16::from_le_bytes([pe_data[pe_offset + 4], pe_data[pe_offset + 5]]);
    assert_eq!(machine_type, 0x8664); // IMAGE_FILE_MACHINE_AMD64

    println!("✅ x64 Hello World PE file generated successfully ({} bytes)", pe_data.len());
}

#[test]
fn test_save_hello_world_files() {
    // 生成并保存 Hello World PE 文件用于测试
    let x86_pe = easy_console_log(Architecture::X86, "Hello from x86!").unwrap();
    let x64_pe = easy_console_log(Architecture::X86_64, "Hello from x64!").unwrap();

    // 保存到测试目录
    let test_dir = "tests/generated";
    fs::create_dir_all(test_dir).ok();

    fs::write(format!("{}/hello_x86.exe", test_dir), &x86_pe).unwrap();
    fs::write(format!("{}/hello_x64.exe", test_dir), &x64_pe).unwrap();

    println!("✅ Generated Hello World PE files saved to {}/", test_dir);
    println!("  - hello_x86.exe ({} bytes)", x86_pe.len());
    println!("  - hello_x64.exe ({} bytes)", x64_pe.len());
}

#[test]
fn test_unsupported_architecture_hello_world() {
    let result = easy_console_log(Architecture::ARM, "Hello");
    assert!(result.is_err());

    println!("✅ Unsupported architecture correctly rejected for Hello World");
}
