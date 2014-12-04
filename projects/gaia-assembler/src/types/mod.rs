//! Gaia 汇编器核心类型定义

use serde::{Deserialize, Serialize};

/// Gaia 类型系统
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GaiaType {
    /// 8位有符号整数
    Integer8,
    /// 16位有符号整数
    Integer16,
    /// 32位有符号整数
    Integer32,
    /// 64位有符号整数
    Integer64,
    /// 32位浮点数
    Float32,
    /// 64位浮点数
    Float64,
    /// 布尔类型
    Boolean,
    /// 字符串类型
    String,
    /// 对象类型
    Object,
    /// 数组类型
    Array(Box<GaiaType>),
    /// 指针类型
    Pointer(Box<GaiaType>),
    /// 空类型
    Void,
    /// 通用整数类型（向后兼容）
    Integer,
    /// 通用浮点类型（向后兼容）
    Float,
    /// 通用双精度浮点类型（向后兼容）
    Double,
}
