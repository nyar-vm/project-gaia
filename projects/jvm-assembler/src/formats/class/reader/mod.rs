use crate::{
    formats::class::{view::ClassInfo, ClassReadConfig},
    program::JvmProgram,
};
use byteorder::BigEndian;
use gaia_types::{BinaryReader, GaiaDiagnostics, GaiaError};
use std::{
    cell::{OnceCell, RefCell},
    io::{Read, Seek},
};

/// jvm class lazy reader
///
/// 可以只读取必要的部分
pub struct ClassReader<'config, R: Read + Seek> {
    config: &'config ClassReadConfig,
    reader: RefCell<BinaryReader<R, BigEndian>>,
    /// 缓存完整的 jvm 程序
    program: OnceCell<JvmProgram>,
    /// 记录一些头信息以及偏移量
    info: OnceCell<ClassInfo>,
}

impl<'config, R: Read + Seek> ClassReader<'config, R> {
    pub fn new(reader: R, config: &'config ClassReadConfig) -> Self {
        Self { config, reader: RefCell::new(BinaryReader::new(reader)), program: Default::default(), info: Default::default() }
    }

    pub fn get_program(&self) -> Result<&JvmProgram, GaiaError> {
        self.program.get_or_try_init(|| self.read_program())
    }
    pub fn get_view(&self) -> Result<&ClassInfo, GaiaError> {
        self.info.get_or_try_init(|| self.read_view())
    }
}

impl<'config, R: Read + Seek> ClassReader<'config, R> {
    pub fn read(mut self) -> GaiaDiagnostics<JvmProgram> {
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
    fn read_program(&self) -> Result<JvmProgram, GaiaError> {
        let reader = self.reader.borrow_mut();
        todo!()
    }

    fn read_view(&self) -> Result<ClassInfo, GaiaError> {
        let reader = self.reader.borrow_mut();
        todo!()
    }
}
