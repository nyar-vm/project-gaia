//! Mini Rust 核心功能测试
//!
//! 这个测试程序验证词法分析器、语法分析器和 AST 生成功能
//! 不依赖于可能有问题的汇编器组件

use crate::{
    ast::{BinaryOperator, Expression, Literal, Statement},
    lexer::{RustLexer, RustTokenType},
    parser::Parser,
};

/// 测试词法分析器
pub fn test_lexer() -> Result<(), String> {
    println!("=== 测试词法分析器 ===");

    let source = r#"fn main() -> i32 {
    let x = 42;
    return x;
}"#;

    let mut lexer = RustLexer::new(source);
    let result = lexer.tokenize();

    if !result.diagnostics.is_empty() {
        return Err(format!("词法分析有错误: {:?}", result.diagnostics));
    }

    let tokens = result.result.map_err(|e| format!("词法分析失败: {:?}", e))?;

    println!("成功解析词法分析");

    // 验证关键 token 类型存在
    let expected_tokens = vec![
        RustTokenType::Fn,
        RustTokenType::Identifier,
        RustTokenType::LeftParen,
        RustTokenType::RightParen,
        RustTokenType::Arrow,
        RustTokenType::Identifier, // i32
        RustTokenType::LeftBrace,
        RustTokenType::Let,
        RustTokenType::Identifier, // x
        RustTokenType::Equal,
        RustTokenType::Integer,
        RustTokenType::Semicolon,
        RustTokenType::Return,
        RustTokenType::Identifier, // x
        RustTokenType::Semicolon,
        RustTokenType::RightBrace,
    ];

    // 简单验证是否包含关键 token
    let mut found_fn = false;
    let mut found_let = false;
    let mut found_return = false;

    for i in 0..tokens.tokens.get_ref().len() {
        if let Ok(token) = tokens.get_token(i) {
            match token.token_type {
                RustTokenType::Fn => found_fn = true,
                RustTokenType::Let => found_let = true,
                RustTokenType::Return => found_return = true,
                _ => {}
            }
        }
    }

    if !found_fn {
        return Err("未找到 fn 关键字".to_string());
    }
    if !found_let {
        return Err("未找到 let 关键字".to_string());
    }
    if !found_return {
        return Err("未找到 return 关键字".to_string());
    }

    println!("✓ 词法分析器测试通过");
    Ok(())
}

/// 测试语法分析器
pub fn test_parser() -> Result<(), String> {
    println!("=== 测试语法分析器 ===");

    let source = r#"fn main() -> i32 {
    let x = 42;
    return x;
}"#;

    let mut lexer = RustLexer::new(source);
    let result = lexer.tokenize();

    if !result.diagnostics.is_empty() {
        return Err(format!("词法分析有错误: {:?}", result.diagnostics));
    }

    let tokens = result.result.map_err(|e| format!("词法分析失败: {:?}", e))?;

    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().map_err(|e| format!("语法分析失败: {:?}", e))?;

    // 验证程序结构
    if program.functions.len() != 1 {
        return Err(format!("期望 1 个函数，实际 {}", program.functions.len()));
    }

    let main_func = &program.functions[0];
    if main_func.name != "main" {
        return Err(format!("期望函数名为 'main'，实际 '{}'", main_func.name));
    }

    if main_func.body.statements.len() != 2 {
        return Err(format!("期望 2 个语句，实际 {}", main_func.body.statements.len()));
    }

    // 验证第一个语句是变量声明
    match &main_func.body.statements[0] {
        Statement::VariableDeclaration { name, initializer, .. } => {
            if name != "x" {
                return Err(format!("期望变量名为 'x'，实际 '{}'", name));
            }
            if let Some(init_expr) = initializer {
                match init_expr {
                    Expression::Literal(Literal::Integer(42)) => {}
                    _ => return Err("期望变量值为整数 42".to_string()),
                }
            }
            else {
                return Err("期望变量有初始值".to_string());
            }
        }
        _ => return Err("期望第一个语句为变量声明".to_string()),
    }

    // 验证第二个语句是返回语句
    match &main_func.body.statements[1] {
        Statement::Return(Some(Expression::Identifier(name))) => {
            if name != "x" {
                return Err(format!("期望返回变量 'x'，实际 '{}'", name));
            }
        }
        _ => return Err("期望第二个语句为返回语句".to_string()),
    }

    println!("✓ 语法分析器测试通过");
    Ok(())
}

/// 测试复杂表达式
pub fn test_complex_expression() -> Result<(), String> {
    println!("=== 测试复杂表达式 ===");

    let source = r#"fn calculate() -> i32 {
    let result = 10 + 5;
    return result;
}"#;

    let mut lexer = RustLexer::new(source);
    let result = lexer.tokenize();

    if !result.diagnostics.is_empty() {
        return Err(format!("词法分析有错误: {:?}", result.diagnostics));
    }

    let tokens = result.result.map_err(|e| format!("词法分析失败: {:?}", e))?;

    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().map_err(|e| format!("语法分析失败: {:?}", e))?;

    let func = &program.functions[0];
    match &func.body.statements[0] {
        Statement::VariableDeclaration { initializer, .. } => {
            // 验证表达式结构 10 + 5
            if let Some(init_expr) = initializer {
                match init_expr {
                    Expression::BinaryOperation { operator: BinaryOperator::Add, .. } => {
                        println!("✓ 正确解析了二元表达式");
                    }
                    _ => return Err("期望二元表达式".to_string()),
                }
            }
            else {
                return Err("期望初始化表达式".to_string());
            }
        }
        _ => return Err("期望变量声明".to_string()),
    }

    println!("✓ 复杂表达式测试通过");
    Ok(())
}

/// 运行所有核心功能测试
pub fn run_all_tests() -> Result<(), String> {
    println!("开始运行 Mini Rust 核心功能测试...\n");

    test_lexer()?;
    println!();

    test_parser()?;
    println!();

    test_complex_expression()?;
    println!();

    println!("🎉 所有核心功能测试通过！");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_functionality() {
        test_lexer().unwrap();
    }

    #[test]
    fn test_parser_functionality() {
        test_parser().unwrap();
    }

    #[test]
    fn test_complex_expressions() {
        test_complex_expression().unwrap();
    }
}
