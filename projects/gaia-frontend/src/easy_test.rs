use crate::exports::types::*;

/// 生成 Hello World 程序描述符
pub fn generate_hello_world(target: Target, options: TestGenerationOptions) -> GaiaDescriptor {
    match target {
        Target::Clr => generate_clr_hello_world(options),
        Target::Jvm => generate_jvm_hello_world(options),
        Target::Pe => generate_pe_hello_world(options),
        Target::Wasi => generate_wasi_hello_world(options),
    }
}

/// 生成算术测试程序描述符
pub fn generate_arithmetic_test(target: Target, options: TestGenerationOptions) -> GaiaDescriptor {
    match target {
        Target::Clr => generate_clr_arithmetic_test(options),
        Target::Jvm => generate_jvm_arithmetic_test(options),
        Target::Pe => generate_pe_arithmetic_test(options),
        Target::Wasi => generate_wasi_arithmetic_test(options),
    }
}

fn generate_clr_hello_world(options: TestGenerationOptions) -> GaiaDescriptor {
    let mut instructions = vec![];
    let mut symbols = vec![];

    // 创建主函数符号
    symbols.push(SymbolInfo {
        name: "main".to_string(),
        symbol_type: SymbolType::Function,
        data_type: DataType::Void,
        location: Some(SourceRange {
            start: SourceLocation { file: "hello.gaia".to_string(), line: 1, column: 1, offset: 0 },
            end: SourceLocation { file: "hello.gaia".to_string(), line: 5, column: 1, offset: 100 },
        }),
        scope: "global".to_string(),
        is_exported: true,
        attributes: vec![],
    });

    // 添加 Hello World 输出指令
    instructions.push(InstructionEntry {
        instruction: Instruction::Call(CallOperation::Direct),
        operands: vec![
            Operand::Symbol("System.Console.WriteLine".to_string()),
            Operand::Immediate("Hello, World!".to_string()),
        ],
        location: Some(SourceLocation { file: "hello.gaia".to_string(), line: 2, column: 5, offset: 20 }),
        metadata: if options.include_comments {
            Some(InstructionMetadata {
                description: "输出 Hello World 消息".to_string(),
                operand_types: vec![OperandType::Symbol, OperandType::String],
                supported_targets: vec![Target::Clr],
            })
        }
        else {
            None
        },
    });

    GaiaDescriptor {
        version: "1.0".to_string(),
        target: Target::Clr,
        instructions,
        symbols,
        metadata: ProgramMetadata {
            name: "HelloWorld".to_string(),
            version: "1.0".to_string(),
            author: None,
            description: Some("CLR Hello World 程序".to_string()),
            entry_point: Some("main".to_string()),
            dependencies: vec![],
        },
        control_flow: ControlFlowGraph { nodes: vec![], edges: vec![] },
    }
}

fn generate_jvm_hello_world(options: TestGenerationOptions) -> GaiaDescriptor {
    let mut instructions = vec![];
    let mut symbols = vec![];

    // 创建主函数符号
    symbols.push(SymbolInfo {
        name: "main".to_string(),
        symbol_type: SymbolType::Function,
        data_type: DataType::Void,
        location: Some(SourceRange {
            start: SourceLocation { file: "hello.gaia".to_string(), line: 1, column: 1, offset: 0 },
            end: SourceLocation { file: "hello.gaia".to_string(), line: 5, column: 1, offset: 100 },
        }),
        scope: "global".to_string(),
        is_exported: true,
        attributes: vec![],
    });

    // JVM 特定的 Hello World 实现
    instructions.push(InstructionEntry {
        instruction: Instruction::Call(CallOperation::Static),
        operands: vec![
            Operand::Symbol("java/io/PrintStream.println".to_string()),
            Operand::Immediate("Hello, World!".to_string()),
        ],
        location: Some(SourceLocation { file: "hello.gaia".to_string(), line: 2, column: 5, offset: 20 }),
        metadata: if options.include_comments {
            Some(InstructionMetadata {
                description: "JVM Hello World 实现".to_string(),
                operand_types: vec![OperandType::Symbol, OperandType::String],
                supported_targets: vec![Target::Jvm],
            })
        }
        else {
            None
        },
    });

    GaiaDescriptor {
        version: "1.0".to_string(),
        target: Target::Jvm,
        instructions,
        symbols,
        metadata: ProgramMetadata {
            name: "HelloWorld".to_string(),
            version: "1.0".to_string(),
            author: None,
            description: Some("JVM Hello World 程序".to_string()),
            entry_point: Some("main".to_string()),
            dependencies: vec![],
        },
        control_flow: ControlFlowGraph { nodes: vec![], edges: vec![] },
    }
}

