//! DLL 文件写入器模块
//!
//! 此模块提供将 PE 结构体写入 DLL 二进制文件的功能，与 reader 模块相对应。

use std::io::{Seek, Write};

use crate::helpers::PeWriter;

/// DLL 文件写入器
#[derive(Debug)]
pub struct DllWriter<W> {
    writer: W,
}

impl<W> DllWriter<W> {
    /// 创建新的 DLL 写入器
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    /// 完成写入并返回底层写入器
    pub fn finish(self) -> W {
        self.writer
    }
}

impl<W: Write + Seek> PeWriter<W> for DllWriter<W> {
    fn get_writer(&mut self) -> &mut W {
        &mut self.writer
    }
}
