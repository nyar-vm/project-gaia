//! Writer 模块，提供文本和二进制数据写入功能
//!
//! 该模块包含用于写入文本和二进制数据的写入器类型。

use byteorder::WriteBytesExt;
use std::fmt::Write;

/// 文本写入器，用于写入文本数据
///
/// 这是一个简单的文本写入器结构体，目前为占位符实现，
/// 后续可以扩展具体的文本写入功能。
#[derive(Copy, Clone, Debug)]
pub struct TextWriter<W: Write> {
    writer: W,
}

/// 二进制写入器，用于从实现了 WriteBytesExt trait 的类型中写入数据
///
/// 这是一个泛型结构体，可以包装任何实现了 WriteBytesExt trait 的类型，
/// 提供二进制数据的写入功能。
#[derive(Copy, Clone, Debug)]
pub struct BinaryWriter<W: WriteBytesExt> {
    writer: W,
}
