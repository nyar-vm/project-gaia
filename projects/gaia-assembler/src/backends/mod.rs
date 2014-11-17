//! Backend compiler module
//!
//! Contains compiler implementations for various target platforms

pub mod jvm;
pub mod msil;
pub mod pe;
pub mod wasi;

// Re-export backend structs
pub use jvm::JvmBackend;
pub use msil::ClrBackend;
pub use pe::PeBackend;
pub use wasi::WasiBackend;

use crate::config::{GaiaConfig, GaiaSettings};
use gaia_types::{
    helpers::{AbiCompatible, ApiCompatible, Architecture, CompilationTarget},
    instruction::GaiaProgram,
    GaiaError, Result,
};
use std::collections::HashMap;

/// 函数映射器
#[derive(Debug)]
pub struct FunctionMapper {
    /// 函数映射表 (平台 -> 源函数 -> 目标函数)
    mappings: HashMap<CompilationTarget, HashMap<String, String>>,
}

impl FunctionMapper {
    /// Create a new function mapper with default mappings
    pub fn new() -> Self {
        let mut mapper = Self { mappings: HashMap::new() };

        // Initialize platform-specific mappings
        mapper.init_il_mappings();
        mapper.init_jvm_mappings();
        mapper.init_pe_mappings();
        mapper.init_wasi_mappings();

        mapper
    }

    /// 从配置创建函数映射器
    pub fn from_config(config: &GaiaSettings) -> Result<Self> {
        let mut mapper = Self::new();

        // 加载所有平台的函数映射
        for mapping in &config.function_mappings {
            for (platform_name, target_name) in &mapping.platform_mappings {
                // 找到对应的编译目标
                for (target, _platform_config) in &config.platforms {
                    if platform_name == &target.build.to_string() {
                        mapper.add_mapping(target, &mapping.common_name, target_name);
                    }
                }
            }
        }

        Ok(mapper)
    }

    /// 添加函数映射
    pub fn add_mapping(&mut self, target: &CompilationTarget, source_func: &str, target_func: &str) {
        self.mappings
            .entry(target.clone())
            .or_insert_with(HashMap::new)
            .insert(source_func.to_string(), target_func.to_string());
    }

    /// 映射函数名
    pub fn map_function(&self, target: &CompilationTarget, function_name: &str) -> Option<&str> {
        self.mappings.get(target).and_then(|platform_mappings| platform_mappings.get(function_name)).map(|s| s.as_str())
    }

    /// Initialize IL (.NET) platform mappings
    fn init_il_mappings(&mut self) {
        let il_target = CompilationTarget {
            build: Architecture::CLR,
            host: AbiCompatible::MicrosoftIntermediateLanguage,
            target: ApiCompatible::ClrRuntime(4),
        };
        self.add_mapping(&il_target, "__builtin_print", "void [mscorlib]System.Console::WriteLine(string)");
        self.add_mapping(&il_target, "__builtin_println", "void [mscorlib]System.Console::WriteLine(string)");
        self.add_mapping(&il_target, "__builtin_read", "string [mscorlib]System.Console::ReadLine()");
        self.add_mapping(&il_target, "malloc", "System.Runtime.InteropServices.Marshal.AllocHGlobal");
        self.add_mapping(&il_target, "free", "System.Runtime.InteropServices.Marshal.FreeHGlobal");
    }

    /// Initialize JVM platform mappings
    fn init_jvm_mappings(&mut self) {
        let jvm_target = CompilationTarget {
            build: Architecture::JVM,
            host: AbiCompatible::JavaAssembly,
            target: ApiCompatible::JvmRuntime(8),
        };
        self.add_mapping(&jvm_target, "__builtin_print", "java.lang.System.out.println");
        self.add_mapping(&jvm_target, "__builtin_println", "java.lang.System.out.println");
        self.add_mapping(&jvm_target, "__builtin_read", "java.util.Scanner.nextLine");
        self.add_mapping(&jvm_target, "malloc", "java.nio.ByteBuffer.allocateDirect");
    }

    /// Initialize PE (Windows) platform mappings
    fn init_pe_mappings(&mut self) {
        let pe_target =
            CompilationTarget { build: Architecture::X86_64, host: AbiCompatible::PE, target: ApiCompatible::MicrosoftVisualC };
        self.add_mapping(&pe_target, "__builtin_print", "printf");
        self.add_mapping(&pe_target, "__builtin_println", "puts");
        self.add_mapping(&pe_target, "__builtin_read", "gets_s");
        self.add_mapping(&pe_target, "malloc", "HeapAlloc");
        self.add_mapping(&pe_target, "free", "HeapFree");
    }

    /// Initialize WASI platform mappings
    fn init_wasi_mappings(&mut self) {
        let wasi_target = CompilationTarget {
            build: Architecture::WASM32,
            host: AbiCompatible::WebAssemblyTextFormat,
            target: ApiCompatible::WASI,
        };
        self.add_mapping(&wasi_target, "__builtin_print", "wasi_print");
        self.add_mapping(&wasi_target, "__builtin_println", "wasi_println");
        self.add_mapping(&wasi_target, "__builtin_read", "wasi_read");
        self.add_mapping(&wasi_target, "malloc", "malloc");
        self.add_mapping(&wasi_target, "free", "free");
    }
}

impl Default for FunctionMapper {
    fn default() -> Self {
        Self::new()
    }
}

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
