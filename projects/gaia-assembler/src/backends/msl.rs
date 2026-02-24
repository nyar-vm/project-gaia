//! MSL (Metal Shading Language) backend compiler

use crate::{config::GaiaConfig, program::GaiaModule, Backend, GeneratedFiles};
use gaia_types::{
    helpers::{AbiCompatible, ApiCompatible, Architecture, CompilationTarget},
    Result,
};
use std::collections::HashMap;

/// MSL Backend implementation
#[derive(Default)]
pub struct MslBackend {}

impl Backend for MslBackend {
    fn name(&self) -> &'static str {
        "MSL"
    }

    fn primary_target(&self) -> CompilationTarget {
        CompilationTarget { build: Architecture::Unknown, host: AbiCompatible::MSL, target: ApiCompatible::Metal }
    }

    fn match_score(&self, target: &CompilationTarget) -> f32 {
        if target.host == AbiCompatible::MSL {
            return 80.0;
        }
        0.0
    }

    fn generate(&self, _program: &GaiaModule, _config: &GaiaConfig) -> Result<GeneratedFiles> {
        #[cfg(feature = "msl")]
        {
            Ok(GeneratedFiles { files: HashMap::new(), diagnostics: vec![] })
        }
        #[cfg(not(feature = "msl"))]
        {
            Err(gaia_types::GaiaError::custom_error("MSL feature is not enabled"))
        }
    }
}
