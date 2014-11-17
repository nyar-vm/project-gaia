#![doc = include_str!("readme.md")]

use crate::{
    reader::{SourcePosition, Token, TokenStream},
    GaiaDiagnostics, GaiaError, SourceLocation,
};
use url::Url;

/// 表示令牌类型。
pub trait TokenType: Copy {
    /// 表示流的结束。
    const END_OF_STREAM: Self;

    /// 检查令牌是否为空格。
    fn is_whitespace(&self) -> bool {
        false
    }

    /// 检查令牌是否应被忽略。
    fn is_ignored(&self) -> bool {
        false
    }
}

/// 词法分析器状态管理实用类
///
/// 这是一个通用的词法分析器状态管理器，提供了完整的词法分析功能，
/// 包括字符位置跟踪、token 收集、错误处理等。
///
/// # 设计目标
///
/// * **通用性**: 支持任意 token 类型，只要实现 `Copy` trait
/// * **性能**: 高效的字符迭代和位置跟踪
/// * **易用性**: 提供丰富的辅助方法简化词法分析
/// * **错误处理**: 集成 Gaia 错误系统
///
/// # 示例
///
/// ```rust
/// use gaia_types::{
///     lexer::LexerState,
///     reader::{SourcePosition, Token},
/// };
///
/// #[derive(Clone, Copy, Debug)]
/// enum MyToken {
///     Identifier,
///     Number,
///     Whitespace,
/// }
///
/// let input = "hello 123";
/// let mut state = LexerState::new(input);
///
/// // 添加 token
/// state.add_token(MyToken::Identifier, 0, 5, 1, 1);
/// state.add_token(MyToken::Whitespace, 5, 1, 1, 6);
/// state.add_token(MyToken::Number, 6, 3, 1, 7);
///
/// // 生成 token 流
/// let token_stream = state.into_token_stream();
/// ```
#[derive(Debug)]
pub struct LexerState<'input, T: TokenType> {
    url: Option<&'input Url>,
    /// 输入字符串
    input: &'input str,
    /// 收集的 tokens
    tokens: Vec<Token<T>>,
    /// 当前行号（从 1 开始）
    line: u32,
    /// 当前列号（从 1 开始，UTF-16 长度）
    column: u32,
    /// 当前字节偏移量（从 0 开始）
    offset: usize,
    diagnostics: Vec<GaiaError>,
}

impl<'input, T: TokenType> LexerState<'input, T> {
    /// 创建新的词法分析器状态
    ///
    /// # 参数
    ///
    /// * `input` - 要分析的输入字符串
    /// * `url` - 输入源的 URL（可选），用于错误定位
    ///
    /// # 返回值
    ///
    /// 返回初始化的词法分析器状态，位置信息设置为 (line: 1, column: 1, offset: 0)
    ///
    /// # 示例
    ///
    /// ```rust
    /// use gaia_types::lexer::LexerState;
    /// use url::Url;
    ///
    /// let input = "let x = 42;";
    /// let url = Url::parse("file:///example.gs").ok();
    /// let state = LexerState::new(input, url);
    /// assert_eq!(state.get_position(), (0, 1, 1));
    /// ```
    pub fn new(input: &'input str, url: Option<&'input Url>) -> Self {
        Self { url, input, tokens: Vec::new(), line: 1, column: 1, offset: 0, diagnostics: vec![] }
    }

    /// 获取当前位置信息
    ///
    /// 返回当前的字节偏移量、行号和列号
    ///
    /// # 返回值
    ///
    /// 返回 `(offset, line, column)` 元组/// 获取当前位置（行、列、偏移量）
    ///
    /// # 返回值
    ///
    /// 返回一个三元组 `(offset, line, column)`，其中：
    /// * `offset` - 字节偏移量（从 0 开始）
    /// * `line` - 行号（从 1 开始）
    /// * `column` - 列号（从 1 开始，UTF-16 长度）
    ///
    /// # 示例
    ///
    /// ```rust
    /// let input = "hello\nworld";
    /// let mut state = LexerState::new(input);
    ///
    /// // 初始位置
    /// assert_eq!(state.get_position(), (0, 1, 1));
    ///
    /// // 移动到 'o' 字符
    /// state.next_char(); // h
    /// state.next_char(); // e
    /// state.next_char(); // l
    /// state.next_char(); // l
    /// state.next_char(); // o
    /// assert_eq!(state.get_position(), (4, 1, 5)); // "hello" 的最后一个字符
    /// ```
    pub fn get_position(&self) -> (usize, u32, u32) {
        (self.offset, self.line, self.column)
    }

