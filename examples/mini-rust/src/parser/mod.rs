//! Mini Rust 语法分析器

use crate::{
    ast::*,
    lexer::{Lexer, Token},
};
use gaia_types::*;

/// 语法分析器
pub struct Parser {
    lexer: Lexer,
    current_token: Token,
}

impl Parser {
    pub fn new(mut lexer: Lexer) -> Result<Self, GaiaError> {
        let current_token = lexer.next_token()?;
        Ok(Self { lexer, current_token })
    }

    fn advance(&mut self) -> Result<(), GaiaError> {
        self.current_token = self.lexer.next_token()?;
        Ok(())
    }

    fn expect(&mut self, expected: Token) -> Result<(), GaiaError> {
        if std::mem::discriminant(&self.current_token) == std::mem::discriminant(&expected) {
            self.advance()
        }
        else {
            Err(GaiaError::syntax_error(
                format!("期望 {:?}, 但得到 {:?}", expected, self.current_token),
                SourceLocation::default(),
            ))
        }
    }

    fn skip_newlines(&mut self) -> Result<(), GaiaError> {
        while self.current_token == Token::Newline {
            self.advance()?;
        }
        Ok(())
    }

    pub fn parse_program(&mut self) -> Result<Program, GaiaError> {
        let mut functions = Vec::new();

        self.skip_newlines()?;

        while self.current_token != Token::Eof {
            functions.push(self.parse_function()?);
            self.skip_newlines()?;
        }

        Ok(Program { name: "main".to_string(), functions })
    }

    fn parse_function(&mut self) -> Result<Function, GaiaError> {
        self.expect(Token::Fn)?;

        let name = match &self.current_token {
            Token::Identifier(name) => name.clone(),
            _ => return Err(GaiaError::syntax_error("期望函数名", SourceLocation::default())),
        };
        self.advance()?;

        self.expect(Token::LeftParen)?;

        let mut parameters = Vec::new();
        while self.current_token != Token::RightParen {
            parameters.push(self.parse_parameter()?);

            if self.current_token == Token::Comma {
                self.advance()?;
            }
            else if self.current_token != Token::RightParen {
                return Err(GaiaError::syntax_error("期望 ',' 或 ')'", SourceLocation::default()));
            }
        }

        self.expect(Token::RightParen)?;

        // 可选的返回类型
        let return_type = if self.current_token == Token::Arrow {
            self.advance()?;
            Some(self.parse_type()?)
        }
        else {
            None
        };

        self.skip_newlines()?;
        let body = self.parse_block()?;

        Ok(Function { name, parameters, return_type, body })
    }

    fn parse_parameter(&mut self) -> Result<Parameter, GaiaError> {
        let name = match &self.current_token {
            Token::Identifier(name) => name.clone(),
            _ => return Err(GaiaError::syntax_error("期望参数名", SourceLocation::default())),
        };
        self.advance()?;

        self.expect(Token::Colon)?;
        let param_type = self.parse_type()?;

        Ok(Parameter { name, param_type })
    }

    fn parse_type(&mut self) -> Result<Type, GaiaError> {
        let type_name = match &self.current_token {
            Token::Identifier(name) => name.clone(),
            _ => return Err(GaiaError::syntax_error("期望类型名", SourceLocation::default())),
        };
        self.advance()?;

        match type_name.as_str() {
            "i32" => Ok(Type::I32),
            "i64" => Ok(Type::I64),
            "f32" => Ok(Type::F32),
            "f64" => Ok(Type::F64),
            "String" => Ok(Type::String),
            "bool" => Ok(Type::Bool),
            "()" => Ok(Type::Unit),
            _ => Err(GaiaError::syntax_error(format!("未知类型: {}", type_name), SourceLocation::default())),
        }
    }

    fn parse_block(&mut self) -> Result<Block, GaiaError> {
        self.expect(Token::LeftBrace)?;
        self.skip_newlines()?;

        let mut statements = Vec::new();

        while self.current_token != Token::RightBrace && self.current_token != Token::Eof {
            statements.push(self.parse_statement()?);
            self.skip_newlines()?;
        }

        self.expect(Token::RightBrace)?;

        Ok(Block { statements })
    }

