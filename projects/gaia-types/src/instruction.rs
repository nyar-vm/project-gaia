//! Gaia 指令和类型定义模块
//!
//! 这个模块包含了 Gaia 项目的核心类型定义，包括指令集、类型系统、
//! 函数和程序结构等。

use serde::{Deserialize, Serialize};

/// Gaia 统一指令集，以 .NET IL 为骨架
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GaiaInstruction {
    // === 栈操作指令 ===
    /// 加载常量到栈
    LoadConstant(GaiaConstant),
    /// 加载局部变量到栈
    LoadLocal(u32),
    /// 存储栈顶值到局部变量
    StoreLocal(u32),
    /// 加载参数到栈
    LoadArgument(u32),
    /// 存储栈顶值到参数
    StoreArgument(u32),
    /// 复制栈顶值
    Duplicate,
    /// 弹出栈顶值
    Pop,

    // === 算术运算指令 ===
    /// 加法
    Add,
    /// 减法
    Subtract,
    /// 乘法
    Multiply,
    /// 除法
    Divide,
    /// 取余
    Remainder,
    /// 按位与
    BitwiseAnd,
    /// 按位或
    BitwiseOr,
    /// 按位异或
    BitwiseXor,
    /// 按位取反
    BitwiseNot,
    /// 左移
    ShiftLeft,
    /// 右移
    ShiftRight,
    /// 取负
    Negate,

    // === 比较指令 ===
    /// 相等比较
    CompareEqual,
    /// 不等比较
    CompareNotEqual,
    /// 小于比较
    CompareLessThan,
    /// 小于等于比较
    CompareLessEqual,
    /// 大于比较
    CompareGreaterThan,
    /// 大于等于比较
    CompareGreaterEqual,

    // === 控制流指令 ===
    /// 无条件跳转
    Branch(String),
    /// 条件跳转（栈顶为真时跳转）
    BranchIfTrue(String),
    /// 条件跳转（栈顶为假时跳转）
    BranchIfFalse(String),
    /// 函数调用
    Call(String),
    /// 返回
    Return,

    // === 内存操作指令 ===
    /// 加载内存地址
    LoadAddress(u32),
    /// 从内存加载值
    LoadIndirect(GaiaType),
    /// 存储值到内存
    StoreIndirect(GaiaType),
    /// 加载对象字段
    LoadField(String),
    /// 存储到对象字段
    StoreField(String),
    /// 创建新对象
    NewObject(String),

    // === 类型转换指令 ===
    /// 类型转换
    Convert(GaiaType, GaiaType),
    /// 装箱
    Box(GaiaType),
    /// 拆箱
    Unbox(GaiaType),

    // === 标签和注释 ===
    /// 标签
    Label(String),
    /// 注释
    Comment(String),
    StringConstant(String),
}

/// Gaia 常量类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GaiaConstant {
    /// 8位整数
    Integer8(i8),
    /// 16位整数
    Integer16(i16),
    /// 32位整数
    Integer32(i32),
    /// 64位整数
    Integer64(i64),
    /// 32位浮点数
    Float32(f32),
    /// 64位浮点数
    Float64(f64),
    /// 字符串
    String(String),
    /// 布尔值
    Boolean(bool),
    /// 空值
    Null,
}

/// Gaia 类型系统
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GaiaType {
    /// 8位整数
    Integer8,
    /// 16位整数
    Integer16,
    /// 32位整数
    Integer32,
    /// 64位整数
    Integer64,
    /// 32位浮点数
    Float32,
    /// 64位浮点数
    Float64,
    /// 字符串
    String,
    /// 布尔值
    Boolean,
    /// 对象引用
    Object,
    /// 指针
    Pointer,
    /// 数组
    Array(Box<GaiaType>),
    /// 自定义类型
    Custom(String),
}

/// Gaia 函数定义
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GaiaFunction {
    /// 函数名
    pub name: String,
    /// 参数类型
    pub parameters: Vec<GaiaType>,
    /// 返回类型
    pub return_type: Option<GaiaType>,
    /// 局部变量类型
    pub locals: Vec<GaiaType>,
    /// 指令序列
    pub instructions: Vec<GaiaInstruction>,
}

/// Gaia 程序
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GaiaProgram {
    /// 程序名
    pub name: String,
    /// 函数列表
    pub functions: Vec<GaiaFunction>,
    /// 全局常量
    pub constants: Vec<(String, GaiaConstant)>,
}

impl GaiaInstruction {
    /// 获取指令的操作码名称
    pub fn opcode_name(&self) -> &'static str {
        match self {
            GaiaInstruction::LoadConstant(_) => "ldconst",
            GaiaInstruction::LoadLocal(_) => "ldloc",
            GaiaInstruction::StoreLocal(_) => "stloc",
            GaiaInstruction::LoadArgument(_) => "ldarg",
            GaiaInstruction::StoreArgument(_) => "starg",
            GaiaInstruction::Duplicate => "dup",
            GaiaInstruction::Pop => "pop",
            GaiaInstruction::Add => "add",
            GaiaInstruction::Subtract => "sub",
            GaiaInstruction::Multiply => "mul",
            GaiaInstruction::Divide => "div",
            GaiaInstruction::Remainder => "rem",
            GaiaInstruction::BitwiseAnd => "and",
            GaiaInstruction::BitwiseOr => "or",
            GaiaInstruction::BitwiseXor => "xor",
            GaiaInstruction::BitwiseNot => "not",
            GaiaInstruction::ShiftLeft => "shl",
            GaiaInstruction::ShiftRight => "shr",
            GaiaInstruction::Negate => "neg",
            GaiaInstruction::CompareEqual => "ceq",
            GaiaInstruction::CompareNotEqual => "cne",
            GaiaInstruction::CompareLessThan => "clt",
            GaiaInstruction::CompareLessEqual => "cle",
            GaiaInstruction::CompareGreaterThan => "cgt",
            GaiaInstruction::CompareGreaterEqual => "cge",
            GaiaInstruction::Branch(_) => "br",
            GaiaInstruction::BranchIfTrue(_) => "brtrue",
            GaiaInstruction::BranchIfFalse(_) => "brfalse",
            GaiaInstruction::Call(_) => "call",
            GaiaInstruction::Return => "ret",
            GaiaInstruction::LoadAddress(_) => "ldloca",
            GaiaInstruction::LoadIndirect(_) => "ldind",
            GaiaInstruction::StoreIndirect(_) => "stind",
            GaiaInstruction::LoadField(_) => "ldfld",
            GaiaInstruction::StoreField(_) => "stfld",
            GaiaInstruction::NewObject(_) => "newobj",
            GaiaInstruction::Convert(_, _) => "conv",
            GaiaInstruction::Box(_) => "box",
            GaiaInstruction::Unbox(_) => "unbox",
            GaiaInstruction::Label(_) => "label",
            GaiaInstruction::Comment(_) => "comment",
            GaiaInstruction::StringConstant(_) => "ldstr",
        }
    }
}
