use crate::{
    backends::{Backend, GeneratedFiles},
    config::GaiaConfig,
    instruction::{DomainInstruction, GaiaInstruction},
    program::{GaiaModule, GaiaTerminator},
};
use gaia_types::{
    helpers::{AbiCompatible, ApiCompatible, Architecture, CompilationTarget},
    neural::NeuralNode,
    Result,
};
use gcn_assembler::{
    instructions::{GcnInstruction, GcnReg},
    program::{GcnKernel, GcnProgram},
    GcnWriter,
};
use std::collections::HashMap;

pub struct GcnBackend {
    writer: GcnWriter,
}

impl GcnBackend {
    pub fn new() -> Self {
        Self { writer: GcnWriter::new() }
    }
}

impl Backend for GcnBackend {
    fn name(&self) -> &'static str {
        "AMD GCN"
    }

    fn primary_target(&self) -> CompilationTarget {
        CompilationTarget {
            build: Architecture::RISCV64, // Placeholder
            host: AbiCompatible::GCN,
            target: ApiCompatible::Unknown,
        }
    }

    fn match_score(&self, target: &CompilationTarget) -> f32 {
        if target.build == Architecture::RISCV64 {
            return 50.0;
        }
        0.0
    }

    fn generate(&self, program: &GaiaModule, _config: &GaiaConfig) -> Result<GeneratedFiles> {
        let mut files = HashMap::new();

        // 转换 GaiaModule -> GcnProgram
        let gcn_program = convert_gaia_to_gcn(program)?;

        // 写入二进制
        let binary = self.writer.write(&gcn_program)?;
        files.insert(format!("{}.hsaco", program.name), binary);

        Ok(GeneratedFiles { files, diagnostics: vec![] })
    }
}

fn convert_gaia_to_gcn(module: &GaiaModule) -> Result<GcnProgram> {
    let mut gcn_program = GcnProgram::new(module.name.clone());

    for function in &module.functions {
        let mut instructions = Vec::new();
        for block in &function.blocks {
            for inst in &block.instructions {
                match inst {
                    GaiaInstruction::Domain(DomainInstruction::Neural(node)) => {
                        // 映射神经网络算子到 GCN 指令
                        if let NeuralNode::Convolution { .. } = node {
                            instructions.push(GcnInstruction::VDot2F32F16 {
                                dst: GcnReg::VGPR(0),
                                src0: GcnReg::VGPR(1),
                                src1: GcnReg::VGPR(2),
                            });
                        }
                    }
                    _ => {
                        instructions.push(GcnInstruction::SNop(0));
                    }
                }
            }

            match &block.terminator {
                GaiaTerminator::Return => {
                    instructions.push(GcnInstruction::SEndPgm);
                }
                _ => {}
            }
        }

        gcn_program.kernels.push(GcnKernel { name: function.name.clone(), instructions, args: Vec::new() });
    }

    Ok(gcn_program)
}
