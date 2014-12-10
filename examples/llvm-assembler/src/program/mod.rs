use crate::formats::llvm::reader;
use gaia_types::Result;
use oak_llvm_ir::ast::LLirRoot;

/// LLVM 程序表示
#[derive(Debug, Clone, Default)]
pub struct LLvmProgram {
    pub root: LLirRoot,
}

impl LLvmProgram {
    pub fn new() -> Self {
        Self::default()
    }

    /// 从源代码加载程序
    pub fn from_source(source: &str) -> Result<Self> {
        let root = reader::parse(source)?;
        Ok(Self { root })
    }
}
