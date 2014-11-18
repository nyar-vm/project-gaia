#![doc = include_str!("readme.md")]

use crate::formats::wasm::{
    view::{
        WasmView, WasmViewCode, WasmViewElement, WasmViewExport, WasmViewFunction, WasmViewFunctionType, WasmViewGlobal,
        WasmViewImport, WasmViewInstruction, WasmViewMemory, WasmViewTable,
    },
    WasmReadConfig,
};
use byteorder::ReadBytesExt;
use gaia_types::{GaiaDiagnostics, GaiaError};
use leb128;
use std::io::{Cursor, Read, Seek, SeekFrom};

/// wasm lazy reader
#[derive(Debug)]
pub struct WasmReader<'config, R> {
    config: &'config WasmReadConfig,
    reader: R,
    view: WasmView,
    errors: Vec<GaiaError>,
}

impl<'config, R> WasmReader<'config, R> {
    pub fn new(reader: R, config: &'config WasmReadConfig) -> Self {
        Self { reader, view: WasmView::default(), errors: vec![], config }
    }

    fn check_magic_head(&self) -> Result<(), GaiaError> {
        if self.config.check_magic_head {
            // \0asm
            if self.view.magic_head != [0x00, 0x61, 0x73, 0x6D] {
                Err(GaiaError::invalid_data("Invalid magic number".to_string()))?
            }
        }
        Ok(())
    }
}

impl<'config, R: Read + Seek> WasmReader<'config, R> {
    pub fn read(mut self) -> GaiaDiagnostics<WasmView> {
        match self.read_to_end() {
            Ok(_) => GaiaDiagnostics { result: Ok(self.view), diagnostics: self.errors },
            Err(fatal) => GaiaDiagnostics { result: Err(fatal), diagnostics: self.errors },
        }
    }

    fn read_to_end(&mut self) -> Result<(), GaiaError> {
        todo!()
    }
}
