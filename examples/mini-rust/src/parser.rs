//! Mini Rust 语法分析器

use crate::{ast::*, lexer::RustTokenType};
use gaia_types::{reader::TokenStream, GaiaError, SourceLocation};

/// 语法分析器
pub struct Parser<'input> {
    tokens: TokenStream<'input, RustTokenType>,
}

impl<'input> Parser<'input> {
    pub fn new(tokens: TokenStream<'input, RustTokenType>) -> Self {
        Self { tokens }
    }

    fn advance(&mut self) {
        self.tokens.tokens.set_position(self.tokens.tokens.position() + 1);
    }

    fn expect(&mut self, expected: RustTokenType) -> Result<(), GaiaError> {
        let current = self.tokens.current()?;
        if current == expected {
            self.advance();
            Ok(())
        }
        else {
            Err(GaiaError::syntax_error(&format!("期望 {:?}, 但得到 {:?}", expected, current), SourceLocation::default()))
        }
    }

    fn skip_newlines(&mut self) -> Result<(), GaiaError> {
        while let Ok(current) = self.tokens.current() {
            if current == RustTokenType::Newline {
                self.advance();
            }
            else {
                break;
            }
        }
        Ok(())
    }

    fn skip_whitespace(&mut self) -> Result<(), GaiaError> {
        while let Ok(current) = self.tokens.current() {
            if current == RustTokenType::Whitespace {
                self.advance();
            }
            else {
                break;
            }
        }
        Ok(())
    }

    fn skip_whitespace_and_newlines(&mut self) -> Result<(), GaiaError> {
        while let Ok(current) = self.tokens.current() {
            if current == RustTokenType::Whitespace || current == RustTokenType::Newline {
                self.advance();
            }
            else {
                break;
            }
        }
        Ok(())
    }

    fn get_identifier_text(&self) -> Result<String, GaiaError> {
        let token = self.tokens.current_token()?;
        let text = self.tokens.get_text(token)?;
        Ok(text.to_string())
    }

    fn get_string_literal_text(&self) -> Result<String, GaiaError> {
        let token = self.tokens.current_token()?;
        let text = self.tokens.get_text(token)?;
        // 移除引号
        if text.len() >= 2 && text.starts_with('"') && text.ends_with('"') {
            Ok(text[1..text.len() - 1].to_string())
        }
        else {
            Ok(text.to_string())
        }
    }

    fn get_integer_text(&self) -> Result<i64, GaiaError> {
        let token = self.tokens.current_token()?;
        let text = self.tokens.get_text(token)?;
        text.parse().map_err(|_| GaiaError::syntax_error(&format!("无效的整数: {}", text), SourceLocation::default()))
    }

    fn get_float_text(&self) -> Result<f64, GaiaError> {
        let token = self.tokens.current_token()?;
        let text = self.tokens.get_text(token)?;
        text.parse().map_err(|_| GaiaError::syntax_error(&format!("无效的浮点数: {}", text), SourceLocation::default()))
    }

    pub fn parse_program(&mut self) -> Result<Program, GaiaError> {
        let mut functions = Vec::new();

        self.skip_whitespace_and_newlines()?;

        while self.tokens.current().is_ok() && self.tokens.current()? != RustTokenType::Eof {
            functions.push(self.parse_function()?);
            self.skip_whitespace_and_newlines()?;
        }

        Ok(Program { name: "main".to_string(), functions })
    }

    fn parse_function(&mut self) -> Result<Function, GaiaError> {
        self.expect(RustTokenType::Fn)?;
        self.skip_whitespace()?;

        let current = self.tokens.current()?;
        if current != RustTokenType::Identifier {
            return Err(GaiaError::syntax_error("期望函数名", SourceLocation::default()));
        }
        let name = self.get_identifier_text()?;
        self.advance();
        self.skip_whitespace()?;

        self.expect(RustTokenType::LeftParen)?;
        self.skip_whitespace()?;

        let mut parameters = Vec::new();
        while self.tokens.current()? != RustTokenType::RightParen {
            parameters.push(self.parse_parameter()?);

            if self.tokens.current()? == RustTokenType::Comma {
                self.advance();
                self.skip_whitespace()?;
            }
            else if self.tokens.current()? != RustTokenType::RightParen {
                return Err(GaiaError::syntax_error("期望 ',' 或 ')'", SourceLocation::default()));
            }
        }

        self.expect(RustTokenType::RightParen)?;
        self.skip_whitespace()?;

        // 可选的返回类型
        let return_type = if self.tokens.current()? == RustTokenType::Arrow {
            self.advance();
            self.skip_whitespace()?;
            Some(self.parse_type()?)
        }
        else {
            None
        };

        self.skip_whitespace_and_newlines()?;
        let body = self.parse_block()?;

        Ok(Function { name, parameters, return_type, body })
    }

