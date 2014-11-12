use crate::formats::luac::{view::LuacView, LuacReadConfig};
use gaia_types::GaiaError;
use std::io::{Read, Seek, SeekFrom};

/// 负责解析 .pyc 文件到 LuacFile 的读取器
#[derive(Debug)]
pub struct LuacReader<'config, R> {
    pub(crate) reader: R,
    pub(crate) view: LuacView,
    // week errors
    pub(crate) errors: Vec<GaiaError>,
    pub(crate) config: &'config LuacReadConfig,
    pub(crate) offset: u64,
}

impl LuacReadConfig {
    pub fn as_reader<R: Read>(&self, reader: R) -> LuacReader<R> {
        LuacReader::new(reader, self)
    }
}

impl<'config, R> LuacReader<'config, R> {
    pub fn new(reader: R, config: &'config LuacReadConfig) -> Self {
        Self { reader, view: LuacView::default(), errors: vec![], config, offset: 0 }
    }

    pub fn get_offset(&self) -> u64 {
        self.offset
    }

    pub fn set_offset(&mut self, offset: u64) -> Result<(), GaiaError>
    where
        R: Seek,
    {
        self.reader.seek(SeekFrom::Start(offset))?;
        Ok(self.offset = offset)
    }

    fn check_magic_head(&mut self) -> Result<(), GaiaError> {
        let expected = [0; 4];
        if self.config.check_magic_head {
            if self.view.magic_head != expected {
                Err(GaiaError::invalid_magic_head(self.view.magic_head.to_vec(), expected.to_vec()))?;
            }
        }
        Ok(())
    }
}

impl<'config, R: Read + Seek> LuacReader<'config, R> {
    pub fn read(mut self) -> Result<LuacView, GaiaError> {
        self.set_offset(0)?;
        self.read_to_end()?;
        Ok(self.view)
    }

    pub fn read_to_end(&mut self) -> Result<(), GaiaError> {
        todo!()
    }
}
