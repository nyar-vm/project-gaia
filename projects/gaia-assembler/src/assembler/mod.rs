//! Gaia 统一编译器
//!
//! 提供统一的编译接口，支持编译到多个目标平台

use crate::{backends::*, program::GaiaModule};
use gaia_types::{helpers::CompilationTarget, GaiaErrorKind, *};

/// Gaia 编译器
pub struct GaiaAssembler {
    backends: Vec<Box<dyn Backend>>,
}

impl GaiaAssembler {
    /// 创建新的编译器实例，包含所有可用的后端
    pub fn new() -> Self {
        #[allow(unused_mut)]
        let mut backends: Vec<Box<dyn Backend>> = vec![];

        #[cfg(feature = "jvm")]
        backends.push(Box::new(JvmBackend {}));
        #[cfg(feature = "pe")]
        backends.push(Box::new(PeBackend {}));
        #[cfg(feature = "wasi")]
        backends.push(Box::new(WasiBackend {}));
        #[cfg(feature = "x86_64")]
        backends.push(Box::new(X86Backend {}));
        #[cfg(feature = "gcn")]
        backends.push(Box::new(GcnBackend::new()));
        #[cfg(feature = "sass")]
        backends.push(Box::new(SassBackend::new()));
        #[cfg(feature = "clr")]
        backends.push(Box::new(ClrBackend {}));
        #[cfg(feature = "elf")]
        backends.push(Box::new(ElfBackend {}));
        #[cfg(feature = "macho")]
        backends.push(Box::new(MachoBackend {}));
        #[cfg(feature = "lua")]
        backends.push(Box::new(LuaBackend {}));
        #[cfg(feature = "llvm")]
        backends.push(Box::new(LlvmBackend {}));
        #[cfg(feature = "msl")]
        backends.push(Box::new(MslBackend {}));
        #[cfg(feature = "spirv")]
        backends.push(Box::new(SpirvBackend {}));
        #[cfg(feature = "python")]
        backends.push(Box::new(PythonBackend {}));

        Self { backends }
    }

    /// 编译 Gaia 程序到指定目标
    pub fn compile(&self, program: &GaiaModule, target: &CompilationTarget) -> Result<GeneratedFiles> {
        // 优先按 host 精确选择后端，避免匹配度导致的误选
        let mut best_backend: Option<&Box<dyn Backend>> = None;
        let mut best_score = 0.0;

        // 先尝试以 backend 的 primary_target 与传入 target 的 host 和 build 精确匹配
        if let Some(candidate) = self.backends.iter().find(|b| {
            let pt = b.primary_target();
            pt.host == target.host && pt.build == target.build
        }) {
            best_backend = Some(candidate);
            best_score = 100.0; // 精确匹配优先
        }

        // 若未找到精确匹配，则退回评分机制
        if best_backend.is_none() {
            for backend in &self.backends {
                let score = backend.match_score(target);
                if score > best_score {
                    best_score = score;
                    best_backend = Some(backend);
                }
            }
        }

        if best_backend.is_none() || best_score <= 0.0 {
            return Err(GaiaErrorKind::UnsupportedTarget { target: target.clone() }.into());
        }

        // 创建配置并传递目标，以便后端依据 host/target 输出正确的文件类型
        let mut config = crate::config::GaiaConfig::default();
        config.target = target.clone();

        // 使用选定的后端进行编译
        best_backend.unwrap().generate(program, &config)
    }

    /// 获取所有可用的后端
    pub fn backends(&self) -> &[Box<dyn Backend>] {
        &self.backends
    }
}

/// 编译到指定平台
pub fn compile_to_platform(program: &GaiaModule, target: CompilationTarget) -> Result<Vec<u8>> {
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