    fn parse_parameter(&mut self) -> Result<Parameter, GaiaError> {
        let current = self.tokens.current()?;
        if current != RustTokenType::Identifier {
            return Err(GaiaError::syntax_error("期望参数名", SourceLocation::default()));
        }
        let name = self.get_identifier_text()?;
        self.advance();

        self.expect(RustTokenType::Colon)?;
        let param_type = self.parse_type()?;

        Ok(Parameter { name, param_type })
    }

    fn parse_type(&mut self) -> Result<Type, GaiaError> {
        let current = self.tokens.current()?;
        if current != RustTokenType::Identifier {
            return Err(GaiaError::syntax_error("期望类型名", SourceLocation::default()));
        }
        let type_name = self.get_identifier_text()?;
        self.advance();

        match type_name.as_str() {
            "i32" => Ok(Type::I32),
            "i64" => Ok(Type::I64),
            "f32" => Ok(Type::F32),
            "f64" => Ok(Type::F64),
            "String" => Ok(Type::String),
            "bool" => Ok(Type::Bool),
            "()" => Ok(Type::Unit),
            _ => Err(GaiaError::syntax_error(&format!("未知类型: {}", type_name), SourceLocation::default())),
        }
    }

    fn parse_block(&mut self) -> Result<Block, GaiaError> {
        self.expect(RustTokenType::LeftBrace)?;
        self.skip_whitespace_and_newlines()?;

        let mut statements = Vec::new();

        while self.tokens.current()? != RustTokenType::RightBrace && self.tokens.current()? != RustTokenType::Eof {
            statements.push(self.parse_statement()?);
            self.skip_whitespace_and_newlines()?;
        }

        self.expect(RustTokenType::RightBrace)?;

        Ok(Block { statements })
    }

    fn parse_statement(&mut self) -> Result<Statement, GaiaError> {
        self.skip_whitespace()?;
        match self.tokens.current()? {
            RustTokenType::Let => self.parse_variable_declaration(),
            RustTokenType::Return => self.parse_return_statement(),
            _ => {
                let expr = self.parse_expression()?;
                if self.tokens.current().is_ok() && self.tokens.current()? == RustTokenType::Semicolon {
                    self.advance();
                }
                Ok(Statement::Expression(expr))
            }
        }
    }

    fn parse_variable_declaration(&mut self) -> Result<Statement, GaiaError> {
        self.expect(RustTokenType::Let)?;

        let current = self.tokens.current()?;
        if current != RustTokenType::Identifier {
            return Err(GaiaError::syntax_error("期望变量名", SourceLocation::default()));
        }
        let name = self.get_identifier_text()?;
        self.advance();

        // 可选的类型注解
        let var_type = if self.tokens.current()? == RustTokenType::Colon {
            self.advance();
            Some(self.parse_type()?)
        }
        else {
            None
        };

        self.expect(RustTokenType::Equal)?;
        let value = self.parse_expression()?;

        if self.tokens.current().is_ok() && self.tokens.current()? == RustTokenType::Semicolon {
            self.advance();
        }

        Ok(Statement::VariableDeclaration { name, var_type, initializer: Some(value) })
    }

    fn parse_return_statement(&mut self) -> Result<Statement, GaiaError> {
        self.expect(RustTokenType::Return)?;

        let value = if self.tokens.current().is_ok()
            && (self.tokens.current()? == RustTokenType::Semicolon || self.tokens.current()? == RustTokenType::Newline)
        {
            None
        }
        else {
            Some(self.parse_expression()?)
        };

        if self.tokens.current().is_ok() && self.tokens.current()? == RustTokenType::Semicolon {
            self.advance();
        }

        Ok(Statement::Return(value))
    }

