//! Backend compiler module
//!
//! Contains compiler implementations for various target platforms

pub mod jvm;
pub mod msil;
pub mod pe;
pub mod wasi;

// Re-export backend structs
pub use self::{jvm::JvmBackend, msil::ClrBackend, pe::PeBackend, wasi::WasiBackend};

use crate::config::{GaiaConfig, GaiaSettings};
use gaia_types::{
    helpers::{AbiCompatible, ApiCompatible, Architecture, CompilationTarget},
    GaiaError, Result,
};
use std::collections::HashMap;
use crate::program::GaiaProgram;


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
    fn generate(&self, program: &GaiaProgram, config: &GaiaConfig) -> Result<GeneratedFiles>;
}

pub struct GeneratedFiles {
    pub files: HashMap<String, Vec<u8>>,
    pub diagnostics: Vec<GaiaError>,
}
