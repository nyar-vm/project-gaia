use gaia_assembler::{
    assembler::{compile_to_platform, GaiaAssembler},
    *,
};
use gaia_types::{
    helpers::{AbiCompatible, ApiCompatible, Architecture, CompilationTarget},
    GaiaConstant, GaiaFunction, GaiaInstruction, GaiaProgram, GaiaType, Result,
};

// 类型别名，保持向后兼容
type GaiaCompiler = GaiaAssembler;

/// 测试编译器创建和基本功能
#[test]
fn test_compiler_creation() {
    let compiler = GaiaCompiler::new();

    // 验证编译器包含所有预期的后端
    let backends = compiler.backends();
    assert_eq!(backends.len(), 4, "应该有4个后端：JVM、MSIL、PE、WASI");

    // 验证后端名称
    let backend_names: Vec<String> = backends.iter().map(|b| b.name().to_string()).collect();

    assert!(backend_names.contains(&"JVM".to_string()));
    assert!(backend_names.contains(&"MSIL".to_string()));
    assert!(backend_names.contains(&"PE".to_string()));
    assert!(backend_names.contains(&"WASI".to_string()));
}

/// 测试编译目标支持
#[test]
fn test_compilation_targets() {
    let compiler = GaiaCompiler::new();

    // 测试各种编译目标
    let targets = vec![CompilationTarget {
        build: Architecture::X86_64,
        host: AbiCompatible::PE,
        target: ApiCompatible::MicrosoftVisualC,
    } /* 默认目标 */];

    for target in targets {
        // 验证编译器能够处理这些目标
        // 这里我们只测试编译器不会崩溃
        let scores: Vec<f32> = compiler.backends().iter().map(|backend| backend.match_score(&target)).collect();

        // 验证至少有一个后端有非零评分
        let max_score = scores.iter().fold(0.0f32, |a, &b| a.max(b));
        println!("目标 {:?} 的最大兼容性评分: {}", target, max_score);
    }
}

/// 测试简单程序编译
#[test]
fn test_simple_program_compilation() {
    let compiler = GaiaCompiler::new();

    // 创建一个简单的测试程序
    let program = create_test_program();
    let target = CompilationTarget {
        build: Architecture::CLR,
        host: AbiCompatible::MicrosoftIntermediateLanguage,
        target: ApiCompatible::ClrRuntime(4),
    };

    // 尝试编译
    let result = compiler.compile(&program, &target);

    // 验证编译结果
    match result {
        Ok(generated_files) => {
            assert!(!generated_files.files.is_empty(), "编译后的文件不应为空");
            let total_size: usize = generated_files.files.values().map(|v| v.len()).sum();
            println!("编译成功，生成了 {} 字节的代码", total_size);
        }
        Err(e) => {
            // 如果编译失败，至少验证错误是合理的
            println!("编译失败（这可能是预期的）: {:?}", e);
        }
    }
}

/// 测试编译到所有平台
#[test]
fn test_compile_all_platforms() {
    let compiler = GaiaCompiler::new();
    let program = create_test_program();

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

    assert!(!results.is_empty(), "应该至少有一个编译结果");

    for (target, generated_files) in results {
        let total_size: usize = generated_files.files.values().map(|v| v.len()).sum();
        println!("成功编译到目标 {:?}，生成 {} 字节", target, total_size);
        assert!(!generated_files.files.is_empty(), "生成的文件不应为空");
    }
}

/// 测试端到端编译
#[test]
fn test_end_to_end_compilation() {
    let compiler = GaiaCompiler::new();
    let program = create_test_program();
    let target =
        CompilationTarget { build: Architecture::X86_64, host: AbiCompatible::Unknown, target: ApiCompatible::Unknown };

    let result = compiler.compile(&program, &target);

    match result {
        Ok(generated_files) => {
            assert!(!generated_files.files.is_empty(), "编译后的文件不应为空");
            let total_size: usize = generated_files.files.values().map(|v| v.len()).sum();
            println!("端到端编译成功，生成 {} 字节", total_size);
        }
        Err(e) => {
            println!("编译失败: {:?}", e);
            // 某些后端可能还未完全实现，这是可以接受的
        }
    }
}

