//! Python bytecode backend compiler

use crate::{config::GaiaConfig, program::GaiaModule, Backend, GeneratedFiles};
use gaia_types::{
    helpers::{AbiCompatible, ApiCompatible, Architecture, CompilationTarget},
    Result,
};
use std::collections::HashMap;

/// Python Backend implementation
#[derive(Default)]
pub struct PythonBackend {}

impl Backend for PythonBackend {
    fn name(&self) -> &'static str {
        "Python"
    }

    fn primary_target(&self) -> CompilationTarget {
        CompilationTarget { build: Architecture::Unknown, host: AbiCompatible::PythonBytecode, target: ApiCompatible::Unknown }
    }

    fn match_score(&self, target: &CompilationTarget) -> f32 {
        if target.host == AbiCompatible::PythonBytecode {
            return 90.0;
        }
        0.0
    }

    fn generate(&self, _program: &GaiaModule, _config: &GaiaConfig) -> Result<GeneratedFiles> {
        #[cfg(feature = "python")]
        {
            Ok(GeneratedFiles { files: HashMap::new(), diagnostics: vec![] })
        }
        #[cfg(not(feature = "python"))]
        {
            Err(gaia_types::GaiaError::custom_error("Python feature is not enabled"))
        }
    }
}
