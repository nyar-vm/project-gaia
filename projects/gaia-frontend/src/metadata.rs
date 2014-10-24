use crate::exports::types::*;

/// 获取程序元数据
pub fn get_program_metadata(descriptor: GaiaDescriptor) -> ProgramMetadata {
    // 从描述符中提取程序信息
    let name = descriptor.program_metadata.name.clone();

    ProgramMetadata {
        name,
        version: descriptor.program_metadata.version.clone(),
        author: descriptor.program_metadata.author.clone(),
        description: descriptor.program_metadata.description.clone(),
        entry_point: find_entry_point_from_symbols(&descriptor.symbols),
        dependencies: descriptor.program_metadata.dependencies.clone(),
    }
}

/// 获取符号表信息
pub fn get_symbols(descriptor: GaiaDescriptor) -> Vec<SymbolInfo> {
    descriptor.symbols
}

/// 获取指令文档
pub fn get_instruction_docs(instruction: Instruction) -> Option<InstructionMetadata> {
    match instruction {
        Instruction::Arithmetic(arith) => Some(InstructionMetadata {
            instruction_type: InstructionType::Arithmetic,
            operand_types: vec![DataType::I32, DataType::I32],
            description: format!("算术运算: {:?}", arith),
            supported_targets: vec![Target::Clr, Target::Jvm, Target::Pe, Target::Wasi],
        }),
        Instruction::Memory(mem) => Some(InstructionMetadata {
            instruction_type: InstructionType::Load,
            operand_types: vec![DataType::I32],
            description: format!("内存操作: {:?}", mem),
            supported_targets: vec![Target::Clr, Target::Jvm, Target::Pe, Target::Wasi],
        }),
        Instruction::ControlFlow(cf) => Some(InstructionMetadata {
            instruction_type: InstructionType::Branch,
            operand_types: vec![DataType::I32],
            description: format!("控制流: {:?}", cf),
            supported_targets: vec![Target::Clr, Target::Jvm, Target::Pe, Target::Wasi],
        }),
        Instruction::Comparison(comp) => Some(InstructionMetadata {
            instruction_type: InstructionType::Compare,
            operand_types: vec![DataType::I32, DataType::I32],
            description: format!("比较操作: {:?}", comp),
            supported_targets: vec![Target::Clr, Target::Jvm, Target::Pe, Target::Wasi],
        }),
        Instruction::Conversion(conv) => Some(InstructionMetadata {
            instruction_type: InstructionType::Convert,
            operand_types: vec![DataType::I32],
            description: format!("类型转换: {:?}", conv),
            supported_targets: vec![Target::Clr, Target::Jvm, Target::Pe, Target::Wasi],
        }),
        Instruction::Call(call) => Some(InstructionMetadata {
            instruction_type: InstructionType::Call,
            operand_types: vec![DataType::Void],
            description: format!("函数调用: {:?}", call),
            supported_targets: vec![Target::Clr, Target::Jvm, Target::Pe, Target::Wasi],
        }),
        Instruction::System(sys) => Some(InstructionMetadata {
            instruction_type: InstructionType::System,
            operand_types: vec![DataType::I32],
            description: format!("系统调用: {:?}", sys),
            supported_targets: vec![Target::Clr, Target::Jvm, Target::Pe, Target::Wasi],
        }),
    }
}

/// 分析程序复杂度
pub fn analyze_complexity(descriptor: GaiaDescriptor) -> ComplexityAnalysis {
    let instruction_count = descriptor.instructions.len() as u32;
    let symbol_count = descriptor.symbols.len() as u32;
    let control_flow_complexity = calculate_control_flow_complexity(&descriptor.control_flow);

    ComplexityAnalysis {
        cyclomatic_complexity: control_flow_complexity,
        instruction_count,
        symbol_count,
        max_nesting_depth: calculate_max_nesting_depth(&descriptor.instructions),
        estimated_execution_time: instruction_count * 10, // 简化估算
    }
}

