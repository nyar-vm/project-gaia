//! Gaia 统一编译器
//!
//! 提供统一的编译接口，支持编译到多个目标平台

use crate::{backends::*, instruction::*};
use gaia_types::{GaiaErrorKind, *};

/// 支持的目标平台
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetPlatform {
    /// .NET IL (Intermediate Language)
    IL,
    /// Java Virtual Machine
    JVM,
    /// Portable Executable (Windows)
    PE,
    /// WebAssembly System Interface
    WASI,
}

impl TargetPlatform {
    /// 获取平台名称
    pub fn name(&self) -> &'static str {
        match self {
            TargetPlatform::IL => "IL",
            TargetPlatform::JVM => "JVM",
            TargetPlatform::PE => "PE",
            TargetPlatform::WASI => "WASI",
        }
    }

    /// 获取文件扩展名
    pub fn file_extension(&self) -> &'static str {
        match self {
            TargetPlatform::IL => "msil",
            TargetPlatform::JVM => "class",
            TargetPlatform::PE => "exe",
            TargetPlatform::WASI => "wasm",
        }
    }

    /// 获取所有支持的平台
    pub fn all() -> Vec<TargetPlatform> {
        vec![TargetPlatform::IL, TargetPlatform::JVM, TargetPlatform::PE, TargetPlatform::WASI]
    }
}

/// Gaia 统一编译器
pub struct GaiaCompiler {
    target: TargetPlatform,
}

impl GaiaCompiler {
    /// 创建新的编译器实例
    pub fn new(target: TargetPlatform) -> Self {
        Self { target }
    }

    /// 编译程序到目标平台
    pub fn compile(&self, program: &GaiaProgram) -> Result<Vec<u8>> {
        match self.target {
            TargetPlatform::IL => msil::MsilBackend::compile(program),
            TargetPlatform::JVM => jvm::JVMBackend::compile(program),
            TargetPlatform::PE => pe::PEBackend::compile(program),
            TargetPlatform::WASI => wasi::WASIBackend::compile(program),
        }
    }

    /// 获取目标平台
    pub fn target(&self) -> TargetPlatform {
        self.target
    }

    /// 编译到所有支持的平台
    pub fn compile_all(program: &GaiaProgram) -> Result<Vec<(TargetPlatform, Vec<u8>)>> {
        let mut results = Vec::new();

        for platform in TargetPlatform::all() {
            let compiler = GaiaCompiler::new(platform);
            match compiler.compile(program) {
                Ok(bytecode) => results.push((platform, bytecode)),
                Err(e) => {
                    // 记录错误但继续编译其他平台
                    eprintln!("编译到 {} 平台失败: {}", platform.name(), e);
                }
            }
        }

        if results.is_empty() {
            Err(GaiaErrorKind::CustomError { message: "所有平台编译都失败了".to_string() }.into())
        }
        else {
            Ok(results)
        }
    }
}

/// 便捷函数：编译到指定平台
pub fn compile_to_platform(program: &GaiaProgram, target: TargetPlatform) -> Result<Vec<u8>> {
    let compiler = GaiaCompiler::new(target);
    compiler.compile(program)
}

/// 便捷函数：编译到所有平台
pub fn compile_to_all_platforms(program: &GaiaProgram) -> Result<Vec<(TargetPlatform, Vec<u8>)>> {
    GaiaCompiler::compile_all(program)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_platform() {
        assert_eq!(TargetPlatform::IL.name(), "IL");
        assert_eq!(TargetPlatform::JVM.file_extension(), "class");
        assert_eq!(TargetPlatform::all().len(), 4);
    }

    #[test]
    fn test_compiler_creation() {
        let compiler = GaiaCompiler::new(TargetPlatform::IL);
        assert_eq!(compiler.target(), TargetPlatform::IL);
    }
}
