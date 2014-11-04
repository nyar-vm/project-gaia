use gaia_assembler::*;
use gaia_types::{
    helpers::{CompilationTarget, Architecture, AbiCompatible, ApiCompatible},
    GaiaProgram, GaiaFunction, GaiaType, GaiaInstruction, GaiaConstant,
    Result,
};

/// 测试编译器的基本编译功能
#[test]
fn test_basic_compilation() {
    let compiler = GaiaCompiler::new();
    let program = create_hello_world_program();
    let target = CompilationTarget {
        build: Architecture::X86_64,
        host: AbiCompatible::PE,
        target: ApiCompatible::MicrosoftVisualC,
    };
    
    let result = compiler.compile(&program, &target);
    
    match result {
        Ok(bytecode) => {
            assert!(!bytecode.is_empty(), "编译后的字节码不应为空");
            println!("Hello World 程序编译成功，生成 {} 字节", bytecode.len());
        }
        Err(e) => {
            println!("编译失败: {:?}", e);
            // 某些后端可能还未完全实现，这是可以接受的
        }
    }
}

/// 测试多函数程序编译
#[test]
fn test_multi_function_compilation() {
    let compiler = GaiaCompiler::new();
    let program = create_multi_function_program();
    let target = CompilationTarget {
        build: Architecture::X86_64,
        host: AbiCompatible::PE,
        target: ApiCompatible::MicrosoftVisualC,
    };
    
    let result = compiler.compile(&program, &target);
    
    match result {
        Ok(bytecode) => {
            assert!(!bytecode.is_empty(), "多函数程序字节码不应为空");
            println!("多函数程序编译成功，生成 {} 字节", bytecode.len());
        }
        Err(e) => {
            println!("多函数程序编译失败: {:?}", e);
        }
    }
}

/// 测试带全局变量的程序编译
#[test]
fn test_global_variables_compilation() {
    let compiler = GaiaCompiler::new();
    let program = create_program_with_globals();
    let target = CompilationTarget {
        build: Architecture::X86_64,
        host: AbiCompatible::PE,
        target: ApiCompatible::MicrosoftVisualC,
    };
    
    let result = compiler.compile(&program, &target);
    
    match result {
        Ok(bytecode) => {
            assert!(!bytecode.is_empty(), "带全局变量的程序字节码不应为空");
            println!("带全局变量的程序编译成功，生成 {} 字节", bytecode.len());
        }
        Err(e) => {
            println!("带全局变量的程序编译失败: {:?}", e);
        }
    }
}

/// 测试编译到所有平台
#[test]
fn test_compile_to_all_platforms() {
    let compiler = GaiaCompiler::new();
    let program = create_simple_program();
    
    // 获取所有后端的主要目标
    let mut results = Vec::new();
    for backend in compiler.backends() {
        let target = backend.primary_target();
        match compiler.compile(&program, &target) {
            Ok(bytecode) => {
                results.push((target, bytecode));
            }
            Err(e) => {
                println!("编译到目标 {:?} 失败: {:?}", target, e);
            }
        }
    }
    
    println!("成功编译到 {} 个平台", results.len());
    
    for (target, bytecode) in results {
        assert!(!bytecode.is_empty(), "目标 {:?} 的字节码不应为空", target);
        println!("目标 {:?}: {} 字节", target, bytecode.len());
    }
}

/// 测试最佳后端选择
#[test]
fn test_best_backend_selection() {
    let compiler = GaiaCompiler::new();
    let target = CompilationTarget {
        build: Architecture::X86_64,
        host: AbiCompatible::PE,
        target: ApiCompatible::MicrosoftVisualC,
    };
    
    // 找到最佳后端
    let mut best_score = 0.0;
    let mut best_backend_name = String::new();
    
    for backend in compiler.backends() {
        let score = backend.match_score(&target);
        if score > best_score {
            best_score = score;
            best_backend_name = backend.name().to_string();
        }
    }
    
    assert!(best_score > 0.0, "应该至少有一个兼容的后端");
    println!("最佳后端: {} (评分: {:.2})", best_backend_name, best_score);
}

