use crate::{
    ast::{JasmClass, JasmInstruction, JasmMethod, JasmRoot},
    lexer::{JasmLexer, JasmTokenType},
};
use gaia_types::{reader::TokenStream, GaiaDiagnostics, GaiaError};

/// JASM 语法分析器
#[derive(Clone, Debug)]
pub struct JasmParser;

impl JasmParser {
    pub fn new() -> Self {
        Self
    }

    /// 解析 JASM 文本
    pub fn parse_text(&self, text: &str) -> GaiaDiagnostics<JasmRoot> {
        let lexer = JasmLexer::new();
        let tokens_result = lexer.tokenize(text);
        match tokens_result.result {
            Ok(tokens) => self.parse(tokens),
            Err(error) => GaiaDiagnostics::failure(error),
        }
    }

    /// 解析 token 流
    pub fn parse(&self, tokens: TokenStream<JasmTokenType>) -> GaiaDiagnostics<JasmRoot> {
        let mut parser_state = ParserState::new(&tokens);
        match self.parse_class(&mut parser_state) {
            Ok(class) => GaiaDiagnostics::success(JasmRoot { class }),
            Err(error) => GaiaDiagnostics::failure(error),
        }
    }

    /// 解析类声明
    fn parse_class(&self, state: &mut ParserState) -> Result<JasmClass, GaiaError> {
        let mut modifiers = Vec::new();
        let mut class_name = String::new();
        let mut version = None;
        let mut methods = Vec::new();
        let mut fields = Vec::new();
        let mut source_file = None;

        // 跳过空白和注释
        state.skip_ignored();

        // 解析访问修饰符
        while let Some(token) = state.current_token() {
            let loop_start_index = state.current_index;

            match token.token_type {
                JasmTokenType::Public => {
                    modifiers.push("public".to_string());
                    state.advance();
                }
                JasmTokenType::Private => {
                    modifiers.push("private".to_string());
                    state.advance();
                }
                JasmTokenType::Protected => {
                    modifiers.push("protected".to_string());
                    state.advance();
                }
                JasmTokenType::Static => {
                    modifiers.push("static".to_string());
                    state.advance();
                }
                JasmTokenType::Final => {
                    modifiers.push("final".to_string());
                    state.advance();
                }
                JasmTokenType::Abstract => {
                    modifiers.push("abstract".to_string());
                    state.advance();
                }
                JasmTokenType::Super => {
                    modifiers.push("super".to_string());
                    state.advance();
                }
                _ => break,
            }
            state.skip_ignored();

            // 强制检查：如果没有前进，强制 advance 并报错
            if state.current_index == loop_start_index {
                state.advance();
                return Err(GaiaError::syntax_error(
                    "Unexpected token in access modifiers",
                    gaia_types::SourceLocation::default(),
                ));
            }
        }

        // 期望 class 关键字
        if !state.expect_token(JasmTokenType::Class) {
            return Err(GaiaError::syntax_error("Expected 'class' keyword", gaia_types::SourceLocation::default()));
        }
        state.skip_ignored();

        // 解析类名
        if let Some(token) = state.current_token() {
            if token.token_type == JasmTokenType::Identifier {
                class_name = state.get_token_text(token)?.to_string();
                state.advance();
            }
            else {
                return Err(GaiaError::syntax_error("Expected class name", gaia_types::SourceLocation::default()));
            }
        }
        else {
            return Err(GaiaError::syntax_error("Expected class name", gaia_types::SourceLocation::default()));
        }
        state.skip_ignored();

        // 解析泛型签名（可选）
        if let Some(token) = state.current_token() {
            if token.token_type == JasmTokenType::Colon {
                state.advance();
                state.skip_ignored();

                // 期望字符串字面量或类型描述符（泛型签名）
                if let Some(signature_token) = state.current_token() {
                    if signature_token.token_type == JasmTokenType::StringLiteral
                        || signature_token.token_type == JasmTokenType::TypeDescriptor
                    {
                        // 跳过泛型签名，暂时不存储
                        state.advance();
                    }
                }
            }
        }
        state.skip_ignored();

        // 解析版本信息（可选）
        if let Some(token) = state.current_token() {
            if token.token_type == JasmTokenType::Version {
                state.advance();
                state.skip_ignored();

                // 解析 major:minor 格式的版本号
                let mut version_string = String::new();

                // 解析主版本号
                if let Some(major_token) = state.current_token() {
                    if major_token.token_type == JasmTokenType::Number {
                        version_string.push_str(state.get_token_text(major_token)?);
                        state.advance();
                        state.skip_ignored();

                        // 解析冒号
                        if let Some(colon_token) = state.current_token() {
                            if colon_token.token_type == JasmTokenType::Colon {
                                version_string.push(':');
                                state.advance();
                                state.skip_ignored();

                                // 解析次版本号
                                if let Some(minor_token) = state.current_token() {
                                    if minor_token.token_type == JasmTokenType::Number {
                                        version_string.push_str(state.get_token_text(minor_token)?);
                                        state.advance();
                                    }
                                }
                            }
                        }

                        version = Some(version_string);
                    }
                }
            }
        }
        state.skip_ignored();

        // 期望左大括号
        if !state.expect_token(JasmTokenType::LeftBrace) {
            return Err(GaiaError::syntax_error("Expected '{'", gaia_types::SourceLocation::default()));
        }
        state.skip_ignored();

        // 解析类体内容
        while let Some(token) = state.current_token() {
            let loop_start_index = state.current_index;

            match token.token_type {
                JasmTokenType::RightBrace => {
                    state.advance();
                    break;
                }
                JasmTokenType::Public
                | JasmTokenType::Private
                | JasmTokenType::Protected
                | JasmTokenType::Static
                | JasmTokenType::Synthetic
                | JasmTokenType::Final
                | JasmTokenType::Deprecated
                | JasmTokenType::Varargs => {
                    // 可能是方法或字段
                    let current_index = state.current_index;
                    match self.parse_method(state) {
                        Ok(method) => {
                            methods.push(method);
                        }
                        Err(e) => {
                            // 打印错误信息以便调试
                            eprintln!("Failed to parse method: {:?}", e);
                            // 如果解析失败，确保前进至少一个 token 以避免无限循环
                            if state.current_index == current_index {
                                state.advance();
                            }
                            // 继续尝试解析下一个元素，而不是直接返回错误
                        }
                    }
                }
                JasmTokenType::SourceFile => {
                    state.advance();
                    state.skip_ignored();

                    if let Some(source_token) = state.current_token() {
                        if source_token.token_type == JasmTokenType::StringLiteral {
                            let source_text = state.get_token_text(source_token)?;
                            // 移除引号
                            source_file = Some(source_text.trim_matches('"').to_string());
                            state.advance();
                        }
                    }

                    // 跳过分号
                    state.skip_ignored();
                    if let Some(token) = state.current_token() {
                        if token.token_type == JasmTokenType::Semicolon {
                            state.advance();
                        }
                    }
                }
                JasmTokenType::InnerClass => {
                    // 跳过 InnerClass 声明
                    state.advance();
                    // 跳过到分号或换行
                    while let Some(token) = state.current_token() {
                        if token.token_type == JasmTokenType::Semicolon || token.token_type == JasmTokenType::Eof {
                            if token.token_type == JasmTokenType::Semicolon {
                                state.advance();
                            }
                            break;
                        }
                        state.advance();
                    }
                }
                JasmTokenType::NestMembers => {
                    // 跳过 NestMembers 声明
                    state.advance();
                    // 跳过到分号或换行
                    while let Some(token) = state.current_token() {
                        if token.token_type == JasmTokenType::Semicolon || token.token_type == JasmTokenType::Eof {
                            if token.token_type == JasmTokenType::Semicolon {
                                state.advance();
                            }
                            break;
                        }
                        state.advance();
                    }
                }
                JasmTokenType::BootstrapMethod => {
                    // 跳过 BootstrapMethod 声明
                    state.advance();
                    // 跳过到分号或换行
                    while let Some(token) = state.current_token() {
                        if token.token_type == JasmTokenType::Semicolon || token.token_type == JasmTokenType::Eof {
                            if token.token_type == JasmTokenType::Semicolon {
                                state.advance();
                            }
                            break;
                        }
                        state.advance();
                    }
                }
                JasmTokenType::Eof => break,
                _ => {
                    state.advance();
                }
            }
            state.skip_ignored();

            // 强制检查：如果没有前进，强制 advance 并报错
            if state.current_index == loop_start_index {
                state.advance();
                return Err(GaiaError::syntax_error("Unexpected token in class body", gaia_types::SourceLocation::default()));
            }
        }

        Ok(JasmClass { modifiers, name: class_name, version, methods, fields, source_file })
    }

