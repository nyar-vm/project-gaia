use gaia_types::lexer::TokenType;

/// WAT (WebAssembly Text) token types for Component Model
///
/// This enum defines all possible token types that can appear in Component Model WAT source code.
/// It includes component-specific keywords, core module elements, and structural tokens.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum WatTokenType {
    // Literals
    /// 字符串字面量 token
    StringLiteral,
    /// 整数字面量 token
    IntegerLiteral,
    /// 浮点数字面量 token
    FloatLiteral,

    // Identifiers and Names
    /// 标识符 token
    Identifier,

    // Component Model Keywords
    /// component 关键字
    Component,
    /// core 关键字
    Core,

    // Import/Export Keywords
    /// import 关键字
    Import,
    /// export 关键字
    Export,

    // Type System Keywords
    /// type 关键字
    Type,
    /// func 关键字
    Func,
    /// result 关键字
    Result,
    /// param 关键字
    Param,
    /// resource 关键字
    Resource,
    /// record 关键字
    Record,
    /// variant 关键字
    Variant,
    /// enum 关键字
    Enum,
    /// union 关键字
    Union,
    /// option 关键字
    Option,
    /// list 关键字
    List,
    /// tuple 关键字
    Tuple,
    /// flags 关键字
    Flags,

    // Canonical Operations
    /// canon 关键字
    Canon,
    /// lower 关键字
    Lower,
    /// lift 关键字
    Lift,
    /// resource.new 关键字
    ResourceNew,
    /// resource.drop 关键字
    ResourceDrop,
    /// resource.rep 关键字
    ResourceRep,

    // Instance and Alias
    /// instance 关键字
    Instance,
    /// alias 关键字
    Alias,
    /// outer 关键字
    Outer,

    // Core WebAssembly Keywords
    /// module 关键字
    Module,
    /// memory 关键字
    Memory,
    /// table 关键字
    Table,
    /// global 关键字
    Global,
    /// start 关键字
    Start,
    /// data 关键字
    Data,
    /// elem 关键字
    Elem,

    // WebAssembly Instructions (common ones)
    /// call 指令
    Call,
    /// call_indirect 指令
    CallIndirect,
    /// local.get 指令
    LocalGet,
    /// local.set 指令
    LocalSet,
    /// local.tee 指令
    LocalTee,
    /// global.get 指令
    GlobalGet,
    /// global.set 指令
    GlobalSet,
    /// i32.const 指令
    I32Const,
    /// i64.const 指令
    I64Const,
    /// f32.const 指令
    F32Const,
    /// f64.const 指令
    F64Const,
    /// i32.load 指令
    I32Load,
    /// i32.store 指令
    I32Store,
    /// drop 指令
    Drop,
    /// select 指令
    Select,
    /// unreachable 指令
    Unreachable,
    /// nop 指令
    Nop,
    /// block 指令
    Block,
    /// loop 指令
    Loop,
    /// if 指令
    If,
    /// else 指令
    Else,
    /// end 指令
    End,
    /// br 指令
    Br,
    /// br_if 指令
    BrIf,
    /// br_table 指令
    BrTable,
    /// return 指令
    Return,

    // WebAssembly Types
    /// i32 类型
    I32,
    /// i64 类型
    I64,
    /// f32 类型
    F32,
    /// f64 类型
    F64,
    /// v128 类型
    V128,
    /// funcref 类型
    Funcref,
    /// externref 类型
    Externref,

    // Modifiers and Attributes
    /// mut 修饰符
    Mut,
    /// shared 修饰符
    Shared,

    // Symbols and Delimiters
    /// ( 符号
    LeftParen,
    /// ) 符号
    RightParen,
    /// { 符号
    LeftBrace,
    /// } 符号
    RightBrace,
    /// [ 符号
    LeftBracket,
    /// ] 符号
    RightBracket,
    /// , 符号
    Comma,
    /// ; 符号
    Semicolon,
    /// : 符号
    Colon,
    /// :: 符号
    DoubleColon,
    /// . 符号
    Dot,
    /// $ 符号
    Dollar,
    /// @ 符号
    At,
    /// = 符号
    Equals,

    // Special tokens
    /// 换行符 token
    Newline,
    /// 空白字符 token（包括空格、制表符等）
    Whitespace,
    /// 注释 token
    Comment,
    /// 文件结束 token
    Eof,

    /// 自定义段关键字
    Custom,
}

impl TokenType for WatTokenType {
    const END_OF_STREAM: Self = WatTokenType::Eof;
}
