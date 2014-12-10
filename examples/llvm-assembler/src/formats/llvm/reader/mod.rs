use gaia_types::Result;
use oak_core::{Builder, ParseSession};
use oak_llvm_ir::{LLirBuilder, LLvmLanguage, LLirRoot};

/// LLVM IR 读取器
pub struct LLvmReader {
    language: LLvmLanguage,
}

impl Default for LLvmReader {
    fn default() -> Self {
        Self::new()
    }
}

impl LLvmReader {
    pub fn new() -> Self {
        Self {
            language: LLvmLanguage::default(),
        }
    }

    /// 从字符串解析 LLVM IR
    pub fn read(&self, source: &str) -> Result<LLirRoot> {
        let builder = LLirBuilder::new(&self.language);
        let mut session_cache = ParseSession::<LLvmLanguage>::default();
        let diagnostics = builder.build(source, &[], &mut session_cache);

        match diagnostics.result {
            Ok(root) => Ok(root),
            Err(e) => {
                let message = e.to_string();
                Err(gaia_types::GaiaError::custom_error(message))
            }
        }
    }
}

/// 解析 LLVM IR 字符串为 AST
pub fn parse(source: &str) -> Result<LLirRoot> {
    LLvmReader::new().read(source)
}
