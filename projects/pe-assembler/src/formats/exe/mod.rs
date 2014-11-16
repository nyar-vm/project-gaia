#![doc = include_str!("readme.md")]

use crate::{formats::exe::writer::ExeWriter, helpers::PeWriter, types::PeProgram};
use gaia_types::{
    helpers::{create_file, Url},
    GaiaError,
};
use std::path::Path;

/// PE EXE 相关模块
pub mod reader;
pub mod writer;

pub fn exe_write_path(pe: &PeProgram, path: &Path) -> Result<Url, GaiaError> {
    let (file, url) = create_file(path)?;
    let mut exe = ExeWriter::new(file);
    exe.write_program(pe)?;
    Ok(url)
}