    fn parse_statement(&mut self) -> Result<Statement, GaiaError> {
        match &self.current_token {
            Token::Let => self.parse_variable_declaration(),
            Token::Return => self.parse_return_statement(),
            _ => {
                let expr = self.parse_expression()?;
                if self.current_token == Token::Semicolon {
                    self.advance()?;
                }
                Ok(Statement::Expression(expr))
            }
        }
    }

    fn parse_variable_declaration(&mut self) -> Result<Statement, GaiaError> {
        self.expect(Token::Let)?;

        let name = match &self.current_token {
            Token::Identifier(name) => name.clone(),
            _ => return Err(GaiaError::syntax_error("期望变量名", SourceLocation::default())),
        };
        self.advance()?;

        // 可选的类型注解
        let var_type = if self.current_token == Token::Colon {
            self.advance()?;
            Some(self.parse_type()?)
        }
        else {
            None
        };

        self.expect(Token::Equal)?;
        let value = self.parse_expression()?;

        if self.current_token == Token::Semicolon {
            self.advance()?;
        }

        Ok(Statement::VariableDeclaration { name, var_type, value })
    }

    fn parse_return_statement(&mut self) -> Result<Statement, GaiaError> {
        self.expect(Token::Return)?;

        let value = if self.current_token == Token::Semicolon || self.current_token == Token::Newline {
            None
        }
        else {
            Some(self.parse_expression()?)
        };

        if self.current_token == Token::Semicolon {
            self.advance()?;
        }

        Ok(Statement::Return(value))
    }

    fn parse_expression(&mut self) -> Result<Expression, GaiaError> {
        self.parse_equality()
    }

