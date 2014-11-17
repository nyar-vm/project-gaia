#![doc = include_str!("readme.md")]

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
pub struct BinaryWriter<W, E> {
    writer: W,
    endian: PhantomData<E>,
}

impl<W: Write, E> Write for BinaryWriter<W, E> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.writer.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}

impl<W: Seek, E> Seek for BinaryWriter<W, E> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.writer.seek(pos)
    }
}

impl<W: Write, E> BinaryWriter<W, E> {
    /// 创建一个新的二进制写入器
    pub fn new(writer: W) -> Self {
        Self { writer, endian: PhantomData }
    }

    /// 获取内部写入器
    pub fn finish(self) -> W {
        self.writer
    }

    /// 将一个 u8 值写入到字节流中。
    ///
    /// # 参数
    ///
    /// * `value` - 要写入的 u8 值。
    pub fn write_u8(&mut self, value: u8) -> std::io::Result<()>
    where
        W: Write,
    {
        self.writer.write_u8(value)
    }

    /// 将一个 u16 值写入到字节流中。
    ///
    /// # 参数
    ///
    /// * `value` - 要写入的 u16 值。
    pub fn write_u16(&mut self, value: u16) -> std::io::Result<()>
    where
        W: Write,
        E: ByteOrder,
    {
        self.writer.write_u16::<E>(value)
    }

    /// 将一个 u32 值写入到字节流中。
    ///
    /// # 参数
    ///
    /// * `value` - 要写入的 u32 值。
    pub fn write_u32(&mut self, value: u32) -> std::io::Result<()>
    where
        W: Write,
        E: ByteOrder,
    {
        self.writer.write_u32::<E>(value)
    }

    /// 将一个 u64 值写入到字节流中。
    ///
    /// # 参数
    ///
    /// * `value` - 要写入的 u64 值。
    pub fn write_u64(&mut self, value: u64) -> std::io::Result<()>
    where
        W: Write,
        E: ByteOrder,
    {
        self.writer.write_u64::<E>(value)
    }

    /// 将字节数组写入到字节流中。
    ///
    /// # 参数
    ///
    /// * `bytes` - 要写入的字节数组。
    pub fn write_bytes(&mut self, bytes: &[u8]) -> std::io::Result<()>
    where
        W: Write,
    {
        self.writer.write_all(bytes)
    }
}
