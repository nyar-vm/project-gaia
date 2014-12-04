#![doc = include_str!("readme.md")]
pub mod to_jasm;
pub mod to_program;

#[derive(Clone, Debug)]
pub struct JasmClass {
    /// 访问修饰符（如 public, private 等）
    pub modifiers: Vec<String>,
    /// 类名
    pub name: String,
    /// 版本信息（如 65:0）
    pub version: Option<String>,
    /// 方法列表
    pub methods: Vec<JasmMethod>,
    /// 字段列表
    pub fields: Vec<JasmField>,
    /// 源文件信息
    pub source_file: Option<String>,
}

/// JASM 方法声明的 AST 节点
#[derive(Clone, Debug)]
pub struct JasmMethod {
    /// 访问修饰符（如 public, static 等）
    pub modifiers: Vec<String>,
    /// 方法名和类型描述符（如 "main":"([Ljava/lang/String;)V"）
    pub name_and_descriptor: String,
    /// 栈大小
    pub stack_size: Option<u32>,
    /// 局部变量数量
    pub locals_count: Option<u32>,
    /// 方法体指令
    pub instructions: Vec<JasmInstruction>,
}

/// JASM 字段声明的 AST 节点
#[derive(Clone, Debug)]
pub struct JasmField {
    /// 访问修饰符
    pub modifiers: Vec<String>,
    /// 字段名和类型描述符
    pub name_and_descriptor: String,
}

/// JASM 指令的 AST 节点
#[derive(Clone, Debug)]
pub enum JasmInstruction {
    /// 简单指令（如 aload_0, return）
    Simple(String),
    /// 带参数的指令（如 ldc "Hello World"）
    WithArgument { instruction: String, argument: String },
    /// 方法调用指令（如 invokespecial Method java/lang/Object.`<init>`:"()V"）
    MethodCall { instruction: String, method_ref: String },
    /// 字段访问指令（如 getstatic Field java/lang/System.out:"Ljava/io/PrintStream;"）
    FieldAccess { instruction: String, field_ref: String },
}

/// JASM 根节点，表示整个 JASM 文件的 AST
#[derive(Clone, Debug)]
pub struct JasmRoot {
    /// 类定义
    pub class: JasmClass,
}
