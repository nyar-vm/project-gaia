//! LLVM IR backend compiler

use crate::{config::GaiaConfig, program::GaiaModule, Backend, GeneratedFiles};
use gaia_types::{
    helpers::{AbiCompatible, ApiCompatible, Architecture, CompilationTarget},
    Result,
};
use std::collections::HashMap;

/// LLVM Backend implementation
#[derive(Default)]
pub struct LlvmBackend {}

impl Backend for LlvmBackend {
    fn name(&self) -> &'static str {
        "LLVM"
    }

    fn primary_target(&self) -> CompilationTarget {
        CompilationTarget { build: Architecture::Unknown, host: AbiCompatible::LlvmIr, target: ApiCompatible::Unknown }
    }

    fn match_score(&self, target: &CompilationTarget) -> f32 {
        if target.host == AbiCompatible::LlvmIr {
            return 80.0;
        }
        0.0
    }

    fn generate(&self, _program: &GaiaModule, _config: &GaiaConfig) -> Result<GeneratedFiles> {
        #[cfg(feature = "llvm")]
        {
            Ok(GeneratedFiles { files: HashMap::new(), diagnostics: vec![] })
        }
        #[cfg(not(feature = "llvm"))]
        {
            Err(gaia_types::GaiaError::custom_error("LLVM feature is not enabled"))
        }
    }
}
