#![doc = include_str!("readme.md")]

pub use self::{token::Token, token_stream::TokenStream};
use crate::{GaiaDiagnostics, GaiaError};
use byteorder::{ByteOrder, ReadBytesExt};
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Seek, SeekFrom},
    marker::PhantomData,
};
use url::Url;

mod token;
mod token_stream;

/// 二进制读取器，用于从实现了 ReadBytesExt trait 的类型中读取数据
///
/// 这是一个泛型结构体，可以包装任何实现了 ReadBytesExt trait 的类型，
/// 提供二进制数据的读取功能。
#[derive(Debug)]
pub struct BinaryReader<R, E> {
    reader: R,
    position: u64,
    endian: PhantomData<E>,
    errors: Vec<GaiaError>,
}

impl<R: Read, E> Read for BinaryReader<R, E> {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        let bytes_read = self.reader.read(buffer)?;
        self.position += bytes_read as u64;
        Ok(bytes_read)
    }
}

impl<R: Seek, E> Seek for BinaryReader<R, E> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        let new_position = self.reader.seek(pos)?;
        self.position = new_position;
        Ok(new_position)
    }
}

impl<R, E> BinaryReader<R, E> {
    /// 创建新的二进制读取器
    ///
    /// # Arguments
    /// * `lexer` - 要读取的数据
    ///
    /// # Returns
    /// 返回新的 BinaryReader 实例
    pub fn new(reader: R) -> Self {
        Self { reader, position: 0, endian: Default::default(), errors: vec![] }
    }

    /// 获取当前读取位置
    ///
    /// # Returns
    /// 返回当前的字节偏移位置
    pub fn get_position(&self) -> u64 {
        self.position
    }

    /// 设置读取位置
    ///
    /// 注意：如果底层reader不支持Seek操作，此函数将返回错误
    ///
    /// # Arguments
    /// * `pos` - 新的读取位置
    ///
    /// # Returns
    /// 返回操作结果
    pub fn set_position(&mut self, position: u64) -> Result<u64, GaiaError>
    where
        R: Seek,
    {
        self.reader.seek(SeekFrom::Start(position))?;
        self.position = position;
        Ok(position)
    }
    /// 完成读取器并返回结果。
    pub fn finish(self) -> GaiaDiagnostics<R> {
        GaiaDiagnostics { result: Ok(self.reader), diagnostics: self.errors }
    }

    /// 获取并清空当前累积的错误列表。
    /// 该方法会返回所有在读取过程中遇到的错误，并重置内部的错误存储。
    pub fn take_errors(&mut self) -> Vec<GaiaError> {
        std::mem::take(&mut self.errors)
    }
}

impl<R: ReadBytesExt, E: ByteOrder> BinaryReader<R, E> {
    /// 读取 u8
    ///
    /// # Returns
    /// 返回读取的 u8 值或 IO 错误
    pub fn read_u8(&mut self) -> std::io::Result<u8> {
        let value = self.reader.read_u8()?;
        self.position += 1;
        Ok(value)
    }

    /// 读取 u16
    ///
    /// # Returns
    /// 返回读取的 u16 值或 IO 错误
    pub fn read_u16(&mut self) -> std::io::Result<u16> {
        let value = self.reader.read_u16::<E>()?;
        self.position += 2;
        Ok(value)
    }

    /// 读取 i16
    ///
    /// # Returns
    /// 返回读取的 i16 值或 IO 错误
    pub fn read_i16(&mut self) -> std::io::Result<i16> {
        let value = self.reader.read_i16::<E>()?;
        self.position += 2;
        Ok(value)
    }

    /// 读取 u32
    ///
    /// # Returns
    /// 返回读取的 u32 值或 IO 错误
    pub fn read_u32(&mut self) -> std::io::Result<u32> {
        let value = self.reader.read_u32::<E>()?;
        self.position += 4;
        Ok(value)
    }

    /// 读取 u64
    ///
    /// # Returns
    /// 返回读取的 u64 值或 IO 错误
    pub fn read_u64(&mut self) -> std::io::Result<u64> {
        let value = self.reader.read_u64::<E>()?;
        self.position += 8;
        Ok(value)
    }

    /// 读取 i32
    ///
    /// # Returns
    /// 返回读取的 i32 值或 IO 错误
    pub fn read_i32(&mut self) -> std::io::Result<i32> {
        let value = self.reader.read_i32::<E>()?;
        self.position += 4;
        Ok(value)
    }

    /// 读取 i64
    ///
    /// # Returns
    /// 返回读取的 i64 值或 IO 错误
    pub fn read_i64(&mut self) -> std::io::Result<i64> {
        let value = self.reader.read_i64::<E>()?;
        self.position += 8;
        Ok(value)
    }

    /// 读取 f32
    ///
    /// # Returns
    /// 返回读取的 f32 值或 IO 错误
    pub fn read_f32(&mut self) -> std::io::Result<f32> {
        let value = self.reader.read_f32::<E>()?;
        self.position += 4;
        Ok(value)
    }

    /// 读取 f64
    ///
    /// # Returns
    /// 返回读取的 f64 值或 IO 错误
    pub fn read_f64(&mut self) -> std::io::Result<f64> {
        let value = self.reader.read_f64::<E>()?;
        self.position += 8;
        Ok(value)
    }