    /// 解析方法声明
    fn parse_method(&self, state: &mut ParserState) -> Result<JasmMethod, GaiaError> {
        let mut modifiers = Vec::new();
        let mut name_and_descriptor = String::new();
        let mut stack_size = None;
        let mut locals_count = None;
        let mut instructions = Vec::new();

        // 解析方法修饰符
        while let Some(token) = state.current_token() {
            let loop_start_index = state.current_index;

            match token.token_type {
                JasmTokenType::Public => {
                    modifiers.push("public".to_string());
                    state.advance();
                }
                JasmTokenType::Private => {
                    modifiers.push("private".to_string());
                    state.advance();
                }
                JasmTokenType::Protected => {
                    modifiers.push("protected".to_string());
                    state.advance();
                }
                JasmTokenType::Static => {
                    modifiers.push("static".to_string());
                    state.advance();
                }
                JasmTokenType::Final => {
                    modifiers.push("final".to_string());
                    state.advance();
                }
                JasmTokenType::Abstract => {
                    modifiers.push("abstract".to_string());
                    state.advance();
                }
                JasmTokenType::Synchronized => {
                    modifiers.push("synchronized".to_string());
                    state.advance();
                }
                JasmTokenType::Native => {
                    modifiers.push("native".to_string());
                    state.advance();
                }
                JasmTokenType::Synthetic => {
                    modifiers.push("synthetic".to_string());
                    state.advance();
                }
                JasmTokenType::Deprecated => {
                    modifiers.push("deprecated".to_string());
                    state.advance();
                }
                JasmTokenType::Varargs => {
                    modifiers.push("varargs".to_string());
                    state.advance();
                }
                _ => break,
            }
            state.skip_ignored();

            // 强制检查：如果没有前进，强制 advance 并报错
            if state.current_index == loop_start_index {
                state.advance();
                return Err(GaiaError::syntax_error(
                    "Unexpected token in method modifiers",
                    gaia_types::SourceLocation::default(),
                ));
            }
        }

        // 期望 Method 关键字
        if !state.expect_token(JasmTokenType::Method) {
            return Err(GaiaError::syntax_error("Expected 'Method' keyword", gaia_types::SourceLocation::default()));
        }
        state.skip_ignored();

        // 解析方法名和描述符 (格式: "name":"descriptor" 或 name:"descriptor")
        if let Some(token) = state.current_token() {
            if token.token_type == JasmTokenType::StringLiteral
                || token.token_type == JasmTokenType::TypeDescriptor
                || token.token_type == JasmTokenType::Identifier
            {
                let method_name_raw = state.get_token_text(token)?.to_string();
                // 去除方法名中的双引号
                let method_name = method_name_raw.trim_matches('"').to_string();
                state.advance();
                state.skip_ignored();

                // 期望冒号
                if !state.expect_token(JasmTokenType::Colon) {
                    return Err(GaiaError::syntax_error(
                        "Expected ':' after method name",
                        gaia_types::SourceLocation::default(),
                    ));
                }
                state.skip_ignored();

                // 解析描述符
                if let Some(descriptor_token) = state.current_token() {
                    if descriptor_token.token_type == JasmTokenType::StringLiteral
                        || descriptor_token.token_type == JasmTokenType::TypeDescriptor
                    {
                        let descriptor_raw = state.get_token_text(descriptor_token)?.to_string();
                        // 去除描述符中的双引号
                        let descriptor = descriptor_raw.trim_matches('"').to_string();
                        name_and_descriptor = format!("{}:{}", method_name, descriptor);
                        state.advance();
                    }
                    else {
                        return Err(GaiaError::syntax_error(
                            "Expected method descriptor",
                            gaia_types::SourceLocation::default(),
                        ));
                    }
                }
                else {
                    return Err(GaiaError::syntax_error("Expected method descriptor", gaia_types::SourceLocation::default()));
                }
            }
            else {
                return Err(GaiaError::syntax_error(
                    "Expected method name and descriptor",
                    gaia_types::SourceLocation::default(),
                ));
            }
        }
        else {
            return Err(GaiaError::syntax_error("Expected method name and descriptor", gaia_types::SourceLocation::default()));
        }
        state.skip_ignored();

        // 解析 stack 和 locals（可选）
        while let Some(token) = state.current_token() {
            let loop_start_index = state.current_index;

            match token.token_type {
                JasmTokenType::Stack => {
                    state.advance();
                    state.skip_ignored();

                    if let Some(stack_token) = state.current_token() {
                        if stack_token.token_type == JasmTokenType::Number {
                            if let Ok(size) = state.get_token_text(stack_token).unwrap_or("0").parse::<u32>() {
                                stack_size = Some(size);
                            }
                            state.advance();
                        }
                    }
                }
                JasmTokenType::Locals => {
                    state.advance();
                    state.skip_ignored();

                    if let Some(locals_token) = state.current_token() {
                        if locals_token.token_type == JasmTokenType::Number {
                            if let Ok(count) = state.get_token_text(locals_token).unwrap_or("0").parse::<u32>() {
                                locals_count = Some(count);
                            }
                            state.advance();
                        }
                    }
                }
                JasmTokenType::LeftBrace => break,
                _ => {
                    state.advance();
                }
            }
            state.skip_ignored();

            // 强制检查：如果没有前进，强制 advance 并报错
            if state.current_index == loop_start_index {
                state.advance();
                return Err(GaiaError::syntax_error(
                    "Unexpected token in method declaration",
                    gaia_types::SourceLocation::default(),
                ));
            }
        }

        // 期望左大括号
        if !state.expect_token(JasmTokenType::LeftBrace) {
            return Err(GaiaError::syntax_error("Expected '{'", gaia_types::SourceLocation::default()));
        }
        state.skip_ignored();

        // 解析方法体指令
        while let Some(token) = state.current_token() {
            let loop_start_index = state.current_index;

            match token.token_type {
                JasmTokenType::RightBrace => {
                    state.advance();
                    break;
                }
                _ => {
                    let current_index = state.current_index;
                    if let Ok(instruction) = self.parse_instruction(state) {
                        instructions.push(instruction);
                    }
                    else {
                        // 如果解析失败，确保前进至少一个 token 以避免无限循环
                        if state.current_index == current_index {
                            state.advance();
                        }
                    }
                }
            }
            state.skip_ignored();

            // 强制检查：如果没有前进，强制 advance 并报错
            if state.current_index == loop_start_index {
                state.advance();
                return Err(GaiaError::syntax_error("Unexpected token in method body", gaia_types::SourceLocation::default()));
            }
        }

        Ok(JasmMethod { modifiers, name_and_descriptor, stack_size, locals_count, instructions })
    }

