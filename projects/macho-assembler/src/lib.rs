//! Mach-O 汇编器库
//! 
//! 这个库提供了用于处理 Mach-O 格式文件的工具和结构。

#![warn(missing_docs, rust_2018_idioms, missing_copy_implementations)]

use crate::{formats::dylib::writer::DylibWriter, helpers::MachoWriter, types::MachoProgram};
use gaia_types::{
    helpers::{create_file, Url},
    Result,
};
use std::path::Path;

/// 类型定义模块
pub mod types;

/// 格式处理模块
pub mod formats;
/// 辅助工具模块
pub mod helpers;

/// 构建器模块
pub mod builder;

/// 将 Mach-O 程序写入到指定路径的动态库文件
///
/// 这是一个高级 API 函数，隐藏了 DylibWriter 的直接使用细节。
///
/// # 参数
///
/// * `macho` - 要写入的 Mach-O 程序
/// * `path` - 输出文件路径
///
/// # 返回值
///
/// 成功时返回文件的 URL，失败时返回 GaiaError
///
/// # 示例
///
/// ```rust,no_run
/// use macho_assembler::dylib_write_path;
/// use std::path::Path;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // let macho_program = /* 创建 Mach-O 程序 */;
/// // let output_path = Path::new("output.dylib");
/// // let url = dylib_write_path(&macho_program, output_path)?;
/// # Ok(())
/// # }
/// ```
pub fn dylib_write_path(macho: &MachoProgram, path: &Path) -> Result<Url> {
    let (file, url) = create_file(path)?;
    let mut dylib = DylibWriter::new(file);
    dylib.write_program(macho)?;
    Ok(url)
}