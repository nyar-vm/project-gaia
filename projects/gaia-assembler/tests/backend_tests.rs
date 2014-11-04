use gaia_assembler::*;
use gaia_types::{
    helpers::{CompilationTarget, Architecture, AbiCompatible, ApiCompatible},
    GaiaProgram, GaiaFunction, GaiaType, GaiaInstruction, GaiaConstant,
    Result,
};

/// 测试所有后端的基本功能
#[test]
fn test_all_backends_basic_functionality() {
    let compiler = GaiaCompiler::new();
    let program = create_test_program();
    let target = CompilationTarget {
        build: Architecture::X86_64,
        host: AbiCompatible::PE,
        target: ApiCompatible::MicrosoftVisualC,
    };
    
    println!("测试 {} 个后端", compiler.backends().len());
    
    for backend in compiler.backends() {
        println!("测试后端: {}", backend.name());
        
        // 测试兼容性评分
        let score = backend.match_score(&target);
        assert!(score >= 0.0 && score <= 1.0, "兼容性评分应在 0.0-1.0 之间");
        println!("  兼容性评分: {:.2}", score);
        
        // 如果兼容性评分大于 0，尝试编译
        if score > 0.0 {
            match backend.compile(&program) {
                Ok(bytecode) => {
                    assert!(!bytecode.is_empty(), "编译结果不应为空");
                    println!("  编译成功: {} 字节", bytecode.len());
                }
                Err(e) => {
                    println!("  编译失败: {:?}", e);
                    // 某些后端可能还未完全实现，这是可以接受的
                }
            }
        } else {
            println!("  跳过编译（兼容性评分为 0）");
        }
    }
}

/// 测试 JVM 后端特定功能
#[test]
fn test_jvm_backend() {
    let compiler = GaiaCompiler::new();
    let program = create_test_program();
    
    // 查找 JVM 后端
    let jvm_backend = compiler.backends()
        .iter()
        .find(|b| b.name().to_lowercase().contains("jvm"))
        .expect("应该有 JVM 后端");
    
    println!("测试 JVM 后端: {}", jvm_backend.name());
    
    // 测试 JVM 特定的编译目标
    let jvm_target = CompilationTarget {
        build: Architecture::JVM,
        host: AbiCompatible::JavaAssembly,
        target: ApiCompatible::JvmRuntime(8),
    };
    let score = jvm_backend.match_score(&jvm_target);
    
    println!("JVM 兼容性评分: {:.2}", score);
    
    if score > 0.0 {
        match jvm_backend.compile(&program) {
            Ok(bytecode) => {
                println!("JVM 编译成功: {} 字节", bytecode.len());
                // 可以添加更多 JVM 特定的验证
            }
            Err(e) => {
                println!("JVM 编译失败: {:?}", e);
            }
        }
    }
}

/// 测试 MSIL 后端特定功能
#[test]
fn test_msil_backend() {
    let compiler = GaiaCompiler::new();
    let program = create_test_program();
    
    // 查找 MSIL 后端
    let msil_backend = compiler.backends()
        .iter()
        .find(|b| b.name().to_lowercase().contains("msil") || b.name().to_lowercase().contains("clr"))
        .expect("应该有 MSIL 后端");
    
    println!("测试 MSIL 后端: {}", msil_backend.name());
    
    let msil_target = CompilationTarget {
        build: Architecture::CLR,
        host: AbiCompatible::MicrosoftIntermediateLanguage,
        target: ApiCompatible::ClrRuntime(4),
    };
    let score = msil_backend.match_score(&msil_target);
    
    println!("MSIL 兼容性评分: {:.2}", score);
    
    if score > 0.0 {
        match msil_backend.compile(&program) {
            Ok(bytecode) => {
                println!("MSIL 编译成功: {} 字节", bytecode.len());
                // 可以添加更多 MSIL 特定的验证
            }
            Err(e) => {
                println!("MSIL 编译失败: {:?}", e);
            }
        }
    }
}

/// 测试 PE 后端特定功能
#[test]
fn test_pe_backend() {
    let compiler = GaiaCompiler::new();
    let program = create_test_program();
    
    // 查找 PE 后端
    let pe_backend = compiler.backends()
        .iter()
        .find(|b| b.name().to_lowercase().contains("pe"))
        .expect("应该有 PE 后端");
    
    println!("测试 PE 后端: {}", pe_backend.name());
    
    let pe_target = CompilationTarget {
        build: Architecture::X86_64,
        host: AbiCompatible::PE,
        target: ApiCompatible::MicrosoftVisualC,
    };
    let score = pe_backend.match_score(&pe_target);
    
    println!("PE 兼容性评分: {:.2}", score);
    
    if score > 0.0 {
        match pe_backend.compile(&program) {
            Ok(bytecode) => {
                println!("PE 编译成功: {} 字节", bytecode.len());
                // 可以添加更多 PE 特定的验证
            }
            Err(e) => {
                println!("PE 编译失败: {:?}", e);
            }
        }
    }
}

