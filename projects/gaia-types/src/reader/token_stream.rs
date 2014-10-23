use crate::{reader::Token, GaiaError};
use std::io::Cursor;

/// 令牌流，用于在词法分析过程中管理和访问令牌序列
///
/// `TokenStream` 包装了一个令牌向量，并提供游标式的访问接口。
/// 它维护了对原始输入字符串的引用，以便能够提取令牌对应的文本内容。
///
/// # 泛型参数
///
/// * `'input` - 输入字符串的生命周期
/// * `T` - 令牌类型，必须实现 `Copy` trait
///
/// # 字段
///
/// * `raw` - 原始输入字符串的引用
/// * `tokens` - 令牌向量的游标，用于跟踪当前位置
#[derive(Clone, Debug)]
pub struct TokenStream<'input, T: Copy> {
    pub raw: &'input str,
    pub tokens: Cursor<Vec<Token<T>>>,
}

impl<'input, T: Copy> TokenStream<'input, T> {
    /// 创建一个新的令牌流实例
    ///
    /// # 参数
    ///
    /// * `raw` - 原始输入字符串的引用
    /// * `tokens` - 令牌向量
    ///
    /// # 返回值
    ///
    /// 返回一个新的 `TokenStream` 实例，游标初始位置在向量开始处
    ///
    /// # 示例
    ///
    /// ```
    /// use gaia_types::reader::{SourcePosition, Token, TokenStream};
    ///
    /// let input = "hello world";
    /// let tokens = vec![
    ///     Token {
    ///         token_type: 1,
    ///         position: SourcePosition { line: 1, column: 1, offset: 0, length: 5 },
    ///     },
    ///     Token {
    ///         token_type: 2,
    ///         position: SourcePosition { line: 1, column: 7, offset: 6, length: 5 },
    ///     },
    /// ];
    /// let stream = TokenStream::new(input, tokens);
    /// ```
    pub fn new(raw: &'input str, tokens: Vec<Token<T>>) -> Self {
        Self { raw, tokens: Cursor::new(tokens) }
    }

    /// 获取当前游标位置的令牌类型
    ///
    /// 此方法尝试从游标的当前位置获取令牌类型。如果游标已经到达向量末尾，
    /// 则返回一个错误，指示已经到达输入流的结尾。
    ///
    /// # 返回值
    ///
    /// * `Ok(T)` - 成功时返回当前位置的令牌类型
    /// * `Err(GaiaError)` - 失败时返回错误，通常是到达流末尾
    ///
    /// # 错误
    ///
    /// 当游标已经到达令牌向量末尾时，返回 `GaiaError` 表示输入流结束
    pub fn current(&self) -> Result<T, GaiaError> {
        let position = self.tokens.position() as usize;
        let tokens = self.tokens.get_ref();

        if position < tokens.len() {
            Ok(tokens[position].token_type)
        }
        else {
            // 到达流末尾，创建适当的错误
            Err(GaiaError::syntax_error("Unexpected end of input stream", crate::SourceLocation::default()))
        }
    }

    /// 获取当前游标位置的令牌引用
    ///
    /// 此方法尝试从游标的当前位置获取完整的令牌引用。如果游标已经到达向量末尾，
    /// 则返回一个错误，指示已经到达输入流的结尾。
    ///
    /// # 返回值
    ///
    /// * `Ok(&Token<T>)` - 成功时返回当前位置的令牌引用
    /// * `Err(GaiaError)` - 失败时返回错误，通常是到达流末尾
    ///
    /// # 错误
    ///
    /// 当游标已经到达令牌向量末尾时，返回 `GaiaError` 表示输入流结束
    pub fn current_token(&self) -> Result<&Token<T>, GaiaError> {
        self.get_token(self.get_index())
    }

    pub fn get_index(&self) -> usize {
        self.tokens.position() as usize
    }

    pub fn get_token(&self, index: usize) -> Result<&Token<T>, GaiaError> {
        let tokens = self.tokens.get_ref();
        if index < tokens.len() {
            Ok(&tokens[index])
        }
        else {
            Err(GaiaError::syntax_error("Unexpected end of input stream", crate::SourceLocation::default()))
        }
    }

    /// 获取当前令牌对应的文本内容
    ///
    /// 此方法使用令牌的 `get_range()` 方法从原始输入字符串中提取对应的文本片段。
    ///
    /// # 参数
    ///
    /// * `token` - 令牌引用
    ///
    /// # 返回值
    ///
    /// * `Ok(&str)` - 成功时返回当前令牌对应的文本切片
    /// * `Err(GaiaError)` - 失败时返回错误
    ///
    /// # 错误
    ///
    /// 可能返回的错误包括：
    /// * 令牌范围超出原始字符串边界
    ///
    /// # 示例
    ///
    /// ```
    /// use gaia_types::{
    ///     reader::{SourcePosition, Token, TokenStream},
    ///     GaiaError,
    /// };
    ///
    /// let input = "hello world";
    /// let tokens = vec![Token {
    ///     token_type: 1,
    ///     position: SourcePosition { line: 1, column: 1, offset: 0, length: 5 },
    /// }];
    /// let stream = TokenStream::new(input, tokens);
    /// let token = stream.current_token().unwrap();
    /// let text = stream.get_text(&token).unwrap();
    /// assert_eq!(text, "hello");
    /// ```
    pub fn get_text(&self, token: &Token<T>) -> Result<&'input str, GaiaError> {
        match self.raw.get(token.get_range()) {
            Some(s) => Ok(s),
            None => Err(GaiaError::invalid_range(self.raw.len(), token.position.offset + token.position.length)),
        }
    }
}
