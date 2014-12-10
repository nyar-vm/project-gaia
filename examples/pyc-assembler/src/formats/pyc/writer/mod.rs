use crate::{formats::pyc::PycWriteConfig, program::PythonProgram};
use byteorder::LittleEndian;
use gaia_types::{BinaryWriter, GaiaError};
use std::io::Write;

/// PycWriter 负责将 PythonProgram 写入到输出流中
#[derive(Debug)]
pub struct PycWriter<W> {
    writer: BinaryWriter<W, LittleEndian>,
    config: PycWriteConfig,
}

impl<W: Write> PycWriter<W> {
    /// 创建一个新的 PycWriter 实例
    pub fn new(writer: W, config: PycWriteConfig) -> Self {
        Self { writer: BinaryWriter::new(writer), config }
    }

    /// 将 PythonProgram 的内容写入到输出流中。
    pub fn write(&mut self, program: &PythonProgram) -> Result<usize, GaiaError> {
        let mut bytes_written = 0;

        // 写入 .pyc 文件头
        self.writer.write_all(&program.header.magic)?;
        self.writer.write_u32(program.header.flags)?;
        self.writer.write_u32(program.header.timestamp)?;
        self.writer.write_u32(program.header.size)?;
        bytes_written += 16;

        // 使用 MarshalWriter 序列化 code_object
        let mut marshal_data = Vec::new();
        let mut marshal_writer = super::marshal::MarshalWriter::new(&mut marshal_data, program.version);
        marshal_writer.write_code_object(&program.code_object).map_err(|e| GaiaError::from(e))?;

        self.writer.write_all(&marshal_data)?;
        bytes_written += marshal_data.len();

        Ok(bytes_written)
    }
}
