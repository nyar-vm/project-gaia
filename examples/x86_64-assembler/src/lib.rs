#![deny(missing_debug_implementations, missing_copy_implementations)]
#![warn(missing_docs, rustdoc::missing_crate_level_docs)]
#![doc = include_str!("readme.md")]
#![doc(html_logo_url = "https://raw.githubusercontent.com/oovm/shape-rs/dev/projects/images/Trapezohedron.svg")]
#![doc(html_favicon_url = "https://raw.githubusercontent.com/oovm/shape-rs/dev/projects/images/Trapezohedron.svg")]

use gaia_types::{helpers::Architecture, GaiaError, Result};

pub mod builder;
pub mod decoder;
pub mod encoder;
pub mod instruction;

use instruction::Instruction;

/// x86_64汇编器主结构体
#[derive(Debug, Clone)]
pub struct X86_64Assembler {
    architecture: Architecture,
}

impl X86_64Assembler {
    /// 创建新的x86_64汇编器
    pub fn new(architecture: Architecture) -> Result<Self> {
        match architecture {
            Architecture::X86 | Architecture::X86_64 => Ok(Self { architecture }),
            _ => Err(GaiaError::unsupported_architecture(architecture)),
        }
    }

    /// 编码指令为字节码
    pub fn encode(&self, instruction: &Instruction) -> Result<Vec<u8>> {
        let encoder = encoder::InstructionEncoder::new(self.architecture.clone());
        encoder.encode(instruction)
    }

    /// 解码字节码为指令序列
    pub fn decode(&self, bytes: &[u8]) -> Result<Vec<Instruction>> {
        let decoder = decoder::InstructionDecoder::new(self.architecture.clone());
        decoder.decode(bytes)
    }

    /// 获取当前汇编器的架构
    pub fn architecture(&self) -> Architecture {
        self.architecture.clone()
    }

    /// 设置汇编器的架构
    pub fn set_architecture(&mut self, architecture: Architecture) -> Result<()> {
        match architecture {
            Architecture::X86 | Architecture::X86_64 => {
                self.architecture = architecture;
                Ok(())
            }
            _ => Err(GaiaError::unsupported_architecture(architecture)),
        }
    }
}
