#![deny(missing_debug_implementations, missing_copy_implementations)]
#![warn(missing_docs, rustdoc::missing_crate_level_docs)]
#![doc = include_str!("readme.md")]
#![doc(html_logo_url = "https://raw.githubusercontent.com/oovm/shape-rs/dev/projects/images/Trapezohedron.svg")]
#![doc(html_favicon_url = "https://raw.githubusercontent.com/oovm/shape-rs/dev/projects/images/Trapezohedron.svg")]

use crate::{formats::exe::writer::ExeWriter, helpers::PeWriter, types::PeProgram};
use gaia_types::{
    helpers::{create_file, Url},
     Result,
};
use std::path::Path;

pub mod formats;
pub mod helpers;
pub mod types;

/// 将 PE 程序写入到指定路径的 EXE 文件
///
/// 这是一个高级 API 函数，隐藏了 ExeWriter 的直接使用细节。
///
/// # 参数
///
/// * `pe` - 要写入的 PE 程序
/// * `path` - 输出文件路径
///
/// # 返回值
///
/// 成功时返回文件的 URL，失败时返回 GaiaError
///
/// # 示例
///
/// ```rust
/// use pe_assembler::exe_write_path;
/// use std::path::Path;
///
/// let pe_program = /* 创建 PE 程序 */;
/// let output_path = Path::new("output.exe");
/// let url = exe_write_path(&pe_program, output_path)?;
/// ```
pub fn exe_write_path(pe: &PeProgram, path: &Path) -> Result<Url> {
    let (file, url) = create_file(path)?;
    let mut exe = ExeWriter::new(file);
    exe.write_program(pe)?;
    Ok(url)
}
