pub use self::token_type::JasmTokenType;
use gaia_types::{lexer::LexerState, reader::TokenStream, GaiaDiagnostics};

mod token_type;

/// JASM 词法分析器
///
/// 用于将 JASM 汇编代码文本转换为 token 流。
///
/// # 示例
///
/// ```rust
/// use jvm_jasm::lexer::JasmLexer;
///
/// let lexer = JasmLexer::new();
/// let result = lexer.tokenize("public class HelloJava");
/// ```
#[derive(Clone, Debug)]
pub struct JasmLexer;

impl Default for JasmLexer {
    fn default() -> Self {
        Self::new()
    }
}

impl JasmLexer {
    /// 创建一个新的 JASM 词法分析器实例
    pub fn new() -> Self {
        Self
    }

    /// 对输入的 JASM 汇编代码进行词法分析
    ///
    /// # 参数
    ///
    /// * `input` - 要分析的 JASM 汇编代码字符串
    ///
    /// # 返回值
    ///
    /// 返回包含 token 流的 `GaiaDiagnostics`，如果分析成功则包含 `TokenStream<JasmTokenType>`
    ///
    /// # 示例
    ///
    /// ```rust
    /// use jvm_jasm::lexer::JasmLexer;
    ///
    /// let lexer = JasmLexer::new();
    /// let result = lexer.tokenize("public class HelloJava");
    ///
    /// if let Ok(token_stream) = result.into_result() {
    ///     // 处理 token 流
    /// }
    /// ```
    pub fn tokenize<'input>(&self, input: &'input str) -> GaiaDiagnostics<TokenStream<'input, JasmTokenType>> {
        let mut state = LexerState::new(input, None);

        while let Some((_offset, ch)) = state.peek() {
            match ch {
                // 处理空白字符
                ch if ch.is_whitespace() => {
                    state.skip_whitespace(JasmTokenType::Whitespace);
                }

                // 处理注释
                '/' => {
                    if state.skip_line_comment(JasmTokenType::Comment, "//").is_none() {
                        // 如果不是注释，则作为普通符号处理
                        let (start_offset, _) = state.next_char().unwrap();
                        let (_, line, column) = state.get_position();
                        state.add_token(JasmTokenType::Slash, start_offset, 1, line, column);
                    }
                }

                // 处理字符串字面量
                '"' => {
                    if let Some(token) = state.read_string_literal(JasmTokenType::StringLiteral, '"') {
                        // 检查是否是类型描述符
                        let content = &input[token.position.offset + 1..token.position.offset + token.position.length - 1];
                        if content.starts_with('(')
                            || content.starts_with('[')
                            || (content.contains('L') && content.contains(';'))
                        {
                            // 更新最后一个 token 的类型为 TypeDescriptor
                            state.update_last_token_type(JasmTokenType::TypeDescriptor);
                        }
                    }
                }

                // 处理数字
                ch if ch.is_ascii_digit() => {
                    state.read_integer(JasmTokenType::Number);
                }

                // 处理标识符和关键字
                ch if ch.is_alphabetic() || ch == '_' || ch == '<' => {
                    if let Some(token) = state.read_identifier(JasmTokenType::Identifier, |c| {
                        c.is_alphanumeric() || c == '_' || c == '/' || c == '$' || c == '>' || c == 'i' || c == 'c'
                    }) {
                        // 获取标识符文本
                        let identifier_text = &input[token.position.offset..token.position.offset + token.position.length];

                        // 匹配关键字和指令
                        let token_type = match identifier_text {
                            // 关键字
                            "class" => JasmTokenType::Class,
                            "version" => JasmTokenType::Version,
                            "Method" => JasmTokenType::Method,
                            "Field" => JasmTokenType::Field,
                            "String" => JasmTokenType::String,
                            "SourceFile" => JasmTokenType::SourceFile,
                            "stack" => JasmTokenType::Stack,
                            "locals" => JasmTokenType::Locals,
                            "end" => JasmTokenType::End,
                            "compiled" => JasmTokenType::Compiled,
                            "from" => JasmTokenType::From,
                            "InnerClass" => JasmTokenType::InnerClass,
                            "NestMembers" => JasmTokenType::NestMembers,
                            "BootstrapMethod" => JasmTokenType::BootstrapMethod,

                            // 访问修饰符
                            "public" => JasmTokenType::Public,
                            "private" => JasmTokenType::Private,
                            "protected" => JasmTokenType::Protected,
                            "static" => JasmTokenType::Static,
                            "super" => JasmTokenType::Super,
                            "final" => JasmTokenType::Final,
                            "abstract" => JasmTokenType::Abstract,
                            "synchronized" => JasmTokenType::Synchronized,
                            "native" => JasmTokenType::Native,
                            "synthetic" => JasmTokenType::Synthetic,
                            "deprecated" => JasmTokenType::Deprecated,
                            "varargs" => JasmTokenType::Varargs,

                            // JVM 指令
                            "aload_0" => JasmTokenType::ALoad0,
                            "aload_1" => JasmTokenType::ALoad1,
                            "aload_2" => JasmTokenType::ALoad2,
                            "aload_3" => JasmTokenType::ALoad3,
                            "iload_0" => JasmTokenType::ILoad0,
                            "iload_1" => JasmTokenType::ILoad1,
                            "iload_2" => JasmTokenType::ILoad2,
                            "iload_3" => JasmTokenType::ILoad3,
                            "ldc" => JasmTokenType::Ldc,
                            "ldc_w" => JasmTokenType::LdcW,
                            "ldc2_w" => JasmTokenType::Ldc2W,
                            "invokespecial" => JasmTokenType::InvokeSpecial,
                            "invokevirtual" => JasmTokenType::InvokeVirtual,
                            "invokestatic" => JasmTokenType::InvokeStatic,
                            "invokeinterface" => JasmTokenType::InvokeInterface,
                            "invokedynamic" => JasmTokenType::InvokeDynamic,
                            "getstatic" => JasmTokenType::GetStatic,
                            "putstatic" => JasmTokenType::PutStatic,
                            "getfield" => JasmTokenType::GetField,
                            "putfield" => JasmTokenType::PutField,
                            "return" => JasmTokenType::Return,
                            "ireturn" => JasmTokenType::IReturn,
                            "areturn" => JasmTokenType::AReturn,
                            "lreturn" => JasmTokenType::LReturn,
                            "freturn" => JasmTokenType::FReturn,
                            "dreturn" => JasmTokenType::DReturn,
                            "nop" => JasmTokenType::Nop,
                            "dup" => JasmTokenType::Dup,
                            "pop" => JasmTokenType::Pop,
                            "new" => JasmTokenType::New,

                            _ => JasmTokenType::Identifier,
                        };

                        // 更新最后一个 token 的类型
                        state.update_last_token_type(token_type);
                    }
                }

                // 处理符号
                '{' => {
                    state.advance_by_char(JasmTokenType::LeftBrace, '{');
                }
                '}' => {
                    state.advance_by_char(JasmTokenType::RightBrace, '}');
                }
                '(' => {
                    state.advance_by_char(JasmTokenType::LeftParen, '(');
                }
                ')' => {
                    state.advance_by_char(JasmTokenType::RightParen, ')');
                }
                '[' => {
                    state.advance_by_char(JasmTokenType::LeftBracket, '[');
                }
                ']' => {
                    state.advance_by_char(JasmTokenType::RightBracket, ']');
                }
                ':' => {
                    state.advance_by_char(JasmTokenType::Colon, ':');
                }
                ';' => {
                    state.advance_by_char(JasmTokenType::Semicolon, ';');
                }
                '.' => {
                    state.advance_by_char(JasmTokenType::Dot, '.');
                }
                ',' => {
                    state.advance_by_char(JasmTokenType::Comma, ',');
                }

                _ => {
                    // 忽略其他字符
                    state.next_char();
                }
            }
        }

        state.success()
    }
}
