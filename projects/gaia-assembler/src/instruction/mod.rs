use crate::{program::GaiaConstant, types::GaiaType};
use serde::{Deserialize, Serialize};

/// Gaia 指令集
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GaiaInstruction {
    // 栈操作
    /// 加载常量到栈顶
    LoadConstant(GaiaConstant),
    /// 加载局部变量
    LoadLocal(usize),
    /// 存储到局部变量
    StoreLocal(usize),
    /// 加载全局变量
    LoadGlobal(String),
    /// 加载方法参数
    LoadArgument(usize),
    /// 存储到全局变量
    StoreGlobal(String),
    /// 复制栈顶元素
    Duplicate,
    /// 弹出栈顶元素
    Pop,

    // 算术运算
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
    /// 取负
    Negate,

    // 位运算
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

    // 比较运算
    /// 等于
    Equal,
    /// 不等于
    NotEqual,
    /// 小于
    LessThan,
    /// 小于等于
    LessThanOrEqual,
    /// 大于
    GreaterThan,
    /// 大于等于
    GreaterThanOrEqual,

    // 逻辑运算
    /// 逻辑与
    LogicalAnd,
    /// 逻辑或
    LogicalOr,
    /// 逻辑非
    LogicalNot,

    // 控制流
    /// 无条件跳转
    Jump(String),
    /// 条件跳转（栈顶为真时跳转）
    JumpIfTrue(String),
    /// 条件跳转（栈顶为假时跳转）
    JumpIfFalse(String),
    /// 函数调用
    Call(String, usize), // 函数名，参数个数
    /// 返回
    Return,
    /// 标签
    Label(String),

    // 内存操作
    /// 加载间接值
    LoadIndirect(GaiaType),
    /// 存储间接值
    StoreIndirect(GaiaType),

    // 类型转换
    /// 类型转换
    Convert(GaiaType, GaiaType), // from, to

    // 对象操作
    /// 装箱
    Box(GaiaType),
    /// 拆箱
    Unbox(GaiaType),

    // 数组操作
    /// 创建数组
    NewArray(GaiaType, usize), // 元素类型，大小
    /// 加载数组元素
    LoadElement(GaiaType),
    /// 存储数组元素
    StoreElement(GaiaType),
    /// 获取数组长度
    ArrayLength,
}