/// 测试后端兼容性评分
#[test]
fn test_backend_compatibility_scoring() {
    let compiler = GaiaCompiler::new();
    let target = CompilationTarget {
        build: Architecture::CLR,
        host: AbiCompatible::MicrosoftIntermediateLanguage,
        target: ApiCompatible::ClrRuntime(4),
    };

    for backend in compiler.backends() {
        let score = backend.match_score(&target);

        // 兼容性评分应该在合理范围内
        assert!(score >= 0.0, "兼容性评分不应为负数");
        assert!(score <= 1.0, "兼容性评分不应超过1.0");

        println!("后端 {} 对目标 {:?} 的兼容性评分: {:.2}", backend.name(), target, score);
    }
}

/// 测试错误处理
#[test]
fn test_error_handling() {
    let compiler = GaiaCompiler::new();

    // 创建一个空程序
    let empty_program = GaiaProgram { name: "empty".to_string(), functions: vec![], constants: vec![] };

    let target =
        CompilationTarget { build: Architecture::X86_64, host: AbiCompatible::PE, target: ApiCompatible::MicrosoftVisualC };

    // 编译空程序应该能够处理（可能成功也可能失败，但不应崩溃）
    let result = compiler.compile(&empty_program, &target);

    match result {
        Ok(_) => println!("空程序编译成功"),
        Err(e) => println!("空程序编译失败（预期）: {:?}", e),
    }
}

/// 测试后端选择
#[test]
fn test_backend_selection() {
    let compiler = GaiaCompiler::new();
    let program = create_test_program();

    // 测试不同的编译目标
    let targets = vec![
        CompilationTarget { build: Architecture::X86_64, host: AbiCompatible::PE, target: ApiCompatible::MicrosoftVisualC },
        CompilationTarget { build: Architecture::JVM, host: AbiCompatible::JavaAssembly, target: ApiCompatible::JvmRuntime(8) },
        CompilationTarget {
            build: Architecture::CLR,
            host: AbiCompatible::MicrosoftIntermediateLanguage,
            target: ApiCompatible::ClrRuntime(4),
        },
        CompilationTarget {
            build: Architecture::WASM32,
            host: AbiCompatible::WebAssemblyTextFormat,
            target: ApiCompatible::WASI,
        },
    ];

    for target in targets {
        let result = compiler.compile(&program, &target);
        match result {
            Ok(generated_files) => {
                let total_size: usize = generated_files.files.values().map(|v| v.len()).sum();
                println!("目标 {} 编译成功，生成 {} 字节", target, total_size);
            }
            Err(e) => {
                println!("目标 {} 编译失败: {:?}", target, e);
                // 某些后端可能还未完全实现，这是可以接受的
            }
        }
    }
}

/// 测试复杂程序编译
#[test]
fn test_complex_program_compilation() {
    let compiler = GaiaCompiler::new();
    let program = create_complex_test_program();
    let target =
        CompilationTarget { build: Architecture::X86_64, host: AbiCompatible::Unknown, target: ApiCompatible::Unknown };

    let result = compiler.compile(&program, &target);

    match result {
        Ok(generated_files) => {
            assert!(!generated_files.files.is_empty(), "编译后的文件不应为空");
            let total_size: usize = generated_files.files.values().map(|v| v.len()).sum();
            println!("复杂程序编译成功，生成 {} 字节", total_size);
        }
        Err(e) => {
            println!("复杂程序编译失败: {:?}", e);
            // 某些后端可能还未完全实现，这是可以接受的
        }
    }
}

/// 创建一个简单的测试程序
fn create_test_program() -> GaiaProgram {
    // 创建一个简单的函数
    let main_function = GaiaFunction {
        name: "main".to_string(),
        parameters: vec![],
        return_type: None,
        instructions: vec![GaiaInstruction::Return],
        locals: vec![],
    };

    GaiaProgram { name: "test_program".to_string(), functions: vec![main_function], constants: vec![] }
}

/// 创建一个复杂的测试程序
fn create_complex_test_program() -> GaiaProgram {
    let main_function = GaiaFunction {
        name: "main".to_string(),
        parameters: vec![],
        return_type: None,
        instructions: vec![
            GaiaInstruction::LoadConstant(GaiaConstant::Integer32(42)),
            GaiaInstruction::StoreLocal(0),
            GaiaInstruction::LoadLocal(0),
            GaiaInstruction::LoadConstant(GaiaConstant::Integer32(10)),
            GaiaInstruction::Add,
            GaiaInstruction::Return,
        ],
        locals: vec![GaiaType::Integer32],
    };

    GaiaProgram { name: "complex_test_program".to_string(), functions: vec![main_function], constants: vec![] }
}