    /// 解析指令
    fn parse_instruction(&self, state: &mut ParserState) -> Result<JasmInstruction, GaiaError> {
        if let Some(token) = state.current_token() {
            let instruction_name = match token.token_type {
                // 简单指令
                JasmTokenType::ALoad0 => "aload_0",
                JasmTokenType::Return => "return",
                JasmTokenType::Nop => "nop",
                JasmTokenType::Dup => "dup",
                JasmTokenType::Pop => "pop",

                // 带参数的指令
                JasmTokenType::Ldc => "ldc",
                JasmTokenType::LdcW => "ldc_w",
                JasmTokenType::Ldc2W => "ldc2_w",

                // 方法调用指令
                JasmTokenType::InvokeSpecial => "invokespecial",
                JasmTokenType::InvokeVirtual => "invokevirtual",
                JasmTokenType::InvokeStatic => "invokestatic",
                JasmTokenType::InvokeInterface => "invokeinterface",
                JasmTokenType::InvokeDynamic => "invokedynamic",

                // 字段访问指令
                JasmTokenType::GetStatic => "getstatic",
                JasmTokenType::PutStatic => "putstatic",
                JasmTokenType::GetField => "getfield",
                JasmTokenType::PutField => "putfield",

                _ => return Err(GaiaError::syntax_error("Unknown instruction", gaia_types::SourceLocation::default())),
            };

            state.advance();
            state.skip_ignored();

            // 检查是否有参数
            if let Some(next_token) = state.current_token() {
                match next_token.token_type {
                    JasmTokenType::Method => {
                        // 方法调用指令
                        state.advance();
                        state.skip_ignored();

                        // 读取完整的方法引用: java/lang/Object."<init>":"()V"
                        let mut method_ref = String::new();

                        // 读取类名部分 (java/lang/Object)
                        if let Some(class_token) = state.current_token() {
                            method_ref.push_str(state.get_token_text(class_token)?);
                            state.advance();
                        }

                        // 读取点号
                        state.skip_ignored();
                        if let Some(dot_token) = state.current_token() {
                            if dot_token.token_type == JasmTokenType::Dot {
                                method_ref.push('.');
                                state.advance();
                            }
                        }

                        // 读取方法名 ("<init>")
                        state.skip_ignored();
                        if let Some(method_name_token) = state.current_token() {
                            method_ref.push_str(state.get_token_text(method_name_token)?);
                            state.advance();
                        }

                        // 读取冒号
                        state.skip_ignored();
                        if let Some(colon_token) = state.current_token() {
                            if colon_token.token_type == JasmTokenType::Colon {
                                method_ref.push(':');
                                state.advance();
                            }
                        }

                        // 读取方法描述符 ("()V")
                        state.skip_ignored();
                        if let Some(descriptor_token) = state.current_token() {
                            method_ref.push_str(state.get_token_text(descriptor_token)?);
                            state.advance();
                        }

                        // 跳过分号
                        state.skip_ignored();
                        if let Some(token) = state.current_token() {
                            if token.token_type == JasmTokenType::Semicolon {
                                state.advance();
                            }
                        }

                        return Ok(JasmInstruction::MethodCall { instruction: instruction_name.to_string(), method_ref });
                    }
                    JasmTokenType::Field => {
                        // 字段访问指令
                        state.advance();
                        state.skip_ignored();

                        // 读取完整的字段引用: java/lang/System.out:"Ljava/io/PrintStream;"
                        let mut field_ref = String::new();

                        // 读取类名部分 (java/lang/System)
                        if let Some(class_token) = state.current_token() {
                            field_ref.push_str(state.get_token_text(class_token)?);
                            state.advance();
                        }

                        // 读取点号
                        state.skip_ignored();
                        if let Some(dot_token) = state.current_token() {
                            if dot_token.token_type == JasmTokenType::Dot {
                                field_ref.push('.');
                                state.advance();
                            }
                        }

                        // 读取字段名 (out)
                        state.skip_ignored();
                        if let Some(field_name_token) = state.current_token() {
                            field_ref.push_str(state.get_token_text(field_name_token)?);
                            state.advance();
                        }

                        // 读取冒号
                        state.skip_ignored();
                        if let Some(colon_token) = state.current_token() {
                            if colon_token.token_type == JasmTokenType::Colon {
                                field_ref.push(':');
                                state.advance();
                            }
                        }

                        // 读取字段描述符 ("Ljava/io/PrintStream;")
                        state.skip_ignored();
                        if let Some(descriptor_token) = state.current_token() {
                            field_ref.push_str(state.get_token_text(descriptor_token)?);
                            state.advance();
                        }

                        // 跳过分号
                        state.skip_ignored();
                        if let Some(token) = state.current_token() {
                            if token.token_type == JasmTokenType::Semicolon {
                                state.advance();
                            }
                        }

                        return Ok(JasmInstruction::FieldAccess { instruction: instruction_name.to_string(), field_ref });
                    }
                    JasmTokenType::String => {
                        // 处理 ldc String "literal" 格式
                        let mut argument = state.get_token_text(next_token)?.to_string();
                        state.advance();

                        // 检查下一个 token 是否是字符串字面量
                        state.skip_ignored();
                        if let Some(string_literal_token) = state.current_token() {
                            if string_literal_token.token_type == JasmTokenType::StringLiteral {
                                argument.push(' ');
                                argument.push_str(state.get_token_text(string_literal_token)?);
                                state.advance();
                            }
                        }

                        // 跳过分号
                        state.skip_ignored();
                        if let Some(token) = state.current_token() {
                            if token.token_type == JasmTokenType::Semicolon {
                                state.advance();
                            }
                        }

                        return Ok(JasmInstruction::WithArgument { instruction: instruction_name.to_string(), argument });
                    }
                    JasmTokenType::StringLiteral | JasmTokenType::TypeDescriptor => {
                        // 带参数的指令
                        let argument = state.get_token_text(next_token)?.to_string();
                        state.advance();

                        // 跳过分号
                        state.skip_ignored();
                        if let Some(token) = state.current_token() {
                            if token.token_type == JasmTokenType::Semicolon {
                                state.advance();
                            }
                        }

                        return Ok(JasmInstruction::WithArgument { instruction: instruction_name.to_string(), argument });
                    }
                    JasmTokenType::Semicolon => {
                        // 简单指令，跳过分号
                        state.advance();
                        return Ok(JasmInstruction::Simple(instruction_name.to_string()));
                    }
                    _ => {
                        // 简单指令，无分号
                        return Ok(JasmInstruction::Simple(instruction_name.to_string()));
                    }
                }
            }

            Ok(JasmInstruction::Simple(instruction_name.to_string()))
        }
        else {
            Err(GaiaError::syntax_error("Expected instruction", gaia_types::SourceLocation::default()))
        }
    }
}

