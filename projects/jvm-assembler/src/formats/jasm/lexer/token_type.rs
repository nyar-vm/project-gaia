use gaia_types::lexer::TokenType;

/// JASM 汇编语言的 Token 类型
///
/// 这个枚举定义了 JASM 汇编语言中所有可能的 token 类型。
/// 所有变体都不包含数据，使得该类型可以实现 Copy trait。
///

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum JasmTokenType {
    // 关键字
    /// class 关键字
    Class,
    /// version 关键字
    Version,
    /// Method 关键字
    Method,
    /// Field 关键字
    Field,
    /// String 关键字
    String,
    /// SourceFile 关键字
    SourceFile,
    /// stack 关键字
    Stack,
    /// locals 关键字
    Locals,
    /// end 关键字
    End,
    /// compiled 关键字
    Compiled,
    /// from 关键字
    From,
    /// InnerClass 关键字
    InnerClass,
    /// NestMembers 关键字
    NestMembers,
    /// BootstrapMethod 关键字
    BootstrapMethod,

    // 访问修饰符
    /// public 访问修饰符
    Public,
    /// private 访问修饰符
    Private,
    /// protected 访问修饰符
    Protected,
    /// static 修饰符
    Static,
    /// super 修饰符
    Super,
    /// final 修饰符
    Final,
    /// abstract 修饰符
    Abstract,
    /// synchronized 修饰符
    Synchronized,
    /// native 修饰符
    Native,
    /// synthetic 修饰符
    Synthetic,
    /// deprecated 修饰符
    Deprecated,
    /// varargs 修饰符
    Varargs,

    // JVM 指令 - 加载指令
    /// aload_0 指令
    ALoad0,
    /// aload_1 指令
    ALoad1,
    /// aload_2 指令
    ALoad2,
    /// aload_3 指令
    ALoad3,
    /// iload_0 指令
    ILoad0,
    /// iload_1 指令
    ILoad1,
    /// iload_2 指令
    ILoad2,
    /// iload_3 指令
    ILoad3,
    /// ldc 指令
    Ldc,
    /// ldc_w 指令
    LdcW,
    /// ldc2_w 指令
    Ldc2W,

    // JVM 指令 - 方法调用
    /// invokespecial 指令
    InvokeSpecial,
    /// invokevirtual 指令
    InvokeVirtual,
    /// invokestatic 指令
    InvokeStatic,
    /// invokeinterface 指令
    InvokeInterface,
    /// invokedynamic 指令
    InvokeDynamic,

    // JVM 指令 - 字段访问
    /// getstatic 指令
    GetStatic,
    /// putstatic 指令
    PutStatic,
    /// getfield 指令
    GetField,
    /// putfield 指令
    PutField,

    // JVM 指令 - 控制流
    /// return 指令
    Return,
    /// ireturn 指令
    IReturn,
    /// areturn 指令
    AReturn,
    /// lreturn 指令
    LReturn,
    /// freturn 指令
    FReturn,
    /// dreturn 指令
    DReturn,

    // JVM 指令 - 其他常用指令
    /// nop 指令
    Nop,
    /// dup 指令
    Dup,
    /// pop 指令
    Pop,
    /// new 指令
    New,

    // 标识符和字面量
    /// 标识符 token
    Identifier,
    /// 字符串字面量 token
    StringLiteral,
    /// 数字字面量 token
    Number,
    /// 类型描述符 token (如 "()V", "[Ljava/lang/String;")
    TypeDescriptor,

    // 符号
    /// { 符号
    LeftBrace,
    /// } 符号
    RightBrace,
    /// ( 符号
    LeftParen,
    /// ) 符号
    RightParen,
    /// [ 符号
    LeftBracket,
    /// ] 符号
    RightBracket,
    /// : 符号
    Colon,
    /// ; 符号
    Semicolon,
    /// . 符号
    Dot,
    /// / 符号
    Slash,
    /// , 符号
    Comma,

    // 特殊
    /// 注释 token
    Comment,
    /// 空白字符 token（包括空格、制表符、换行符等）
    Whitespace,
    /// 文件结束 token
    Eof,
}

impl TokenType for JasmTokenType {
    const END_OF_STREAM: Self = Self::Eof;
}
