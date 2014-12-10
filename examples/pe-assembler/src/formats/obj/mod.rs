#![doc = include_str!("readme.md")]

pub mod reader;

use crate::types::CoffObject;
use gaia_types::{helpers::open_file, GaiaError};
pub use reader::ObjReader;
use std::{io::BufReader, path::Path};

// 便利函数，保持向后兼容
pub fn coff_from_file<P: AsRef<Path>>(path: P) -> Result<CoffObject, GaiaError> {
    let (file, _url) = open_file(path.as_ref())?;
    let reader = ObjReader::new(BufReader::new(file));
    reader.read_object().result
}
