#![doc = include_str!("readme.md")]

pub mod reader;
pub mod writer;

pub use reader::{read_lib_from_file, LibReader};
pub use writer::WriteConfig;