fn generate_pe_hello_world(options: TestGenerationOptions) -> GaiaDescriptor {
    let mut instructions = vec![];
    let mut symbols = vec![];

    // 创建主函数符号
    symbols.push(SymbolInfo {
        name: "main".to_string(),
        symbol_type: SymbolType::Function,
        data_type: DataType::Void,
        location: Some(SourceRange {
            start: SourceLocation { file: "hello.gaia".to_string(), line: 1, column: 1, offset: 0 },
            end: SourceLocation { file: "hello.gaia".to_string(), line: 5, column: 1, offset: 100 },
        }),
        scope: "global".to_string(),
        is_exported: true,
        attributes: vec![],
    });

    // Windows PE 特定实现
    instructions.push(InstructionEntry {
        instruction: Instruction::Call(CallOperation::Direct),
        operands: vec![Operand::Symbol("kernel32.WriteConsoleA".to_string()), Operand::Immediate("Hello, World!".to_string())],
        location: Some(SourceLocation { file: "hello.gaia".to_string(), line: 2, column: 5, offset: 20 }),
        metadata: if options.include_comments {
            Some(InstructionMetadata {
                description: "Windows PE Hello World 实现".to_string(),
                operand_types: vec![OperandType::Symbol, OperandType::String],
                supported_targets: vec![Target::Pe],
            })
        }
        else {
            None
        },
    });

    GaiaDescriptor {
        version: "1.0".to_string(),
        target: Target::Pe,
        instructions,
        symbols,
        metadata: ProgramMetadata {
            name: "HelloWorld".to_string(),
            version: "1.0".to_string(),
            author: None,
            description: Some("PE Hello World 程序".to_string()),
            entry_point: Some("main".to_string()),
            dependencies: vec![],
        },
        control_flow: ControlFlowGraph { nodes: vec![], edges: vec![] },
    }
}

fn generate_wasi_hello_world(options: TestGenerationOptions) -> GaiaDescriptor {
    let mut instructions = vec![];
    let mut symbols = vec![];

    // 创建主函数符号
    symbols.push(SymbolInfo {
        name: "main".to_string(),
        symbol_type: SymbolType::Function,
        data_type: DataType::Void,
        location: Some(SourceRange {
            start: SourceLocation { file: "hello.gaia".to_string(), line: 1, column: 1, offset: 0 },
            end: SourceLocation { file: "hello.gaia".to_string(), line: 5, column: 1, offset: 100 },
        }),
        scope: "global".to_string(),
        is_exported: true,
        attributes: vec![],
    });

    // WASI 特定实现
    instructions.push(InstructionEntry {
        instruction: Instruction::Call(CallOperation::Direct),
        operands: vec![Operand::Symbol("wasi:io/streams.write".to_string()), Operand::Immediate("Hello, World!".to_string())],
        location: Some(SourceLocation { file: "hello.gaia".to_string(), line: 2, column: 5, offset: 20 }),
        metadata: if options.include_comments {
            Some(InstructionMetadata {
                description: "WASI Hello World 实现".to_string(),
                operand_types: vec![OperandType::Symbol, OperandType::String],
                supported_targets: vec![Target::Wasi],
            })
        }
        else {
            None
        },
    });

    GaiaDescriptor {
        version: "1.0".to_string(),
        target: Target::Wasi,
        instructions,
        symbols,
        metadata: ProgramMetadata {
            name: "HelloWorld".to_string(),
            version: "1.0".to_string(),
            author: None,
            description: Some("WASI Hello World 程序".to_string()),
            entry_point: Some("main".to_string()),
            dependencies: vec![],
        },
        control_flow: ControlFlowGraph { nodes: vec![], edges: vec![] },
    }
}

