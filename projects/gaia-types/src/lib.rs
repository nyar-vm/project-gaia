#![feature(try_trait_v2)]
#![deny(missing_debug_implementations, missing_copy_implementations)]
#![warn(missing_docs, rustdoc::missing_crate_level_docs)]
#![doc = include_str!("readme.md")]
#![doc(html_logo_url = "https://raw.githubusercontent.com/oovm/shape-rs/dev/projects/images/Trapezohedron.svg")]
#![doc(html_favicon_url = "https://raw.githubusercontent.com/oovm/shape-rs/dev/projects/images/Trapezohedron.svg")]

pub mod assembler;
mod errors;
pub mod helpers;
pub mod lexer;
pub mod parser;
pub mod reader;
pub mod writer;

pub use crate::{
    assembler::BinaryWriter,
    errors::{GaiaDiagnostics, GaiaError, GaiaErrorKind, Result},
    reader::{BinaryReader, SourceLocation, SourcePosition},
    writer::TextWriter,
};
