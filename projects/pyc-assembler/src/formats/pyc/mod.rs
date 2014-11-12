#![doc = include_str!("readme.md")]

use crate::program::{PythonProgram, PythonVersion};
use gaia_types::{
    helpers::{open_file, Url},
    GaiaError,
};
use std::{
    fmt::Debug,
    io::{BufReader, Read, Seek},
    path::Path,
};

pub mod reader;
pub mod view;
/// writer 模块包含用于将 PycView 写入输出流的逻辑。
pub mod writer;

/// PycReadConfig 结构体用于配置 .pyc 文件的读取行为。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PycReadConfig {
    /// 文件的 URL，如果存在的话。
    pub url: Option<Url>,
    /// Python 版本信息。
    pub version: PythonVersion,
}

/// PycWriteConfig 结构体用于配置 .pyc 文件的写入行为。
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct PycWriteConfig {}

impl Default for PycReadConfig {
    fn default() -> Self {
        Self { url: None, version: PythonVersion::Unknown }
    }
}

/// 从指定路径读取 .pyc 文件并将其转换为 PythonProgram。
pub fn pyc_read_path(path: &Path) -> Result<PythonProgram, GaiaError> {
    let (mut file, url) = open_file(path)?;
    let mut config = PycReadConfig::default();
    let mut magic = [0u8; 4];
    file.read_exact(&mut magic)?;
    config.version = PythonVersion::from_magic(magic);
    config.url = Some(url);
    file.seek(std::io::SeekFrom::Start(0))?;
    let mut reader = config.as_reader(BufReader::new(file));
    reader.read_to_end()?;
    reader.view.to_program(&config).result
}
