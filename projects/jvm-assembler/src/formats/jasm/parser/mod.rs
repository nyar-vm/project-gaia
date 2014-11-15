//! JASM 语法分析器
//!
//! 这个模块实现了 JASM 汇编语言的语法分析器，将 token 流转换为抽象语法树 (AST)。

use crate::formats::jasm::{ast::*, lexer::JasmTokenType};
use gaia_types::{
    helpers::Url,
    reader::{SourcePosition, Token, TokenStream},
    GaiaError, Result, SourceLocation,
};

/// JASM 语法分析器
#[derive(Debug)]
pub struct JasmParser {
    /// 源文件 URL（可选）
    url: Option<Url>,
}

impl JasmParser {
    /// 创建新的 JASM 解析器
    pub fn new(url: Option<Url>) -> Self {
        Self { url }
    }

    /// 解析 token 流为 JASM AST
    ///
    /// # 参数
    /// * `tokens` - 要解析的 token 流
    ///
    /// # 返回值
    /// 返回解析后的 `JasmRoot` AST 或错误
    pub fn parse(&self, tokens: TokenStream<JasmTokenType>) -> Result<JasmRoot> {
        let mut parser_state = ParserState::new(self.url.clone(), tokens.raw, tokens.tokens.into_inner());

        parser_state.parse_root()
    }
}

/// 解析器内部状态
struct ParserState<'input> {
    /// 源文件 URL
    url: Option<Url>,
    /// 源代码
    source: &'input str,
    /// Token 列表
    tokens: Vec<Token<JasmTokenType>>,
    /// 当前位置
    position: usize,
}

impl<'input> ParserState<'input> {
    /// 创建新的解析器状态
    fn new(url: Option<Url>, source: &'input str, tokens: Vec<Token<JasmTokenType>>) -> Self {
        Self { url, source, tokens, position: 0 }
    }

    /// 获取当前 token
    fn current(&self) -> Option<&Token<JasmTokenType>> {
        self.tokens.get(self.position)
    }

    /// 前进到下一个 token
    fn advance(&mut self) -> Option<&Token<JasmTokenType>> {
        if self.position < self.tokens.len() {
            self.position += 1;
        }
        self.current()
    }

    /// 检查当前 token 是否匹配指定类型
    fn check(&self, token_type: JasmTokenType) -> bool {
        self.current().map(|t| t.token_type) == Some(token_type)
    }

    /// 消费指定类型的 token
    fn consume(&mut self, token_type: JasmTokenType) -> Result<Token<JasmTokenType>> {
        if self.check(token_type) {
            let token = self.current().unwrap().clone();
            self.advance();
            Ok(token)
        }
        else {
            Err(GaiaError::custom_error(format!(
                "Expected {:?}, found {:?} at {:?}",
                token_type,
                self.current().map(|t| t.token_type),
                self.location_from_token(self.current().unwrap())
            )))
        }
    }

    /// 跳过空白和注释
    fn skip_whitespace_and_comments(&mut self) {
        while let Some(token) = self.current() {
            match token.token_type {
                JasmTokenType::Whitespace | JasmTokenType::Comment => {
                    self.advance();
                }
                _ => break,
            }
        }
    }

    /// 解析根节点
    fn parse_root(&mut self) -> Result<JasmRoot> {
        self.skip_whitespace_and_comments();

        let class = self.parse_class()?;

        Ok(JasmRoot { class })
    }

