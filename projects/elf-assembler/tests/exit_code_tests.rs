//! Exit Code ELF 文件生成测试

use elf_assembler::generator::easy_exit_code;
use gaia_types::helpers::Architecture;
use std::fs;

#[test]
fn test_exit_code_x64() {
    let elf_data = easy_exit_code(Architecture::X86_64, 42).unwrap();

    // 验证 ELF 魔数
    assert_eq!(&elf_data[0..4], b"\x7fELF");

    // 验证 64 位
    assert_eq!(elf_data[4], 2); // EI_CLASS = ELFCLASS64

    // 验证小端序
    assert_eq!(elf_data[5], 1); // EI_DATA = ELFDATA2LSB

    // 验证机器类型 (x86-64)
    let machine_type = u16::from_le_bytes([elf_data[18], elf_data[19]]);
    assert_eq!(machine_type, 0x3E); // EM_X86_64

    println!("✅ x64 Exit Code ELF file generated successfully ({} bytes)", elf_data.len());
}

#[test]
fn test_save_exit_code_files() {
    // 生成退出代码文件
    let exit_x64 = easy_exit_code(Architecture::X86_64, 123).unwrap();

    // 保存到测试目录
    let test_dir = "tests/generated";
    fs::create_dir_all(test_dir).ok();

    fs::write(format!("{}/exit_x64", test_dir), &exit_x64).unwrap();

    println!("✅ Generated Exit Code ELF files saved to {}/", test_dir);
    println!("  - exit_x64 ({} bytes)", exit_x64.len());
}

#[test]
fn test_different_exit_codes() {
    // 测试不同的退出代码
    let exit_codes = [0, 1, 42, 123, 255];

    for &code in &exit_codes {
        let elf_data = easy_exit_code(Architecture::X86_64, code).unwrap();

        // 验证基本 ELF 结构
        assert_eq!(&elf_data[0..4], b"\x7fELF");
        assert_eq!(elf_data[4], 2); // 64-bit
        assert_eq!(elf_data[5], 1); // little-endian

        let machine_type = u16::from_le_bytes([elf_data[18], elf_data[19]]);
        assert_eq!(machine_type, 0x3E); // x86-64

        println!("✅ Exit code {} ELF file generated successfully ({} bytes)", code, elf_data.len());
    }
}

#[test]
fn test_unsupported_architecture_exit_code() {
    let result = easy_exit_code(Architecture::ARM32, 42);
    assert!(result.is_err());

    println!("✅ Unsupported architecture correctly rejected for Exit Code");
}
