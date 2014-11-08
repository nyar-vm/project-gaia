use gaia_types::lexer::TokenType;

/// IL 汇编语言的 Token 类型
///
/// 这个枚举定义了 IL 汇编语言中所有可能的 token 类型。
/// 所有变体都不包含数据，使得该类型可以实现 Copy trait。
///
/// # 示例
///
/// ```rust
/// use il_assembler::reader::il_lexer::IlTokenType;
///
/// let token = IlTokenType::Assembly;
/// assert_eq!(token, IlTokenType::Assembly);
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MsilTokenType {
    // 指令关键字
    /// .assembly 指令
    Assembly,
    /// extern 关键字
    Extern,
    /// .module 指令
    Module,
    /// .class 指令
    Class,
    /// .method 指令
    Method,
    /// .ver 指令
    Ver,
    /// .hash 指令
    Hash,
    /// algorithm 关键字
    Algorithm,
    /// .maxstack 指令
    Maxstack,
    /// .locals 指令
    Locals,
    /// init 关键字
    Init,
    /// .entrypoint 指令
    Entrypoint,
    /// .publickeytoken 指令
    Publickeytoken,
    /// .ctor 构造函数标识符
    Ctor,

    // 访问修饰符
    /// public 访问修饰符
    Public,
    /// private 访问修饰符
    Private,
    /// protected 访问修饰符
    Protected,
    /// internal 访问修饰符
    Internal,

    // 方法修饰符
    /// hidebysig 修饰符
    Hidebysig,
    /// virtual 修饰符
    Virtual,
    /// static 修饰符
    Static,
    /// abstract 修饰符
    Abstract,
    /// sealed 修饰符
    Sealed,
    /// override 修饰符
    Override,
    /// specialname 修饰符
    Specialname,
    /// rtspecialname 修饰符
    Rtspecialname,

    // 类型修饰符
    /// auto 修饰符
    Auto,
    /// ansi 修饰符
    Ansi,
    /// unicode 修饰符
    Unicode,
    /// beforefieldinit 修饰符
    Beforefieldinit,

    // 调用约定
    /// instance 调用约定
    Instance,
    /// cil 调用约定
    Cil,
    /// managed 调用约定
    Managed,

    // IL 指令
    /// nop 指令
    Nop,
    /// ldstr 指令
    Ldstr,
    /// ldarg 指令
    Ldarg,
    /// call 指令
    Call,
    /// ret 指令
    Ret,
    /// extends 关键字
    Extends,

    // 标识符和字面量
    /// 标识符 token
    Identifier,
    /// 字符串字面量 token
    StringLiteral,
    /// 数字字面量 token
    Number,

    // 符号
    /// . 符号
    Dot,
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
    /// :: 符号
    DoubleColon,
    /// , 符号
    Comma,
    /// ; 符号
    Semicolon,
    /// / 符号
    Slash,

    // 特殊类型
    /// 类型描述符
    TypeDescriptor,

    // 特殊
    /// 注释 token
    Comment,
    /// 空白字符 token（包括空格、制表符、换行符等）
    Whitespace,
    /// 文件结束 token
    Eof,
}

impl TokenType for MsilTokenType {
    const END_OF_STREAM: Self = MsilTokenType::Eof;
}
