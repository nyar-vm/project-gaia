//! Gaia 统一编译器
//!
//! 提供统一的编译接口，支持编译到多个目标平台

use crate::{backends::*, instruction::*};
use gaia_types::{
    helpers::{AbiCompatible, ApiCompatible, Architecture, CompilationTarget},
    GaiaErrorKind, *,
};



/// Gaia 编译器
pub struct GaiaCompiler {
    backends: Vec<Box<dyn Backend>>,
}

impl GaiaCompiler {
    /// 创建新的编译器实例，包含所有可用的后端
    pub fn new() -> Self {
        let backends: Vec<Box<dyn Backend>> = vec![
            Box::new(ClrBackend {}),
            Box::new(JvmBackend {}),
            Box::new(PeBackend {}),
            Box::new(WasiBackend {}),
        ];
        
        Self { backends }
    }

    /// 编译 Gaia 程序到指定目标
    pub fn compile(&self, program: &GaiaProgram, target: &CompilationTarget) -> Result<Vec<u8>> {
        // 计算每个后端的匹配度并找到最佳匹配
        let mut best_backend: Option<&Box<dyn Backend>> = None;
        let mut best_score = 0.0;

        for backend in &self.backends {
            let score = backend.match_score(target);
            if score > best_score {
                best_score = score;
                best_backend = Some(backend);
            }
        }

        if best_score == 0.0 || best_backend.is_none() {
            return Err(GaiaErrorKind::UnsupportedTarget { target: target.clone() }.into());
        }

        // 使用最佳匹配的后端进行编译
        best_backend.unwrap().compile(program)
    }

    /// 获取所有可用的后端
    pub fn backends(&self) -> &[Box<dyn Backend>] {
        &self.backends
    }
}

/// 编译到指定平台
pub fn compile_to_platform(program: &GaiaProgram, target: CompilationTarget) -> Result<Vec<u8>> {
    let compiler = GaiaCompiler::new();
    compiler.compile(program, &target)
}
