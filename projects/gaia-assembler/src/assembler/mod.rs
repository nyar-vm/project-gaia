//! Gaia 统一编译器
//!
//! 提供统一的编译接口，支持编译到多个目标平台

use crate::backends::*;
use gaia_types::{helpers::CompilationTarget, GaiaErrorKind, *};

/// Gaia 编译器
pub struct GaiaAssembler {
    backends: Vec<Box<dyn Backend>>,
}

impl GaiaAssembler {
    /// 创建新的编译器实例，包含所有可用的后端
    pub fn new() -> Self {
        let backends: Vec<Box<dyn Backend>> =
            vec![Box::new(ClrBackend {}), Box::new(JvmBackend {}), Box::new(PeBackend {}), Box::new(WasiBackend {})];

        Self { backends }
    }

    /// 编译 Gaia 程序到指定目标
    pub fn compile(&self, program: &GaiaProgram, target: &CompilationTarget) -> Result<GeneratedFiles> {
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

        // 创建默认配置
        let config = crate::config::GaiaConfig::default();

        // 使用最佳匹配的后端进行编译
        best_backend.unwrap().generate(program, &config)
    }

    /// 获取所有可用的后端
    pub fn backends(&self) -> &[Box<dyn Backend>] {
        &self.backends
    }
}

/// 编译到指定平台
pub fn compile_to_platform(program: &GaiaProgram, target: CompilationTarget) -> Result<Vec<u8>> {
    let compiler = GaiaAssembler::new();
    let generated_files = compiler.compile(program, &target)?;

    // 从生成的文件中提取主要的二进制文件
    // 通常第一个文件是主要的输出文件
    if let Some((_, bytes)) = generated_files.files.iter().next() {
        Ok(bytes.clone())
    }
    else {
        Err(GaiaError::invalid_data("No output files generated"))
    }
}