/// 获取依赖关系图
pub fn get_dependencies(descriptor: GaiaDescriptor) -> DependencyGraph {
    let mut nodes = vec![];
    let mut edges = vec![];

    // 为每个符号创建节点
    for (index, symbol) in descriptor.symbols.iter().enumerate() {
        nodes.push(DependencyNode {
            id: format!("symbol_{}", index),
            name: symbol.name.clone(),
            node_type: match symbol.symbol_type {
                SymbolType::Function => DependencyNodeType::Function,
                SymbolType::Variable => DependencyNodeType::Variable,
                SymbolType::Label => DependencyNodeType::Label,
                SymbolType::Constant => DependencyNodeType::Constant,
            },
        });
    }

    // 分析指令中的依赖关系
    for (index, instruction_entry) in descriptor.instructions.iter().enumerate() {
        if let Some(symbol_name) = extract_symbol_reference(&instruction_entry.instruction) {
            if let Some(symbol_index) = descriptor.symbols.iter().position(|s| s.name == symbol_name) {
                edges.push(DependencyEdge {
                    from: format!("instruction_{}", index),
                    to: format!("symbol_{}", symbol_index),
                    dependency_type: DependencyType::Uses,
                });
            }
        }
    }

    DependencyGraph { nodes, edges }
}

/// 获取平台信息
pub fn get_platform_info(target: Target) -> PlatformInfo {
    match target {
        Target::Clr => PlatformInfo {
            name: ".NET Common Language Runtime".to_string(),
            version: "8.0".to_string(),
            architecture: "x64".to_string(),
            pointer_size: 64,
            supported_features: vec![
                "garbage_collection".to_string(),
                "jit_compilation".to_string(),
                "managed_memory".to_string(),
            ],
        },
        Target::Jvm => PlatformInfo {
            name: "Java Virtual Machine".to_string(),
            version: "21".to_string(),
            architecture: "x64".to_string(),
            pointer_size: 64,
            supported_features: vec![
                "garbage_collection".to_string(),
                "bytecode_verification".to_string(),
                "cross_platform".to_string(),
            ],
        },
        Target::Pe => PlatformInfo {
            name: "Windows Portable Executable".to_string(),
            version: "11".to_string(),
            architecture: "x64".to_string(),
            pointer_size: 64,
            supported_features: vec!["native_execution".to_string(), "dll_loading".to_string(), "windows_api".to_string()],
        },
        Target::Wasi => PlatformInfo {
            name: "WebAssembly System Interface".to_string(),
            version: "0.2".to_string(),
            architecture: "wasm32".to_string(),
            pointer_size: 32,
            supported_features: vec![
                "sandboxed_execution".to_string(),
                "capability_security".to_string(),
                "portable_bytecode".to_string(),
            ],
        },
    }
}

fn find_entry_point_from_symbols(symbols: &[SymbolInfo]) -> Option<String> {
    // 查找入口点函数
    for symbol in symbols {
        if symbol.symbol_type == SymbolType::Function {
            if symbol.name == "main" || symbol.name == "start" {
                return Some(symbol.name.clone());
            }
        }
    }
    None
}

fn calculate_control_flow_complexity(cfg: &ControlFlowGraph) -> u32 {
    // 简化的圈复杂度计算
    let edge_count = cfg.edges.len() as u32;
    let node_count = cfg.nodes.len() as u32;
    if node_count > 0 {
        edge_count - node_count + 2
    }
    else {
        1
    }
}

fn calculate_max_nesting_depth(instructions: &[InstructionEntry]) -> u32 {
    let mut max_depth = 0;
    let mut current_depth = 0;

    for instruction_entry in instructions {
        match &instruction_entry.instruction {
            Instruction::ControlFlow(cf) => {
                // 简化的嵌套深度计算
                current_depth += 1;
                max_depth = max_depth.max(current_depth);
            }
            _ => {}
        }
    }

    max_depth
}

fn extract_symbol_reference(instruction: &Instruction) -> Option<String> {
    // 简化的符号引用提取
    match instruction {
        Instruction::Call(call_op) => {
            // 假设调用操作数包含函数名
            Some(format!("function_call_{:?}", call_op))
        }
        _ => None,
    }
}