    /// 解析类定义
    fn parse_class(&mut self) -> Result<JasmClass> {
        // 解析访问修饰符
        let mut modifiers = Vec::new();
        while let Some(token) = self.current() {
            match token.token_type {
                JasmTokenType::Public => {
                    modifiers.push("public".to_string());
                    self.advance();
                }
                JasmTokenType::Private => {
                    modifiers.push("private".to_string());
                    self.advance();
                }
                JasmTokenType::Protected => {
                    modifiers.push("protected".to_string());
                    self.advance();
                }
                JasmTokenType::Static => {
                    modifiers.push("static".to_string());
                    self.advance();
                }
                JasmTokenType::Final => {
                    modifiers.push("final".to_string());
                    self.advance();
                }
                JasmTokenType::Super => {
                    modifiers.push("super".to_string());
                    self.advance();
                }
                JasmTokenType::Abstract => {
                    modifiers.push("abstract".to_string());
                    self.advance();
                }
                _ => break,
            }
            self.skip_whitespace_and_comments();
        }

        // 消费 "class" 关键字
        self.consume(JasmTokenType::Class)?;
        self.skip_whitespace_and_comments();

        // 解析类名
        let name_token = self.consume(JasmTokenType::Identifier)?;
        let name = self.token_text(&name_token).to_string();
        self.skip_whitespace_and_comments();

        // 解析版本信息（可选）
        let mut version = None;
        if self.check(JasmTokenType::Version) {
            self.advance(); // 消费 "version"
            self.skip_whitespace_and_comments();
            let version_token = self.consume(JasmTokenType::Number)?;
            let version_text = self.token_text(&version_token).to_string(); // 克隆字符串

            // 检查是否有冒号和次版本号
            if self.check(JasmTokenType::Colon) {
                self.advance(); // 消费 ":"
                self.skip_whitespace_and_comments();
                let minor_token = self.consume(JasmTokenType::Number)?;
                let minor_text = self.token_text(&minor_token).to_string(); // 克隆字符串
                version = Some(format!("{}:{}", version_text, minor_text));
            }
            else {
                version = Some(version_text);
            }
            self.skip_whitespace_and_comments();
        }

        // 消费左大括号
        self.consume(JasmTokenType::LeftBrace)?;
        self.skip_whitespace_and_comments();

        // 解析类体
        let mut methods = Vec::new();
        let mut fields = Vec::new();
        let mut source_file = None;

        while !self.check(JasmTokenType::RightBrace) && self.current().is_some() {
            self.skip_whitespace_and_comments();

            if let Some(token) = self.current() {
                match token.token_type {
                    JasmTokenType::Method
                    | JasmTokenType::Public
                    | JasmTokenType::Private
                    | JasmTokenType::Protected
                    | JasmTokenType::Static
                    | JasmTokenType::Final => {
                        methods.push(self.parse_method()?);
                    }
                    JasmTokenType::Field => {
                        fields.push(self.parse_field()?);
                    }
                    JasmTokenType::SourceFile => {
                        source_file = Some(self.parse_source_file()?);
                    }
                    _ => {
                        return Err(GaiaError::custom_error(format!(
                            "Unexpected token in class body: {:?} at {:?}",
                            token.token_type,
                            self.location_from_token(token)
                        )));
                    }
                }
            }
            self.skip_whitespace_and_comments();
        }

        // 消费右大括号
        self.consume(JasmTokenType::RightBrace)?;

        Ok(JasmClass { modifiers, name, version, methods, fields, source_file })
    }

    /// 解析方法定义
    fn parse_method(&mut self) -> Result<JasmMethod> {
        // 解析访问修饰符
        let mut modifiers = Vec::new();
        while let Some(token) = self.current() {
            match token.token_type {
                JasmTokenType::Public => {
                    modifiers.push("public".to_string());
                    self.advance();
                }
                JasmTokenType::Private => {
                    modifiers.push("private".to_string());
                    self.advance();
                }
                JasmTokenType::Protected => {
                    modifiers.push("protected".to_string());
                    self.advance();
                }
                JasmTokenType::Static => {
                    modifiers.push("static".to_string());
                    self.advance();
                }
                JasmTokenType::Final => {
                    modifiers.push("final".to_string());
                    self.advance();
                }
                JasmTokenType::Synchronized => {
                    modifiers.push("synchronized".to_string());
                    self.advance();
                }
                JasmTokenType::Native => {
                    modifiers.push("native".to_string());
                    self.advance();
                }
                _ => break,
            }
            self.skip_whitespace_and_comments();
        }

        // 消费 "Method" 关键字
        self.consume(JasmTokenType::Method)?;
        self.skip_whitespace_and_comments();

        // 解析方法名和描述符
        let name_and_descriptor = self.parse_method_signature()?;
        self.skip_whitespace_and_comments();

        // 解析栈大小和局部变量数（可选）
        let mut stack_size = None;
        let mut locals_count = None;

        if self.check(JasmTokenType::Stack) {
            self.advance(); // 消费 "stack"
            self.skip_whitespace_and_comments();
            let stack_token = self.consume(JasmTokenType::Number)?;
            stack_size = Some(self.token_text(&stack_token).parse().unwrap_or(0));
            self.skip_whitespace_and_comments();
        }

        if self.check(JasmTokenType::Locals) {
            self.advance(); // 消费 "locals"
            self.skip_whitespace_and_comments();
            let locals_token = self.consume(JasmTokenType::Number)?;
            locals_count = Some(self.token_text(&locals_token).parse().unwrap_or(0));
            self.skip_whitespace_and_comments();
        }

        // 消费左大括号
        self.consume(JasmTokenType::LeftBrace)?;
        self.skip_whitespace_and_comments();

        // 解析方法体指令
        let mut instructions = Vec::new();
        while !self.check(JasmTokenType::RightBrace) && self.current().is_some() {
            self.skip_whitespace_and_comments();
            if let Some(instruction) = self.parse_instruction()? {
                instructions.push(instruction);
            }
            self.skip_whitespace_and_comments();
        }

        // 消费右大括号
        self.consume(JasmTokenType::RightBrace)?;

        Ok(JasmMethod { modifiers, name_and_descriptor, stack_size, locals_count, instructions })
    }

