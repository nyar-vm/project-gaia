#![doc = include_str!("readme.md")]

pub mod reader;

pub use reader::{read_coff_from_file, ObjReader};
