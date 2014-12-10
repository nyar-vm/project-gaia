#![doc = include_str!("readme.md")]

pub mod reader;
pub mod writer;

use crate::{
    helpers::CoffReader,
    types::{CoffFileType, CoffInfo, StaticLibrary},
};
use gaia_types::{helpers::open_file, GaiaError};
pub use reader::LibReader;
use std::{
    io::{BufReader, Cursor},
    path::Path,
};
pub use writer::WriteConfig;

#[derive(Copy, Clone, Debug)]
pub struct LibReadConfig {}

// 便利函数，保持向后兼容
pub fn lib_from_bytes(data: &[u8]) -> Result<StaticLibrary, GaiaError> {
    let reader = LibReader::new(Cursor::new(data));
    reader.finish().result
}

pub fn lib_from_file<P>(path: P) -> Result<StaticLibrary, GaiaError>
where
    P: AsRef<Path>,
{
    let (file, url) = open_file(path.as_ref())?;
    let reader = LibReader::new(file).with_url(url);
    reader.finish().result
}

pub fn lib_file_type<P: AsRef<Path>>(path: P) -> Result<CoffFileType, GaiaError> {
    let (file, url) = open_file(path.as_ref())?;
    let mut reader = LibReader::new(BufReader::new(file)).with_url(url);
    reader.get_coff_header()?;
    todo!()
}

/// 获取文件信息
pub fn lib_file_info<P: AsRef<Path>>(path: P) -> Result<CoffInfo, GaiaError> {
    let (file, url) = open_file(path.as_ref())?;
    let mut reader = LibReader::new(BufReader::new(file)).with_url(url);
    reader.get_coff_header()?;
    todo!()
}
