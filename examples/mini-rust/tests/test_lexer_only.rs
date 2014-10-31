use crate::lexer::RustLexer;

pub fn test_lexer_functionality() {
    println!("开始测试词法分析器...");

    // 测试简单的函数定义
    let input = "fn main() { let x = 42; }";
    let mut lexer = RustLexer::new(input);
    let diagnostics = lexer.tokenize();

    match diagnostics.result {
        Ok(token_stream) => {
            println!("✓ 词法分析成功");
            println!("  输入: {}", input);
            println!("  生成的 token 数量: {}", token_stream.tokens.len());

            // 打印前几个 token
            for (i, token) in token_stream.tokens.iter().take(5).enumerate() {
                println!("  Token {}: {:?}", i, token.token_type);
            }
        }
        Err(e) => {
            println!("✗ 词法分析失败: {:?}", e);
        }
    }

    // 测试更复杂的代码
    let complex_input = r#"
        fn add(a: i32, b: i32) -> i32 {
            return a + b;
        }
    "#;

    let mut lexer2 = RustLexer::new(complex_input);
    let diagnostics2 = lexer2.tokenize();

    match diagnostics2.result {
        Ok(token_stream) => {
            println!("✓ 复杂代码词法分析成功");
            println!("  生成的 token 数量: {}", token_stream.tokens.len());
        }
        Err(e) => {
            println!("✗ 复杂代码词法分析失败: {:?}", e);
        }
    }

    println!("词法分析器测试完成");
}
