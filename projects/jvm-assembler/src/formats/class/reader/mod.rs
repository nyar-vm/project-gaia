use crate::program::JvmProgram;
use byteorder::BigEndian;
use gaia_types::{BinaryReader, GaiaDiagnostics};
use std::io::{Read, Seek};

/// jvm class lazy reader
///
/// 可以只读取必要的部分
pub struct ClassReader<R> {
    reader: BinaryReader<R, BigEndian>,
}

impl<R> ClassReader<R> {
    pub fn new(reader: R) -> Self {
        Self { reader: BinaryReader::new(reader) }
    }

    pub fn finish(self) -> R {
        self.reader.finish()
    }
}

impl<R: Read> ClassReader<R> {
    pub fn read(&mut self) -> GaiaDiagnostics<JvmProgram>
    where
        R: Seek,
    {
        todo!()
    }
}
