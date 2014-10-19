use std::fmt::{Display, Formatter};
pub use url::Url;

/// 支持的处理器架构类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Architecture {
    /// x86 32位架构
    X86,
    /// x86-64 64位架构
    X86_64,
    /// ARM 32位架构
    ARM,
    /// ARM64/AArch64 64位架构
    ARM64,
    /// RISC-V 32位架构
    RISCV32,
    /// RISC-V 64位架构
    RISCV64,
    /// MIPS 32位架构
    MIPS,
    /// MIPS 64位架构
    MIPS64,
    /// 其他自定义架构，包含架构名称
    Other(String),
}

impl Display for Architecture {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Architecture::X86 => write!(f, "x86"),
            Architecture::X86_64 => write!(f, "x64"),
            Architecture::ARM => write!(f, "arm"),
            Architecture::ARM64 => write!(f, "arm64"),
            Architecture::RISCV32 => write!(f, "riscv32"),
            Architecture::RISCV64 => write!(f, "riscv64"),
            Architecture::MIPS => write!(f, "mips"),
            Architecture::MIPS64 => write!(f, "mips64"),
            Architecture::Other(name) => write!(f, "{}", name),
        }
    }
}
