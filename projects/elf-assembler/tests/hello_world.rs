//! Hello World ELF 文件生成器
//!
//! 此模块提供生成简单控制台输出 ELF 文件的功能。
//! 使用新的人体工程学友好的汇编器API。

use elf_assembler::writer::{create_hello_world_elf, ElfBuilder};

/// 生成 Hello World ELF 文件 - 使用新的汇编器API
pub fn generate_hello_world_elf() -> Vec<u8> {
    // 使用预定义的 Hello World ELF 生成器
    let elf_file = create_hello_world_elf();
    elf_file.to_bytes()
}

/// 生成简单的退出程序ELF文件
pub fn generate_exit_elf(exit_code: i32) -> Vec<u8> {
    let mut builder = ElfBuilder::new();

    // 简单的退出程序机器码 (x86-64 Linux)
    let exit_code_bytes = (exit_code as u32).to_le_bytes();
    let exit_code = vec![
        // mov rax, 60 (sys_exit)
        0x48,
        0xc7,
        0xc0,
        0x3c,
        0x00,
        0x00,
        0x00,
        // mov rdi, exit_code
        0x48,
        0xc7,
        0xc7,
        exit_code_bytes[0],
        exit_code_bytes[1],
        exit_code_bytes[2],
        exit_code_bytes[3],
        // syscall
        0x0f,
        0x05,
    ];

    builder.set_entry_point(0x401000).add_code_segment(exit_code).to_bytes()
}

/// 生成简单的输出程序ELF文件
pub fn generate_output_elf(message: &str) -> Vec<u8> {
    let mut builder = ElfBuilder::new();

    // 创建包含消息的机器码
    let mut code = vec![
        // mov rax, 1 (sys_write)
        0x48,
        0xc7,
        0xc0,
        0x01,
        0x00,
        0x00,
        0x00,
        // mov rdi, 1 (stdout)
        0x48,
        0xc7,
        0xc7,
        0x01,
        0x00,
        0x00,
        0x00,
        // mov rsi, message_addr (消息地址)
        0x48,
        0xc7,
        0xc6,
        0x20,
        0x10,
        0x40,
        0x00,
        // mov rdx, message_length
        0x48,
        0xc7,
        0xc2,
        (message.len() as u8),
        0x00,
        0x00,
        0x00,
        // syscall
        0x0f,
        0x05,
        // mov rax, 60 (sys_exit)
        0x48,
        0xc7,
        0xc0,
        0x3c,
        0x00,
        0x00,
        0x00,
        // mov rdi, 0 (exit code)
        0x48,
        0xc7,
        0xc7,
        0x00,
        0x00,
        0x00,
        0x00,
        // syscall
        0x0f,
        0x05,
    ];

    // 添加消息字符串
    code.extend_from_slice(message.as_bytes());

    builder.set_entry_point(0x401000).add_code_segment(code).to_bytes()
}

/// 传统的手动构建方式 - 保留作为参考
#[allow(dead_code)]
pub fn generate_hello_world_elf_manual() -> Vec<u8> {
    use elf_assembler::types::{ElfFile, ElfHeader64, ElfMachine, ElfType, ProgramHeader64};

    // 创建 ELF 头
    let mut elf_header = ElfHeader64::new();
    elf_header.e_type = ElfType::Exec as u16;
    elf_header.e_machine = ElfMachine::X86_64 as u16;
    elf_header.e_entry = 0x401000;
    elf_header.e_phoff = 64; // 程序头表偏移
    elf_header.e_phentsize = 56; // 程序头表项大小
    elf_header.e_phnum = 1; // 程序头表项数量

    // 创建程序头
    let program_header = ProgramHeader64::new_load_segment(0x1000, 0x401000, 64);

    // Hello World 的机器码
    let code = vec![
        // mov rax, 1 (sys_write)
        0x48, 0xc7, 0xc0, 0x01, 0x00, 0x00, 0x00, // mov rdi, 1 (stdout)
        0x48, 0xc7, 0xc7, 0x01, 0x00, 0x00, 0x00, // mov rsi, hello_msg
        0x48, 0xc7, 0xc6, 0x20, 0x10, 0x40, 0x00, // mov rdx, 13 (message length)
        0x48, 0xc7, 0xc2, 0x0d, 0x00, 0x00, 0x00, // syscall
        0x0f, 0x05, // mov rax, 60 (sys_exit)
        0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00, // mov rdi, 0 (exit code)
        0x48, 0xc7, 0xc7, 0x00, 0x00, 0x00, 0x00, // syscall
        0x0f, 0x05, // "Hello, World!" string
        b'H', b'e', b'l', b'l', b'o', b',', b' ', b'W', b'o', b'r', b'l', b'd', b'!', b'\n',
    ];

    // 创建 ELF 文件
    let mut elf_file = ElfFile::new();
    elf_file.header = elf_header;
    elf_file.add_program_header(program_header);
    elf_file.data = code;

    elf_file.to_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_hello_world_elf() {
        let elf_bytes = generate_hello_world_elf();

        // 检查 ELF 魔数
        assert_eq!(&elf_bytes[0..4], &[0x7f, b'E', b'L', b'F']);

        // 检查是否为 64 位
        assert_eq!(elf_bytes[4], 2);

        // 检查是否为小端序
        assert_eq!(elf_bytes[5], 1);

        // 检查文件大小合理
        assert!(elf_bytes.len() > 100);
    }

    #[test]
    fn test_generate_exit_elf() {
        let elf_bytes = generate_exit_elf(42);

        // 检查 ELF 魔数
        assert_eq!(&elf_bytes[0..4], &[0x7f, b'E', b'L', b'F']);

        // 检查文件大小合理
        assert!(elf_bytes.len() > 50);
    }

    #[test]
    fn test_generate_output_elf() {
        let message = "Test message";
        let elf_bytes = generate_output_elf(message);

        // 检查 ELF 魔数
        assert_eq!(&elf_bytes[0..4], &[0x7f, b'E', b'L', b'F']);

        // 检查文件包含消息
        let elf_string = String::from_utf8_lossy(&elf_bytes);
        assert!(elf_string.contains(message));
    }

    #[test]
    fn test_manual_elf_generation() {
        let elf_bytes = generate_hello_world_elf_manual();

        // 检查 ELF 魔数
        assert_eq!(&elf_bytes[0..4], &[0x7f, b'E', b'L', b'F']);

        // 检查是否为可执行文件
        assert_eq!(elf_bytes[16], 2); // ET_EXEC

        // 检查机器架构
        assert_eq!(elf_bytes[18], 0x3e); // EM_X86_64
    }
}
