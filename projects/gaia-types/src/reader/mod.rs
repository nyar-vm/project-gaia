pub use self::{token::Token, token_stream::TokenStream};
use byteorder::ReadBytesExt;
use std::io::Cursor;
use url::Url;
use serde::{Deserialize, Serialize};

mod token;
mod token_stream;

/// 二进制读取器，用于从实现了 ReadBytesExt trait 的类型中读取数据
///
/// 这是一个泛型结构体，可以包装任何实现了 ReadBytesExt trait 的类型，
/// 提供二进制数据的读取功能。
#[derive(Debug)]
pub struct BinaryReader<R: ReadBytesExt> {
    reader: Cursor<R>,
}

/// 源代码位置信息，表示代码在源文件中的位置
///
/// 该结构体用于跟踪源代码的位置信息，包括行号、列号等。
#[derive(Copy, Clone, Debug)]
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
        Self {
            line: 1,
            column: 1,
            url: None,
        }
    }
}