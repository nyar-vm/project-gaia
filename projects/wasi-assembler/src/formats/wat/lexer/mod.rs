#![doc = include_str!("readme.md")]

use gaia_types::{lexer::LexerState, reader::TokenStream, GaiaDiagnostics};

/// Token type definitions for WAT lexer
mod token_type;
pub use token_type::WatTokenType;

/// WAT (WebAssembly Text) lexer for Component Model
///
/// This lexer tokenizes Component Model WAT source code, handling:
/// - Component-specific syntax (component, core module, etc.)
/// - Block comments `(; ... ;)`
/// - Line comments `;;`
/// - String literals with escape sequences
/// - Numeric literals (integers and floats)
/// - Identifiers and keywords
/// - Symbols and delimiters
#[derive(Debug, Copy, Clone)]
pub struct WatLexer;

impl WatLexer {
    /// Creates a new WAT lexer instance
    pub fn new() -> Self {
        Self
    }

    /// Tokenizes the input WAT source code
    ///
    /// # Arguments
    /// * `input` - The WAT source code to tokenize
    ///
    /// # Returns
    /// A `GaiaDiagnostics` containing the token stream or error information
    pub fn tokenize<'input>(&self, input: &'input str) -> GaiaDiagnostics<TokenStream<'input, WatTokenType>> {
        let mut state = LexerState::new(input, None);

        while !state.rest_text().is_empty() {
            if let Some((_, ch)) = state.peek() {
                match ch {
                    // Skip whitespace
                    ' ' | '\t' | '\r' => {
                        state.next_char();
                    }

                    // Handle newlines
                    '\n' => {
                        state.advance_by_char(WatTokenType::Newline, '\n');
                    }

                    // Handle comments and parentheses
                    '(' => {
                        let rest = state.rest_text();
                        if rest.len() > 1 && rest.chars().nth(1) == Some(';') {
                            self.skip_block_comment(&mut state);
                        }
                        else {
                            state.advance_by_char(WatTokenType::LeftParen, '(');
                        }
                    }

                    ')' => {
                        state.advance_by_char(WatTokenType::RightParen, ')');
                    }

                    // Handle line comments
                    ';' => {
                        let rest = state.rest_text();
                        if rest.len() > 1 && rest.chars().nth(1) == Some(';') {
                            self.skip_line_comment(&mut state);
                        }
                        else {
                            state.advance_by_char(WatTokenType::Semicolon, ';');
                        }
                    }

                    // Handle string literals
                    '"' => {
                        self.read_string_literal(&mut state);
                    }

                    // Handle symbols
                    '{' => state.advance_by_char(WatTokenType::LeftBrace, '{'),
                    '}' => state.advance_by_char(WatTokenType::RightBrace, '}'),
                    '[' => state.advance_by_char(WatTokenType::LeftBracket, '['),
                    ']' => state.advance_by_char(WatTokenType::RightBracket, ']'),
                    ',' => state.advance_by_char(WatTokenType::Comma, ','),
                    ':' => {
                        let rest = state.rest_text();
                        if rest.len() > 1 && rest.chars().nth(1) == Some(':') {
                            state.next_char(); // consume first ':'
                            state.advance_by_char(WatTokenType::DoubleColon, ':');
                        }
                        else {
                            state.advance_by_char(WatTokenType::Colon, ':');
                        }
                    }
                    '.' => state.advance_by_char(WatTokenType::Dot, '.'),
                    '$' => state.advance_by_char(WatTokenType::Dollar, '$'),
                    '@' => state.advance_by_char(WatTokenType::At, '@'),
                    '=' => state.advance_by_char(WatTokenType::Equals, '='),

                    // Handle numeric literals
                    '0'..='9' => {
                        self.read_numeric_literal(&mut state);
                    }

                    // Handle negative numbers
                    '-' => {
                        let rest = state.rest_text();
                        if rest.len() > 1 && matches!(rest.chars().nth(1), Some('0'..='9')) {
                            self.read_numeric_literal(&mut state);
                        }
                        else {
                            // Treat as identifier (e.g., in instruction names)
                            self.read_identifier(&mut state);
                        }
                    }

                    // Handle identifiers and keywords
                    'a'..='z' | 'A'..='Z' | '_' => {
                        self.read_identifier(&mut state);
                    }

                    // Handle unexpected characters
                    c => {
                        state.mark_error(format!("Unexpected character: '{}'", c));
                        state.next_char();
                    }
                }
            }
        }

        state.success()
    }

    /// Skips a line comment `;;`
    fn skip_line_comment(&self, state: &mut LexerState<WatTokenType>) {
        while !state.rest_text().is_empty() {
            if let Some((_, ch)) = state.peek() {
                if ch == '\n' {
                    break;
                }
                state.next_char();
            }
            else {
                break;
            }
        }
    }

    /// Skips a block comment `(; ... ;)`
    fn skip_block_comment(&self, state: &mut LexerState<WatTokenType>) {
        state.next_char(); // consume '('
        state.next_char(); // consume ';'

        let mut depth = 1;
        while !state.rest_text().is_empty() && depth > 0 {
            if let Some((_, ch)) = state.peek() {
                match ch {
                    '(' => {
                        let rest = state.rest_text();
                        if rest.len() > 1 && rest.chars().nth(1) == Some(';') {
                            state.next_char(); // consume '('
                            state.next_char(); // consume ';'
                            depth += 1;
                        }
                        else {
                            state.next_char();
                        }
                    }
                    ';' => {
                        let rest = state.rest_text();
                        if rest.len() > 1 && rest.chars().nth(1) == Some(')') {
                            state.next_char(); // consume ';'
                            state.next_char(); // consume ')'
                            depth -= 1;
                        }
                        else {
                            state.next_char();
                        }
                    }
                    _ => {
                        state.next_char();
                    }
                }
            }
            else {
                break;
            }
        }

        if depth > 0 {
            state.mark_error("Unterminated block comment".to_string());
        }
    }

    /// Reads a string literal with escape sequence support
    fn read_string_literal(&self, state: &mut LexerState<WatTokenType>) {
        let (start, line, column) = state.mark_position();
        state.next_char(); // consume opening quote

        while !state.rest_text().is_empty() {
            if let Some((_, ch)) = state.peek() {
                if ch == '"' {
                    state.next_char(); // consume closing quote
                    let length = state.get_position().0 - start;
                    state.add_token(WatTokenType::StringLiteral, start, length, line, column);
                    return;
                }
                else if ch == '\\' {
                    state.next_char(); // consume backslash
                    if !state.rest_text().is_empty() {
                        state.next_char(); // consume escaped character
                    }
                }
                else {
                    state.next_char();
                }
            }
            else {
                break;
            }
        }

        state.mark_error("Unterminated string literal".to_string());
    }

    /// Reads a numeric literal (integer or float)
    fn read_numeric_literal(&self, state: &mut LexerState<WatTokenType>) {
        let (start, line, column) = state.mark_position();

        // Handle optional negative sign
        if let Some((_, ch)) = state.peek() {
            if ch == '-' {
                state.next_char();
            }
        }

        // Handle hexadecimal numbers (0x...)
        if let Some((_, ch)) = state.peek() {
            if ch == '0' {
                let rest = state.rest_text();
                if rest.len() > 1 && matches!(rest.chars().nth(1), Some('x' | 'X')) {
                    state.next_char(); // consume '0'
                    state.next_char(); // consume 'x' or 'X'

                    while let Some((_, ch)) = state.peek() {
                        if matches!(ch, '0'..='9' | 'a'..='f' | 'A'..='F' | '_') {
                            state.next_char();
                        }
                        else {
                            break;
                        }
                    }

                    let length = state.get_position().0 - start;
                    state.add_token(WatTokenType::IntegerLiteral, start, length, line, column);
                    return;
                }
            }
        }

        // Read integer part
        while let Some((_, ch)) = state.peek() {
            if matches!(ch, '0'..='9' | '_') {
                state.next_char();
            }
            else {
                break;
            }
        }

        // Check for float (decimal point)
        let mut is_float = false;
        if let Some((_, ch)) = state.peek() {
            if ch == '.' {
                let rest = state.rest_text();
                if rest.len() > 1 && matches!(rest.chars().nth(1), Some('0'..='9')) {
                    is_float = true;
                    state.next_char(); // consume '.'

                    // Read fractional part
                    while let Some((_, ch)) = state.peek() {
                        if matches!(ch, '0'..='9' | '_') {
                            state.next_char();
                        }
                        else {
                            break;
                        }
                    }
                }
            }
        }

        // Check for exponent
        if let Some((_, ch)) = state.peek() {
            if matches!(ch, 'e' | 'E') {
                is_float = true;
                state.next_char(); // consume 'e' or 'E'

                if let Some((_, ch)) = state.peek() {
                    if matches!(ch, '+' | '-') {
                        state.next_char(); // consume sign
                    }
                }

                while let Some((_, ch)) = state.peek() {
                    if matches!(ch, '0'..='9' | '_') {
                        state.next_char();
                    }
                    else {
                        break;
                    }
                }
            }
        }

        let length = state.get_position().0 - start;
        if is_float {
            state.add_token(WatTokenType::FloatLiteral, start, length, line, column);
        }
        else {
            state.add_token(WatTokenType::IntegerLiteral, start, length, line, column);
        }
    }

    /// Reads an identifier or keyword token
    fn read_identifier(&self, state: &mut LexerState<WatTokenType>) {
        let (start, line, column) = state.mark_position();

        // Read identifier characters
        while let Some((_, ch)) = state.peek() {
            if matches!(ch, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-' | '.' | '$') {
                state.next_char();
            }
            else {
                break;
            }
        }

        let length = state.get_position().0 - start;
        // We'll classify based on the token type for now, and let the parser handle the text
        let token_type = WatTokenType::Identifier; // Default to identifier, parser can reclassify
        state.add_token(token_type, start, length, line, column);
    }

    /// Classifies a text as a keyword or identifier
    fn classify_keyword(&self, text: &str) -> WatTokenType {
        match text {
            // Component Model Keywords
            "component" => WatTokenType::Component,
            "core" => WatTokenType::Core,
            "custom" => WatTokenType::Custom,
            "import" => WatTokenType::Import,
            "export" => WatTokenType::Export,
            "instance" => WatTokenType::Instance,
            "alias" => WatTokenType::Alias,
            "outer" => WatTokenType::Outer,

            // Type System Keywords
            "type" => WatTokenType::Type,
            "func" => WatTokenType::Func,
            "result" => WatTokenType::Result,
            "param" => WatTokenType::Param,
            "resource" => WatTokenType::Resource,
            "record" => WatTokenType::Record,
            "variant" => WatTokenType::Variant,
            "enum" => WatTokenType::Enum,
            "union" => WatTokenType::Union,
            "option" => WatTokenType::Option,
            "list" => WatTokenType::List,
            "tuple" => WatTokenType::Tuple,
            "flags" => WatTokenType::Flags,

            // Canonical Operations
            "canon" => WatTokenType::Canon,
            "lower" => WatTokenType::Lower,
            "lift" => WatTokenType::Lift,
            "resource.new" => WatTokenType::ResourceNew,
            "resource.drop" => WatTokenType::ResourceDrop,
            "resource.rep" => WatTokenType::ResourceRep,

            // Core WebAssembly Keywords
            "module" => WatTokenType::Module,
            "memory" => WatTokenType::Memory,
            "table" => WatTokenType::Table,
            "global" => WatTokenType::Global,
            "start" => WatTokenType::Start,
            "data" => WatTokenType::Data,
            "elem" => WatTokenType::Elem,

            // WebAssembly Instructions
            "call" => WatTokenType::Call,
            "call_indirect" => WatTokenType::CallIndirect,
            "local.get" => WatTokenType::LocalGet,
            "local.set" => WatTokenType::LocalSet,
            "local.tee" => WatTokenType::LocalTee,
            "global.get" => WatTokenType::GlobalGet,
            "global.set" => WatTokenType::GlobalSet,
            "i32.const" => WatTokenType::I32Const,
            "i64.const" => WatTokenType::I64Const,
            "f32.const" => WatTokenType::F32Const,
            "f64.const" => WatTokenType::F64Const,
            "i32.load" => WatTokenType::I32Load,
            "i32.store" => WatTokenType::I32Store,
            "drop" => WatTokenType::Drop,
            "select" => WatTokenType::Select,
            "unreachable" => WatTokenType::Unreachable,
            "nop" => WatTokenType::Nop,
            "block" => WatTokenType::Block,
            "loop" => WatTokenType::Loop,
            "if" => WatTokenType::If,
            "else" => WatTokenType::Else,
            "end" => WatTokenType::End,
            "br" => WatTokenType::Br,
            "br_if" => WatTokenType::BrIf,
            "br_table" => WatTokenType::BrTable,
            "return" => WatTokenType::Return,

            // WebAssembly Types
            "i32" => WatTokenType::I32,
            "i64" => WatTokenType::I64,
            "f32" => WatTokenType::F32,
            "f64" => WatTokenType::F64,
            "v128" => WatTokenType::V128,
            "funcref" => WatTokenType::Funcref,
            "externref" => WatTokenType::Externref,

            // Modifiers
            "mut" => WatTokenType::Mut,
            "shared" => WatTokenType::Shared,

            // Default to identifier
            _ => WatTokenType::Identifier,
        }
    }
}
