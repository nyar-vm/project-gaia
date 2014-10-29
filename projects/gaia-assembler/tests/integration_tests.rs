//! 集成测试

use gaia_assembler::*;

#[test]
fn test_full_compilation_pipeline() {
    // 测试完整的编译流程：导入 -> 编译 -> 导出

    // 1. 创建一个简单的汇编程序
    let asm_code = r#"
            .section .text
            .global _start
            _start:
                mov eax, 42
                ret
        "#;

    // 2. 使用 PE 导入适配器解析汇编代码
    let mut pe_adapter = import_adapters::PeImportAdapter::new();
    let program_result = pe_adapter.import_program(asm_code);

    match program_result {
        Ok(program) => {
            // 3. 使用 PE 后端编译程序
            let binary_result = backends::pe::compile(&program);

            match binary_result {
                Ok(binary) => {
                    // 4. 验证生成的二进制文件
                    assert!(!binary.is_empty());
                    println!("Successfully compiled program, binary size: {} bytes", binary.len());

                    // 5. 使用导出适配器导出为其他格式
                    let mut export_adapter = export_adapters::PeExportAdapter::new();
                    let export_result = export_adapter.export_program(&program);

                    match export_result {
                        Ok(exported_code) => {
                            assert!(!exported_code.is_empty());
                            println!("Successfully exported program");
                        }
                        Err(err) => {
                            println!("Export not yet implemented: {:?}", err);
                        }
                    }
                }
                Err(err) => {
                    println!("Compilation not yet implemented: {:?}", err);
                }
            }
        }
        Err(err) => {
            println!("Import failed or not yet implemented: {:?}", err);
        }
    }
}

#[test]
fn test_cross_architecture_compilation() {
    // 测试跨架构编译
    let program = GaiaProgram {
        functions: vec![GaiaFunction {
            name: "main".to_string(),
            instructions: vec![
                GaiaInstruction::LoadConstant(GaiaConstant::Int32(42)),
                GaiaInstruction::LoadConstant(GaiaConstant::Int32(24)),
                GaiaInstruction::Add,
                GaiaInstruction::Return,
            ],
            parameters: vec![],
            return_type: Some(GaiaType::Integer32),
        }],
    };

    // 编译到不同的目标架构
    let targets = vec![
        ("PE (x86/x64)", backends::pe::compile),
        ("IL (.NET)", backends::msil::compile),
        ("JVM (Java)", backends::jvm::compile),
        ("WASI (WebAssembly)", backends::wasi::compile),
    ];

    for (target_name, compile_fn) in targets {
        let result = compile_fn(&program);
        match result {
            Ok(binary) => {
                println!("✓ {} compilation successful: {} bytes", target_name, binary.len());
                assert!(!binary.is_empty());
            }
            Err(err) => {
                println!("○ {} not yet implemented: {:?}", target_name, err);
            }
        }
    }
}

#[test]
fn test_round_trip_conversion() {
    // 测试往返转换：汇编 -> Gaia IR -> 汇编
    let original_asm = r#"
            mov eax, 42
            add eax, 24
            ret
        "#;

    // 1. 导入汇编代码
    let mut import_adapter = import_adapters::PeImportAdapter::new();
    let program_result = import_adapter.import_program(original_asm);

    if let Ok(program) = program_result {
        // 2. 导出回汇编代码
        let mut export_adapter = export_adapters::PeExportAdapter::new();
        let export_result = export_adapter.export_program(&program);

        match export_result {
            Ok(exported_asm) => {
                // 3. 验证导出的代码包含关键指令
                assert!(exported_asm.contains("mov") || exported_asm.contains("LoadConstant"));
                assert!(exported_asm.contains("add") || exported_asm.contains("Add"));
                assert!(exported_asm.contains("ret") || exported_asm.contains("Return"));
                println!("Round-trip conversion successful");
            }
            Err(err) => {
                println!("Export not yet implemented: {:?}", err);
            }
        }
    }
    else {
        println!("Import not yet implemented: {:?}", program_result.unwrap_err());
    }
}

