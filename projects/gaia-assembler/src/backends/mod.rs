//! Backend compiler module
//!
//! Contains compiler implementations for various target platforms

pub mod jvm;
pub mod msil;
pub mod pe;
pub mod wasi;

#[cfg(test)]
mod test_mapper;

use crate::instruction::GaiaProgram;
use gaia_types::Result;
use std::collections::HashMap;

/// Target platform enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TargetPlatform {
    IL,
    JVM,
    PE,
    WASI,
}

/// Function mapper for cross-platform function name mapping
pub struct FunctionMapper {
    mappings: HashMap<TargetPlatform, HashMap<String, String>>,
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

    /// Add a function mapping for a specific platform
    pub fn add_mapping(&mut self, platform: TargetPlatform, source_name: &str, target_name: &str) {
        self.mappings.entry(platform).or_insert_with(HashMap::new).insert(source_name.to_string(), target_name.to_string());
    }

    /// Map a function name to the target platform equivalent
    pub fn map_function(&self, func_name: &str, platform: TargetPlatform) -> String {
        self.mappings
            .get(&platform)
            .and_then(|map| map.get(func_name))
            .map(|s| s.clone())
            .unwrap_or_else(|| func_name.to_string())
    }

    /// Initialize IL (.NET) platform mappings
    fn init_il_mappings(&mut self) {
        self.add_mapping(TargetPlatform::IL, "__builtin_print", "void [mscorlib]System.Console::WriteLine(string)");
        self.add_mapping(TargetPlatform::IL, "__builtin_println", "void [mscorlib]System.Console::WriteLine(string)");
        self.add_mapping(TargetPlatform::IL, "__builtin_read", "string [mscorlib]System.Console::ReadLine()");
        self.add_mapping(TargetPlatform::IL, "malloc", "System.Runtime.InteropServices.Marshal.AllocHGlobal");
        self.add_mapping(TargetPlatform::IL, "free", "System.Runtime.InteropServices.Marshal.FreeHGlobal");
    }

    /// Initialize JVM platform mappings
    fn init_jvm_mappings(&mut self) {
        self.add_mapping(TargetPlatform::JVM, "__builtin_print", "java.lang.System.out.println");
        self.add_mapping(TargetPlatform::JVM, "__builtin_println", "java.lang.System.out.println");
        self.add_mapping(TargetPlatform::JVM, "__builtin_read", "java.util.Scanner.nextLine");
        self.add_mapping(TargetPlatform::JVM, "malloc", "java.nio.ByteBuffer.allocateDirect");
    }

    /// Initialize PE (Windows) platform mappings
    fn init_pe_mappings(&mut self) {
        self.add_mapping(TargetPlatform::PE, "__builtin_print", "printf");
        self.add_mapping(TargetPlatform::PE, "__builtin_println", "puts");
        self.add_mapping(TargetPlatform::PE, "__builtin_read", "gets_s");
        self.add_mapping(TargetPlatform::PE, "malloc", "HeapAlloc");
        self.add_mapping(TargetPlatform::PE, "free", "HeapFree");
    }

    /// Initialize WASI platform mappings
    fn init_wasi_mappings(&mut self) {
        self.add_mapping(TargetPlatform::WASI, "__builtin_print", "wasi_print");
        self.add_mapping(TargetPlatform::WASI, "__builtin_println", "wasi_println");
        self.add_mapping(TargetPlatform::WASI, "__builtin_read", "wasi_read");
        self.add_mapping(TargetPlatform::WASI, "malloc", "malloc");
        self.add_mapping(TargetPlatform::WASI, "free", "free");
    }
}

impl Default for FunctionMapper {
    fn default() -> Self {
        Self::new()
    }
}

/// Backend compiler trait
pub trait Backend {
    const IS_BINARY: bool = true;
    /// Compile Gaia program to target platform
    fn compile(program: &GaiaProgram) -> Result<Vec<u8>>;

    /// Get backend name
    fn name() -> &'static str;

    /// Get output file extension
    fn file_extension() -> &'static str;
}
