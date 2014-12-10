//! Backend compiler module
//!
//! Contains compiler implementations for various target platforms

pub mod gcn;
pub mod jvm;
pub mod msil;
pub mod pe;
pub mod sass;
pub mod wasi;
pub mod x86;

// Re-export backend structs
#[cfg(feature = "clr")]
pub use self::msil::ClrBackend;
pub use self::{gcn::GcnBackend, jvm::JvmBackend, pe::PeBackend, sass::SassBackend, wasi::WasiBackend, x86::X86Backend};

use crate::{config::GaiaConfig, program::GaiaModule};
use gaia_types::{helpers::CompilationTarget, Result};
use std::collections::HashMap;

/// Backend compiler trait
pub trait Backend {
    /// Get backend name
    fn name(&self) -> &'static str;

    /// 获取此后端支持的主要编译目标
    fn primary_target(&self) -> CompilationTarget;

    /// 计算与给定编译目标的匹配度 (0-100)
    /// 0 表示不支持
    fn match_score(&self, target: &CompilationTarget) -> f32;

    /// Compile Gaia program to target platform
    fn generate(&self, program: &GaiaModule, config: &GaiaConfig) -> Result<GeneratedFiles>;
}

pub struct GeneratedFiles {
    pub files: HashMap<String, Vec<u8>>,
    pub diagnostics: Vec<gaia_types::GaiaError>,
}
