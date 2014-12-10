#![doc = include_str!("readme.md")]
use crate::instructions::PythonInstruction;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// 表示 Python 字节码的版本。
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PythonVersion {
    #[default]
    /// 未知 Python 版本。
    Unknown,
    /// Python 3.7 版本。
    Python3_7,
    /// Python 3.8 版本。
    Python3_8,
    /// Python 3.9 版本。
    Python3_9,
    /// Python 3.10 版本。
    Python3_10,
    /// Python 3.11 版本。
    Python3_11,
    /// Python 3.12 版本。
    Python3_12,
    /// Python 3.13 版本。
    Python3_13,
    /// Python 3.14 版本。
    Python3_14,
}

impl PythonVersion {
    /// 根据 .pyc 头部的 MAGIC_NUMBER 推断 Python 版本
    /// 参考 importlib.util.MAGIC_NUMBER 值和 CPython 源码
    pub fn from_magic(magic: [u8; 4]) -> Self {
        match magic {
            // Python 3.7.x - 0x420d0d0a
            [66, 13, 13, 10] => PythonVersion::Python3_7,
            // Python 3.8.x - 0x550d0d0a
            [85, 13, 13, 10] => PythonVersion::Python3_8,
            // Python 3.9.x - 0x610d0d0a
            [97, 13, 13, 10] => PythonVersion::Python3_9,
            // Python 3.10.x - 0x700d0d0a
            [112, 13, 13, 10] => PythonVersion::Python3_10,
            // Python 3.11.x - 0xa80d0d0a
            [168, 13, 13, 10] => PythonVersion::Python3_11,
            // Python 3.12.x - 0xcb0d0d0a
            [203, 13, 13, 10] => PythonVersion::Python3_12,
            // Python 3.13.x - 0xf30d0d0a
            [243, 13, 13, 10] => PythonVersion::Python3_13,
            // Python 3.14.x - 0x2b0e0d0a (可能在正式版本中变化)
            [43, 14, 13, 10] => PythonVersion::Python3_14,
            _ => PythonVersion::Unknown,
        }
    }

    /// 将 Python 版本转换为对应的 MAGIC_NUMBER
    pub fn as_magic(&self) -> [u8; 4] {
        match self {
            // Python 3.7.x - 0x420d0d0a
            PythonVersion::Python3_7 => [66, 13, 13, 10],
            // Python 3.8.x - 0x550d0d0a
            PythonVersion::Python3_8 => [85, 13, 13, 10],
            // Python 3.9.x - 0x610d0d0a
            PythonVersion::Python3_9 => [97, 13, 13, 10],
            // Python 3.10.x - 0x700d0d0a
            PythonVersion::Python3_10 => [112, 13, 13, 10],
            // Python 3.11.x - 0xa80d0d0a
            PythonVersion::Python3_11 => [168, 13, 13, 10],
            // Python 3.12.x - 0xcb0d0d0a
            PythonVersion::Python3_12 => [203, 13, 13, 10],
            // Python 3.13.x - 0xf30d0d0a
            PythonVersion::Python3_13 => [243, 13, 13, 10],
            // Python 3.14.x - 0x2b0e0d0a (可能在正式版本中变化)
            PythonVersion::Python3_14 => [43, 14, 13, 10],
            PythonVersion::Unknown => [0, 0, 0, 0],
        }
    }
}

/// .pyc 文件的头部信息。
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PycHeader {
    /// Magic number identifying the Python version.
    pub magic: [u8; 4],
    /// Bit flags for the .pyc file.
    pub flags: u32,
    /// Timestamp of the .pyc file creation.
    pub timestamp: u32,
    /// Size of the .pyc file.
    pub size: u32,
}

impl Default for PycHeader {
    fn default() -> Self {
        Self { magic: [0; 4], flags: 0, timestamp: 0, size: 0 }
    }
}

/// Upvalue 信息
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Upvalue {
    /// 若在栈中则为1，若在外部upvalue中则为0
    pub in_stack: u8,
    /// 寄存器或upvalue的索引
    pub idx: u8,
    /// 调试信息用的名称
    pub name: String,
}

/// 局部变量信息
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalVar {
    /// 局部变量的名称。
    pub name: String,
    /// 局部变量作用域的起始程序计数器。
    pub start_pc: u32,
    /// 局部变量作用域的结束程序计数器。
    pub end_pc: u32,
}

/// 表示不同类型的 Python 对象。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum PythonObject {
    /// Python 字符串对象。
    Str(String),
    /// Python 整数对象（32位）。
    Int(i32),
    /// Python 整数对象（64位）。
    Integer(i64),
    /// Python 浮点数对象。
    Float(f64),
    /// Python 布尔对象。
    Bool(bool),
    /// 另一个 Python 字符串对象（与 Str 重复，但为了兼容性保留）。
    String(String),
    /// Python 列表对象。
    List(Vec<PythonObject>),
    /// Python 元组对象。
    Tuple(Vec<PythonObject>),
    /// Python 字节数组对象。
    Bytes(Vec<u8>),
    /// Python 代码对象。
    Code(PythonCodeObject),
    #[default]
    /// 表示 Python 的 None 对象。
    None,
}

/// Python 代码对象，包含字节码指令、常量、变量等信息。
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct PythonCodeObject {
    /// 代码对象的名称。
    pub name: String,
    /// 代码对象的限定名称。
    pub qualname: String,
    /// 源文件名。
    pub source_name: String,
    /// 代码对象的起始行号。
    pub first_line: u32,
    /// 代码对象的结束行号。
    pub last_line: u32,
    /// 参数数量。
    pub co_argcount: u8,
    /// 仅限位置参数的数量。
    pub co_posonlyargcount: u8,
    /// 仅限关键字参数的数量。
    pub co_kwonlyargcount: u8,
    /// 局部变量数量。
    pub co_nlocals: u8,
    /// 所需的最大栈大小。
    pub co_stacksize: u8,
    /// 代码对象标志位。
    pub co_flags: u32,
    /// 字节码指令。
    pub co_code: Vec<PythonInstruction>,
    /// 代码对象使用的常量。
    pub co_consts: Vec<PythonObject>,
    /// 代码对象使用的名称。
    pub co_names: Vec<String>,
    /// 局部变量和自由变量名称。
    pub co_localsplusnames: Vec<String>,
    /// 局部变量和自由变量类型。
    pub co_localspluskinds: Vec<u8>,
    /// 用于调试的行号信息。
    pub co_linetable: Vec<u8>,
    /// 异常处理表。
    pub co_exceptiontable: Vec<u8>,
}

/// Python .pyc 程序的高层语义定义（以指令序列为核心）
/// Python .pyc 程序的高层语义定义（以指令序列为核心）
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct PythonProgram {
    /// .pyc 文件的头部。
    pub header: PycHeader,
    /// Python 程序的主代码对象。
    pub code_object: PythonCodeObject,
    /// 与 .pyc 文件关联的 Python 版本。
    pub version: PythonVersion,
}