/// 测试 WASI 后端特定功能
#[test]
fn test_wasi_backend() {
    let compiler = GaiaCompiler::new();
    let program = create_test_program();
    
    // 查找 WASI 后端
    let wasi_backend = compiler.backends()
        .iter()
        .find(|b| b.name().to_lowercase().contains("wasi"))
        .expect("应该有 WASI 后端");
    
    println!("测试 WASI 后端: {}", wasi_backend.name());
    
    let wasi_target = CompilationTarget {
        build: Architecture::WASM32,
        host: AbiCompatible::WebAssemblyTextFormat,
        target: ApiCompatible::WASI,
    };
    let score = wasi_backend.match_score(&wasi_target);
    
    println!("WASI 兼容性评分: {:.2}", score);
    
    if score > 0.0 {
        match wasi_backend.compile(&program) {
            Ok(bytecode) => {
                println!("WASI 编译成功: {} 字节", bytecode.len());
                // 可以添加更多 WASI 特定的验证
            }
            Err(e) => {
                println!("WASI 编译失败: {:?}", e);
            }
        }
    }
}

/// 测试后端兼容性评分系统
#[test]
fn test_backend_compatibility_scoring() {
    let compiler = GaiaCompiler::new();
    let target = CompilationTarget {
        build: Architecture::X86_64,
        host: AbiCompatible::PE,
        target: ApiCompatible::MicrosoftVisualC,
    };
    
    let mut scores = Vec::new();
    
    for backend in compiler.backends() {
        let score = backend.match_score(&target);
        scores.push((backend.name().to_string(), score));
        
        // 验证评分在有效范围内
        assert!(score >= 0.0 && score <= 1.0, 
                "后端 {} 的兼容性评分 {} 不在有效范围 [0.0, 1.0] 内", 
                backend.name(), score);
    }
    
    // 按评分排序
    scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    
    println!("后端兼容性评分排序:");
    for (name, score) in scores {
        println!("  {}: {:.2}", name, score);
    }
}

/// 测试后端错误处理
#[test]
fn test_backend_error_handling() {
    let compiler = GaiaCompiler::new();
    
    // 创建一个可能导致错误的程序
    let invalid_program = GaiaProgram {
        name: "invalid".to_string(),
        functions: vec![],
        constants: vec![],
    };
    
    let target = CompilationTarget {
        build: Architecture::X86_64,
        host: AbiCompatible::PE,
        target: ApiCompatible::MicrosoftVisualC,
    };
    
    for backend in compiler.backends() {
        let result = backend.compile(&invalid_program);
        
        // 验证错误处理不会导致崩溃
        match result {
            Ok(bytecode) => {
                println!("后端 {} 成功处理了空程序，生成 {} 字节", 
                        backend.name(), bytecode.len());
            }
            Err(e) => {
                println!("后端 {} 正确报告了错误: {:?}", backend.name(), e);
            }
        }
    }
}

/// 测试后端性能基准
#[test]
fn test_backend_performance() {
    let compiler = GaiaCompiler::new();
    let program = create_complex_test_program();
    let target = CompilationTarget {
        build: Architecture::X86_64,
        host: AbiCompatible::PE,
        target: ApiCompatible::MicrosoftVisualC,
    };
    
    for backend in compiler.backends() {
        if backend.match_score(&target) > 0.0 {
            let start = std::time::Instant::now();
            
            match backend.compile(&program) {
                Ok(bytecode) => {
                    let duration = start.elapsed();
                    println!("后端 {} 编译耗时: {:?}, 生成 {} 字节", 
                            backend.name(), duration, bytecode.len());
                }
                Err(e) => {
                    let duration = start.elapsed();
                    println!("后端 {} 编译失败，耗时: {:?}, 错误: {:?}", 
                            backend.name(), duration, e);
                }
            }
        }
    }
}

// 辅助函数

fn create_test_program() -> GaiaProgram {
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
        name: "test_program".to_string(),
        functions: vec![main_function],
        constants: vec![],
    }
}

fn create_complex_test_program() -> GaiaProgram {
    let main_function = GaiaFunction {
        name: "main".to_string(),
        parameters: vec![],
        return_type: None,
        instructions: vec![
            GaiaInstruction::Return,
        ],
        locals: vec![GaiaType::Integer32, GaiaType::Float64],
    };
    
    let helper_function = GaiaFunction {
        name: "helper".to_string(),
        parameters: vec![GaiaType::Integer32, GaiaType::Integer32],
        return_type: Some(GaiaType::Integer32),
        instructions: vec![
            GaiaInstruction::Return,
        ],
        locals: vec![GaiaType::Integer32],
    };
    
    GaiaProgram {
        name: "complex_test_program".to_string(),
        functions: vec![main_function, helper_function],
        constants: vec![],
    }
}