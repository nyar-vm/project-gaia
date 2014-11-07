//! Hello World ELF 文件生成测试

use elf_assembler::generator::easy_console_log;
use gaia_types::helpers::Architecture;
use std::fs;

#[test]
fn test_hello_world_x86() {
    let elf_data = easy_console_log(Architecture::X86, "Hello World!").unwrap();

    // 验证 ELF 魔数
    assert_eq!(&elf_data[0..4], b"\x7fELF");

    // 验证架构 (x86)
    assert_eq!(elf_data[4], 1); // EI_CLASS = ELFCLASS32
    let machine_type = u16::from_le_bytes([elf_data[18], elf_data[19]]);
    assert_eq!(machine_type, 3); // EM_386

    println!("✅ x86 Hello World ELF file generated successfully ({} bytes)", elf_data.len());
}

#[test]
fn test_hello_world_x64() {
    let elf_data = easy_console_log(Architecture::X86_64, "Hello World!").unwrap();

    // 验证 ELF 魔数
    assert_eq!(&elf_data[0..4], b"\x7fELF");

    // 验证架构 (x64)
    assert_eq!(elf_data[4], 2); // EI_CLASS = ELFCLASS64
    let machine_type = u16::from_le_bytes([elf_data[18], elf_data[19]]);
    assert_eq!(machine_type, 62); // EM_X86_64

    println!("✅ x64 Hello World ELF file generated successfully ({} bytes)", elf_data.len());
}

#[test]
fn test_save_hello_world_files() {
    // 生成并保存 Hello World ELF 文件用于测试
    let x86_elf = easy_console_log(Architecture::X86, "Hello from x86!").unwrap();
    let x64_elf = easy_console_log(Architecture::X86_64, "Hello from x64!").unwrap();

    // 保存到测试目录
    let test_dir = "tests/generated";
    fs::create_dir_all(test_dir).ok();

    fs::write(format!("{}/hello_x86", test_dir), &x86_elf).unwrap();
    fs::write(format!("{}/hello_x64", test_dir), &x64_elf).unwrap();

    println!("✅ Generated Hello World ELF files saved to {}/", test_dir);
    println!("  - hello_x86 ({} bytes)", x86_elf.len());
    println!("  - hello_x64 ({} bytes)", x64_elf.len());
}

#[test]
fn test_unsupported_architecture_hello_world() {
    let result = easy_console_log(Architecture::ARM32, "Hello");
    assert!(result.is_err());

    println!("✅ Unsupported architecture correctly rejected for Hello World");
}
