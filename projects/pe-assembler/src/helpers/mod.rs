#![doc = include_str!("readme.md")]

mod builder;
mod coff_reader;
mod pe_reader;
mod pe_writer;

pub use self::{
    coff_reader::{CoffReader, CoffViewer},
    pe_reader::PeReader,
    pe_writer::PeWriter,
};
pub use builder::*;
