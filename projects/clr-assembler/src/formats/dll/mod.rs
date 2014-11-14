use crate::program::ClrProgram;
use gaia_types::{GaiaError, SourceLocation};

pub mod reader;
pub mod writer;

/// 从文件路径读取 .NET 程序集
pub fn read_from_file(file_path: &str) -> Result<ClrProgram, GaiaError> {
    if !std::path::Path::new(file_path).exists() {
        return Err(GaiaError::syntax_error("无效的文件路径".to_string(), SourceLocation::default()));
    }

    // TODO: 实现从文件读取 .NET 程序集的逻辑
    let program = ClrProgram::new("DefaultAssembly");
    Ok(program)
}

/// 从字节数组读取 .NET 程序集
pub fn read_from_bytes(_bytes: &[u8]) -> Result<ClrProgram, GaiaError> {
    // TODO: 实现从字节数组读取 .NET 程序集的逻辑
    Err(GaiaError::syntax_error("暂不支持从字节数组读取".to_string(), SourceLocation::default()))
}
