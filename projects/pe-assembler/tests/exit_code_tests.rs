//! Exit Code PE 文件生成测试

use gaia_types::helpers::Architecture;
use pe_assembler::generator::easy_exit_code;
use std::fs;

#[test]
fn test_exit_code_x86() {
    let pe_data = easy_exit_code(Architecture::X86, 42).unwrap();

    // 验证 DOS 头
    assert_eq!(pe_data[0], 0x4D); // 'M'
    assert_eq!(pe_data[1], 0x5A); // 'Z'

    // 验证 PE 签名位置
    let pe_offset = u32::from_le_bytes([pe_data[60], pe_data[61], pe_data[62], pe_data[63]]) as usize;
    assert_eq!(&pe_data[pe_offset..pe_offset + 4], b"PE\0\0");

    // 验证机器类型 (x86)
    let machine_type = u16::from_le_bytes([pe_data[pe_offset + 4], pe_data[pe_offset + 5]]);
    assert_eq!(machine_type, 0x014C); // IMAGE_FILE_MACHINE_I386

    println!("✅ x86 Exit Code PE file generated successfully ({} bytes)", pe_data.len());
}

#[test]
fn test_exit_code_x64() {
    let pe_data = easy_exit_code(Architecture::X86_64, 42).unwrap();

    // 验证 DOS 头
    assert_eq!(pe_data[0], 0x4D); // 'M'
    assert_eq!(pe_data[1], 0x5A); // 'Z'

    // 验证 PE 签名位置
    let pe_offset = u32::from_le_bytes([pe_data[60], pe_data[61], pe_data[62], pe_data[63]]) as usize;
    assert_eq!(&pe_data[pe_offset..pe_offset + 4], b"PE\0\0");

    // 验证机器类型 (x64)
    let machine_type = u16::from_le_bytes([pe_data[pe_offset + 4], pe_data[pe_offset + 5]]);
    assert_eq!(machine_type, 0x8664); // IMAGE_FILE_MACHINE_AMD64

    println!("✅ x64 Exit Code PE file generated successfully ({} bytes)", pe_data.len());
}

#[test]
fn test_save_exit_code_files() {
    // 生成退出代码文件
    let exit_x86 = easy_exit_code(Architecture::X86, 123).unwrap();
    let exit_x64 = easy_exit_code(Architecture::X86_64, 456).unwrap();

    // 保存到测试目录
    let test_dir = "tests/generated";
    fs::create_dir_all(test_dir).ok();

    fs::write(format!("{}/exit_x86.exe", test_dir), &exit_x86).unwrap();
    fs::write(format!("{}/exit_x64.exe", test_dir), &exit_x64).unwrap();

    println!("✅ Generated Exit Code PE files saved to {}/", test_dir);
    println!("  - exit_x86.exe ({} bytes)", exit_x86.len());
    println!("  - exit_x64.exe ({} bytes)", exit_x64.len());
}

#[test]
fn test_different_exit_codes() {
    // 测试不同的退出代码
    let exit_codes = [0, 1, 42, 123, 255];

    for &code in &exit_codes {
        let pe_data = easy_exit_code(Architecture::X86_64, code).unwrap();

        // 验证基本 PE 结构
        assert_eq!(pe_data[0], 0x4D); // 'M'
        assert_eq!(pe_data[1], 0x5A); // 'Z'

        let pe_offset = u32::from_le_bytes([pe_data[60], pe_data[61], pe_data[62], pe_data[63]]) as usize;
        assert_eq!(&pe_data[pe_offset..pe_offset + 4], b"PE\0\0");

        println!("✅ Exit code {} PE file generated successfully ({} bytes)", code, pe_data.len());
    }
}

#[test]
fn test_unsupported_architecture_exit_code() {
    let result = easy_exit_code(Architecture::ARM, 42);
    assert!(result.is_err());

    println!("✅ Unsupported architecture correctly rejected for Exit Code");
}
