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
use sass_assembler::{
    instructions::{SassInstruction, SassReg},
    program::{SassKernel, SassProgram},
    SassWriter,
};
use std::collections::HashMap;

pub struct SassBackend {
    writer: SassWriter,
}

impl SassBackend {
    pub fn new() -> Self {
        Self { writer: SassWriter::new() }
    }
}

impl Backend for SassBackend {
    fn name(&self) -> &'static str {
        "NVIDIA SASS"
    }

    fn primary_target(&self) -> CompilationTarget {
        CompilationTarget { build: Architecture::NvSass, host: AbiCompatible::PTX, target: ApiCompatible::Unknown }
    }

    fn match_score(&self, target: &CompilationTarget) -> f32 {
        if target.build == Architecture::NvSass {
            return 100.0;
        }
        0.0
    }

    fn generate(&self, program: &GaiaModule, _config: &GaiaConfig) -> Result<GeneratedFiles> {
        let mut files = HashMap::new();

        // 转换 GaiaModule -> SassProgram
        let sass_program = convert_gaia_to_sass(program)?;

        // 写入二进制
        let binary = self.writer.write(&sass_program)?;
        files.insert(format!("{}.cubin", program.name), binary);

        Ok(GeneratedFiles { files, diagnostics: vec![] })
    }
}

fn convert_gaia_to_sass(module: &GaiaModule) -> Result<SassProgram> {
    let mut sass_program = SassProgram::new(module.name.clone());

    for function in &module.functions {
        let mut instructions = Vec::new();
        for block in &function.blocks {
            for inst in &block.instructions {
                match inst {
                    GaiaInstruction::Domain(DomainInstruction::Neural(node)) => {
                        // 映射神经网络算子到 SASS 指令
                        if let NeuralNode::Convolution { .. } = node {
                            instructions.push(SassInstruction::Imma {
                                dst: SassReg::R(0),
                                src0: SassReg::R(1),
                                src1: SassReg::R(2),
                                src2: SassReg::R(0),
                            });
                        }
                    }
                    _ => {
                        instructions.push(SassInstruction::Nop);
                    }
                }
            }

            match &block.terminator {
                GaiaTerminator::Return => {
                    instructions.push(SassInstruction::Exit);
                }
                _ => {}
            }
        }

        sass_program.kernels.push(SassKernel { name: function.name.clone(), instructions });
    }

    Ok(sass_program)
}