    /// 解析方法签名（名称和描述符）
    fn parse_method_signature(&mut self) -> Result<String> {
        let mut signature = String::new();

        // 解析方法名（可能包含特殊字符如 <init>）
        if self.check(JasmTokenType::StringLiteral) || self.check(JasmTokenType::Identifier) {
            let token = self.current().unwrap();
            let text = self.token_text(token);
            if text.starts_with('"') && text.ends_with('"') {
                signature.push_str(&text[1..text.len() - 1]); // 去掉引号
            }
            else {
                signature.push_str(text);
            }
            self.advance();
        }

        self.skip_whitespace_and_comments();

        // 消费冒号
        if self.check(JasmTokenType::Colon) {
            signature.push(':');
            self.advance();
            self.skip_whitespace_and_comments();
        }

        // 解析类型描述符
        if self.check(JasmTokenType::TypeDescriptor) || self.check(JasmTokenType::StringLiteral) {
            let token = self.current().unwrap();
            let text = self.token_text(token);
            if text.starts_with('"') && text.ends_with('"') {
                signature.push_str(&text[1..text.len() - 1]); // 去掉引号
            }
            else {
                signature.push_str(text);
            }
            self.advance();
        }

        Ok(signature)
    }

    /// 解析指令
    fn parse_instruction(&mut self) -> Result<Option<JasmInstruction>> {
        if let Some(token) = self.current() {
            match token.token_type {
                // 简单指令
                JasmTokenType::ALoad0
                | JasmTokenType::ALoad1
                | JasmTokenType::ALoad2
                | JasmTokenType::ALoad3
                | JasmTokenType::ILoad0
                | JasmTokenType::ILoad1
                | JasmTokenType::ILoad2
                | JasmTokenType::ILoad3
                | JasmTokenType::Return
                | JasmTokenType::IReturn
                | JasmTokenType::AReturn
                | JasmTokenType::Nop
                | JasmTokenType::Dup
                | JasmTokenType::Pop => {
                    let instruction_name = self.token_text(token).to_string();
                    self.advance();
                    self.skip_whitespace_and_comments();

                    // 消费分号
                    if self.check(JasmTokenType::Semicolon) {
                        self.advance();
                    }

                    Ok(Some(JasmInstruction::Simple(instruction_name)))
                }

                // 带参数的指令
                JasmTokenType::Ldc | JasmTokenType::LdcW | JasmTokenType::Ldc2W => {
                    let instruction_name = self.token_text(token).to_string();
                    self.advance();
                    self.skip_whitespace_and_comments();

                    // 解析参数
                    let argument = if self.check(JasmTokenType::String) {
                        self.advance(); // 消费 "String"
                        self.skip_whitespace_and_comments();
                        if self.check(JasmTokenType::StringLiteral) {
                            let arg_token = self.current().unwrap();
                            let arg_text = self.token_text(arg_token).to_string(); // 克隆字符串
                            self.advance();
                            format!("String {}", arg_text)
                        }
                        else {
                            return Err(GaiaError::custom_error(format!(
                                "Expected string literal after String at {:?}",
                                self.location_from_token(self.current().unwrap_or(&Token {
                                    token_type: JasmTokenType::Eof,
                                    position: SourcePosition { line: 0, column: 0, offset: 0, length: 0 }
                                }))
                            )));
                        }
                    }
                    else if self.check(JasmTokenType::StringLiteral) || self.check(JasmTokenType::Number) {
                        let arg_token = self.current().unwrap();
                        let arg_text = self.token_text(arg_token).to_string();
                        self.advance();
                        arg_text
                    }
                    else {
                        return Err(GaiaError::custom_error(format!(
                            "Expected argument for ldc instruction at {:?}",
                            self.location_from_token(self.current().unwrap_or(&Token {
                                token_type: JasmTokenType::Eof,
                                position: SourcePosition { line: 0, column: 0, offset: 0, length: 0 }
                            }))
                        )));
                    };

                    self.skip_whitespace_and_comments();

                    // 消费分号
                    if self.check(JasmTokenType::Semicolon) {
                        self.advance();
                    }

                    Ok(Some(JasmInstruction::WithArgument { instruction: instruction_name, argument }))
                }

                // 方法调用指令
                JasmTokenType::InvokeSpecial
                | JasmTokenType::InvokeVirtual
                | JasmTokenType::InvokeStatic
                | JasmTokenType::InvokeInterface => {
                    let instruction_name = self.token_text(token).to_string();
                    self.advance();
                    self.skip_whitespace_and_comments();

                    // 消费 "Method" 关键字
                    self.consume(JasmTokenType::Method)?;
                    self.skip_whitespace_and_comments();

                    // 解析方法引用
                    let method_ref = self.parse_method_reference()?;
                    self.skip_whitespace_and_comments();

                    // 消费分号
                    if self.check(JasmTokenType::Semicolon) {
                        self.advance();
                    }

                    Ok(Some(JasmInstruction::MethodCall { instruction: instruction_name, method_ref }))
                }

                // 字段访问指令
                JasmTokenType::GetStatic | JasmTokenType::PutStatic | JasmTokenType::GetField | JasmTokenType::PutField => {
                    let instruction_name = self.token_text(token).to_string();
                    self.advance();
                    self.skip_whitespace_and_comments();

                    // 消费 "Field" 关键字
                    self.consume(JasmTokenType::Field)?;
                    self.skip_whitespace_and_comments();

                    // 解析字段引用
                    let field_ref = self.parse_field_reference()?;
                    self.skip_whitespace_and_comments();

                    // 消费分号
                    if self.check(JasmTokenType::Semicolon) {
                        self.advance();
                    }

                    Ok(Some(JasmInstruction::FieldAccess { instruction: instruction_name, field_ref }))
                }

                _ => {
                    // 跳过未知的 token
                    self.advance();
                    Ok(None)
                }
            }
        }
        else {
            Ok(None)
        }
    }