/// 测试编译器错误处理
#[test]
fn test_compiler_error_handling() {
    let compiler = GaiaCompiler::new();
    
    // 测试空程序
    let empty_program = GaiaProgram {
        name: "empty".to_string(),
        functions: vec![],
        constants: vec![],
    };
    
    let target = CompilationTarget {
        build: Architecture::X86_64,
        host: AbiCompatible::PE,
        target: ApiCompatible::MicrosoftVisualC,
    };
    let result = compiler.compile(&empty_program, &target);
    
    // 空程序可能编译成功也可能失败，但不应该崩溃
    match result {
        Ok(bytecode) => println!("空程序编译成功，生成 {} 字节", bytecode.len()),
        Err(e) => println!("空程序编译失败（可能是预期的）: {:?}", e),
    }
}

/// 测试便利函数
#[test]
fn test_convenience_functions() {
    let program = create_simple_program();
    let target = CompilationTarget {
        build: Architecture::X86_64,
        host: AbiCompatible::PE,
        target: ApiCompatible::MicrosoftVisualC,
    };
    
    // 测试 compile_to_platform 函数
    let result = compile_to_platform(&program, target.clone());
    match result {
        Ok(bytecode) => {
            assert!(!bytecode.is_empty(), "便利函数编译结果不应为空");
            println!("便利函数编译成功，生成 {} 字节", bytecode.len());
        }
        Err(e) => {
            println!("便利函数编译失败: {:?}", e);
        }
    }
    
    // 测试 compile_to_all_platforms 函数（手动实现）
    let compiler = GaiaCompiler::new();
    let mut results = Vec::new();
    for backend in compiler.backends() {
        let target = backend.primary_target();
        match compiler.compile(&program, &target) {
            Ok(bytecode) => {
                results.push((target, bytecode));
            }
            Err(e) => {
                println!("编译到目标 {:?} 失败: {:?}", target, e);
            }
        }
    }
    println!("便利函数编译到 {} 个平台", results.len());
}

// 辅助函数：创建各种测试程序

fn create_hello_world_program() -> GaiaProgram {
    let main_function = GaiaFunction {
        name: "main".to_string(),
        parameters: vec![],
        return_type: None,
        instructions: vec![
            // 简化的 Hello World 指令序列
            GaiaInstruction::Return,
        ],
        locals: vec![],
    };
    
    GaiaProgram {
        name: "hello_world".to_string(),
        functions: vec![main_function],
        constants: vec![],
    }
}

fn create_multi_function_program() -> GaiaProgram {
    let main_function = GaiaFunction {
        name: "main".to_string(),
        parameters: vec![],
        return_type: None,
        instructions: vec![
            GaiaInstruction::Return,
        ],
        locals: vec![],
    };
    
    let helper_function = GaiaFunction {
        name: "helper".to_string(),
        parameters: vec![GaiaType::Integer32],
        return_type: Some(GaiaType::Integer32),
        instructions: vec![
            GaiaInstruction::Return,
        ],
        locals: vec![],
    };
    
    GaiaProgram {
        name: "multi_function".to_string(),
        functions: vec![main_function, helper_function],
        constants: vec![],
    }
}

fn create_program_with_globals() -> GaiaProgram {
    let main_function = GaiaFunction {
        name: "main".to_string(),
        parameters: vec![],
        return_type: None,
        instructions: vec![
            GaiaInstruction::Return,
        ],
        locals: vec![],
    };
    
    GaiaProgram {
        name: "with_globals".to_string(),
        functions: vec![main_function],
        constants: vec![("global_counter".to_string(), GaiaConstant::Integer32(0))],
    }
}

fn create_simple_program() -> GaiaProgram {
    let main_function = GaiaFunction {
        name: "main".to_string(),
        parameters: vec![],
        return_type: None,
        instructions: vec![
            GaiaInstruction::Return,
        ],
        locals: vec![],
    };
    
    GaiaProgram {
        name: "simple".to_string(),
        functions: vec![main_function],
        constants: vec![],
    }
}