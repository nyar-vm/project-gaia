use gaia_types::{GaiaError, SourceLocation, Result};
use std::collections::HashMap;

/// CLR 版本号信息
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClrVersion {
    pub major: u16,
    pub minor: u16,
    pub build: u16,
    pub revision: u16,
}

impl Default for ClrVersion {
    fn default() -> Self {
        Self { major: 0, minor: 0, build: 0, revision: 0 }
    }
}

/// CLR 访问修饰符和标志
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClrAccessFlags {
    pub is_public: bool,
    pub is_private: bool,
    pub is_security_transparent: bool,
    pub is_retargetable: bool,
}

impl Default for ClrAccessFlags {
    fn default() -> Self {
        Self { is_public: false, is_private: true, is_security_transparent: false, is_retargetable: false }
    }
}

/// CLR 方法实现标志
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClrMethodImplFlags {
    pub is_managed: bool,
    pub is_native: bool,
    pub is_runtime: bool,
    pub is_inline: bool,
    pub is_no_inline: bool,
    pub is_synchronized: bool,
}

impl Default for ClrMethodImplFlags {
    fn default() -> Self {
        Self {
            is_managed: true,
            is_native: false,
            is_runtime: false,
            is_inline: false,
            is_no_inline: false,
            is_synchronized: false,
        }
    }
}

/// CLR 引用类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ClrReferenceType {
    Type,
    Method,
    Field,
    Member,
}

/// CLR 类型引用
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ClrTypeReference {
    Primitive(String),
    Type(String, Option<String>), // (name, namespace)
    Array(Box<ClrTypeReference>),
    Generic(Box<ClrTypeReference>, Vec<ClrTypeReference>),
}

/// CLR 外部程序集引用
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClrExternalAssembly {
    pub name: String,
    pub version: ClrVersion,
    pub public_key_token: Option<Vec<u8>>,
    pub culture: Option<String>,
    pub hash_value: Option<Vec<u8>>,
}

/// CLR 模块定义
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClrModule {
    pub name: String,
    pub mvid: [u8; 16], // GUID
}

/// CLR 特性/属性
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClrAttribute {
    pub name: String,
    pub constructor_args: Vec<ClrAttributeValue>,
    pub named_args: HashMap<String, ClrAttributeValue>,
}

/// CLR 特性参数值
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClrAttributeValue {
    String(String),
    Int32(i32),
    Boolean(bool),
    Enum(String, i32),
    Array(Vec<ClrAttributeValue>),
}

/// CLR 字段定义
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClrField {
    pub name: String,
    pub field_type: ClrTypeReference,
    pub access_flags: ClrAccessFlags,
    pub attributes: Vec<ClrAttribute>,
    pub initial_value: Option<Vec<u8>>,
}

/// CLR 方法参数定义
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClrParameter {
    pub name: String,
    pub parameter_type: ClrTypeReference,
    pub attributes: Vec<ClrAttribute>,
}

/// CLR 局部变量定义
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClrLocalVariable {
    pub index: u32,
    pub variable_type: ClrTypeReference,
    pub name: Option<String>,
}

/// CLR 异常处理器
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClrExceptionHandler {
    pub handler_type: ClrExceptionHandlerType,
    pub try_start: u32,
    pub try_end: u32,
    pub handler_start: u32,
    pub handler_end: u32,
    pub catch_type: Option<ClrTypeReference>,
    pub filter_start: Option<u32>,
}

/// CLR 异常处理器类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClrExceptionHandlerType {
    Catch,
    Filter,
    Finally,
    Fault,
}