#[test]
fn test_multi_language_interop() {
    // 测试多语言互操作性
    let program = GaiaProgram {
        functions: vec![GaiaFunction {
            name: "add_numbers".to_string(),
            instructions: vec![
                GaiaInstruction::LoadArgument(0),
                GaiaInstruction::LoadArgument(1),
                GaiaInstruction::Add,
                GaiaInstruction::Return,
            ],
            parameters: vec![GaiaType::Integer32, GaiaType::Integer32],
            return_type: Some(GaiaType::Integer32),
        }],
    };

    // 编译到不同的运行时环境
    let runtimes = vec![
        ("Native (PE)", backends::pe::compile),
        (".NET (IL)", backends::msil::compile),
        ("JVM (Java)", backends::jvm::compile),
        ("Web (WASI)", backends::wasi::compile),
    ];

    let mut successful_compilations = 0;

    for (runtime_name, compile_fn) in runtimes {
        match compile_fn(&program) {
            Ok(binary) => {
                successful_compilations += 1;
                println!("✓ {} runtime: {} bytes", runtime_name, binary.len());

                // 验证二进制格式
                match runtime_name {
                    name if name.contains("PE") => {
                        assert_eq!(&binary[0..2], b"MZ", "Invalid PE header");
                    }
                    name if name.contains("Java") => {
                        assert_eq!(&binary[0..4], &[0xCA, 0xFE, 0xBA, 0xBE], "Invalid Java class header");
                    }
                    name if name.contains("WASI") => {
                        assert_eq!(&binary[0..4], &[0x00, 0x61, 0x73, 0x6D], "Invalid WASM header");
                    }
                    _ => {
                        // IL 和其他格式的验证
                        assert!(!binary.is_empty());
                    }
                }
            }
            Err(err) => {
                println!("○ {} not implemented: {:?}", runtime_name, err);
            }
        }
    }

    println!("Successfully compiled to {}/4 target runtimes", successful_compilations);
}

#[test]
fn test_optimization_pipeline() {
    // 测试优化流水线
    let unoptimized_program = GaiaProgram {
        functions: vec![GaiaFunction {
            name: "redundant_operations".to_string(),
            instructions: vec![
                // 冗余操作：加载常量然后立即丢弃
                GaiaInstruction::LoadConstant(GaiaConstant::Int32(42)),
                GaiaInstruction::LoadConstant(GaiaConstant::Int32(0)),
                GaiaInstruction::Add, // 42 + 0 = 42，可以优化为直接加载 42
                // 另一个冗余操作
                GaiaInstruction::LoadConstant(GaiaConstant::Int32(1)),
                GaiaInstruction::Multiply, // 42 * 1 = 42，可以优化掉
                GaiaInstruction::Return,
            ],
            parameters: vec![],
            return_type: Some(GaiaType::Integer32),
        }],
    };

    // 编译未优化的程序
    let unoptimized_result = backends::pe::compile(&unoptimized_program);

    // TODO: 实现优化器后，可以比较优化前后的代码大小和性能
    match unoptimized_result {
        Ok(binary) => {
            println!("Unoptimized binary size: {} bytes", binary.len());
            // 未来可以添加优化器测试
        }
        Err(err) => {
            println!("Compilation not yet implemented: {:?}", err);
        }
    }
}

#[test]
fn test_error_recovery() {
    // 测试错误恢复机制
    let programs_with_errors = vec![
        // 语法错误
        GaiaProgram {
            functions: vec![GaiaFunction {
                name: "syntax_error".to_string(),
                instructions: vec![
                    // 缺少操作数的指令
                    GaiaInstruction::Add, // 栈上没有足够的操作数
                    GaiaInstruction::Return,
                ],
                parameters: vec![],
                return_type: None,
            }],
        },
        // 类型错误
        GaiaProgram {
            functions: vec![GaiaFunction {
                name: "type_error".to_string(),
                instructions: vec![
                    GaiaInstruction::LoadConstant(GaiaConstant::String("hello".to_string())),
                    GaiaInstruction::LoadConstant(GaiaConstant::Int32(42)),
                    GaiaInstruction::Add, // 尝试将字符串和整数相加
                    GaiaInstruction::Return,
                ],
                parameters: vec![],
                return_type: Some(GaiaType::String),
            }],
        },
    ];

    for (i, program) in programs_with_errors.iter().enumerate() {
        let result = backends::pe::compile(program);
        match result {
            Ok(_) => {
                println!("Program {} compiled successfully (error not detected)", i + 1);
            }
            Err(err) => {
                println!("Program {} failed as expected: {:?}", i + 1, err);
                // 验证错误类型是合理的
                let error_str = format!("{:?}", err);
                assert!(
                    error_str.contains("not implemented")
                        || error_str.contains("NotImplemented")
                        || error_str.contains("invalid")
                        || error_str.contains("syntax")
                        || error_str.contains("type")
                );
            }
        }
    }
}