    /// 标记当前位置并返回位置信息
    ///
    /// 这是一个便利方法，用于在需要记录 token 开始位置时使用。
    /// 返回当前的字节偏移量、行号和列号，通常用于后续的 `add_token` 调用。
    ///
    /// # 返回值
    ///
    /// `(offset, line, column)` - 当前位置的三元组
    ///
    /// # 示例
    ///
    /// ```rust
    /// let input = "hello";
    /// let mut state = LexerState::new(input, None);
    ///
    /// let (start_offset, start_line, start_column) = state.mark_position();
    /// // 处理一些字符...
    /// state.add_token(SomeTokenType::Identifier, start_offset, 5, start_line, start_column);
    /// ```
    pub fn mark_position(&self) -> (usize, u32, u32) {
        (self.offset, self.line, self.column)
    }

    /// 获取当前源代码位置
    ///
    /// 返回一个 `SourcePosition` 结构体，包含完整的位置信息
    pub fn get_source_position(&self) -> SourcePosition {
        SourcePosition { offset: self.offset, length: 0, line: self.line, column: self.column }
    }

    /// 查看下一个字符但不消耗它
    ///
    /// 返回下一个字符的副本及其字节偏移量，但不移动位置指针。可以多次调用 `peek` 获取相同的字符。
    ///
    /// # 返回值
    ///
    /// * `Some((offset, ch))` - 下一个字符 `ch` 及其字节偏移量
    /// * `None` - 已到达输入流末尾
    ///
    /// # 示例
    ///
    /// ```rust
    /// let input = "hello";
    /// let mut state = LexerState::new(input);
    ///
    /// // 多次 peek 返回相同的字符
    /// assert_eq!(state.peek(), Some((0, 'h')));
    /// assert_eq!(state.peek(), Some((0, 'h')));
    /// assert_eq!(state.peek(), Some((0, 'h')));
    ///
    /// // 位置信息不变
    /// assert_eq!(state.get_position(), (0, 1, 1));
    ///
    /// // next_char 会消耗字符并移动位置
    /// assert_eq!(state.next_char(), Some((0, 'h')));
    /// assert_eq!(state.peek(), Some((1, 'e')));
    /// assert_eq!(state.get_position(), (1, 1, 2));
    /// ```
    pub fn peek(&mut self) -> Option<(usize, char)> {
        let char = self.rest_text().chars().next()?;
        Some((self.offset, char))
    }