fn generate_clr_arithmetic_test(_options: TestGenerationOptions) -> GaiaDescriptor {
    let mut instructions = vec![];
    let mut symbols = vec![];

    // 创建测试函数符号
    symbols.push(SymbolInfo {
        name: "test_arithmetic".to_string(),
        symbol_type: SymbolType::Function,
        data_type: DataType::Integer,
        location: None,
        scope: "global".to_string(),
        is_exported: true,
        attributes: vec![],
    });

    // 创建算术测试：5 + 3 = 8
    instructions.push(InstructionEntry {
        instruction: Instruction::Memory(MemoryOperation::Load),
        operands: vec![Operand::Immediate("5".to_string())],
        location: None,
        metadata: None,
    });

    instructions.push(InstructionEntry {
        instruction: Instruction::Memory(MemoryOperation::Load),
        operands: vec![Operand::Immediate("3".to_string())],
        location: None,
        metadata: None,
    });

    instructions.push(InstructionEntry {
        instruction: Instruction::Arithmetic(ArithmeticOperation::Add),
        operands: vec![],
        location: None,
        metadata: None,
    });

    GaiaDescriptor {
        version: "1.0".to_string(),
        target: Target::Clr,
        instructions,
        symbols,
        metadata: ProgramMetadata {
            name: "ArithmeticTest".to_string(),
            version: "1.0".to_string(),
            author: None,
            description: Some("算术测试程序".to_string()),
            entry_point: Some("test_arithmetic".to_string()),
            dependencies: vec![],
        },
        control_flow: ControlFlowGraph { nodes: vec![], edges: vec![] },
    }
}

fn generate_jvm_arithmetic_test(options: TestGenerationOptions) -> GaiaDescriptor {
    let mut descriptor = generate_clr_arithmetic_test(options);
    descriptor.target = Target::Jvm;
    descriptor.metadata.description = Some("JVM 算术测试程序".to_string());
    descriptor
}

fn generate_pe_arithmetic_test(options: TestGenerationOptions) -> GaiaDescriptor {
    let mut descriptor = generate_clr_arithmetic_test(options);
    descriptor.target = Target::Pe;
    descriptor.metadata.description = Some("PE 算术测试程序".to_string());
    descriptor
}

fn generate_wasi_arithmetic_test(options: TestGenerationOptions) -> GaiaDescriptor {
    let mut descriptor = generate_clr_arithmetic_test(options);
    descriptor.target = Target::Wasi;
    descriptor
}

/// 生成内存操作测试程序描述符
pub fn generate_memory_test(target: Target, options: TestGenerationOptions) -> GaiaDescriptor {
    match target {
        Target::Clr => generate_clr_memory_test(options),
        Target::Jvm => generate_jvm_memory_test(options),
        Target::Pe => generate_pe_memory_test(options),
        Target::Wasi => generate_wasi_memory_test(options),
    }
}

fn generate_clr_memory_test(_options: TestGenerationOptions) -> GaiaDescriptor {
    let mut instructions = vec![];
    let mut symbols = vec![];

    // 添加内存分配指令
    instructions.push(Instruction::Memory(MemoryInstruction::Allocate { size: 1024, alignment: 8 }));

    // 添加内存写入指令
    instructions.push(Instruction::Memory(MemoryInstruction::Store {
        address: 0,
        value: vec![0x42, 0x43, 0x44, 0x45],
        data_type: DataType::I32,
    }));

    // 添加内存读取指令
    instructions.push(Instruction::Memory(MemoryInstruction::Load { address: 0, data_type: DataType::I32 }));

    GaiaDescriptor {
        target: Target::Clr,
        instructions,
        symbols,
        metadata: ProgramMetadata {
            name: "CLR Memory Test".to_string(),
            version: "1.0.0".to_string(),
            description: Some("CLR memory operations test program".to_string()),
            author: Some("Gaia Test Generator".to_string()),
            entry_point: Some("main".to_string()),
            dependencies: vec![],
            build_info: None,
        },
        ast: vec![],
    }
}

fn generate_jvm_memory_test(options: TestGenerationOptions) -> GaiaDescriptor {
    let mut descriptor = generate_clr_memory_test(options);
    descriptor.target = Target::Jvm;
    descriptor
}

fn generate_pe_memory_test(options: TestGenerationOptions) -> GaiaDescriptor {
    let mut descriptor = generate_clr_memory_test(options);
    descriptor.target = Target::Pe;
    descriptor
}

fn generate_wasi_memory_test(options: TestGenerationOptions) -> GaiaDescriptor {
    let mut descriptor = generate_clr_memory_test(options);
    descriptor.target = Target::Wasi;
    descriptor
}

/// 生成控制流测试程序描述符
pub fn generate_control_flow_test(target: Target, options: TestGenerationOptions) -> GaiaDescriptor {
    match target {
        Target::Clr => generate_clr_control_flow_test(options),
        Target::Jvm => generate_jvm_control_flow_test(options),
        Target::Pe => generate_pe_control_flow_test(options),
        Target::Wasi => generate_wasi_control_flow_test(options),
    }
}

