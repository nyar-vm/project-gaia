use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// 支持的处理器架构类型
///
/// 表示底层运行时架构，包括物理处理器架构和虚拟机架构。
/// 注意：不能是 unknown，必须明确指定架构类型。
///
/// # 支持类型
///
/// ## 物理架构
/// - x86系列：X86, X86_64
/// - ARM系列：ARM32, ARM64
/// - RISC-V系列：RISCV32, RISCV64
/// - MIPS系列：MIPS32, MIPS64
/// - WebAssembly系列：WASM32, WASM64
///
/// ## 虚拟机架构
/// - JVM：Java虚拟机
/// - CLR：.NET公共语言运行时
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Architecture {
    /// 未知架构, 无法生成目标代码
    Unknown,
    /// x86 32位架构
    X86,
    /// x86-64 64位架构
    X86_64,
    /// ARM 32位架构
    ARM32,
    /// ARM64/AArch64 64位架构
    ARM64,
    /// RISC-V 32位架构
    RISCV32,
    /// RISC-V 64位架构
    RISCV64,
    /// MIPS 32位架构
    MIPS32,
    /// MIPS 64位架构
    MIPS64,
    /// WebAssembly 32位架构
    WASM32,
    /// WebAssembly 64位架构
    WASM64,
    /// Java 虚拟机。
    JVM,
    /// 公共语言运行时。
    CLR,
    /// NVIDIA GPU SASS 架构
    NvSass,
    /// 其他架构类型
    Other(String),
}

impl Display for Architecture {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Architecture::Unknown => f.write_str("unknown"),
            Architecture::X86 => f.write_str("x86"),
            Architecture::X86_64 => f.write_str("x64"),
            Architecture::ARM32 => f.write_str("arm"),
            Architecture::ARM64 => f.write_str("arm64"),
            Architecture::RISCV32 => f.write_str("riscv32"),
            Architecture::RISCV64 => f.write_str("riscv64"),
            Architecture::MIPS32 => f.write_str("mips"),
            Architecture::MIPS64 => f.write_str("mips64"),
            Architecture::WASM32 => f.write_str("wasm32"),
            Architecture::WASM64 => f.write_str("wasm64"),
            Architecture::JVM => f.write_str("jvm"),
            Architecture::CLR => f.write_str("clr"),
            Architecture::NvSass => f.write_str("nv_sass"),
            Architecture::Other(name) => f.write_str(name),
        }
    }
}

impl std::str::FromStr for Architecture {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "unknown" => Ok(Architecture::Unknown),
            "x86" => Ok(Architecture::X86),
            "x64" | "x86_64" => Ok(Architecture::X86_64),
            "arm" | "arm32" => Ok(Architecture::ARM32),
            "arm64" => Ok(Architecture::ARM64),
            "riscv32" => Ok(Architecture::RISCV32),
            "riscv64" => Ok(Architecture::RISCV64),
            "mips" | "mips32" => Ok(Architecture::MIPS32),
            "mips64" => Ok(Architecture::MIPS64),
            "wasm32" => Ok(Architecture::WASM32),
            "wasm64" => Ok(Architecture::WASM64),
            "jvm" => Ok(Architecture::JVM),
            "clr" => Ok(Architecture::CLR),
            "nv_sass" | "sass" => Ok(Architecture::NvSass),
            name => Ok(Architecture::Other(name.to_string())),
        }
    }
}
