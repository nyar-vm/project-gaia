use gaia_types::lexer::TokenType;

/// Token 类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RustTokenType {
    // 关键字
    Fn,
    Let,
    Return,
    If,
    Else,
    True,
    False,

    // 标识符和字面量
    Identifier,
    Integer,
    Float,
    StringLiteral,

    // 运算符
    Plus,         // +
    Minus,        // -
    Star,         // *
    Slash,        // /
    Equal,        // =
    EqualEqual,   // ==
    NotEqual,     // !=
    Less,         // <
    LessEqual,    // <=
    Greater,      // >
    GreaterEqual, // >=
    Bang,         // !

    // 分隔符
    LeftParen,  // (
    RightParen, // )
    LeftBrace,  // {
    RightBrace, // }
    Semicolon,  // ;
    Comma,      // ,
    Dot,        // .
    Arrow,      // ->
    Colon,      // :

    // 特殊
    Whitespace,
    Comment,
    Newline,
    Eof,
}

impl TokenType for RustTokenType {
    const END_OF_STREAM: Self = Self::Eof;
}
