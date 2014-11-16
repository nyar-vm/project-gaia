use crate::{
    helpers::PeReader,
    types::{PeHeader, PeInfo, PeProgram, SectionHeader},
};
use byteorder::LittleEndian;
use gaia_types::{BinaryReader, GaiaDiagnostics, GaiaError};
use std::io::{Read, Seek};

/// EXE 结构，惰性读取器
#[derive(Debug)]
pub struct ExeReader<R> {
    /// 二进制读取器（已定位到 DOS 头起始位置）
    viewer: BinaryReader<R, LittleEndian>,
    lazy_header: Option<PeHeader>,
    lazy_section_headers: Option<Vec<SectionHeader>>,
    lazy_program: Option<PeProgram>,
    lazy_info: Option<PeInfo>,
    diagnostics: Vec<GaiaError>,
}

impl<R> ExeReader<R> {
    pub fn new(reader: R) -> Self {
        Self {
            viewer: BinaryReader::new(reader),
            lazy_header: None,
            lazy_section_headers: None,
            lazy_program: None,
            lazy_info: None,
            diagnostics: vec![],
        }
    }
}

impl<W: Read + Seek> PeReader<W> for ExeReader<W> {
    fn get_viewer(&mut self) -> &mut BinaryReader<W, LittleEndian> {
        &mut self.viewer
    }

    fn add_diagnostics(&mut self, _error: impl Into<GaiaError>) {
        // 暂时不实现，避免编译错误
    }

    fn get_cached_section_headers(&self) -> Option<&Vec<SectionHeader>> {
        self.lazy_section_headers.as_ref()
    }

    fn set_cached_section_headers(&mut self, headers: Vec<SectionHeader>) {
        self.lazy_section_headers = Some(headers);
    }

    fn read_header_once(&mut self) -> Result<&PeHeader, GaiaError> {
        if self.lazy_header.is_none() {
            self.lazy_header = Some(self.read_header_force()?);
        }
        match self.lazy_header.as_ref() {
            Some(s) => Ok(s),
            None => unreachable!(),
        }
    }

    fn read_program_once(&mut self) -> Result<&PeProgram, GaiaError> {
        if self.lazy_program.is_none() {
            self.lazy_program = Some(self.read_program_force()?);
        }
        match self.lazy_program.as_ref() {
            Some(s) => Ok(s),
            None => unreachable!(),
        }
    }
}

impl<W: Read + Seek> ExeReader<W> {
    /// 读取完整的 PE 程序（惰性读取，会缓存结果）
    pub fn read_program(mut self) -> GaiaDiagnostics<PeProgram> {
        match self.read_program_once() {
            Ok(_) => match self.lazy_program {
                Some(program) => GaiaDiagnostics { result: Ok(program), diagnostics: self.diagnostics },
                None => unreachable!(),
            },
            Err(error) => GaiaDiagnostics { result: Err(error), diagnostics: self.diagnostics },
        }
    }

    /// 查看 EXE 文件信息
    pub fn view(&mut self) -> Result<PeInfo, GaiaError> {
        if let Some(ref info) = self.lazy_info {
            return Ok(info.clone());
        }

        let info = self.create_pe_info()?;
        self.lazy_info = Some(info.clone());
        Ok(info)
    }
}
