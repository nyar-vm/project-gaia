//! Backend compiler module
//!
//! Contains compiler implementations for various target platforms

#[cfg(feature = "gcn")]
pub mod gcn;
#[cfg(feature = "jvm")]
pub mod jvm;
#[cfg(feature = "clr")]
pub mod msil;
#[cfg(feature = "pe")]
pub mod pe;
#[cfg(feature = "sass")]
pub mod sass;
#[cfg(feature = "wasi")]
pub mod wasi;
#[cfg(feature = "x86_64")]
pub mod x86;
#[cfg(feature = "elf")]
pub mod elf;
#[cfg(feature = "macho")]
pub mod macho;
#[cfg(feature = "lua")]
pub mod lua;
#[cfg(feature = "llvm")]
pub mod llvm;
#[cfg(feature = "msl")]
pub mod msl;
#[cfg(feature = "spirv")]
pub mod spirv;
#[cfg(feature = "python")]
pub mod python;

// Re-export backend structs
#[cfg(feature = "clr")]
pub use self::msil::ClrBackend;
#[cfg(feature = "wasi")]
pub use self::wasi::WasiBackend;
#[cfg(feature = "jvm")]
pub use self::jvm::JvmBackend;
#[cfg(feature = "x86_64")]
pub use self::x86::X86Backend;
#[cfg(feature = "pe")]
pub use self::pe::PeBackend;
#[cfg(feature = "gcn")]
pub use self::gcn::GcnBackend;
#[cfg(feature = "sass")]
pub use self::sass::SassBackend;
#[cfg(feature = "elf")]
pub use self::elf::ElfBackend;
#[cfg(feature = "macho")]
pub use self::macho::MachoBackend;
#[cfg(feature = "lua")]
pub use self::lua::LuaBackend;
#[cfg(feature = "llvm")]
pub use self::llvm::LlvmBackend;
#[cfg(feature = "msl")]
pub use self::msl::MslBackend;
#[cfg(feature = "spirv")]
pub use self::spirv::SpirvBackend;
#[cfg(feature = "python")]
pub use self::python::PythonBackend;

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
