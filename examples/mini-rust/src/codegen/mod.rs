//! Mini Rust 代码生成器

use crate::ast::*;
use gaia_assembler::{instruction::GaiaInstruction, program::{GaiaConstant, GaiaFunction, GaiaProgram}, types::*};
use gaia_types::*;
use std::collections::HashMap;

/// 代码生成器
pub struct CodeGenerator {
    /// 局部变量映射
    locals: HashMap<String, usize>,
    /// 当前局部变量索引
    local_index: usize,
}

impl CodeGenerator {
    pub fn new() -> Self {
        Self { locals: HashMap::new(), local_index: 0 }
    }

    /// 生成 GaiaProgram
    pub fn generate(&mut self, program: &Program) -> Result<GaiaProgram> {
        let mut functions = Vec::new();

        for function in &program.functions {
            functions.push(self.generate_function(function)?);
        }

        Ok(GaiaProgram { name: program.name.clone(), functions, constants: Vec::new(), globals: None })
    }

    /// 生成函数
    fn generate_function(&mut self, function: &Function) -> Result<GaiaFunction> {
        // 重置局部变量状态
        self.locals.clear();
        self.local_index = 0;

        // 添加参数到局部变量
        for param in &function.parameters {
            self.locals.insert(param.name.clone(), self.local_index);
            self.local_index += 1;
        }

        let mut instructions = Vec::new();

        // 生成函数体
        self.generate_block(&function.body, &mut instructions)?;

        // 如果函数没有显式返回，添加默认返回
        if !instructions.iter().any(|inst| matches!(inst, GaiaInstruction::Return)) {
            instructions.push(GaiaInstruction::Return);
        }

        Ok(GaiaFunction {
            name: function.name.clone(),
            parameters: function.parameters.iter().map(|p| p.param_type.to_gaia_type()).collect(),
            return_type: function.return_type.as_ref().map(|t| t.to_gaia_type()),
            locals: Vec::new(), // 暂时为空，后续可以优化
            instructions,
        })
    }

    /// 生成代码块
    fn generate_block(&mut self, block: &Block, instructions: &mut Vec<GaiaInstruction>) -> Result<()> {
        for statement in &block.statements {
            self.generate_statement(statement, instructions)?;
        }
        Ok(())
    }

    /// 生成语句
    fn generate_statement(&mut self, statement: &Statement, instructions: &mut Vec<GaiaInstruction>) -> Result<()> {
        match statement {
            Statement::Expression(expr) => {
                self.generate_expression(expr, instructions)?;
                // 表达式语句需要弹出结果
                instructions.push(GaiaInstruction::Pop);
            }
            Statement::VariableDeclaration { name, var_type: _, initializer } => {
                if let Some(init_expr) = initializer {
                    // 生成初始值
                    self.generate_expression(init_expr, instructions)?;
                }
                else {
                    // 如果没有初始值，使用默认值 0
                    instructions.push(GaiaInstruction::LoadConstant(GaiaConstant::Integer32(0)));
                }

                // 分配局部变量
                let index = self.local_index;
                self.locals.insert(name.clone(), index);
                self.local_index += 1;

                // 存储到局部变量
                instructions.push(GaiaInstruction::StoreLocal(index));
            }
            Statement::Return(expr) => {
                if let Some(expr) = expr {
                    self.generate_expression(expr, instructions)?;
                }
                instructions.push(GaiaInstruction::Return);
            }
        }
        Ok(())
    }

