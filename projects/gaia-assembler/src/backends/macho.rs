//! Mach-O (Mach Object) backend compiler

use crate::{config::GaiaConfig, program::GaiaModule, Backend, GeneratedFiles};
use gaia_types::{
    helpers::{AbiCompatible, ApiCompatible, Architecture, CompilationTarget},
    Result,
};
use std::collections::HashMap;

/// Mach-O Backend implementation
#[derive(Default)]
pub struct MachoBackend {}

impl Backend for MachoBackend {
    fn name(&self) -> &'static str {
        "Mach-O"
    }

    fn primary_target(&self) -> CompilationTarget {
        CompilationTarget { build: Architecture::X86_64, host: AbiCompatible::MachO, target: ApiCompatible::Apple }
    }

    fn match_score(&self, target: &CompilationTarget) -> f32 {
        if target.host == AbiCompatible::MachO {
            return 80.0;
        }
        0.0
    }

    fn generate(&self, _program: &GaiaModule, _config: &GaiaConfig) -> Result<GeneratedFiles> {
        #[cfg(feature = "macho")]
        {
            Ok(GeneratedFiles { files: HashMap::new(), diagnostics: vec![] })
        }
        #[cfg(not(feature = "macho"))]
        {
            Err(gaia_types::GaiaError::custom_error("Mach-O feature is not enabled"))
        }
    }
}
