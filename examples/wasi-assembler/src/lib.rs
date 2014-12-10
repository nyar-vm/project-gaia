#![feature(once_cell_try)]
#![warn(missing_docs)]
#![doc = include_str!("../readme.md")]

pub mod formats;
pub mod helpers;
pub mod program;

// 重新导出常用模块
#[cfg(feature = "wat")]
pub use formats::wat;
use gaia_types::GaiaError;
pub use program::{
    WasiAlias, WasiAliasTarget, WasiCanonicalOperation, WasiComponentItem, WasiCoreFunc, WasiCoreInstance, WasiCoreModule,
    WasiCoreType, WasiCustomSection, WasiDataSegment, WasiElementSegment, WasiExport, WasiFunction, WasiFunctionType,
    WasiGlobal, WasiImport, WasiInstance, WasiInstanceArg, WasiInstanceType, WasiInstruction, WasiMemory, WasiPrimitiveType,
    WasiProgram, WasiProgramBuilder, WasiProgramType, WasiRecordField, WasiResourceMethod, WasiResourceType, WasiSymbol,
    WasiSymbolType, WasiTable, WasiType, WasiTypeDefinition, WasiVariantCase, WasmExportType, WasmGlobalType, WasmImportType,
    WasmInfo, WasmLocal, WasmMemoryType, WasmReferenceType, WasmTableType, WasmValueType,
};

/// WASI 错误类型
#[derive(Debug)]
pub enum WasiError {
    /// 解析错误
    ParseError(String),
    /// 验证错误
    ValidationError(String),
    /// 编译错误
    CompilationError(String),
    /// 其他错误
    Other(String),
}

impl std::fmt::Display for WasiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WasiError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            WasiError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            WasiError::CompilationError(msg) => write!(f, "Compilation error: {}", msg),
            WasiError::Other(msg) => write!(f, "Other error: {}", msg),
        }
    }
}

impl std::error::Error for WasiError {}

impl From<GaiaError> for WasiError {
    fn from(error: GaiaError) -> Self {
        WasiError::Other(error.to_string())
    }
}

/// WASI 汇编器
pub struct WasiAssembler {
    target: String,
}

impl WasiAssembler {
    /// 创建新的汇编器实例
    pub fn new() -> Self {
        Self { target: "wasm32-wasi".to_string() }
    }

    /// 设置目标架构
    pub fn set_target(&mut self, target: &str) {
        self.target = target.to_string();
    }

    /// 从字符串汇编 WASI 代码
    pub fn assemble_from_str(&self, _source: &str) -> std::result::Result<Vec<u8>, WasiError> {
        // 这是一个占位实现
        Ok(vec![0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00])
    }

    /// 从文件汇编 WASI 代码
    pub fn assemble_from_file(&self, path: impl AsRef<std::path::Path>) -> std::result::Result<Vec<u8>, WasiError> {
        let source = std::fs::read_to_string(path).map_err(|e| WasiError::Other(e.to_string()))?;
        self.assemble_from_str(&source)
    }
}

impl Default for WasiAssembler {
    fn default() -> Self {
        Self::new()
    }
}
