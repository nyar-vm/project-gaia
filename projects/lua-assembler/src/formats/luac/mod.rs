use crate::program::{LuaProgram, LuaVersion};
use gaia_types::{
    helpers::{open_file, Url},
    GaiaError,
};
use std::{fmt::Debug, io::BufReader, path::Path};

pub mod reader;
pub mod view;
pub mod writer;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LuacReadConfig {
    pub url: Option<Url>,
    pub version: LuaVersion,
    pub check_magic_head: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct LuacWriteConfig {}

impl Default for LuacReadConfig {
    fn default() -> Self {
        Self { url: None, version: LuaVersion::Unknown, check_magic_head: false }
    }
}

pub fn luac_read_path(path: &Path) -> Result<LuaProgram, GaiaError> {
    let (file, url) = open_file(path)?;
    let mut config = LuacReadConfig::default();
    config.url = Some(url);
    let mut reader = config.as_reader(BufReader::new(file));
    reader.read_to_end()?;
    reader.view.to_program().result
}
