#![warn(missing_docs)]

pub mod instructions;
pub mod program;

use crate::{instructions::GcnInstruction, program::GcnProgram};
use gaia_types::Result;

/// GCN 汇编器/写入器
pub struct GcnWriter {}

impl GcnWriter {
    pub fn new() -> Self {
        Self {}
    }

    pub fn write(&self, program: &GcnProgram) -> Result<Vec<u8>> {
        let mut code = String::new();
        for kernel in &program.kernels {
            code.push_str(&format!(".text\n.globl {}\n{}:\n", kernel.name, kernel.name));
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

    fn format_instruction(&self, inst: &GcnInstruction) -> String {
        match inst {
            GcnInstruction::VAddF32 { dst, src0, src1 } => format!("v_add_f32 {}, {}, {}", dst, src0, src1),
            GcnInstruction::VMulF32 { dst, src0, src1 } => format!("v_mul_f32 {}, {}, {}", dst, src0, src1),
            GcnInstruction::VDot2F32F16 { dst, src0, src1 } => format!("v_dot2_f32_f16 {}, {}, {}", dst, src0, src1),
            GcnInstruction::GlobalLoadDword { dst, addr, offset } => {
                format!("global_load_dword {}, {}, offset:{}", dst, addr, offset)
            }
            GcnInstruction::GlobalStoreDword { addr, src, offset } => {
                format!("global_store_dword {}, {}, offset:{}", addr, src, offset)
            }
            GcnInstruction::SEndPgm => "s_endpgm".to_string(),
            GcnInstruction::SNop(n) => format!("s_nop {}", n),
        }
    }
}
