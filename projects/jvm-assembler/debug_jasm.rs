use jvm_assembler::compile_jasm_to_class;
use jvm_jasm::{lexer::JasmLexer, parser::JasmParser};
use std::fs;

fn main() {
    // 读取 HelloJava.jasm 文件
    let jasm_content = fs::read_to_string("tests/HelloJava.jasm")
        .expect("Failed to read HelloJava.jasm");
    
    println!("JASM content:");
    println!("{}", jasm_content);
    println!("\n--- Testing lexer ---");
    
    // 测试词法分析
    let lexer = JasmLexer::new();
    let tokens_result = lexer.tokenize(&jasm_content);
    
    match tokens_result.result {
        Ok(tokens) => {
            println!("Lexer success! {} tokens", tokens.tokens.get_ref().len());
            
            // 显示所有tokens
            for (i, token) in tokens.tokens.get_ref().iter().enumerate() {
                let text = tokens.get_text(token).unwrap_or("<error>");
                println!("  {}: {:?} - '{}'", i, token.token_type, text);
            }
            
            println!("\n--- Testing parser ---");
            let parser = JasmParser::new();
            let parse_result = parser.parse(tokens);
            
            match parse_result.result {
                Ok(ast) => {
                    println!("Parser success!");
                    println!("Class name: {}", ast.class.name);
                    println!("Class modifiers: {:?}", ast.class.modifiers);
                    println!("Methods: {}", ast.class.methods.len());
                    
                    println!("\n--- Attempting to compile ---");
                    // 尝试编译
                    match compile_jasm_to_class(&jasm_content) {
                        Ok(class_bytes) => {
                            println!("Success! Generated {} bytes", class_bytes.len());
                            
                            // 写入新的 .class 文件
                            fs::write("tests/HelloJava_compiled.class", &class_bytes)
                                .expect("Failed to write compiled class file");
                            
                            println!("Written to tests/HelloJava_compiled.class");
                        }
                        Err(e) => {
                            println!("Compile Error: {:?}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("Parser Error: {:?}", e);
                }
            }
        }
        Err(e) => {
            println!("Lexer Error: {:?}", e);
        }
    }
}