use crate::formats::pyc::PycWriteConfig;
use crate::program::PythonProgram;
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
        Self {
            writer: BinaryWriter::new(writer),
            config,
        }
    }

    /// 将 PythonProgram 的内容写入到输出流中。
    pub fn write(&mut self, program: &PythonProgram) -> Result<usize, GaiaError> {
        // 写入 .pyc 文件头
        self.writer.write_all(&program.header.magic)?;
        self.writer.write_u32(program.header.flags)?;
        self.writer.write_u32(program.header.timestamp)?;
        self.writer.write_u32(program.header.size)?;

        // TODO: 实现 marshal 数据的序列化
        // 这里需要将 PythonProgram 的 code_object 序列化为 marshal 格式
        
        Ok(16) // 暂时返回头部大小
    }
}
