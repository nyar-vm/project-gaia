//! ELF 文件生成器模块
//!
//! 此模块提供简单的 ELF 文件生成功能，用于创建基本的可执行文件。

use crate::writer::{create_hello_world_elf, ElfBuilder};
use gaia_types::{helpers::Architecture, GaiaError};

/// ELF 文件生成器 trait
pub trait ElfGenerator {
    /// 生成 ELF 文件
    fn generate(&self, arch: Architecture) -> Result<Vec<u8>, GaiaError>;
}

/// 简单的 Hello World ELF 生成器
#[derive(Debug, Copy, Clone)]
pub struct HelloWorldGenerator;

impl ElfGenerator for HelloWorldGenerator {
    fn generate(&self, arch: Architecture) -> Result<Vec<u8>, GaiaError> {
        match arch {
            Architecture::X86_64 => Ok(create_hello_world_elf().to_bytes()),
            _ => Err(GaiaError::unsupported_architecture(arch)),
        }
    }
}

/// 生成简单的 Hello World ELF 文件
pub fn easy_hello_world(arch: Architecture) -> Result<Vec<u8>, GaiaError> {
    let generator = HelloWorldGenerator;
    generator.generate(arch)
}

/// 生成简单的退出代码 ELF 文件
pub fn easy_exit_code(arch: Architecture, exit_code: u8) -> Result<Vec<u8>, GaiaError> {
    match arch {
        Architecture::X86_64 => {
            let mut builder = ElfBuilder::new();

            // x86-64 Linux 退出程序的机器码
            let exit_code_bytes = [
                0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00, // mov rax, 60 (sys_exit)
                0x48, 0xc7, 0xc7, exit_code, 0x00, 0x00, 0x00, // mov rdi, exit_code
                0x0f, 0x05, // syscall
            ];

            builder.add_code_segment(exit_code_bytes.to_vec());
            Ok(builder.build().to_bytes())
        }
        _ => Err(GaiaError::unsupported_architecture(arch)),
    }
}

/// 生成简单的控制台输出 ELF 文件
pub fn easy_console_log(arch: Architecture, message: &str) -> Result<Vec<u8>, GaiaError> {
    match arch {
        Architecture::X86_64 => {
            let mut builder = ElfBuilder::new();

            // 将消息添加到代码段末尾
            let message_bytes = message.as_bytes().to_vec();
            let message_len = message_bytes.len();

            // x86-64 Linux 输出程序的机器码
            let mut code = vec![
                // write syscall
                0x48,
                0xc7,
                0xc0,
                0x01,
                0x00,
                0x00,
                0x00, // mov rax, 1 (sys_write)
                0x48,
                0xc7,
                0xc7,
                0x01,
                0x00,
                0x00,
                0x00, // mov rdi, 1 (stdout)
                0x48,
                0xc7,
                0xc6,
                0x00,
                0x20,
                0x40,
                0x00, // mov rsi, 0x402000 (message address)
                0x48,
                0xc7,
                0xc2,
                (message_len & 0xff) as u8,
                ((message_len >> 8) & 0xff) as u8,
                0x00,
                0x00, // mov rdx, message_len
                0x0f,
                0x05, // syscall
                // exit syscall
                0x48,
                0xc7,
                0xc0,
                0x3c,
                0x00,
                0x00,
                0x00, // mov rax, 60 (sys_exit)
                0x48,
                0xc7,
                0xc7,
                0x00,
                0x00,
                0x00,
                0x00, // mov rdi, 0
                0x0f,
                0x05, // syscall
            ];

            // 将消息附加到代码后面
            code.extend(message_bytes);

            builder.add_code_segment(code);
            Ok(builder.build().to_bytes())
        }
        _ => Err(GaiaError::unsupported_architecture(arch)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello_world_generator() {
        let result = easy_hello_world(Architecture::X86_64);
        assert!(result.is_ok());

        let elf_data = result.unwrap();
        // 验证 ELF 魔数
        assert_eq!(&elf_data[0..4], b"\x7fELF");
    }

    #[test]
    fn test_exit_code_generator() {
        let result = easy_exit_code(Architecture::X86_64, 42);
        assert!(result.is_ok());

        let elf_data = result.unwrap();
        // 验证 ELF 魔数
        assert_eq!(&elf_data[0..4], b"\x7fELF");
    }

    #[test]
    fn test_console_log_generator() {
        let result = easy_console_log(Architecture::X86_64, "Hello, World!");
        assert!(result.is_ok());

        let elf_data = result.unwrap();
        // 验证 ELF 魔数
        assert_eq!(&elf_data[0..4], b"\x7fELF");
    }

    #[test]
    fn test_unsupported_architecture() {
        let result = easy_hello_world(Architecture::ARM32);
        assert!(result.is_err());
    }
}
