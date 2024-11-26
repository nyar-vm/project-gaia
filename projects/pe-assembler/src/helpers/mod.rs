#![doc = include_str!("readme.md")]

mod builder;
mod coff_reader;
pub mod pe_reader;
mod pe_writer;

pub(crate) use self::coff_reader::{CoffReader, read_coff_header, read_section_headers, read_coff_object};
pub use self::{pe_reader::PeReader, pe_writer::PeWriter};
pub use builder::*;