    /// 解析方法引用
    fn parse_method_reference(&mut self) -> Result<String> {
        let mut method_ref = String::new();

        // 解析类名
        if self.check(JasmTokenType::Identifier) {
            let token = self.current().unwrap();
            method_ref.push_str(self.token_text(token));
            self.advance();
        }

        // 消费点号
        if self.check(JasmTokenType::Dot) {
            method_ref.push('.');
            self.advance();
        }

        // 解析方法名和描述符
        if self.check(JasmTokenType::StringLiteral) || self.check(JasmTokenType::Identifier) {
            let token = self.current().unwrap();
            let text = self.token_text(token);
            if text.starts_with('"') && text.ends_with('"') {
                method_ref.push_str(&text[1..text.len() - 1]); // 去掉引号
            }
            else {
                method_ref.push_str(text);
            }
            self.advance();
        }

        // 消费冒号
        if self.check(JasmTokenType::Colon) {
            method_ref.push(':');
            self.advance();
        }

        // 解析类型描述符
        if self.check(JasmTokenType::TypeDescriptor) || self.check(JasmTokenType::StringLiteral) {
            let token = self.current().unwrap();
            let text = self.token_text(token);
            if text.starts_with('"') && text.ends_with('"') {
                method_ref.push_str(&text[1..text.len() - 1]); // 去掉引号
            }
            else {
                method_ref.push_str(text);
            }
            self.advance();
        }

        Ok(method_ref)
    }