/// 解析器状态管理
struct ParserState<'a> {
    tokens: &'a TokenStream<'a, JasmTokenType>,
    current_index: usize,
}

impl<'a> ParserState<'a> {
    fn new(tokens: &'a TokenStream<'a, JasmTokenType>) -> Self {
        Self { tokens, current_index: 0 }
    }

    fn current_token(&self) -> Option<&gaia_types::reader::Token<JasmTokenType>> {
        let token_vec = self.tokens.tokens.get_ref();
        if self.current_index < token_vec.len() {
            Some(&token_vec[self.current_index])
        }
        else {
            None
        }
    }

    fn advance(&mut self) {
        self.current_index += 1;
    }

    fn skip_ignored(&mut self) {
        let token_vec = self.tokens.tokens.get_ref();
        while self.current_index < token_vec.len() {
            match token_vec[self.current_index].token_type {
                JasmTokenType::Whitespace | JasmTokenType::Comment => {
                    self.current_index += 1;
                }
                _ => break,
            }
        }
    }

    fn expect_token(&mut self, expected: JasmTokenType) -> bool {
        if let Some(token) = self.current_token() {
            if token.token_type == expected {
                self.advance();
                return true;
            }
        }
        false
    }

    fn get_token_text(&self, token: &gaia_types::reader::Token<JasmTokenType>) -> Result<&str, GaiaError> {
        self.tokens.get_text(token)
    }
}

impl Default for JasmParser {
    fn default() -> Self {
        Self::new()
    }
}
