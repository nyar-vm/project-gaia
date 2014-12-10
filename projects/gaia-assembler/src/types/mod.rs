//! Gaia 汇编器核心类型定义

use serde::{Deserialize, Serialize};

/// Gaia 地址空间
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AddressSpace {
    /// 通用/默认地址空间
    Generic,
    /// 栈/局部内存
    Local,
    /// 全局内存
    Global,
    /// 共享内存 (GPU LDS)
    Shared,
    /// 常量内存
    Constant,
    /// 托管堆 (JVM/CLR)
    Managed,
}

/// Gaia 类型系统
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GaiaType {
    // --- 标量类型 ---
    /// 1位布尔类型
    Bool,
    /// 8位有符号整数
    I8,
    /// 8位无符号整数
    U8,
    /// 16位有符号整数
    I16,
    /// 16位无符号整数
    U16,
    /// 32位有符号整数
    I32,
    /// 32位无符号整数
    U32,
    /// 64位有符号整数
    I64,
    /// 64位无符号整数
    U64,
    /// 16位半精度浮点数
    F16,
    /// 32位单精度浮点数
    F32,
    /// 64位双精度浮点数
    F64,

    // --- 复合类型 ---
    /// 指针类型 (指向类型, 地址空间)
    Pointer(Box<GaiaType>, AddressSpace),
    /// 数组类型 (元素类型, 长度)
    Array(Box<GaiaType>, usize),
    /// 向量类型 (元素类型, 数量)
    Vector(Box<GaiaType>, usize),
    /// 结构体类型 (名称/ID)
    Struct(String),
    /// 字符串类型 (托管或原始)
    String,

    // --- 托管类型 (Managed) ---
    /// 动态对象类型 (Python/JS)
    Object,
    /// 托管类 (JVM/CLR)
    Class(String),
    /// 接口/特征 (Interface/Trait)
    Interface(String),
    /// 动态/任意类型 (Variant)
    Any,

    // --- 领域特定类型 (Domain) ---
    /// 张量类型 (元素类型, 形状)
    /// 形状中使用 -1 表示动态维度
    Tensor(Box<GaiaType>, Vec<isize>),

    // --- 特殊类型 ---
    /// 空类型
    Void,
    /// 不透明类型 (用于外部引用)
    Opaque(String),
    /// 函数指针
    FunctionPtr(Box<GaiaSignature>),
}

/// 函数签名
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GaiaSignature {
    pub params: Vec<GaiaType>,
    pub return_type: GaiaType,
}
