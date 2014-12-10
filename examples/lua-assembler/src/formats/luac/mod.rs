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
        Self { url: None, version: LuaVersion::Unknown, check_magic_head: true }
    }
}

pub fn luac_read_path(path: &Path) -> Result<LuaProgram, GaiaError> {
    let (file, url) = open_file(path)?;
    let mut config = LuacReadConfig::default();
    config.url = Some(url);
    let reader = config.as_reader(BufReader::new(file));
    let result = reader.finish();
    result.result
}

// 为了兼容文档测试，提供简化的函数别名
pub fn read_luac_file(path: &Path) -> Result<LuaProgram, GaiaError> {
    luac_read_path(path)
}

pub fn write_luac_file(_path: &Path, _program: &LuaProgram) -> Result<(), GaiaError> {
    // TODO: 实现写入功能
    Err(GaiaError::custom_error("write_luac_file not implemented yet"))
}
