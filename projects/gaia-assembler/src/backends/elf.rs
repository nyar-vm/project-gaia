//! ELF (Executable and Linkable Format) backend compiler

use crate::{config::GaiaConfig, program::GaiaModule, Backend, GeneratedFiles};
use gaia_types::{
    helpers::{AbiCompatible, ApiCompatible, Architecture, CompilationTarget},
    Result,
};
use std::collections::HashMap;

/// ELF Backend implementation
#[derive(Default)]
pub struct ElfBackend {}

impl Backend for ElfBackend {
    fn name(&self) -> &'static str {
        "ELF"
    }

    fn primary_target(&self) -> CompilationTarget {
        CompilationTarget { build: Architecture::X86_64, host: AbiCompatible::ELF, target: ApiCompatible::Linux }
    }

    fn match_score(&self, target: &CompilationTarget) -> f32 {
        if target.host == AbiCompatible::ELF {
            return 80.0;
        }
        0.0
    }

    fn generate(&self, _program: &GaiaModule, _config: &GaiaConfig) -> Result<GeneratedFiles> {
        #[cfg(feature = "elf")]
        {
            // TODO: Implement conversion using elf-assembler
            Ok(GeneratedFiles { files: HashMap::new(), diagnostics: vec![] })
        }
        #[cfg(not(feature = "elf"))]
        {
            Err(gaia_types::GaiaError::custom_error("ELF feature is not enabled"))
        }
    }
}
