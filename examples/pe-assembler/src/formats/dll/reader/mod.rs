use crate::{
    helpers::{
        pe_reader::{read_pe_head, read_pe_program, read_pe_section_headers},
        PeReader,
    },
    types::{PeHeader, PeInfo, PeProgram, SectionHeader},
};
use gaia_types::{GaiaDiagnostics, GaiaError};
use std::io::{Read, Seek, SeekFrom};

/// DLL 结构，惰性读取器
#[derive(Debug)]
pub struct DllReader<R> {
    reader: R,
    dll_header: Option<PeHeader>,
    dll_info: Option<PeInfo>,
    dll_section_headers: Option<Vec<SectionHeader>>,
    dll_program: Option<PeProgram>,
    errors: Vec<GaiaError>,
}

impl<R: Read> Read for DllReader<R> {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        self.reader.read(buffer)
    }
}

impl<R: Seek> Seek for DllReader<R> {
    fn seek(&mut self, position: SeekFrom) -> std::io::Result<u64> {
        self.reader.seek(position)
    }
}

impl<R> DllReader<R> {
    pub fn new(reader: R) -> Self {
        Self { reader, dll_header: None, dll_section_headers: None, dll_program: None, dll_info: None, errors: vec![] }
    }
    pub fn finish(mut self) -> GaiaDiagnostics<PeProgram>
    where
        R: Read + Seek,
    {
        if self.dll_program.is_none() {
            if let Err(e) = read_pe_program(&mut self) {
                return GaiaDiagnostics { result: Err(e), diagnostics: self.errors };
            }
        }
        unsafe {
            let exe = self.dll_program.unwrap_unchecked();
            GaiaDiagnostics { result: Ok(exe), diagnostics: self.errors }
        }
    }
}

impl<R: Read + Seek> PeReader<R> for DllReader<R> {
    fn get_viewer(&mut self) -> &mut R {
        &mut self.reader
    }

    fn add_diagnostics(&mut self, error: impl Into<GaiaError>) {
        self.errors.push(error.into());
    }

    fn get_section_headers(&mut self) -> Result<&[SectionHeader], GaiaError> {
        if self.dll_section_headers.is_none() {
            self.dll_section_headers = Some(read_pe_section_headers(self)?);
        }
        unsafe { Ok(self.dll_section_headers.as_ref().unwrap_unchecked()) }
    }

    fn get_pe_header(&mut self) -> Result<&PeHeader, GaiaError> {
        if self.dll_header.is_none() {
            self.dll_header = Some(read_pe_head(self)?)
        }
        unsafe { Ok(self.dll_header.as_ref().unwrap_unchecked()) }
    }

    fn get_program(&mut self) -> Result<&PeProgram, GaiaError> {
        if self.dll_program.is_none() {
            self.dll_program = Some(read_pe_program(self)?);
        }
        unsafe { Ok(self.dll_program.as_ref().unwrap_unchecked()) }
    }
}
