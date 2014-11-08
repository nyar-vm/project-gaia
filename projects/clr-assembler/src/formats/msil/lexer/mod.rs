pub use self::token_type::MsilTokenType;
use crate::formats::msil::MsilReadConfig;
use gaia_types::{lexer::LexerState, reader::TokenStream, GaiaDiagnostics};

mod token_type;

/// MSIL 词法分析器
#[derive(Clone, Debug)]
pub struct MsilLexer<'config> {
    config: &'config MsilReadConfig,
}

impl<'config> MsilLexer<'config> {
    pub fn new(config: &'config MsilReadConfig) -> Self {
        Self { config }
    }

    /// 对输入的 MSIL 汇编代码进行词法分析
    pub fn tokenize<'input>(&self, input: &'input str) -> GaiaDiagnostics<TokenStream<'input, MsilTokenType>>
    where
        'config: 'input,
    {
        let mut state = LexerState::new(input, self.config.url.as_ref());

        while let Some((_offset, ch)) = state.peek() {
            match ch {
                // 处理空白字符
                ch if ch.is_whitespace() => {
                    state.skip_whitespace(MsilTokenType::Whitespace);
                }

                // 处理注释
                '/' => {
                    if state.skip_line_comment(MsilTokenType::Comment, "//").is_none() {
                        state.next_char(); // 如果不是注释，跳过这个字符
                    }
                }

                // 处理字符串字面量
                '"' => {
                    state.read_string_literal(MsilTokenType::StringLiteral, '"');
                }

                // 处理数字
                ch if ch.is_ascii_digit() => {
                    state.read_integer(MsilTokenType::Number);
                }

                // 处理标识符和关键字
                ch if ch.is_alphabetic() || ch == '_' => {
                    if let Some(token) = state.read_identifier(MsilTokenType::Identifier, |c| c.is_alphanumeric() || c == '_') {
                        // 获取标识符文本
                        let identifier_text = &input[token.position.offset..token.position.offset + token.position.length];

                        // 匹配关键字和指令
                        let token_type = match identifier_text {
                            "assembly" => MsilTokenType::Assembly,
                            "extern" => MsilTokenType::Extern,
                            "module" => MsilTokenType::Module,
                            "class" => MsilTokenType::Class,
                            "method" => MsilTokenType::Method,
                            "public" => MsilTokenType::Public,
                            "private" => MsilTokenType::Private,
                            "auto" => MsilTokenType::Auto,
                            "ansi" => MsilTokenType::Ansi,
                            "beforefieldinit" => MsilTokenType::Beforefieldinit,
                            "extends" => MsilTokenType::Extends,
                            "hidebysig" => MsilTokenType::Hidebysig,
                            "virtual" => MsilTokenType::Virtual,
                            "instance" => MsilTokenType::Instance,
                            "cil" => MsilTokenType::Cil,
                            "managed" => MsilTokenType::Managed,
                            "specialname" => MsilTokenType::Specialname,
                            "rtspecialname" => MsilTokenType::Rtspecialname,
                            "maxstack" => MsilTokenType::Maxstack,
                            "locals" => MsilTokenType::Locals,
                            "init" => MsilTokenType::Init,
                            "entrypoint" => MsilTokenType::Entrypoint,
                            "publickeytoken" => MsilTokenType::Publickeytoken,
                            "static" => MsilTokenType::Static,
                            "ctor" => MsilTokenType::Ctor,
                            "nop" => MsilTokenType::Nop,
                            "ldstr" => MsilTokenType::Ldstr,
                            "ldarg" => MsilTokenType::Ldarg,
                            "call" => MsilTokenType::Call,
                            "ret" => MsilTokenType::Ret,
                            "ver" => MsilTokenType::Ver,
                            "hash" => MsilTokenType::Hash,
                            "algorithm" => MsilTokenType::Algorithm,
                            _ => MsilTokenType::Identifier,
                        };

                        // 更新最后一个 token 的类型
                        state.update_last_token_type(token_type);
                    }
                }

                // 处理符号
                '{' => {
                    let (start, line, column) = state.mark_position();
                    state.next_char();
                    state.add_token(MsilTokenType::LeftBrace, start, 1, line, column);
                }
                '}' => {
                    let (start, line, column) = state.mark_position();
                    state.next_char();
                    state.add_token(MsilTokenType::RightBrace, start, 1, line, column);
                }
                '(' => {
                    let (start, line, column) = state.mark_position();
                    state.next_char();
                    state.add_token(MsilTokenType::LeftParen, start, 1, line, column);
                }
                ')' => {
                    let (start, line, column) = state.mark_position();
                    state.next_char();
                    state.add_token(MsilTokenType::RightParen, start, 1, line, column);
                }
                '[' => {
                    let (start, line, column) = state.mark_position();
                    state.next_char();
                    state.add_token(MsilTokenType::LeftBracket, start, 1, line, column);
                }
                ']' => {
                    let (start, line, column) = state.mark_position();
                    state.next_char();
                    state.add_token(MsilTokenType::RightBracket, start, 1, line, column);
                }
                ':' => {
                    let (start, line, column) = state.mark_position();
                    state.next_char();
                    state.add_token(MsilTokenType::Colon, start, 1, line, column);
                }
                ';' => {
                    let (start, line, column) = state.mark_position();
                    state.next_char();
                    state.add_token(MsilTokenType::Semicolon, start, 1, line, column);
                }
                '.' => {
                    let (start, line, column) = state.mark_position();
                    state.next_char();
                    state.add_token(MsilTokenType::Dot, start, 1, line, column);
                }
                ',' => {
                    let (start, line, column) = state.mark_position();
                    state.next_char();
                    state.add_token(MsilTokenType::Comma, start, 1, line, column);
                }

                _ => {
                    // 跳过未知字符
                    state.next_char();
                }
            }
        }

        state.success()
    }
}
