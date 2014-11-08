//! MSIL (Microsoft Intermediate Language) 抽象语法树 (AST) 模块
//!
//! 这个模块定义了 MSIL 汇编语言的抽象语法树结构，用于表示解析后的 MSIL 代码。
//! AST 节点对应于 MSIL 汇编语言中的各种构造，如程序集、模块、类、方法等。

/// MSIL 程序的根节点
///
/// 表示一个完整的 MSIL 程序，包含程序中的所有语句。
///
/// # 示例
///
/// ```rust
/// use clr_msil::ast::{MsilRoot, MsilStatement};
///
/// let root = MsilRoot { statements: vec![MsilStatement::Assembly("MyAssembly".to_string())] };
/// ```
#[derive(Clone, Debug)]
pub struct MsilRoot {
    /// 程序中的所有语句
    pub statements: Vec<MsilStatement>,
}

/// MSIL 语句枚举
///
/// 表示 MSIL 程序中的各种顶级语句。
///
/// # 变体
///
/// - `AssemblyExtern`: 外部程序集引用声明
/// - `Assembly`: 程序集声明
/// - `Module`: 模块声明
/// - `Class`: 类声明
#[derive(Clone, Debug)]
pub enum MsilStatement {
    /// 外部程序集引用声明
    ///
    /// 例如：`.assembly extern UnityEngine`
    AssemblyExtern(String),
    /// 程序集声明
    ///
    /// 例如：`.assembly MyAssembly`
    Assembly(String),
    /// 模块声明
    ///
    /// 例如：`.module MyModule.dll`
    Module(String),
    /// 类声明
    ///
    /// 包含类的完整定义，包括修饰符、名称、基类和成员方法。
    Class(MsilClass),
}

/// MSIL 类定义
///
/// 表示一个 MSIL 类，包含类的所有属性和方法。
///
/// # 示例
///
/// ```rust
/// use clr_msil::ast::{MsilClass, MsilMethod};
///
/// let class = MsilClass {
///     modifiers: vec!["public".to_string(), "auto".to_string()],
///     name: "MyClass".to_string(),
///     extends: Some("System.Object".to_string()),
///     methods: vec![],
/// };
/// ```
#[derive(Clone, Debug)]
pub struct MsilClass {
    /// 类修饰符（如 public, private, auto, ansi 等）
    pub modifiers: Vec<String>,
    /// 类名（可能包含命名空间）
    pub name: String,
    /// 基类名称（如果存在继承）
    pub extends: Option<String>,
    /// 类中包含的方法
    pub methods: Vec<MsilMethod>,
}

/// MSIL 方法定义
///
/// 表示一个 MSIL 方法，包含方法签名和方法体。
///
/// # 示例
///
/// ```rust
/// use clr_msil::ast::{MsilMethod, MsilMethodBody, MsilParameter};
///
/// let method = MsilMethod {
///     modifiers: vec!["public".to_string(), "static".to_string()],
///     return_type: "void".to_string(),
///     name: "Main".to_string(),
///     parameters: vec![MsilParameter {
///         param_type: "string[]".to_string(),
///         name: Some("args".to_string()),
///     }],
///     body: None,
/// };
/// ```
#[derive(Clone, Debug)]
pub struct MsilMethod {
    /// 方法修饰符（如 public, static, virtual 等）
    pub modifiers: Vec<String>,
    /// 返回类型
    pub return_type: String,
    /// 方法名
    pub name: String,
    /// 方法参数列表
    pub parameters: Vec<MsilParameter>,
    /// 方法体（包含指令和局部变量）
    pub body: Option<MsilMethodBody>,
}

/// MSIL 方法参数定义
///
/// 表示方法的一个参数。
#[derive(Clone, Debug)]
pub struct MsilParameter {
    /// 参数类型
    pub param_type: String,
    /// 参数名（可选）
    pub name: Option<String>,
}

/// MSIL 方法体定义
///
/// 包含方法的实现细节，如最大栈大小、局部变量和指令序列。
///
/// # 示例
///
/// ```rust
/// use clr_msil::ast::{MsilInstruction, MsilLocal, MsilMethodBody};
///
/// let body = MsilMethodBody {
///     maxstack: Some(8),
///     locals: vec![MsilLocal {
///         index: Some(0),
///         local_type: "bool".to_string(),
///         name: Some("V_0".to_string()),
///     }],
///     instructions: vec![MsilInstruction {
///         opcode: "ldstr".to_string(),
///         operands: vec!["Hello World!".to_string()],
///         label: None,
///     }],
/// };
/// ```
#[derive(Clone, Debug)]
pub struct MsilMethodBody {
    /// 最大栈大小
    pub maxstack: Option<u32>,
    /// 局部变量列表
    pub locals: Vec<MsilLocal>,
    /// 指令序列
    pub instructions: Vec<MsilInstruction>,
}

/// MSIL 局部变量定义
///
/// 表示方法中的一个局部变量。
#[derive(Clone, Debug)]
pub struct MsilLocal {
    /// 变量索引（可选）
    pub index: Option<u32>,
    /// 变量类型
    pub local_type: String,
    /// 变量名（可选）
    pub name: Option<String>,
}

/// MSIL 指令定义
///
/// 表示一条 MSIL 指令，包含操作码和操作数。
///
/// # 示例
///
/// ```rust
/// use clr_msil::ast::MsilInstruction;
///
/// let instruction = MsilInstruction {
///     opcode: "call".to_string(),
///     operands: vec!["void [UnityEngine]UnityEngine.Debug::Log(object)".to_string()],
///     label: Some("IL_0006".to_string()),
/// };
/// ```
#[derive(Clone, Debug)]
pub struct MsilInstruction {
    /// 指令操作码（如 ldstr, call, ret 等）
    pub opcode: String,
    /// 指令操作数
    pub operands: Vec<String>,
    /// 指令标签（可选，用于跳转目标）
    pub label: Option<String>,
}
