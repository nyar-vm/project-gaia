#![doc = include_str!("readme.md")]
pub use self::compilation_target::{AbiCompatible, ApiCompatible, Architecture, CompilationTarget};
use crate::GaiaError;
use std::{fs::File, path::Path};
pub use url::Url;

mod compilation_target;
// pub mod dwarf;

/// 从文件路径创建 URL。
pub fn url_from_path(path: &Path) -> Result<Url, GaiaError> {
    match Url::from_file_path(path) {
        Ok(url) => Ok(url),
        Err(_) => Err(GaiaError::invalid_data("path not valid")),
    }
}

/// 打开文件并返回文件句柄和 URL。
pub fn open_file(path: &Path) -> Result<(File, Url), GaiaError> {
    let url = url_from_path(path)?;
    match File::open(path) {
        Ok(file) => Ok((file, url)),
        Err(e) => Err(GaiaError::io_error(e, url)),
    }
}

/// 创建文件并返回文件句柄和 URL。
pub fn create_file(path: &Path) -> Result<(File, Url), GaiaError> {
    let url = url_from_path(path)?;
    match File::create(path) {
        Ok(file) => Ok((file, url)),
        Err(e) => Err(GaiaError::io_error(e, url)),
    }
}

/// 将数据保存为 JSON 格式。
#[cfg(feature = "serde_json")]
pub fn save_json<T: serde::Serialize>(analyses: &T, output_path: &Path) -> Result<Url, GaiaError> {
    use std::io::Write;
    let (mut file, url) = create_file(output_path)?;
    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let mut buf = Vec::new();
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
    analyses.serialize(&mut ser)?;
    file.write_all(&buf)?;
    Ok(url)
}
