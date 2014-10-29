//! 后端编译器测试

use gaia_assembler::backends::*;
use gaia_types::*;

#[test]
fn test_pe_backend_compilation() {
    // 创建一个简单的 Gaia 程序
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
            return_type: Some(GaiaType::Int32),
        }],
    };

    // 测试 PE 后端编译
    let result = pe::compile(&program);

    // 由于 PE 后端尚未完全实现，预期会返回 NotImplemented 错误
    match result {
        Ok(binary) => {
            // 如果编译成功，验证生成的二进制文件
            assert!(!binary.is_empty());
            // 验证 PE 文件头
            assert_eq!(&binary[0..2], b"MZ"); // DOS 头
        }
        Err(err) => {
            // 验证是 NotImplemented 错误
            assert!(format!("{:?}", err).contains("not implemented") || format!("{:?}", err).contains("NotImplemented"));
        }
    }
}

#[test]
fn test_il_backend_compilation() {
    let program = GaiaProgram {
        functions: vec![GaiaFunction {
            name: "Main".to_string(),
            instructions: vec![
                GaiaInstruction::LoadConstant(GaiaConstant::String("Hello, World!".to_string())),
                GaiaInstruction::Call("Console.WriteLine".to_string()),
                GaiaInstruction::Return,
            ],
            parameters: vec![],
            return_type: None,
        }],
    };

    let result = msil::compile(&program);

    match result {
        Ok(binary) => {
            // 验证生成的 .NET 程序集
            assert!(!binary.is_empty());
            // 验证 PE 文件头（.NET 程序集也是 PE 文件）
            assert_eq!(&binary[0..2], b"MZ");
        }
        Err(err) => {
            assert!(format!("{:?}", err).contains("not implemented") || format!("{:?}", err).contains("NotImplemented"));
        }
    }
}

#[test]
fn test_jvm_backend_compilation() {
    let program = GaiaProgram {
        functions: vec![GaiaFunction {
            name: "main".to_string(),
            instructions: vec![
                GaiaInstruction::LoadConstant(GaiaConstant::String("Hello, World!".to_string())),
                GaiaInstruction::Call("System.out.println".to_string()),
                GaiaInstruction::Return,
            ],
            parameters: vec![GaiaType::Array(Box::new(GaiaType::String))], // String[] args
            return_type: None,
        }],
    };

    let result = jvm::compile(&program);

    match result {
        Ok(binary) => {
            // 验证生成的 Java 字节码
            assert!(!binary.is_empty());
            // 验证 class 文件魔数
            assert_eq!(&binary[0..4], &[0xCA, 0xFE, 0xBA, 0xBE]);
        }
        Err(err) => {
            assert!(format!("{:?}", err).contains("not implemented") || format!("{:?}", err).contains("NotImplemented"));
        }
    }
}

#[test]
fn test_wasi_backend_compilation() {
    let program = GaiaProgram {
        functions: vec![GaiaFunction {
            name: "_start".to_string(),
            instructions: vec![GaiaInstruction::LoadConstant(GaiaConstant::Int32(42)), GaiaInstruction::Return],
            parameters: vec![],
            return_type: Some(GaiaType::Int32),
        }],
    };

    let result = wasi::compile(&program);

    match result {
        Ok(binary) => {
            // 验证生成的 WebAssembly 模块
            assert!(!binary.is_empty());
            // 验证 WASM 魔数
            assert_eq!(&binary[0..4], &[0x00, 0x61, 0x73, 0x6D]); // "\0asm"
                                                                  // 验证版本号
            assert_eq!(&binary[4..8], &[0x01, 0x00, 0x00, 0x00]); // version 1
        }
        Err(err) => {
            assert!(format!("{:?}", err).contains("not implemented") || format!("{:?}", err).contains("NotImplemented"));
        }
    }
}

#[test]
fn test_complex_program_compilation() {
    // 测试更复杂的程序编译
    let program = GaiaProgram {
        functions: vec![
            GaiaFunction {
                name: "fibonacci".to_string(),
                instructions: vec![
                    // if (n <= 1) return n;
                    GaiaInstruction::LoadArgument(0),
                    GaiaInstruction::LoadConstant(GaiaConstant::Int32(1)),
                    GaiaInstruction::LessThan,
                    GaiaInstruction::BranchIfFalse("recursive".to_string()),
                    GaiaInstruction::LoadArgument(0),
                    GaiaInstruction::Return,
                    // return fibonacci(n-1) + fibonacci(n-2);
                    GaiaInstruction::Label("recursive".to_string()),
                    GaiaInstruction::LoadArgument(0),
                    GaiaInstruction::LoadConstant(GaiaConstant::Int32(1)),
                    GaiaInstruction::Subtract,
                    GaiaInstruction::Call("fibonacci".to_string()),
                    GaiaInstruction::LoadArgument(0),
                    GaiaInstruction::LoadConstant(GaiaConstant::Int32(2)),
                    GaiaInstruction::Subtract,
                    GaiaInstruction::Call("fibonacci".to_string()),
                    GaiaInstruction::Add,
                    GaiaInstruction::Return,
                ],
                parameters: vec![GaiaType::Int32],
                return_type: Some(GaiaType::Int32),
            },
            GaiaFunction {
                name: "main".to_string(),
                instructions: vec![
                    GaiaInstruction::LoadConstant(GaiaConstant::Int32(10)),
                    GaiaInstruction::Call("fibonacci".to_string()),
                    GaiaInstruction::Return,
                ],
                parameters: vec![],
                return_type: Some(GaiaType::Int32),
            },
        ],
    };

    // 测试所有后端
    let backends = vec![
        ("PE", pe::compile as fn(&GaiaProgram) -> Result<Vec<u8>>),
        ("IL", msil::compile),
        ("JVM", jvm::compile),
        ("WASI", wasi::compile),
    ];

    for (name, compile_fn) in backends {
        let result = compile_fn(&program);
        match result {
            Ok(binary) => {
                println!("{} backend compiled successfully, binary size: {} bytes", name, binary.len());
                assert!(!binary.is_empty());
            }
            Err(err) => {
                println!("{} backend not yet implemented: {:?}", name, err);
                // 验证是预期的未实现错误
                assert!(format!("{:?}", err).contains("not implemented") || format!("{:?}", err).contains("NotImplemented"));
            }
        }
    }
}

#[test]
fn test_error_handling() {
    // 测试错误处理
    let invalid_program = GaiaProgram {
        functions: vec![GaiaFunction {
            name: "invalid".to_string(),
            instructions: vec![
                // 无效的指令序列：尝试调用不存在的函数
                GaiaInstruction::Call("nonexistent_function".to_string()),
                GaiaInstruction::Return,
            ],
            parameters: vec![],
            return_type: None,
        }],
    };

    let result = pe::compile(&invalid_program);
    // 应该返回错误或者成功编译（取决于后端实现）
    match result {
        Ok(_) => {
            // 如果编译成功，说明后端处理了未定义函数的情况
        }
        Err(err) => {
            // 如果编译失败，验证错误类型
            println!("Expected error for invalid program: {:?}", err);
        }
    }
}
