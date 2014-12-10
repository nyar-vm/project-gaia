#![doc = include_str!("readme.md")]

pub mod reader;
pub mod writer;

#[derive(Copy, Clone, Debug)]
pub struct WasmReadConfig {
    pub check_magic_head: bool,
}
