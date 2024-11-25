#![doc = include_str!("readme.md")]

use crate::{
    formats::wasm::WasmReadConfig,
    program::{WasiProgram, WasmInfo},
};
use byteorder::LittleEndian;
use gaia_types::{BinaryReader, GaiaDiagnostics, GaiaError};
use leb128;
use std::{
    cell::{OnceCell, RefCell},
    io::{Read, Seek},
};

/// wasm lazy reader
#[derive(Debug)]
pub struct WasmReader<'config, R> {
    config: &'config WasmReadConfig,
    reader: RefCell<BinaryReader<R, LittleEndian>>,
    view: OnceCell<WasmInfo>,
    program: OnceCell<WasiProgram>,
}

impl WasmReadConfig {
    pub fn as_reader<R: Read + Seek>(&self, reader: R) -> WasmReader<R> {
        WasmReader::new(reader, self)
    }
}

impl<'config, R> WasmReader<'config, R> {
    pub fn new(reader: R, config: &'config WasmReadConfig) -> Self {
        Self { reader: RefCell::new(BinaryReader::new(reader)), view: Default::default(), program: Default::default(), config }
    }
    pub fn finish(mut self) -> GaiaDiagnostics<WasiProgram>
    where
        R: Read + Seek,
    {
        match self.get_program() {
            Ok(_) => {
                let errors = self.reader.borrow_mut().take_errors();
                GaiaDiagnostics { result: self.program.take().ok_or(GaiaError::unreachable()), diagnostics: errors }
            }
            Err(e) => {
                let errors = self.reader.borrow_mut().take_errors();
                GaiaDiagnostics { result: Err(e), diagnostics: errors }
            }
        }
    }
}

impl<'config, R: Read + Seek> WasmReader<'config, R> {
    pub fn get_program(&self) -> Result<&WasiProgram, GaiaError> {
        self.program.get_or_try_init(|| self.read_program())
    }
    fn read_program(&self) -> Result<WasiProgram, GaiaError> {
        let reader = self.reader.borrow_mut();
        todo!()
    }
    pub fn get_view(&self) -> Result<&WasmInfo, GaiaError> {
        self.view.get_or_try_init(|| self.read_view())
    }
    fn read_view(&self) -> Result<WasmInfo, GaiaError> {
        let reader = self.reader.borrow_mut();
        todo!()
    }
}