    /// 解析字段引用
    fn parse_field_reference(&mut self) -> Result<String> {
        let mut field_ref = String::new();

        // 解析类名
        if self.check(JasmTokenType::Identifier) {
            let token = self.current().unwrap();
            field_ref.push_str(self.token_text(token));
            self.advance();
        }

        // 消费点号
        if self.check(JasmTokenType::Dot) {
            field_ref.push('.');
            self.advance();
        }

        // 解析字段名
        if self.check(JasmTokenType::Identifier) {
            let token = self.current().unwrap();
            field_ref.push_str(self.token_text(token));
            self.advance();
        }

        // 消费冒号
        if self.check(JasmTokenType::Colon) {
            field_ref.push(':');
            self.advance();
        }

        // 解析类型描述符
        if self.check(JasmTokenType::TypeDescriptor) || self.check(JasmTokenType::StringLiteral) {
            let token = self.current().unwrap();
            let text = self.token_text(token);
            if text.starts_with('"') && text.ends_with('"') {
                field_ref.push_str(&text[1..text.len() - 1]); // 去掉引号
            }
            else {
                field_ref.push_str(text);
            }
            self.advance();
        }

        Ok(field_ref)
    }

    /// 解析字段定义
    fn parse_field(&mut self) -> Result<JasmField> {
        // 解析访问修饰符
        let mut modifiers = Vec::new();
        while let Some(token) = self.current() {
            match token.token_type {
                JasmTokenType::Public => {
                    modifiers.push("public".to_string());
                    self.advance();
                }
                JasmTokenType::Private => {
                    modifiers.push("private".to_string());
                    self.advance();
                }
                JasmTokenType::Protected => {
                    modifiers.push("protected".to_string());
                    self.advance();
                }
                JasmTokenType::Static => {
                    modifiers.push("static".to_string());
                    self.advance();
                }
                JasmTokenType::Final => {
                    modifiers.push("final".to_string());
                    self.advance();
                }
                _ => break,
            }
            self.skip_whitespace_and_comments();
        }

        // 消费 "Field" 关键字
        self.consume(JasmTokenType::Field)?;
        self.skip_whitespace_and_comments();

        // 解析字段名和描述符
        let name_and_descriptor = self.parse_field_signature()?;
        self.skip_whitespace_and_comments();

        // 消费分号
        if self.check(JasmTokenType::Semicolon) {
            self.advance();
        }

        Ok(JasmField { modifiers, name_and_descriptor })
    }

    /// 解析字段签名
    fn parse_field_signature(&mut self) -> Result<String> {
        let mut signature = String::new();

        // 解析字段名
        if self.check(JasmTokenType::Identifier) {
            let token = self.current().unwrap();
            signature.push_str(self.token_text(token));
            self.advance();
        }

        self.skip_whitespace_and_comments();

        // 消费冒号
        if self.check(JasmTokenType::Colon) {
            signature.push(':');
            self.advance();
            self.skip_whitespace_and_comments();
        }

        // 解析类型描述符
        if self.check(JasmTokenType::TypeDescriptor) || self.check(JasmTokenType::StringLiteral) {
            let token = self.current().unwrap();
            let text = self.token_text(token);
            if text.starts_with('"') && text.ends_with('"') {
                signature.push_str(&text[1..text.len() - 1]); // 去掉引号
            }
            else {
                signature.push_str(text);
            }
            self.advance();
        }

        Ok(signature)
    }

    /// 解析源文件信息
    fn parse_source_file(&mut self) -> Result<String> {
        // 消费 "SourceFile" 关键字
        self.consume(JasmTokenType::SourceFile)?;
        self.skip_whitespace_and_comments();

        // 解析文件名
        let filename_token = self.consume(JasmTokenType::StringLiteral)?;
        let filename = self.token_text(&filename_token).to_string(); // 克隆字符串

        // 去掉引号
        let filename =
            if filename.starts_with('"') && filename.ends_with('"') { &filename[1..filename.len() - 1] } else { &filename };

        self.skip_whitespace_and_comments();

        // 消费分号
        if self.check(JasmTokenType::Semicolon) {
            self.advance();
        }

        Ok(filename.to_string())
    }

    /// 获取 token 的文本内容
    fn token_text(&self, token: &Token<JasmTokenType>) -> &str {
        let range = token.get_range();
        &self.source[range]
    }

    /// 从 token 创建源位置
    fn location_from_token(&self, token: &Token<JasmTokenType>) -> SourceLocation {
        SourceLocation { line: token.position.line, column: token.position.column, url: self.url.clone().map(|u| u.clone()) }
    }
}

impl Default for JasmParser {
    fn default() -> Self {
        Self::new(None)
    }
}
