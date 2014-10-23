use gaia_types::lexer::{LexerState, TokenType};

mod utf16_test;

#[derive(Copy, Clone)]
pub enum MyToken {
    Eof,
}

impl TokenType for MyToken {
    const END_OF_STREAM: Self = Self::Eof;
}

pub struct MyState<'input> {
    /// 词法分析器状态
    pub state: LexerState<'input, MyToken>,
    /// 配置信息（只读）
    pub config: Config,
    /// 可变状态
    pub mutable: Mutable,
}

/// 配置信息结构体
#[derive(Clone, Debug)]
pub struct Config {
    /// 是否区分大小写
    pub case_sensitive: bool,
    /// 是否支持 Unicode 标识符
    pub unicode_identifiers: bool,
    /// 注释标记（如 "//", "#" 等）
    pub comment_markers: Vec<String>,
}

/// 可变状态结构体
#[derive(Clone, Debug)]
pub struct Mutable {
    /// 缩进栈（用于 Python 等缩进敏感语言）
    pub indent_stack: Vec<usize>,
    /// 括号计数器
    pub paren_count: i32,
    /// 当前括号类型栈
    pub paren_stack: Vec<char>,
}
