#![doc = include_str!("readme.md")]

/// Marshal 序列化模块
pub mod marshal;
pub mod reader;
/// 写入器模块，负责将 PythonProgram 写入到 .pyc 文件
pub mod writer;

use crate::program::PythonVersion;
use gaia_types::{GaiaDiagnostics, GaiaError};
use std::{fs::File, io::BufReader, path::Path};

/// 配置结构体，用于指定读取 .pyc 文件时的参数
#[derive(Debug, Clone, Copy)]
pub struct PycReadConfig {
    /// 指定 Python 版本，如果为 Unknown，则从文件头部推断
    pub version: PythonVersion,
}

impl Default for PycReadConfig {
    fn default() -> Self {
        Self { version: PythonVersion::Unknown }
    }
}

/// 配置结构体，用于指定写入 .pyc 文件时的参数
#[derive(Debug, Clone, Copy)]
pub struct PycWriteConfig {
    /// 指定 Python 版本
    pub version: PythonVersion,
}

impl Default for PycWriteConfig {
    fn default() -> Self {
        Self { version: PythonVersion::Python3_9 }
    }
}

/// 从指定路径读取 .pyc 文件并解析为 PythonProgram
pub fn pyc_read_path<P: AsRef<Path>>(path: P, config: &PycReadConfig) -> GaiaDiagnostics<crate::program::PythonProgram> {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => return GaiaDiagnostics { result: Err(GaiaError::from(e)), diagnostics: Vec::new() },
    };

    let buf_reader = BufReader::new(file);
    let reader = config.as_reader(buf_reader);
    reader.finish()
}
