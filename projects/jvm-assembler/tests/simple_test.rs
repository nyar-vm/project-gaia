use jvm_jasm::lexer::JasmLexer;

#[test]
fn test_minimal_lexer() {
    println!("Starting minimal lexer test...");

    let input = "class";
    let lexer = JasmLexer::new();
    let tokens = lexer.tokenize(input);

    println!("Tokens: {:?}", tokens);
    println!("Test completed successfully!");
}

#[test]
fn test_minimal_parser() {
    println!("Starting minimal parser test...");

    let input = "class";
    let lexer = jvm_jasm::lexer::JasmLexer::new();
    let tokens_result = lexer.tokenize(input);

    match tokens_result.result {
        Ok(tokens) => {
            let parser = jvm_jasm::parser::JasmParser::new();
            let result = parser.parse(tokens);

            println!("Parse result: {:?}", result);
        }
        Err(e) => {
            println!("Tokenization failed: {:?}", e);
        }
    }

    println!("Test completed!");
}

#[test]
fn test_single_token() {
    println!("Starting single token test...");

    let input = "public class";
    let lexer = jvm_jasm::lexer::JasmLexer::new();
    let tokens_result = lexer.tokenize(input);

    match tokens_result.result {
        Ok(tokens) => {
            println!("Tokens generated successfully");

            // 打印所有 token
            let token_vec = tokens.tokens.get_ref();
            for (i, token) in token_vec.iter().enumerate() {
                println!("Token {}: {:?}", i, token);
            }
        }
        Err(e) => {
            println!("Tokenization failed: {:?}", e);
        }
    }

    println!("Single token test completed!");
}

#[test]
fn test_simple_parser() {
    println!("Starting simple parser test...");

    let input = "public class";
    let lexer = jvm_jasm::lexer::JasmLexer::new();
    let tokens_result = lexer.tokenize(input);

    match tokens_result.result {
        Ok(tokens) => {
            println!("Tokens generated successfully");

            let parser = jvm_jasm::parser::JasmParser::new();
            let result = parser.parse(tokens);

            println!("Parse result: {:?}", result);
        }
        Err(e) => {
            println!("Tokenization failed: {:?}", e);
        }
    }

    println!("Simple parser test completed!");
}
