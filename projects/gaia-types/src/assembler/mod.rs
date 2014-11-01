//! Writer 模块，提供文本和二进制数据写入功能
//!
//! 该模块包含用于写入文本和二进制数据的写入器类型。

use byteorder::{ByteOrder, WriteBytesExt};
use std::{
    io::{Seek, SeekFrom, Write},
    marker::PhantomData,
};

/// 二进制写入器，用于从实现了 WriteBytesExt trait 的类型中写入数据
///
/// 这是一个泛型结构体，可以包装任何实现了 WriteBytesExt trait 的类型，
/// 提供二进制数据的写入功能。
#[derive(Copy, Clone, Debug)]
pub struct BinaryAssembler<W, E> {
    writer: W,
    endian: PhantomData<E>,
}

impl<W: Write, E> Write for BinaryAssembler<W, E> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.writer.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}

impl<W: Seek, E> Seek for BinaryAssembler<W, E> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.writer.seek(pos)
    }
}

impl<W, E> BinaryAssembler<W, E> {
    /// 创建一个新的二进制写入器
    pub fn new(writer: W) -> Self {
        Self { writer, endian: PhantomData }
    }

    pub fn write_u16(&mut self, value: u16) -> std::io::Result<()>
    where
        W: Write,
        E: ByteOrder,
    {
        self.writer.write_u16::<E>(value)
    }

    pub fn write_u32(&mut self, value: u32) -> std::io::Result<()>
    where
        W: Write,
        E: ByteOrder,
    {
        self.writer.write_u32::<E>(value)
    }

    pub fn write_u64(&mut self, value: u64) -> std::io::Result<()>
    where
        W: Write,
        E: ByteOrder,
    {
        self.writer.write_u64::<E>(value)
    }
}
