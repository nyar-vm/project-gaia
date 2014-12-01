#![doc = include_str!("readme.md")]

mod builder;
mod coff_reader;
pub mod pe_reader;
mod pe_writer;

pub(crate) use self::coff_reader::{read_coff_header, read_coff_object, read_section_headers, CoffReader};
pub use self::{pe_reader::PeReader, pe_writer::PeWriter};
pub use builder::*;
