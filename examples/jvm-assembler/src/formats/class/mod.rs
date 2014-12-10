#![doc = include_str!("readme.md")]

pub mod reader;

pub mod view;
pub mod writer;

/// Class 文件读取配置
#[derive(Copy, Clone, Debug)]
pub struct ClassReadConfig {}