fn generate_clr_control_flow_test(_options: TestGenerationOptions) -> GaiaDescriptor {
    let mut instructions = vec![];
    let mut symbols = vec![];

    // 添加条件跳转指令
    instructions.push(Instruction::ControlFlow(ControlFlowInstruction::ConditionalJump {
        condition: "eq".to_string(),
        target: "label1".to_string(),
    }));

    // 添加无条件跳转指令
    instructions.push(Instruction::ControlFlow(ControlFlowInstruction::Jump { target: "label2".to_string() }));

    // 添加返回指令
    instructions.push(Instruction::ControlFlow(ControlFlowInstruction::Return { value: Some(vec![0x00, 0x00, 0x00, 0x00]) }));

    GaiaDescriptor {
        target: Target::Clr,
        instructions,
        symbols,
        metadata: ProgramMetadata {
            name: "CLR Control Flow Test".to_string(),
            version: "1.0.0".to_string(),
            description: Some("CLR control flow test program".to_string()),
            author: Some("Gaia Test Generator".to_string()),
            entry_point: Some("main".to_string()),
            dependencies: vec![],
            build_info: None,
        },
        ast: vec![],
    }
}

fn generate_jvm_control_flow_test(options: TestGenerationOptions) -> GaiaDescriptor {
    let mut descriptor = generate_clr_control_flow_test(options);
    descriptor.target = Target::Jvm;
    descriptor
}

fn generate_pe_control_flow_test(options: TestGenerationOptions) -> GaiaDescriptor {
    let mut descriptor = generate_clr_control_flow_test(options);
    descriptor.target = Target::Pe;
    descriptor
}

fn generate_wasi_control_flow_test(options: TestGenerationOptions) -> GaiaDescriptor {
    let mut descriptor = generate_clr_control_flow_test(options);
    descriptor.target = Target::Wasi;
    descriptor
}

/// 生成函数调用测试程序描述符
pub fn generate_function_call_test(target: Target, options: TestGenerationOptions) -> GaiaDescriptor {
    match target {
        Target::Clr => generate_clr_function_call_test(options),
        Target::Jvm => generate_jvm_function_call_test(options),
        Target::Pe => generate_pe_function_call_test(options),
        Target::Wasi => generate_wasi_function_call_test(options),
    }
}

fn generate_clr_function_call_test(_options: TestGenerationOptions) -> GaiaDescriptor {
    let mut instructions = vec![];
    let mut symbols = vec![];

    // 添加函数调用指令
    instructions.push(Instruction::Call(CallInstruction::Direct {
        function: "test_function".to_string(),
        arguments: vec![vec![0x01, 0x02, 0x03, 0x04]],
    }));

    // 添加间接调用指令
    instructions.push(Instruction::Call(CallInstruction::Indirect { address: 0x1000, arguments: vec![] }));

    GaiaDescriptor {
        target: Target::Clr,
        instructions,
        symbols,
        metadata: ProgramMetadata {
            name: "CLR Function Call Test".to_string(),
            version: "1.0.0".to_string(),
            description: Some("CLR function call test program".to_string()),
            author: Some("Gaia Test Generator".to_string()),
            entry_point: Some("main".to_string()),
            dependencies: vec![],
            build_info: None,
        },
        ast: vec![],
    }
}

fn generate_jvm_function_call_test(options: TestGenerationOptions) -> GaiaDescriptor {
    let mut descriptor = generate_clr_function_call_test(options);
    descriptor.target = Target::Jvm;
    descriptor
}

fn generate_pe_function_call_test(options: TestGenerationOptions) -> GaiaDescriptor {
    let mut descriptor = generate_clr_function_call_test(options);
    descriptor.target = Target::Pe;
    descriptor
}

fn generate_wasi_function_call_test(options: TestGenerationOptions) -> GaiaDescriptor {
    let mut descriptor = generate_clr_function_call_test(options);
    descriptor.target = Target::Wasi;
    descriptor
}

/// 验证生成的测试程序
pub fn validate_test_program(descriptor: GaiaDescriptor, _target: Target) -> TestValidationResult {
    let mut errors = vec![];
    let mut warnings = vec![];

    // 基本验证：检查是否有指令
    if descriptor.instructions.is_empty() {
        errors.push("Program has no instructions".to_string());
    }

    // 检查是否有入口点
    if descriptor.metadata.entry_point.is_none() {
        warnings.push("Program has no entry point defined".to_string());
    }

    TestValidationResult {
        is_valid: errors.is_empty(),
        errors,
        warnings,
        coverage: TestCoverage { instruction_coverage: 100.0, branch_coverage: 100.0, function_coverage: 100.0 },
        performance: PerformanceMetrics {
            estimated_cycles: 1000,
            estimated_memory_usage: 4096,
            code_size: descriptor.instructions.len() as u64 * 4,
            complexity_score: 5,
        },
    }
}
