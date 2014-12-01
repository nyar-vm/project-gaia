use crate::{
    helpers::{read_coff_header, read_coff_object, read_section_headers, CoffReader},
    types::coff::{CoffHeader, CoffInfo, CoffObject, SectionHeader},
};
use gaia_types::{GaiaDiagnostics, GaiaError};
use std::io::{Read, Seek};

/// COFF 对象文件 (.obj) 结构，惰性读取器
#[derive(Debug)]
pub struct ObjReader<R> {
    /// 二进制读取器（已定位到对象文件起始位置）
    reader: R,
    lazy_header: Option<CoffHeader>,
    lazy_section_headers: Option<Vec<SectionHeader>>,
    lazy_object: Option<CoffObject>,
    lazy_info: Option<CoffInfo>,
    diagnostics: Vec<GaiaError>,
}

impl<R> ObjReader<R> {
    pub fn new(reader: R) -> Self {
        Self { reader, lazy_header: None, lazy_section_headers: None, lazy_object: None, lazy_info: None, diagnostics: vec![] }
    }
}

impl<W: Read + Seek> CoffReader<W> for ObjReader<W> {
    fn get_viewer(&mut self) -> &mut W {
        &mut self.reader
    }

    fn add_diagnostics(&mut self, error: impl Into<GaiaError>) {
        self.diagnostics.push(error.into());
    }

    fn get_coff_header(&mut self) -> Result<&CoffHeader, GaiaError> {
        if self.lazy_header.is_none() {
            let header = read_coff_header(self)?;
            self.lazy_header = Some(header);
        }
        Ok(self.lazy_header.as_ref().unwrap())
    }

    fn set_coff_header(&mut self, head: CoffHeader) -> Option<CoffHeader> {
        self.lazy_header.replace(head)
    }

    fn get_section_headers(&mut self) -> Result<&[SectionHeader], GaiaError> {
        if self.lazy_section_headers.is_none() {
            let headers = read_section_headers(self)?;
            self.lazy_section_headers = Some(headers);
        }
        Ok(self.lazy_section_headers.as_ref().unwrap())
    }

    fn set_section_headers(&mut self, headers: Vec<SectionHeader>) -> Vec<SectionHeader> {
        self.lazy_section_headers.replace(headers).unwrap_or_default()
    }

    fn get_coff_object(&mut self) -> Result<&CoffObject, GaiaError> {
        if self.lazy_object.is_none() {
            let object = read_coff_object(self)?;
            self.lazy_object = Some(object);
        }
        Ok(self.lazy_object.as_ref().unwrap())
    }

    fn set_coff_object(&mut self, object: CoffObject) -> Option<CoffObject> {
        self.lazy_object.replace(object)
    }

    fn get_coff_info(&mut self) -> Result<&CoffInfo, GaiaError> {
        if self.lazy_info.is_none() {
            let info = self.create_coff_info()?;
            self.lazy_info = Some(info);
        }
        Ok(self.lazy_info.as_ref().unwrap())
    }

    fn set_coff_info(&mut self, info: CoffInfo) -> Option<CoffInfo> {
        self.lazy_info.replace(info)
    }
}

impl<W: Read + Seek> ObjReader<W> {
    /// 读取完整的 COFF 对象（惰性读取，会缓存结果）
    pub fn read_object(mut self) -> GaiaDiagnostics<CoffObject> {
        match self.get_coff_object() {
            Ok(_) => match self.lazy_object {
                Some(object) => GaiaDiagnostics { result: Ok(object), diagnostics: self.diagnostics },
                None => unreachable!(),
            },
            Err(error) => GaiaDiagnostics { result: Err(error), diagnostics: self.diagnostics },
        }
    }

    /// 查看 COFF 对象文件信息
    pub fn view(&mut self) -> Result<CoffInfo, GaiaError> {
        Ok(self.get_coff_info()?.clone())
    }
}
