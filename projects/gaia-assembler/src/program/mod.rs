use crate::{
    instruction::GaiaInstruction,
    types::{GaiaSignature, GaiaType},
};
use serde::{Deserialize, Serialize};

/// Gaia 程序模块
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GaiaModule {
    pub name: String,
    pub functions: Vec<GaiaFunction>,
    pub structs: Vec<GaiaStruct>,
    pub classes: Vec<GaiaClass>,
    pub constants: Vec<(String, GaiaConstant)>,
    pub globals: Vec<GaiaGlobal>,
    pub imports: Vec<GaiaImport>,
}

/// 外部导入项
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GaiaImport {
    pub library: String,
    pub symbol: String,
}

/// Gaia 类 (用于托管运行时)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GaiaClass {
    pub name: String,
    pub parent: Option<String>,
    pub interfaces: Vec<String>,
    pub fields: Vec<GaiaField>,
    pub methods: Vec<GaiaFunction>,
    pub attributes: Vec<String>,
}

/// Gaia 字段
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GaiaField {
    pub name: String,
    pub ty: GaiaType,
    pub is_static: bool,
    pub visibility: Visibility,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Visibility {
    Public,
    Private,
    Protected,
    Internal,
}

/// Gaia 函数
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GaiaFunction {
    pub name: String,
    pub signature: GaiaSignature,
    pub blocks: Vec<GaiaBlock>,
    pub is_external: bool,
}

/// Gaia 基本块
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GaiaBlock {
    pub label: String,
    pub instructions: Vec<GaiaInstruction>,
    pub terminator: GaiaTerminator,
}

/// 终结指令 (控制流)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GaiaTerminator {
    /// 跳转
    Jump(String),
    /// 条件跳转 (真标签, 假标签)
    Branch { true_label: String, false_label: String },
    /// 函数返回
    Return,
    /// 调用并跳转 (函数名, 参数, 返回后跳转的标签)
    Call { callee: String, args_count: usize, next_block: String },
    /// 停止/退出
    Halt,
}

/// Gaia 结构体
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GaiaStruct {
    pub name: String,
    pub fields: Vec<(String, GaiaType)>,
}

/// Gaia 全局变量
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GaiaGlobal {
    pub name: String,
    pub ty: GaiaType,
    pub initial_value: Option<GaiaConstant>,
}

/// Gaia 常量
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GaiaConstant {
    Bool(bool),
    I8(i8),
    U8(u8),
    I16(i16),
    U16(u16),
    I32(i32),
    U32(u32),
    I64(i64),
    U64(u64),
    F32(f32),
    F64(f64),
    String(String),
    /// 原始二进制数据 (用于权重、常量数组等)
    Blob(Vec<u8>),
    /// 常量数组
    Array(Vec<GaiaConstant>),
    /// 空值/空引用
    Null,
}