    /// 生成表达式
    fn generate_expression(&mut self, expression: &Expression, instructions: &mut Vec<GaiaInstruction>) -> Result<()> {
        match expression {
            Expression::Literal(literal) => {
                let constant = literal.to_gaia_constant();
                instructions.push(GaiaInstruction::LoadConstant(constant));
            }
            Expression::Identifier(name) => {
                if let Some(&index) = self.locals.get(name) {
                    instructions.push(GaiaInstruction::LoadLocal(index));
                }
                else {
                    return Err(GaiaError::syntax_error(format!("未定义的变量: {}", name), SourceLocation::default()));
                }
            }
            Expression::BinaryOperation { left, operator, right } => {
                self.generate_expression(left, instructions)?;
                self.generate_expression(right, instructions)?;

                let instruction = match operator {
                    BinaryOperator::Add => GaiaInstruction::Add,
                    BinaryOperator::Subtract => GaiaInstruction::Subtract,
                    BinaryOperator::Multiply => GaiaInstruction::Multiply,
                    BinaryOperator::Divide => GaiaInstruction::Divide,
                    BinaryOperator::Equal => GaiaInstruction::Equal,
                    BinaryOperator::NotEqual => GaiaInstruction::NotEqual,
                    BinaryOperator::Less => GaiaInstruction::LessThan,
                    BinaryOperator::LessEqual => GaiaInstruction::LessThanOrEqual,
                    BinaryOperator::Greater => GaiaInstruction::GreaterThan,
                    BinaryOperator::GreaterEqual => GaiaInstruction::GreaterThanOrEqual,
                };

                instructions.push(instruction);
            }
            Expression::UnaryOperation { operator, operand } => {
                match operator {
                    UnaryOperator::Negate => {
                        // 数值取负：0 - operand
                        instructions.push(GaiaInstruction::LoadConstant(GaiaConstant::Integer32(0)));
                        self.generate_expression(operand, instructions)?;
                        instructions.push(GaiaInstruction::Subtract);
                    }
                    UnaryOperator::Not => {
                        // 逻辑非
                        self.generate_expression(operand, instructions)?;
                        instructions.push(GaiaInstruction::LogicalNot);
                    }
                }
            }
            Expression::FunctionCall { name, arguments } => {
                // 生成参数
                for arg in arguments {
                    self.generate_expression(arg, instructions)?;
                }

                // 直接调用函数名，由适配器完成映射
                if name == "console" {
                    return Err(GaiaError::syntax_error("console 不是函数，应该使用 console.log()", SourceLocation::default()));
                }

                instructions.push(GaiaInstruction::Call(name.clone(), arguments.len()));
            }
            Expression::MethodCall { object, method, arguments } => {
                // 处理 console.log
                if let Expression::Identifier(obj_name) = object.as_ref() {
                    if obj_name == "console" && method == "log" {
                        // 生成参数
                        for arg in arguments {
                            self.generate_expression(arg, instructions)?;
                        }

                        // 调用通用函数名，由适配器完成映射
                        instructions.push(GaiaInstruction::Call("console.log".to_string(), arguments.len()));
                        return Ok(());
                    }
                }

                // 其他方法调用暂不支持
                return Err(GaiaError::syntax_error(
                    format!("不支持的方法调用: {}.{}", self.expression_to_string(object), method),
                    SourceLocation::default(),
                ));
            }
            Expression::MacroCall { name, arguments } => {
                // 处理 println! 宏
                if name == "println" {
                    // 生成参数
                    for arg in arguments {
                        self.generate_expression(arg, instructions)?;
                    }

                    // 调用通用函数名，由适配器完成映射
                    instructions.push(GaiaInstruction::Call("println".to_string(), arguments.len()));
                }
                else {
                    return Err(GaiaError::syntax_error(format!("不支持的宏: {}!", name), SourceLocation::default()));
                }
            }
        }
        Ok(())
    }

    /// 将表达式转换为字符串（用于错误消息）
    fn expression_to_string(&self, expr: &Expression) -> String {
        match expr {
            Expression::Identifier(name) => name.clone(),
            Expression::Literal(Literal::String(s)) => format!("\"{}\"", s),
            Expression::Literal(Literal::Integer(i)) => i.to_string(),
            Expression::Literal(Literal::Float(f)) => f.to_string(),
            Expression::Literal(Literal::Boolean(b)) => b.to_string(),
            _ => "<expression>".to_string(),
        }
    }
}

/// Mini Rust 解析器
pub struct MiniRustParser;

impl MiniRustParser {
    pub fn parse(source: &str) -> Result<GaiaProgram> {
        use crate::{lexer::RustLexer, parser::Parser};

        let mut lexer = RustLexer::new(source);
        let diagnostics = lexer.tokenize();
        let token_stream = diagnostics.result?;

        // 调试：打印所有 tokens
        println!("Generated tokens:");
        let tokens_vec = token_stream.tokens.get_ref();
        for (i, token) in tokens_vec.iter().enumerate() {
            println!("  {}: {:?}", i, token);
        }

        let mut parser = Parser::new(token_stream);
        let ast = parser.parse_program()?;

        let mut codegen = CodeGenerator::new();
        codegen.generate(&ast)
    }
}
