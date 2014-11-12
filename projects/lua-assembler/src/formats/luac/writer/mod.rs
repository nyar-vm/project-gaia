use crate::formats::luac::LuacWriteConfig;
use gaia_types::GaiaError;
use std::io::Write;

#[derive(Debug)]
pub struct LuacWriter<'config, W> {
    pub(crate) writer: W,
    pub(crate) config: &'config LuacWriteConfig,
    pub(crate) errors: Vec<GaiaError>,
}

impl LuacWriteConfig {
    pub fn as_writer<W: Write>(&self, writer: W) -> LuacWriter<W> {
        LuacWriter::new(writer, self)
    }
}

impl<'config, W> LuacWriter<'config, W> {
    pub fn new(writer: W, config: &'config LuacWriteConfig) -> Self {
        LuacWriter { writer, config, errors: vec![] }
    }
}