    /// 读取指定长度的字节数组
    ///
    /// # Arguments
    /// * `len` - 要读取的字节数
    ///
    /// # Returns
    /// 返回读取的字节数组或 IO 错误
    pub fn read_bytes(&mut self, len: usize) -> std::io::Result<Vec<u8>> {
        let mut buf = vec![0u8; len];
        self.reader.read_exact(&mut buf)?;
        self.position += len as u64;
        Ok(buf)
    }

    /// 读取固定长度的字节数组
    ///
    /// # Returns
    /// 返回读取的字节数组或 IO 错误
    pub fn read_array<const N: usize>(&mut self) -> std::io::Result<[u8; N]> {
        let mut buf = [0u8; N];
        self.reader.read_exact(&mut buf)?;
        self.position += N as u64;
        Ok(buf)
    }

    /// 跳过指定数量的字节
    ///
    /// 注意：如果底层reader不支持Seek操作，此函数将返回错误
    ///
    /// # Arguments
    /// * `count` - 要跳过的字节数
    ///
    /// # Returns
    /// 返回操作结果
    pub fn skip(&mut self, count: u64) -> std::io::Result<u64>
    where
        R: Seek,
    {
        let new_pos = self.reader.seek(SeekFrom::Current(count as i64))?;
        self.position = new_pos;
        Ok(new_pos)
    }

    /// 读取 LEB128 编码的无符号 32 位整数
    ///
    /// # Returns
    /// 返回读取的 u32 值或 IO 错误
    pub fn read_u32_leb128(&mut self) -> std::io::Result<u32> {
        let mut result = 0u32;
        let mut shift = 0;
        
        loop {
            let byte = self.read_u8()?;
            result |= ((byte & 0x7F) as u32) << shift;
            
            if byte & 0x80 == 0 {
                break;
            }
            
            shift += 7;
            if shift >= 32 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "LEB128 value too large for u32"
                ));
            }
        }
        
        Ok(result)
    }

    /// 读取 LEB128 编码的有符号 32 位整数
    ///
    /// # Returns
    /// 返回读取的 i32 值或 IO 错误
    pub fn read_i32_leb128(&mut self) -> std::io::Result<i32> {
        let mut result = 0i32;
        let mut shift = 0;
        let mut byte;
        
        loop {
            byte = self.read_u8()?;
            result |= ((byte & 0x7F) as i32) << shift;
            shift += 7;
            
            if byte & 0x80 == 0 {
                break;
            }
            
            if shift >= 32 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "LEB128 value too large for i32"
                ));
            }
        }
        
        // 符号扩展
        if shift < 32 && (byte & 0x40) != 0 {
            result |= !0 << shift;
        }
        
        Ok(result)
    }

    /// 读取 LEB128 编码的有符号 64 位整数
    ///
    /// # Returns
    /// 返回读取的 i64 值或 IO 错误
    pub fn read_i64_leb128(&mut self) -> std::io::Result<i64> {
        let mut result = 0i64;
        let mut shift = 0;
        let mut byte;
        
        loop {
            byte = self.read_u8()?;
            result |= ((byte & 0x7F) as i64) << shift;
            shift += 7;
            
            if byte & 0x80 == 0 {
                break;
            }
            
            if shift >= 64 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "LEB128 value too large for i64"
                ));
            }
        }
        
        // 符号扩展
        if shift < 64 && (byte & 0x40) != 0 {
            result |= !0 << shift;
        }
        
        Ok(result)
    }
}

impl<R, E> BinaryReader<R, E> {
    /// 计算 LEB128 编码值的字节长度（静态方法）
    ///
    /// # Arguments
    /// * `value` - 要计算长度的值
    ///
    /// # Returns
    /// 返回编码后的字节长度
    pub fn leb128_size(mut value: u32) -> u32 {
        let mut size = 0;
        loop {
            value >>= 7;
            size += 1;
            if value == 0 {
                break;
            }
        }
        size
    }
}

/// 源代码位置信息，表示代码在源文件中的位置
///
/// 该结构体用于跟踪源代码的位置信息，包括行号、列号等。
#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SourcePosition {
    /// 行号，从 1 开始计数
    ///
    /// 表示当前位置所在的行号，第一行为 1。
    pub line: u32,
    /// 列号，从 1 开始计数
    ///
    /// 表示当前位置所在的列号，第一列为 1。
    pub column: u32,
    /// 字节偏移量，从 0 开始计数
    ///
    /// 表示从文件开始到当前位置的字节偏移量。
    pub offset: usize,
    /// 长度，表示该位置所覆盖的字节数
    ///
    /// 通常用于表示标记或符号的长度。
    pub length: usize,
}

/// 源代码位置，包含文件 URL 和位置信息
///
/// 该结构体扩展了 SourcePosition，增加了文件 URL 信息，
/// 可以表示代码在特定文件中的位置。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceLocation {
    /// 行号，从 1 开始计数
    ///
    /// 表示当前位置所在的行号。
    pub line: u32,
    /// 列号，从 1 开始计数
    ///
    /// 表示当前位置所在的列号。
    pub column: u32,
    /// 源文件的 URL，可选
    ///
    /// 如果存在，表示包含该代码的文件的 URL 或路径。
    /// 可以是文件系统路径或网络 URL。
    pub url: Option<Url>,
}

impl Default for SourceLocation {
    fn default() -> Self {
        Self { line: 1, column: 1, url: None }
    }
}
