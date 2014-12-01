//! JVM Class 文件写入器
//!
//! 这个模块实现了将 JVM 程序转换为 Class 文件字节码的功能。

use crate::program::*;
use byteorder::BigEndian;
use gaia_types::{BinaryWriter, GaiaDiagnostics, Result};
use std::io::Write;

/// Class 文件写入器
pub struct ClassWriter<W> {
    /// 二进制汇编器
    writer: BinaryWriter<W, BigEndian>,
}

impl<W> ClassWriter<W> {
    /// 创建新的 Class 写入器
    pub fn new(writer: W) -> Self {
        Self { writer: BinaryWriter::new(writer) }
    }

    /// 完成写入并返回底层写入器
    pub fn finish(self) -> W {
        self.writer.finish()
    }
}

impl<W: Write> ClassWriter<W> {
    /// 将 ClassView 写入为二进制 Class 格式
    pub fn write(mut self, program: &JvmProgram) -> GaiaDiagnostics<W> {
        match self.write_class_file(program) {
            Ok(_) => GaiaDiagnostics::success(self.finish()),
            Err(error) => GaiaDiagnostics::failure(error),
        }
    }

    /// 写入 Class 文件
    fn write_class_file(&mut self, _program: &JvmProgram) -> Result<()> {
        todo!()
    }
}
