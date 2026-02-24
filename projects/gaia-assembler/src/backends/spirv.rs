//! SPIR-V (Standard Portable Intermediate Representation - V) backend compiler

use crate::{config::GaiaConfig, program::GaiaModule, Backend, GeneratedFiles};
use gaia_types::{
    helpers::{AbiCompatible, ApiCompatible, Architecture, CompilationTarget},
    Result,
};
use std::collections::HashMap;

/// SPIR-V Backend implementation
#[derive(Default)]
pub struct SpirvBackend {}

impl Backend for SpirvBackend {
    fn name(&self) -> &'static str {
        "SPIR-V"
    }

    fn primary_target(&self) -> CompilationTarget {
        CompilationTarget { build: Architecture::Unknown, host: AbiCompatible::SPIRV, target: ApiCompatible::Unknown }
    }

    fn match_score(&self, target: &CompilationTarget) -> f32 {
        if target.host == AbiCompatible::SPIRV {
            return 95.0;
        }
        0.0
    }

    fn generate(&self, _program: &GaiaModule, _config: &GaiaConfig) -> Result<GeneratedFiles> {
        #[cfg(feature = "spirv")]
        {
            Ok(GeneratedFiles { files: HashMap::new(), diagnostics: vec![] })
        }
        #[cfg(not(feature = "spirv"))]
        {
            Err(gaia_types::GaiaError::custom_error("SPIR-V feature is not enabled"))
        }
    }
}
