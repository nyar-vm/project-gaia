#![warn(missing_docs)]

pub mod instructions;
pub mod program;

use crate::{instructions::SassInstruction, program::SassProgram};
use gaia_types::Result;

/// SASS 汇编器/写入器
pub struct SassWriter {}

impl SassWriter {
    pub fn new() -> Self {
        Self {}
    }

    pub fn write(&self, program: &SassProgram) -> Result<Vec<u8>> {
        let mut code = String::new();
        for kernel in &program.kernels {
            code.push_str(&format!(".section .text.{}\n.global {}\n{}:\n", kernel.name, kernel.name, kernel.name));
            for inst in &kernel.instructions {
                code.push_str("    ");
                code.push_str(&self.format_instruction(inst));
                code.push('\n');
            }
        }

        // 模拟生成二进制 (ELF Magic + 汇编内容)
        let mut binary = Vec::from("\x7fELF".as_bytes());
        binary.extend_from_slice(code.as_bytes());
        Ok(binary)
    }

    fn format_instruction(&self, inst: &SassInstruction) -> String {
        match inst {
            SassInstruction::FAdd { dst, src0, src1 } => format!("FADD {}, {}, {}", dst, src0, src1),
            SassInstruction::FMul { dst, src0, src1 } => format!("FMUL {}, {}, {}", dst, src0, src1),
            SassInstruction::Imma { dst, src0, src1, src2 } => {
                format!("IMMA.16.16.16.F32 {}, {}, {}, {}", dst, src0, src1, src2)
            }
            SassInstruction::Ldg { dst, addr } => format!("LDG.E {}, [{}]", dst, addr),
            SassInstruction::Stg { addr, src } => format!("STG.E [{}], {}", addr, src),
            SassInstruction::Exit => "EXIT".to_string(),
            SassInstruction::Nop => "NOP".to_string(),
        }
    }
}