    /// 返回剩余的文本。
    pub fn rest_text(&self) -> &'input str {
        unsafe {
            debug_assert!(self.offset <= self.input.len());
            self.input.get_unchecked(self.offset..)
        }
    }

    /// 获取下一个字符并更新位置信息
    ///
    /// 从输入流中读取下一个字符，并自动更新行号、列号和偏移量。
    /// 列号计算使用 UTF-16 长度，与 LSP 协议兼容。
    ///
    /// # 返回值
    ///
    /// * `Some((offset, ch))` - 成功读取字符 `ch` 及其字节偏移量
    /// * `None` - 已到达输入流末尾
    ///
    /// # 示例
    ///
    /// ```rust
    /// let input = "a\n😀";
    /// let mut state = LexerState::new(input);
    ///
    /// // 读取 'a'
    /// assert_eq!(state.next_char(), Some((0, 'a')));
    /// assert_eq!(state.get_position(), (1, 1, 2)); // UTF-16 长度: 'a' = 1
    ///
    /// // 读取换行符
    /// assert_eq!(state.next_char(), Some((1, '\n')));
    /// assert_eq!(state.get_position(), (2, 2, 1)); // 新行开始
    ///
    /// // 读取表情符号（代理对，UTF-16 长度 = 2）
    /// assert_eq!(state.next_char(), Some((2, '😀')));
    /// assert_eq!(state.get_position(), (6, 2, 3)); // UTF-16 长度: '😀' = 2，所以 column = 1 + 2 = 3
    /// ```
    pub fn next_char(&mut self) -> Option<(usize, char)> {
        let (_, this_char) = self.peek()?;
        self.consume_char(this_char);
        self.peek()
    }

    /// 更新位置信息（处理换行和 Unicode）
    ///
    /// 正确处理各种换行符（\n, \r, \r\n）和 Unicode 字符。
    /// 列号计算使用 UTF-16 长度，与 LSP 协议兼容。
    ///
    /// # UTF-16 编码规则
    ///
    /// * 基本多语言平面 (BMP) 字符（U+0000 到 U+FFFF）：1 个 UTF-16 码元
    /// * 辅助平面字符（U+10000 到 U+10FFFF）：2 个 UTF-16 码元（代理对）
    ///
    /// # 示例
    ///
    /// ```rust
    /// // 基本拉丁字符：每个字符 1 个 UTF-16 码元
    /// "abc" -> 列号: 1, 2, 3
    ///
    /// // 中文字符：每个字符 1 个 UTF-16 码元（在 BMP 范围内）
    /// "你好" -> 列号: 1, 2
    ///
    /// // 表情符号：每个字符 2 个 UTF-16 码元（代理对）
    /// "😀" -> 列号: 1, 3（跳过 2，因为代理对占用 2 个码元）
    /// ```
    ///
    /// # 测试
    ///
    /// ```rust
    /// # use gaia_types::lexer::{LexerState, TokenType};
    /// # #[derive(Clone, Copy, Debug)] enum TestToken { Char, Eof }
    /// # impl TokenType for TestToken { const EOF: Self = TestToken::Eof; }
    /// let mut state = LexerState::<TestToken>::new("😀", None);
    /// assert_eq!(state.get_position(), (0, 1, 1)); // 初始位置
    /// state.next_char(); // 读取表情符号
    /// assert_eq!(state.get_position(), (4, 1, 3)); // UTF-16长度=2，所以column=1+2=3
    /// ```
    /// 消耗单个字符并更新位置信息
    ///
    /// 这是一个内部方法，用于消耗单个字符并正确更新行号和列号。
    /// 它会正确处理换行符和 UTF-16 列计算。
    ///
    /// # 参数
    ///
    /// * `ch` - 要消耗的字符
    ///
    /// # 重要说明
    ///
    /// **此方法仅处理单个字符**，不适用于多字符序列如 `\r\n`：
    /// * 对于 `\r\n` 换行符，请使用 `consume_str` 方法
    /// * 此方法会将 `\r` 和 `\n` 分别作为独立的字符处理，可能导致行号计算错误
    ///
    /// # 示例
    ///
    /// ```rust
    /// let input = "hello\nworld";
    /// let mut state = LexerState::new(input, None);
    ///
    /// // 处理换行符
    /// state.consume_char('\n');
    /// assert_eq!(state.get_position(), (6, 2, 1)); // 行号增加，列号重置
    ///
    /// // 处理普通字符
    /// state.consume_char('w');
    /// assert_eq!(state.get_position(), (7, 2, 2)); // 列号增加
    /// ```
    ///
    /// # 注意
    ///
    /// 对于换行符处理：
    /// * `\n` - 正确增加行号，列号重置为 1
    /// * `\r` - 单独增加行号，列号重置为 1（不推荐，可能导致问题）
    /// * `\r\n` - **不支持**，应使用 `consume_str(state, "\r\n")`
    fn consume_char(&mut self, ch: char) -> usize {
        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        }
        else if ch == '\r' {
            self.line += 1;
            self.column = 1;
        }
        else {
            // 计算 UTF-16 长度，与 LSP 协议兼容
            let utf16_len = ch.len_utf16();
            self.column += utf16_len as u32;
        }
        self.offset += ch.len_utf8();
        ch.len_utf8()
    }

    /// 消耗指定字符串并更新位置信息
    ///
    /// 这是一个内部方法，用于消耗字符串并正确更新行号和列号。
    /// 它会正确处理各种换行符（包括 `\r\n`）和 UTF-16 列计算。
    ///
    /// # 参数
    ///
    /// * `expected` - 要消耗的字符串
    ///
    /// # 返回值
    ///
    /// 返回消耗的字节长度
    ///
    /// # 重要特性
    ///
    /// **正确处理 `\r\n` 换行符**：
    /// * 对于 `\r\n` 序列，会正确识别为单个换行符
    /// * 行号只增加一次，列号正确重置
    /// * 这是与 `consume_char` 方法的主要区别
    ///
    /// # 示例
    ///
    /// ```rust
    /// let input = "hello\r\nworld";
    /// let mut state = LexerState::new(input, None);
    ///
    /// // 正确处理 \r\n 换行符
    /// let length = state.consume_str("\r\n");
    /// assert_eq!(length, 2);
    /// assert_eq!(state.get_position(), (2, 2, 1)); // 行号只增加一次
    /// ```
    fn consume_str(&mut self, expected: &str) -> usize {
        #[cfg(debug_assertions)]
        unsafe {
            let start = self.offset;
            let end = start + expected.len();
            let input = self.input.get_unchecked(start..end);
            assert_eq!(input, expected);
        }

        let mut chars = expected.chars();
        while let Some(c) = chars.next() {
            match c {
                '\n' => {
                    self.line += 1;
                    self.column = 1;
                    self.offset += 1;
                }
                '\r' => match chars.next() {
                    Some('\n') => {
                        self.line += 1;
                        self.column = 1;
                        self.offset += 2;
                    }
                    Some(other) => {
                        self.line += 1;
                        self.column = 1;
                        self.offset += 1 + other.len_utf8();
                    }
                    None => {
                        self.line += 1;
                        self.column = 1;
                        self.offset += 1;
                    }
                },
                _ => {
                    self.column += c.len_utf16() as u32;
                    self.offset += c.len_utf8();
                }
            }
        }
        expected.len()
    }

    /// 添加 token 到收集列表
    ///
    /// 这是一个核心方法，用于将 token 添加到内部的 token 列表中。
    /// 它会根据提供的位置信息创建 `SourcePosition` 和 `Token` 对象。
    ///
    /// # 参数
    ///
    /// * `token_type` - token 类型
    /// * `start_offset` - token 起始字节偏移量（相对于输入字符串的开始）
    /// * `length` - token 长度（字节）
    /// * `start_line` - token 起始行号（从 1 开始）
    /// * `start_column` - token 起始列号（从 1 开始，UTF-16 列）
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// // 假设 MyToken 是实现了 TokenType 的类型
    /// state.add_token(MyToken::Keyword, 0, 3, 1, 1);   // "let"
    /// state.add_token(MyToken::Identifier, 4, 1, 1, 5); // "x"
    /// ```
    pub fn add_token(&mut self, token_type: T, start_offset: usize, length: usize, start_line: u32, start_column: u32) {
        let position = SourcePosition { offset: start_offset, length, line: start_line, column: start_column };
        self.tokens.push(Token { token_type, position });
    }

    /// 从当前位置添加 token（便捷方法）
    ///
    /// 这个方法使用当前的位置信息自动创建 token，比 `add_token` 更方便。
    ///
    /// # 参数
    ///
    /// * `token_type` - token 类型
    /// * `start_offset` - token 起始字节偏移量（相对于输入字符串的开始）
    /// * `length` - token 长度（字节）
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// // 假设 MyToken 是实现了 TokenType 的类型
    /// state.add_token_at_current(MyToken::Operator, 0, 1); // "+"
    /// ```
    pub fn add_token_at_current(&mut self, token_type: T, start_offset: usize, length: usize) {
        self.add_token(token_type, start_offset, length, self.line, self.column);
    }

    /// 添加单字符 token 并自动推进位置
    ///
    /// 这是一个便利方法，用于处理单字符 token（如括号、运算符等）。
    /// 它会自动标记当前位置，消耗指定字符，并添加 token。
    ///
    /// # 重要说明
    ///
    /// **此方法仅处理单个字符**，不适用于多字符序列如 `\r\n`：
    /// * 对于 `\r\n` 换行符，请使用 `advance_by_str` 方法
    /// * 此方法会将 `\r` 和 `\n` 分别作为独立的字符处理，可能导致行号计算错误
    ///
    /// # 参数
    ///
    /// * `token_type` - token 类型
    /// * `expected_char` - 期望的字符
    ///
    /// # 返回值
    ///
    /// * `Some(Token<T>)` - 成功创建的 token
    /// * `None` - 如果下一个字符不匹配或已到达输入末尾
    ///
    /// # 示例
    ///
    /// ```rust
    /// let input = "{hello}";
    /// let mut state = LexerState::new(input, None);
    ///
    /// // 处理左大括号
    /// if let Some(token) = state.advance_char(MyToken::LeftBrace, '{') {
    ///     println!("Found left brace at position {:?}", token.position);
    /// }
    /// ```
    ///
    /// # 注意
    ///
    /// 对于换行符处理：
    /// * `\n` - 正确增加行号，列号重置为 1
    /// * `\r` - 单独增加行号，列号重置为 1（不推荐，可能导致问题）
    /// * `\r\n` - **不支持**，应使用 `advance_by_str(state, "\r\n")`
    pub fn advance_by_char(&mut self, token_type: T, expected: char) {
        let (start, line, column) = self.mark_position();
        let length = self.consume_char(expected);
        self.add_token(token_type, start, length, line, column);
    }

    /// 添加字符串 token 并自动推进位置
    ///
    /// 这是一个便利方法，用于处理多字符 token（如运算符、关键字、换行符等）。
    /// 它会自动标记当前位置，消耗指定字符串，并添加 token。
    ///
    /// # 重要特性
    ///
    /// **正确处理 `\r\n` 换行符**：
    /// * 对于 `\r\n` 序列，会正确识别为单个换行符
    /// * 行号只增加一次，列号正确重置
    /// * 这是与 `advance_by_char` 方法的主要区别
    ///
    /// # 参数
    ///
    /// * `token_type` - token 类型
    /// * `expected` - 期望的字符串
    ///
    /// # 示例
    ///
    /// ```rust
    /// let input = "hello\r\nworld";
    /// let mut state = LexerState::new(input, None);
    ///
    /// // 正确处理 \r\n 换行符
    /// state.advance_by_str(MyToken::Newline, "\r\n");
    /// assert_eq!(state.get_position(), (2, 2, 1)); // 行号只增加一次
    ///
    /// // 处理多字符运算符
    /// state.advance_by_str(MyToken::Arrow, "=>");
    /// assert_eq!(state.get_position(), (4, 2, 3));
    /// ```
    ///
    /// # 注意
    ///
    /// 对于换行符处理：
    /// * `\n` - 正确增加行号，列号重置为 1
    /// * `\r` - 正确增加行号，列号重置为 1
    /// * `\r\n` - **正确识别为单个换行符**，行号只增加一次
    pub fn advance_by_str(&mut self, token_type: T, expected: &str) {
        let (start, line, column) = self.mark_position();
        let length = self.consume_str(expected);
        self.add_token(token_type, start, length, line, column);
    }

    /// 更新最后一个 token 的类型
    ///
    /// 这是一个便利方法，用于在需要根据内容重新分类 token 时使用。
    ///
    /// # 参数
    ///
    /// * `new_token_type` - 新的 token 类型
    ///
    /// # 返回值
    ///
    /// * `true` - 成功更新
    /// * `false` - 没有 token 可以更新
    ///
    /// # 示例
    ///
    /// ```rust
    /// # use gaia_types::lexer::{LexerState, TokenType};
    /// # #[derive(Clone, Copy, Debug, PartialEq)] enum TestToken { StringLiteral, TypeDescriptor, Eof }
    /// # impl TokenType for TestToken { const EOF: Self = TestToken::Eof; }
    /// let input = "string";
    /// let mut state = LexerState::new(input, None);
    ///
    /// // 先添加一个字符串字面量
    /// state.read_string_literal(TestToken::StringLiteral, '"');
    /// // 根据内容重新分类为类型描述符
    /// state.update_last_token_type(TestToken::TypeDescriptor);
    /// ```
    pub fn update_last_token_type(&mut self, new_token_type: T) -> bool {
        if let Some(last_token) = self.tokens.last_mut() {
            last_token.token_type = new_token_type;
            true
        }
        else {
            false
        }
    }

    /// 跳过空白字符
    ///
    /// 消耗并收集连续的空白字符作为单个 whitespace token
    ///
    /// # 参数
    ///
    /// * `token_type` - whitespace token 的类型
    pub fn skip_whitespace(&mut self, token_type: T) -> Option<Token<T>> {
        if let Some((start_offset, ch)) = self.peek() {
            if ch.is_whitespace() {
                let start_line = self.line;
                let start_column = self.column;
                let mut length = 0;

                // 消耗连续的空白字符
                while let Some((_offset, ch)) = self.peek() {
                    if !ch.is_whitespace() {
                        break;
                    }
                    length += ch.len_utf8();
                    self.next_char();
                }

                let position = SourcePosition { offset: start_offset, length, line: start_line, column: start_column };
                let token = Token { token_type, position };
                self.tokens.push(token.clone());
                return Some(token);
            }
        }
        None
    }

    /// 跳过单行注释
    ///
    /// 如果当前位置是注释开始，则消耗整个注释
    ///
    /// # 参数
    ///
    /// * `comment_type` - 注释 token 的类型
    /// * `start_marker` - 注释开始标记（如 "//"）
    ///
    /// # 返回值
    ///
    /// 如果成功跳过注释则返回注释 token，否则返回 `None`
    pub fn skip_line_comment(&mut self, comment_type: T, start_marker: &str) -> Option<Token<T>> {
        if let Some((start_offset, _)) = self.peek() {
            // 检查是否匹配注释开始标记
            let remaining = &self.input[start_offset..];
            if remaining.starts_with(start_marker) {
                let start_line = self.line;
                let start_column = self.column;
                let mut length = start_marker.len();

                // 消耗注释内容直到行尾
                self.next_char(); // 跳过第一个字符
                if start_marker.len() > 1 {
                    for _ in 1..start_marker.len() {
                        self.next_char();
                    }
                }

                while let Some((_, ch)) = self.peek() {
                    if ch == '\n' || ch == '\r' {
                        break;
                    }
                    length += ch.len_utf8();
                    self.next_char();
                }

                let position = SourcePosition { offset: start_offset, length, line: start_line, column: start_column };
                let token = Token { token_type: comment_type, position };
                self.tokens.push(token.clone());
                return Some(token);
            }
        }
        None
    }

    /// 读取标识符
    ///
    /// 读取连续的字母、数字、下划线等作为标识符
    ///
    /// # 参数
    ///
    /// * `identifier_type` - 标识符 token 的类型
    /// * `is_identifier_char` - 判断字符是否属于标识符的函数
    ///
    /// # 返回值
    ///
    /// 如果成功读取标识符则返回标识符 token，否则返回 `None`
    pub fn read_identifier<F>(&mut self, identifier_type: T, is_identifier_char: F) -> Option<Token<T>>
    where
        F: Fn(char) -> bool,
    {
        if let Some((start_offset, ch)) = self.peek() {
            if is_identifier_char(ch) {
                let start_line = self.line;
                let start_column = self.column;
                let mut length = ch.len_utf8();

                self.next_char();

                // 读取连续的标识符字符
                while let Some((_, ch)) = self.peek() {
                    if !is_identifier_char(ch) {
                        break;
                    }
                    length += ch.len_utf8();
                    self.next_char();
                }

                let position = SourcePosition { offset: start_offset, length, line: start_line, column: start_column };
                let token = Token { token_type: identifier_type, position };
                self.tokens.push(token.clone());
                return Some(token);
            }
        }
        None
    }

    /// 读取字符串字面量
    ///
    /// 读取完整的字符串字面量，包括引号
    ///
    /// # 参数
    ///
    /// * `string_type` - 字符串 token 的类型
    /// * `quote_char` - 字符串引号字符（如 '"' 或 '\''）
    ///
    /// # 返回值
    ///
    /// 如果成功读取字符串则返回字符串 token，否则返回 `None`
    pub fn read_string_literal(&mut self, string_type: T, quote_char: char) -> Option<Token<T>> {
        if let Some((start_offset, ch)) = self.peek() {
            if ch == quote_char {
                let start_line = self.line;
                let start_column = self.column;
                let mut length = ch.len_utf8();

                self.next_char(); // 跳过开始引号

                // 读取字符串内容
                while let Some((_, ch)) = self.peek() {
                    if ch == quote_char {
                        length += ch.len_utf8();
                        self.next_char(); // 消耗结束引号
                        break;
                    }
                    if ch == '\\' {
                        // 处理转义字符
                        length += ch.len_utf8();
                        self.next_char();
                        if let Some((_, escaped_ch)) = self.peek() {
                            length += escaped_ch.len_utf8();
                            self.next_char();
                        }
                    }
                    else {
                        length += ch.len_utf8();
                        self.next_char();
                    }
                }

                let position = SourcePosition { offset: start_offset, length, line: start_line, column: start_column };
                let token = Token { token_type: string_type, position };
                self.tokens.push(token.clone());
                return Some(token);
            }
        }
        None
    }

    /// 读取数字字面量
    ///
    /// 读取连续的数字字符
    ///
    /// # 参数
    ///
    /// * `number_type` - 数字 token 的类型
    ///
    /// # 返回值
    ///
    /// 如果成功读取数字则返回数字 token，否则返回 `None`
    pub fn read_integer(&mut self, number_type: T) -> Option<Token<T>> {
        if let Some((start_offset, ch)) = self.peek() {
            if ch.is_ascii_digit() {
                let start_line = self.line;
                let start_column = self.column;
                let mut length = ch.len_utf8();

                self.next_char();

                // 读取连续的数字字符
                while let Some((_, ch)) = self.peek() {
                    if !ch.is_ascii_digit() && ch != '.' {
                        break;
                    }
                    length += ch.len_utf8();
                    self.next_char();
                }

                let position = SourcePosition { offset: start_offset, length, line: start_line, column: start_column };
                let token = Token { token_type: number_type, position };
                self.tokens.push(token.clone());
                return Some(token);
            }
        }
        None
    }

    /// 获取已收集的 token 数量
    ///
    /// 返回当前已收集的 token 总数
    ///
    /// # 返回值
    ///
    /// 返回 `usize` 类型的 token 数量
    ///
    /// # 示例
    ///
    /// ```rust
    /// # use gaia_types::lexer::{LexerState, TokenType};
    /// # #[derive(Clone, Copy, Debug, PartialEq)] enum TestToken { Keyword, Identifier, Eof }
    /// # impl TokenType for TestToken { const EOF: Self = TestToken::Eof; }
    /// let input = "let x = 42";
    /// let mut state = LexerState::new(input, None);
    ///
    /// assert_eq!(state.token_count(), 0); // 初始为空
    ///
    /// // 添加一些 token
    /// state.add_token(TestToken::Keyword, 0, 3, 1, 1); // "let"
    /// state.add_token(TestToken::Identifier, 4, 1, 1, 5); // "x"
    ///
    /// assert_eq!(state.token_count(), 2);
    /// ```
    pub fn token_count(&self) -> usize {
        self.tokens.len()
    }

    /// 获取指定索引的 token
    ///
    /// 根据索引获取已收集的 token 引用
    ///
    /// # 参数
    ///
    /// * `index` - token 的索引位置（从 0 开始）
    ///
    /// # 返回值
    ///
    /// * `Some(&Token<T>)` - 成功获取的 token 引用
    /// * `None` - 索引超出范围
    ///
    /// # 示例
    ///
    /// ```rust
    /// let input = "let x";
    /// let mut state = LexerState::new(input, None);
    ///
    /// state.add_token(MyToken::Keyword, 0, 3, 1, 1); // "let"
    /// state.add_token(MyToken::Identifier, 4, 1, 1, 5); // "x"
    ///
    /// if let Some(token) = state.get_token(0) {
    ///     println!("第一个 token: {:?}", token.token_type);
    /// }
    ///
    /// assert!(state.get_token(2).is_none()); // 超出范围
    /// ```
    pub fn get_token(&self, index: usize) -> Option<&Token<T>> {
        self.tokens.get(index)
    }

    /// 标记语法错误
    ///
    /// 在当前位置创建一个语法错误并添加到诊断信息中
    ///
    /// # 参数
    ///
    /// * `message` - 错误信息，可以是任何实现了 `ToString` 的类型
    ///
    /// # 示例
    ///
    /// ```rust
    /// # use gaia_types::lexer::{LexerState, TokenType};
    /// # use gaia_types::errors::{GaiaError, SourceLocation};
    /// # #[derive(Clone, Copy, Debug, PartialEq)] enum TestToken { Eof }
    /// # impl TokenType for TestToken { const EOF: Self = TestToken::Eof; }
    /// let input = "let x = @";
    /// let mut state = LexerState::new(input, None);
    ///
    /// // 移动到非法字符位置
    /// state.next_char();
    /// state.next_char();
    /// state.next_char();
    /// state.next_char();
    /// state.next_char();
    /// state.next_char();
    /// state.next_char();
    ///
    /// // 标记错误
    /// state.mark_error("意外的字符 '@'");
    ///
    /// // 获取诊断结果时会包含这个错误
    /// let diagnostics = state.success(TestToken::Eof);
    /// assert!(!diagnostics.diagnostics.is_empty());
    /// ```
    pub fn mark_error(&mut self, message: impl ToString) {
        let location = SourceLocation { line: self.line, column: self.column, url: self.url.cloned() };
        let error = GaiaError::syntax_error(message, location);
        self.diagnostics.push(error);
    }

    /// 创建成功的诊断结果
    ///
    /// 创建包含 token 流的成功诊断结果，并自动添加 EOF token
    ///
    /// 这个方法会消耗 `LexerState`，返回完整的词法分析结果。
    /// 它会自动在 token 流末尾添加 EOF token，表示输入结束。
    ///
    /// # 参数
    ///
    /// * `eof_type` - EOF token 的类型
    ///
    /// # 返回值
    ///
    /// 返回 `GaiaDiagnostics<TokenStream<T>>` 实例，包含：
    /// * `Ok(TokenStream<T>)` - 成功的 token 流
    /// * `diagnostics` - 收集到的所有诊断信息（警告、非致命错误）
    ///
    /// # 示例
    ///
    /// ```rust
    /// # use gaia_types::lexer::{LexerState, TokenType};
    /// # #[derive(Clone, Copy, Debug, PartialEq)] enum TestToken { Keyword, Identifier, Eof }
    /// # impl TokenType for TestToken { const EOF: Self = TestToken::Eof; }
    /// let input = "let x = 42";
    /// let mut state = LexerState::new(input, None);
    ///
    /// // ... 添加 token ...
    ///
    /// let diagnostics = state.success(TestToken::Eof);
    /// match diagnostics.result {
    ///     Ok(token_stream) => {
    ///         println!("词法分析成功，共 {} 个 token", token_stream.tokens.len());
    ///     }
    ///     Err(error) => {
    ///         println!("词法分析失败: {}", error);
    ///     }
    /// }
    /// ```
    pub fn success(mut self) -> GaiaDiagnostics<TokenStream<'input, T>> {
        let position = SourcePosition { offset: self.input.len(), length: 0, line: self.line, column: self.column };
        self.tokens.push(Token { token_type: T::END_OF_STREAM, position });
        GaiaDiagnostics { result: Ok(TokenStream::new(self.input, self.tokens)), diagnostics: self.diagnostics }
    }

    /// 创建失败的诊断结果
    ///
    /// 创建包含致命错误的失败诊断结果
    ///
    /// 当遇到无法恢复的错误时使用此方法，它会返回一个致命错误，
    /// 词法分析过程应该立即停止。
    ///
    /// # 参数
    ///
    /// * `fatal` - 致命错误
    ///
    /// # 返回值
    ///
    /// 返回 `GaiaDiagnostics<TokenStream<T>>` 实例，其中：
    /// * `result` 是 `Err(fatal)`
    /// * `diagnostics` 包含之前收集的所有诊断信息
    ///
    /// # 示例
    ///
    /// ```rust
    /// # use gaia_types::lexer::{LexerState, TokenType};
    /// # use gaia_types::errors::{GaiaError, SourceLocation};
    /// # #[derive(Clone, Copy, Debug, PartialEq)] enum TestToken { Eof }
    /// # impl TokenType for TestToken { const EOF: Self = TestToken::Eof; }
    /// let input = "let x = @";
    /// let mut state = LexerState::new(input, None);
    ///
    /// // ... 处理输入 ...
    ///
    /// // 遇到致命错误
    /// let fatal_error = GaiaError::syntax_error("无法识别的字符",
    ///     SourceLocation { line: 1, column: 9, url: None });
    /// let diagnostics = state.failure(fatal_error);
    ///
    /// assert!(diagnostics.result.is_err());
    /// ```
    pub fn failure(self, fatal: GaiaError) -> GaiaDiagnostics<TokenStream<'input, T>> {
        GaiaDiagnostics { result: Err(fatal), diagnostics: self.diagnostics }
    }
}