    fn parse_expression(&mut self) -> Result<Expression, GaiaError> {
        self.parse_equality()
    }

    fn parse_equality(&mut self) -> Result<Expression, GaiaError> {
        let mut expr = self.parse_comparison()?;

        while self.tokens.current().is_ok() {
            match self.tokens.current()? {
                RustTokenType::EqualEqual => {
                    self.advance();
                    let right = self.parse_comparison()?;
                    expr = Expression::BinaryOperation {
                        left: Box::new(expr),
                        operator: BinaryOperator::Equal,
                        right: Box::new(right),
                    };
                }
                RustTokenType::NotEqual => {
                    self.advance();
                    let right = self.parse_comparison()?;
                    expr = Expression::BinaryOperation {
                        left: Box::new(expr),
                        operator: BinaryOperator::NotEqual,
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<Expression, GaiaError> {
        let mut expr = self.parse_term()?;

        while self.tokens.current().is_ok() {
            match self.tokens.current()? {
                RustTokenType::Greater => {
                    self.advance();
                    let right = self.parse_term()?;
                    expr = Expression::BinaryOperation {
                        left: Box::new(expr),
                        operator: BinaryOperator::Greater,
                        right: Box::new(right),
                    };
                }
                RustTokenType::GreaterEqual => {
                    self.advance();
                    let right = self.parse_term()?;
                    expr = Expression::BinaryOperation {
                        left: Box::new(expr),
                        operator: BinaryOperator::GreaterEqual,
                        right: Box::new(right),
                    };
                }
                RustTokenType::Less => {
                    self.advance();
                    let right = self.parse_term()?;
                    expr = Expression::BinaryOperation {
                        left: Box::new(expr),
                        operator: BinaryOperator::Less,
                        right: Box::new(right),
                    };
                }
                RustTokenType::LessEqual => {
                    self.advance();
                    let right = self.parse_term()?;
                    expr = Expression::BinaryOperation {
                        left: Box::new(expr),
                        operator: BinaryOperator::LessEqual,
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn parse_term(&mut self) -> Result<Expression, GaiaError> {
        let mut expr = self.parse_factor()?;

        while self.tokens.current().is_ok() {
            match self.tokens.current()? {
                RustTokenType::Plus => {
                    self.advance();
                    let right = self.parse_factor()?;
                    expr = Expression::BinaryOperation {
                        left: Box::new(expr),
                        operator: BinaryOperator::Add,
                        right: Box::new(right),
                    };
                }
                RustTokenType::Minus => {
                    self.advance();
                    let right = self.parse_factor()?;
                    expr = Expression::BinaryOperation {
                        left: Box::new(expr),
                        operator: BinaryOperator::Subtract,
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn parse_factor(&mut self) -> Result<Expression, GaiaError> {
        let mut expr = self.parse_unary()?;

        while self.tokens.current().is_ok() {
            match self.tokens.current()? {
                RustTokenType::Star => {
                    self.advance();
                    let right = self.parse_unary()?;
                    expr = Expression::BinaryOperation {
                        left: Box::new(expr),
                        operator: BinaryOperator::Multiply,
                        right: Box::new(right),
                    };
                }
                RustTokenType::Slash => {
                    self.advance();
                    let right = self.parse_unary()?;
                    expr = Expression::BinaryOperation {
                        left: Box::new(expr),
                        operator: BinaryOperator::Divide,
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expression, GaiaError> {
        match self.tokens.current()? {
            RustTokenType::Bang => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expression::UnaryOperation { operator: UnaryOperator::Not, operand: Box::new(expr) })
            }
            RustTokenType::Minus => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expression::UnaryOperation { operator: UnaryOperator::Negate, operand: Box::new(expr) })
            }
            _ => self.parse_call(),
        }
    }

    fn parse_call(&mut self) -> Result<Expression, GaiaError> {
        let mut expr = self.parse_primary()?;

        loop {
            match self.tokens.current() {
                Ok(RustTokenType::LeftParen) => {
                    self.advance();
                    let mut arguments = Vec::new();

                    while self.tokens.current()? != RustTokenType::RightParen {
                        arguments.push(self.parse_expression()?);

                        if self.tokens.current()? == RustTokenType::Comma {
                            self.advance();
                        }
                        else if self.tokens.current()? != RustTokenType::RightParen {
                            return Err(GaiaError::syntax_error("期望 ',' 或 ')'", SourceLocation::default()));
                        }
                    }

                    self.expect(RustTokenType::RightParen)?;

                    expr = Expression::FunctionCall {
                        name: match expr {
                            Expression::Identifier(name) => name,
                            _ => return Err(GaiaError::syntax_error("只能调用标识符", SourceLocation::default())),
                        },
                        arguments,
                    };
                }
                Ok(RustTokenType::Bang) => {
                    // 处理宏调用，如 println!
                    self.advance();
                    if self.tokens.current()? == RustTokenType::LeftParen {
                        self.advance();
                        let mut arguments = Vec::new();

                        while self.tokens.current()? != RustTokenType::RightParen {
                            arguments.push(self.parse_expression()?);

                            if self.tokens.current()? == RustTokenType::Comma {
                                self.advance();
                            }
                            else if self.tokens.current()? != RustTokenType::RightParen {
                                return Err(GaiaError::syntax_error("期望 ',' 或 ')'", SourceLocation::default()));
                            }
                        }

                        self.expect(RustTokenType::RightParen)?;

                        expr = Expression::MacroCall {
                            name: match expr {
                                Expression::Identifier(name) => name,
                                _ => return Err(GaiaError::syntax_error("只能调用标识符宏", SourceLocation::default())),
                            },
                            arguments,
                        };
                    }
                    else {
                        return Err(GaiaError::syntax_error("期望 '(' 在宏调用中", SourceLocation::default()));
                    }
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expression, GaiaError> {
        self.skip_whitespace()?;
        match self.tokens.current()? {
            RustTokenType::True => {
                self.advance();
                Ok(Expression::Literal(Literal::Boolean(true)))
            }
            RustTokenType::False => {
                self.advance();
                Ok(Expression::Literal(Literal::Boolean(false)))
            }
            RustTokenType::Integer => {
                let value = self.get_integer_text()?;
                self.advance();
                Ok(Expression::Literal(Literal::Integer(value)))
            }
            RustTokenType::Float => {
                let value = self.get_float_text()?;
                self.advance();
                Ok(Expression::Literal(Literal::Float(value)))
            }
            RustTokenType::StringLiteral => {
                let value = self.get_string_literal_text()?;
                self.advance();
                Ok(Expression::Literal(Literal::String(value)))
            }
            RustTokenType::Identifier => {
                let name = self.get_identifier_text()?;
                self.advance();
                Ok(Expression::Identifier(name))
            }
            RustTokenType::LeftParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(RustTokenType::RightParen)?;
                Ok(expr)
            }
            _ => {
                Err(GaiaError::syntax_error(&format!("意外的 token: {:?}", self.tokens.current()?), SourceLocation::default()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::RustLexer;

    #[test]
    fn test_parse_simple_function() {
        let input = "fn main() { }";
        let mut lexer = RustLexer::new(input);
        let diagnostics = lexer.tokenize();
        assert!(diagnostics.result.is_ok());

        let token_stream = diagnostics.result.unwrap();
        let mut parser = Parser::new(token_stream);
        let program = parser.parse_program().unwrap();

        assert_eq!(program.functions.len(), 1);
        assert_eq!(program.functions[0].name, "main");
    }

    #[test]
    fn test_parse_function_with_parameters() {
        let input = "fn add(a: i32, b: i32) -> i32 { return a + b; }";
        let mut lexer = RustLexer::new(input);
        let diagnostics = lexer.tokenize();
        assert!(diagnostics.result.is_ok());

        let token_stream = diagnostics.result.unwrap();
        let mut parser = Parser::new(token_stream);
        let program = parser.parse_program().unwrap();

        assert_eq!(program.functions.len(), 1);
        let func = &program.functions[0];
        assert_eq!(func.name, "add");
        assert_eq!(func.parameters.len(), 2);
        assert_eq!(func.return_type, Some(Type::I32));
    }
}
