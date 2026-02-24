//! Lua backend compiler

use crate::{config::GaiaConfig, program::GaiaModule, Backend, GeneratedFiles};
use gaia_types::{
    helpers::{AbiCompatible, ApiCompatible, Architecture, CompilationTarget},
    Result,
};
use std::collections::HashMap;

/// Lua Backend implementation
#[derive(Default)]
pub struct LuaBackend {}

impl Backend for LuaBackend {
    fn name(&self) -> &'static str {
        "Lua"
    }

    fn primary_target(&self) -> CompilationTarget {
        CompilationTarget { build: Architecture::Unknown, host: AbiCompatible::LuaBytecode, target: ApiCompatible::Unknown }
    }

    fn match_score(&self, target: &CompilationTarget) -> f32 {
        if target.host == AbiCompatible::LuaBytecode {
            return 80.0;
        }
        0.0
    }

    fn generate(&self, _program: &GaiaModule, _config: &GaiaConfig) -> Result<GeneratedFiles> {
        #[cfg(feature = "lua")]
        {
            Ok(GeneratedFiles { files: HashMap::new(), diagnostics: vec![] })
        }
        #[cfg(not(feature = "lua"))]
        {
            Err(gaia_types::GaiaError::custom_error("Lua feature is not enabled"))
        }
    }
}
