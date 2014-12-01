//! # pyc 模块
//!
//! 本模块提供了对 Python 字节码文件（`.pyc` 文件）的读取、解析和写入功能。
//! 它包含了以下主要组件：
//!
//! - `reader`: 负责从字节流中读取和解析 `.pyc` 文件的内容。
//! - `writer`: 负责将 PythonProgram 序列化为 `.pyc` 文件的字节流。
//!
//! ## 设计目标
//!
//! - **高效性**：通过优化的数据结构和算法，确保读取和写入操作的高效性。
//! - **准确性**：严格遵循 `.pyc` 文件格式规范，确保生成和解析的文件符合标准。
//! - **易用性**：提供简洁明了的 API，使得用户可以轻松地进行 `.pyc` 文件的操作。
//! - **可扩展性**：模块化的设计使得功能可以根据需要进行扩展和定制。

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
        Self {
            version: PythonVersion::Unknown,
        }
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
        Self {
            version: PythonVersion::Python3_9,
        }
    }
}

/// 从指定路径读取 .pyc 文件并解析为 PythonProgram
pub fn pyc_read_path<P: AsRef<Path>>(
    path: P,
    config: &PycReadConfig,
) -> GaiaDiagnostics<crate::program::PythonProgram> {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            return GaiaDiagnostics {
                result: Err(GaiaError::from(e)),
                diagnostics: Vec::new(),
            }
        }
    };

    let buf_reader = BufReader::new(file);
    let reader = config.as_reader(buf_reader);
    reader.finish()
}