    fn parse_equality(&mut self) -> Result<Expression, GaiaError> {
        let mut expr = self.parse_comparison()?;

        while matches!(self.current_token, Token::EqualEqual | Token::NotEqual) {
            let operator = match self.current_token {
                Token::EqualEqual => BinaryOperator::Equal,
                Token::NotEqual => BinaryOperator::NotEqual,
                _ => unreachable!(),
            };
            self.advance()?;
            let right = self.parse_comparison()?;
            expr = Expression::BinaryOperation { left: Box::new(expr), operator, right: Box::new(right) };
        }

        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<Expression, GaiaError> {
        let mut expr = self.parse_term()?;

        while matches!(self.current_token, Token::Greater | Token::GreaterEqual | Token::Less | Token::LessEqual) {
            let operator = match self.current_token {
                Token::Greater => BinaryOperator::Greater,
                Token::GreaterEqual => BinaryOperator::GreaterEqual,
                Token::Less => BinaryOperator::Less,
                Token::LessEqual => BinaryOperator::LessEqual,
                _ => unreachable!(),
            };
            self.advance()?;
            let right = self.parse_term()?;
            expr = Expression::BinaryOperation { left: Box::new(expr), operator, right: Box::new(right) };
        }

        Ok(expr)
    }

    fn parse_term(&mut self) -> Result<Expression, GaiaError> {
        let mut expr = self.parse_factor()?;

        while matches!(self.current_token, Token::Plus | Token::Minus) {
            let operator = match self.current_token {
                Token::Plus => BinaryOperator::Add,
                Token::Minus => BinaryOperator::Subtract,
                _ => unreachable!(),
            };
            self.advance()?;
            let right = self.parse_factor()?;
            expr = Expression::BinaryOperation { left: Box::new(expr), operator, right: Box::new(right) };
        }

        Ok(expr)
    }

    fn parse_factor(&mut self) -> Result<Expression, GaiaError> {
        let mut expr = self.parse_unary()?;

        while matches!(self.current_token, Token::Star | Token::Slash) {
            let operator = match self.current_token {
                Token::Star => BinaryOperator::Multiply,
                Token::Slash => BinaryOperator::Divide,
                _ => unreachable!(),
            };
            self.advance()?;
            let right = self.parse_unary()?;
            expr = Expression::BinaryOperation { left: Box::new(expr), operator, right: Box::new(right) };
        }

        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expression, GaiaError> {
        match &self.current_token {
            Token::Bang => {
                self.advance()?;
                let expr = self.parse_unary()?;
                Ok(Expression::UnaryOperation { operator: UnaryOperator::Not, operand: Box::new(expr) })
            }
            Token::Minus => {
                self.advance()?;
                let expr = self.parse_unary()?;
                Ok(Expression::UnaryOperation { operator: UnaryOperator::Negate, operand: Box::new(expr) })
            }
            _ => self.parse_call(),
        }
    }

    fn parse_call(&mut self) -> Result<Expression, GaiaError> {
        let mut expr = self.parse_primary()?;

        loop {
            match &self.current_token {
                Token::LeftParen => {
                    self.advance()?;
                    let mut arguments = Vec::new();

                    while self.current_token != Token::RightParen {
                        arguments.push(self.parse_expression()?);

                        if self.current_token == Token::Comma {
                            self.advance()?;
                        }
                        else if self.current_token != Token::RightParen {
                            return Err(GaiaError::syntax_error("期望 ',' 或 ')'", SourceLocation::default()));
                        }
                    }

                    self.expect(Token::RightParen)?;

                    expr = match expr {
                        Expression::Identifier(name) => Expression::FunctionCall { name, arguments },
                        _ => return Err(GaiaError::syntax_error("只能调用标识符", SourceLocation::default())),
                    };
                }
                Token::Dot => {
                    self.advance()?;
                    let method_name = match &self.current_token {
                        Token::Identifier(name) => name.clone(),
                        _ => return Err(GaiaError::syntax_error("期望方法名", SourceLocation::default())),
                    };
                    self.advance()?;

                    self.expect(Token::LeftParen)?;
                    let mut arguments = Vec::new();

                    while self.current_token != Token::RightParen {
                        arguments.push(self.parse_expression()?);

                        if self.current_token == Token::Comma {
                            self.advance()?;
                        }
                        else if self.current_token != Token::RightParen {
                            return Err(GaiaError::syntax_error("期望 ',' 或 ')'", SourceLocation::default()));
                        }
                    }

                    self.expect(Token::RightParen)?;

                    expr = Expression::MethodCall { object: Box::new(expr), method_name, arguments };
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expression, GaiaError> {
        match &self.current_token.clone() {
            Token::Integer(value) => {
                let value = *value;
                self.advance()?;
                Ok(Expression::Literal(Literal::Integer(value)))
            }
            Token::Float(value) => {
                let value = *value;
                self.advance()?;
                Ok(Expression::Literal(Literal::Float(value)))
            }
            Token::StringLiteral(value) => {
                let value = value.clone();
                self.advance()?;
                Ok(Expression::Literal(Literal::String(value)))
            }
            Token::True => {
                self.advance()?;
                Ok(Expression::Literal(Literal::Boolean(true)))
            }
            Token::False => {
                self.advance()?;
                Ok(Expression::Literal(Literal::Boolean(false)))
            }
            Token::Println => {
                self.advance()?; // 跳过 println!

                // 解析参数列表
                self.expect(Token::LeftParen)?;
                let mut arguments = Vec::new();

                while self.current_token != Token::RightParen {
                    arguments.push(self.parse_expression()?);

                    if self.current_token == Token::Comma {
                        self.advance()?;
                    }
                    else if self.current_token != Token::RightParen {
                        return Err(GaiaError::syntax_error("期望 ',' 或 ')'", SourceLocation::default()));
                    }
                }

                self.expect(Token::RightParen)?;

                Ok(Expression::MacroCall { name: "println".to_string(), arguments })
            }
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance()?;
                Ok(Expression::Identifier(name))
            }
            Token::LeftParen => {
                self.advance()?;
                let expr = self.parse_expression()?;
                self.expect(Token::RightParen)?;
                Ok(expr)
            }
            _ => Err(GaiaError::syntax_error(format!("意外的 token: {:?}", self.current_token), SourceLocation::default())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_function() {
        let input = r#"
            fn main() {
                console.log("Hello World");
            }
        "#;

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer).unwrap();
        let program = parser.parse_program().unwrap();

        assert_eq!(program.functions.len(), 1);
        assert_eq!(program.functions[0].name, "main");
        assert_eq!(program.functions[0].parameters.len(), 0);
    }

    #[test]
    fn test_parse_function_with_parameters() {
        let input = r#"
            fn add(a: i32, b: i32) -> i32 {
                return a + b;
            }
        "#;

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer).unwrap();
        let program = parser.parse_program().unwrap();

        assert_eq!(program.functions.len(), 1);
        let func = &program.functions[0];
        assert_eq!(func.name, "add");
        assert_eq!(func.parameters.len(), 2);
        assert_eq!(func.parameters[0].name, "a");
        assert_eq!(func.parameters[1].name, "b");
        assert!(func.return_type.is_some());
    }
}
