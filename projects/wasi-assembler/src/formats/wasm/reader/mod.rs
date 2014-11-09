#![doc = include_str!("readme.md")]

use crate::program::WasiProgram;
use byteorder::LittleEndian;
use gaia_types::{BinaryReader, GaiaDiagnostics};
use std::io::{Read, Seek};

/// wasm lazy reader
#[derive(Debug)]
pub struct WasmReader<W> {
    reader: BinaryReader<W, LittleEndian>,
}

impl<R> WasmReader<R> {
    pub fn new(reader: R) -> Self {
        Self { reader: BinaryReader::new(reader) }
    }

    pub fn finish(self) -> R {
        self.reader.finish()
    }
}

impl<R: Read> WasmReader<R> {
    pub fn read(&mut self) -> GaiaDiagnostics<WasiProgram>
    where
        R: Seek,
    {
        todo!()
    }
}
