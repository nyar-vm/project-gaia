use crate::{
    helpers::CoffReader,
    types::coff::{CoffHeader, CoffInfo, CoffObject, SectionHeader},
};
use byteorder::LittleEndian;
use gaia_types::{BinaryReader, GaiaDiagnostics, GaiaError};
use std::{
    fs::File,
    io::{Read, Seek},
    path::Path,
};

/// COFF 对象文件 (.obj) 结构，惰性读取器
#[derive(Debug)]
pub struct ObjReader<R> {
    /// 二进制读取器（已定位到对象文件起始位置）
    viewer: BinaryReader<R, LittleEndian>,
    lazy_header: Option<CoffHeader>,
    lazy_section_headers: Option<Vec<SectionHeader>>,
    lazy_object: Option<CoffObject>,
    lazy_info: Option<CoffInfo>,
    diagnostics: Vec<GaiaError>,
}

impl<R> ObjReader<R> {
    pub fn new(reader: R) -> Self {
        Self {
            viewer: BinaryReader::new(reader),
            lazy_header: None,
            lazy_section_headers: None,
            lazy_object: None,
            lazy_info: None,
            diagnostics: vec![],
        }
    }
}

impl<W: Read + Seek> CoffReader<W> for ObjReader<W> {
    fn get_viewer(&mut self) -> &mut BinaryReader<W, LittleEndian> {
        &mut self.viewer
    }

    fn add_diagnostics(&mut self, error: impl Into<GaiaError>) {
        self.diagnostics.push(error.into());
    }

    fn get_cached_section_headers(&self) -> Option<&Vec<SectionHeader>> {
        self.lazy_section_headers.as_ref()
    }

    fn set_cached_section_headers(&mut self, headers: Vec<SectionHeader>) {
        self.lazy_section_headers = Some(headers);
    }

    fn read_header_once(&mut self) -> Result<&CoffHeader, GaiaError> {
        if self.lazy_header.is_none() {
            self.lazy_header = Some(self.read_header_force()?);
        }
        match self.lazy_header.as_ref() {
            Some(s) => Ok(s),
            None => unreachable!(),
        }
    }

    fn read_object_once(&mut self) -> Result<&CoffObject, GaiaError> {
        if self.lazy_object.is_none() {
            self.lazy_object = Some(self.read_object_force()?);
        }
        match self.lazy_object.as_ref() {
            Some(s) => Ok(s),
            None => unreachable!(),
        }
    }
}

impl<W: Read + Seek> ObjReader<W> {
    /// 读取完整的 COFF 对象（惰性读取，会缓存结果）
    pub fn read_object(mut self) -> GaiaDiagnostics<CoffObject> {
        match self.read_object_once() {
            Ok(_) => match self.lazy_object {
                Some(object) => GaiaDiagnostics { result: Ok(object), diagnostics: self.diagnostics },
                None => unreachable!(),
            },
            Err(error) => GaiaDiagnostics { result: Err(error), diagnostics: self.diagnostics },
        }
    }

    /// 查看 COFF 对象文件信息
    pub fn view(&mut self) -> Result<CoffInfo, GaiaError> {
        if let Some(ref info) = self.lazy_info {
            return Ok(info.clone());
        }

        let info = self.create_coff_info()?;
        self.lazy_info = Some(info.clone());
        Ok(info)
    }
}

impl ObjReader<File> {
    /// 从文件创建 COFF 读取器
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, GaiaError> {
        let file = File::open(path).map_err(|e| GaiaError::invalid_data(&format!("无法打开文件: {}", e)))?;
        Ok(Self::new(file))
    }
}

// 便利函数，保持向后兼容
pub fn read_coff_from_file<P: AsRef<Path>>(path: P) -> Result<CoffObject, GaiaError> {
    let reader = ObjReader::<File>::from_file(path)?;
    let result = reader.read_object();
    result.result
}
