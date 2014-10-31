//! Mini Rust 抽象语法树定义

use serde::{Deserialize, Serialize};

/// Mini Rust 程序
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Program {
    pub name: String,
    pub functions: Vec<Function>,
}

/// 函数定义
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub body: Block,
}

/// 函数参数
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub param_type: Type,
}

/// 代码块
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Block {
    pub statements: Vec<Statement>,
}

/// 语句
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Statement {
    /// 表达式语句
    Expression(Expression),
    /// 变量声明
    VariableDeclaration { name: String, var_type: Option<Type>, initializer: Option<Expression> },
    /// 返回语句
    Return(Option<Expression>),
}

/// 表达式
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expression {
    /// 字面量
    Literal(Literal),
    /// 标识符
    Identifier(String),
    /// 函数调用
    FunctionCall { name: String, arguments: Vec<Expression> },
    /// 宏调用（如 println!）
    MacroCall { name: String, arguments: Vec<Expression> },
    /// 方法调用 (如 console.log)
    MethodCall { object: Box<Expression>, method: String, arguments: Vec<Expression> },
    /// 二元运算
    BinaryOperation { left: Box<Expression>, operator: BinaryOperator, right: Box<Expression> },
    /// 一元运算
    UnaryOperation { operator: UnaryOperator, operand: Box<Expression> },
}

/// 字面量
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Literal {
    /// 整数
    Integer(i64),
    /// 浮点数
    Float(f64),
    /// 字符串
    String(String),
    /// 布尔值
    Boolean(bool),
}

/// 二元运算符
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

/// 一元运算符
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UnaryOperator {
    Negate,
    Not,
}

/// 类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Type {
    /// 32位整数
    I32,
    /// 64位整数
    I64,
    /// 32位浮点数
    F32,
    /// 64位浮点数
    F64,
    /// 字符串
    String,
    /// 布尔值
    Bool,
    /// 空类型
    Unit,
}

impl Type {
    /// 转换为 Gaia 类型
    pub fn to_gaia_type(&self) -> gaia_assembler::GaiaType {
        match self {
            Type::I32 => gaia_assembler::GaiaType::Integer32,
            Type::I64 => gaia_assembler::GaiaType::Integer64,
            Type::F32 => gaia_assembler::GaiaType::Float32,
            Type::F64 => gaia_assembler::GaiaType::Float64,
            Type::String => gaia_assembler::GaiaType::String,
            Type::Bool => gaia_assembler::GaiaType::Boolean,
            Type::Unit => gaia_assembler::GaiaType::Object, // 使用 Object 表示 unit 类型
        }
    }
}

impl Literal {
    /// 转换为 Gaia 常量
    pub fn to_gaia_constant(&self) -> gaia_assembler::GaiaConstant {
        match self {
            Literal::Integer(i) => gaia_assembler::GaiaConstant::Integer32(*i as i32),
            Literal::Float(f) => gaia_assembler::GaiaConstant::Float32(*f as f32),
            Literal::String(s) => gaia_assembler::GaiaConstant::String(s.clone()),
            Literal::Boolean(b) => gaia_assembler::GaiaConstant::Boolean(*b),
        }
    }
}
