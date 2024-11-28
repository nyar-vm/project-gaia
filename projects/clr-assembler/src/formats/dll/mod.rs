pub use self::{reader::DllReader, writer::DllWriter};
use crate::program::ClrProgram;
use gaia_types::{helpers::open_file, GaiaError};
use std::{io::Cursor, path::Path};

pub mod reader;
pub mod writer;

/// .NET PE 文件惰性读取器
///
/// 该类负责读取和解析 .NET 程序集文件，提供以下功能：
/// - 检查文件是否为有效的 .NET 程序集
/// - 解析 CLR 头和元数据
/// - 提取程序集的基本信息
/// - 验证程序集的完整性
/// - 支持惰性读取和完整解析两种模式
#[derive(Clone, Debug)]
pub struct DllReadConfig {
    pub assembly_ref_fallback_names: Vec<String>,
}

impl Default for DllReadConfig {
    fn default() -> Self {
        Self { assembly_ref_fallback_names: Vec::new() }
    }
}

/// 从文件路径读取 .NET 程序集
pub fn dll_from_file(file_path: &Path) -> Result<ClrProgram, GaiaError> {
    let config = DllReadConfig::default();
    let (file, url) = open_file(file_path)?;
    let mut dll_reader = DllReader::new(file, &config);
    dll_reader.to_clr_program()
}

/// 从字节数组读取 .NET 程序集
pub fn dll_from_bytes(_bytes: &[u8]) -> Result<ClrProgram, GaiaError> {
    let config = DllReadConfig::default();
    let mut dll_reader = DllReader::new(Cursor::new(_bytes), &config);
    dll_reader.to_clr_program()
}

/// 检查文件是否为 .NET 程序集（DLL）
pub fn is_dotnet_dll(_file_path: &Path) -> Result<bool, GaiaError> {
    // TODO: 实现检查逻辑
    todo!()
}

/// 从文件路径读取 .NET 程序集，返回诊断结果
pub fn read_dotnet_assembly(file_path: &Path, options: &DllReadConfig) -> Result<ClrProgram, GaiaError> {
    let (file, url) = open_file(file_path)?;
    let mut dll_reader = DllReader::new(file, options);
    dll_reader.to_clr_program()
}
