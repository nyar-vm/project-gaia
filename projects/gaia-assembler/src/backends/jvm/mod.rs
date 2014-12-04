//! JVM (Java Virtual Machine) backend compiler

use crate::{
    backends::{Backend, GeneratedFiles},
    config::GaiaConfig,
    instruction::GaiaInstruction,
    program::{GaiaFunction, GaiaProgram},
    types::GaiaType,
};
use crate::adapters::FunctionMapper;
use gaia_types::{
    helpers::{AbiCompatible, ApiCompatible, Architecture, CompilationTarget},
    GaiaError, Result,
};
use jvm_assembler::{
    formats::{class::writer::ClassWriter, jasm::ast::to_jasm::JvmToJasmConverter},
    program::{JvmAccessFlags, JvmField, JvmInstruction, JvmMethod, JvmProgram, JvmVersion},
};
use std::collections::HashMap;
use crate::program::{GaiaConstant, GaiaGlobal};

/// JVM Backend implementation
#[derive(Default)]
pub struct JvmBackend {}

impl Backend for JvmBackend {
    fn name(&self) -> &'static str {
        "JVM"
    }

    fn primary_target(&self) -> CompilationTarget {
        CompilationTarget { build: Architecture::JVM, host: AbiCompatible::JavaAssembly, target: ApiCompatible::JvmRuntime(8) }
    }

    fn match_score(&self, target: &CompilationTarget) -> f32 {
        match target.build {
            Architecture::JVM => match target.host {
                // jasm output, 5% support
                AbiCompatible::JavaAssembly => 5.0,
                // bytecode output, 30% support
                AbiCompatible::Unknown => 30.0,
                _ => -100.0,
            },
            _ => -100.0,
        }
    }

    fn generate(&self, program: &GaiaProgram, config: &GaiaConfig) -> Result<GeneratedFiles> {
        let mut files = HashMap::new();

        // 将 GaiaProgram 转换为 JvmProgram（带配置与函数映射）
        let jvm_program = convert_gaia_to_jvm(program, config)?;

        match config.target.host {
            AbiCompatible::Unknown => {
                // 生成 .class 字节码文件
                let buffer = Vec::new();
                let mut class_writer = ClassWriter::new(buffer);
                let class_bytes = class_writer.write(&jvm_program).result?;
                files.insert("main.class".to_string(), class_bytes);
            }
            AbiCompatible::JavaAssembly => {
                // 生成 .jasm 汇编文件
                let mut converter = JvmToJasmConverter::new();
                let jasm_result = converter.convert(jvm_program);
                match jasm_result.result {
                    Ok(jasm_root) => {
                        let jasm_string = format!("{:#?}", jasm_root);
                        files.insert("main.jasm".to_string(), jasm_string.into_bytes());
                    }
                    Err(error) => return Err(error),
                }
            }
            _ => return Err(GaiaError::custom_error(&format!("Unsupported host ABI: {:?}", config.target.host))),
        }

        Ok(GeneratedFiles { files, diagnostics: vec![] })
    }
}

impl JvmBackend {
    /// Generate JVM program from Gaia program
    pub fn generate_program(program: &GaiaProgram) -> Result<JvmProgram> {
        // 使用默认配置生成（保持向后兼容）
        let default_config = GaiaConfig::default();
        convert_gaia_to_jvm(program, &default_config)
    }
}

/// JVM 编译上下文，携带函数映射与目标信息
struct JvmContext {
    function_mapper: FunctionMapper,
}

/// Convert GaiaProgram to JvmProgram
fn convert_gaia_to_jvm(program: &GaiaProgram, config: &GaiaConfig) -> Result<JvmProgram> {
    let mut jvm_program = JvmProgram::new(program.name.clone());

    // Set version information
    jvm_program.version = JvmVersion { major: 52, minor: 0 }; // Java 8

    // Set access flags
    jvm_program.access_flags = JvmAccessFlags::PUBLIC;

    // 构建上下文（从配置初始化函数映射）
    let ctx = JvmContext {
        function_mapper: FunctionMapper::from_config(&config.setting)?,
    };

    // Convert functions（带上下文）
    for function in &program.functions {
        let jvm_method = convert_gaia_function_to_jvm(function, &ctx)?;
        jvm_program.add_method(jvm_method);
    }

    // Convert global variables to fields
    if let Some(globals) = &program.globals {
        for global in globals {
            let jvm_field = convert_gaia_global_to_jvm_field(global)?;
            jvm_program.add_field(jvm_field);
        }
    }

    Ok(jvm_program)
}

/// Convert GaiaFunction to JvmMethod
fn convert_gaia_function_to_jvm(function: &GaiaFunction, ctx: &JvmContext) -> Result<JvmMethod> {
    // 构建方法描述符
    let descriptor = build_method_descriptor(&function.parameters, &function.return_type);

    let mut method = JvmMethod::new(function.name.clone(), descriptor);

    // 设置访问标志
    method = method.with_public().with_static();

    // 转换指令（可能产生多条 JVM 指令）
    for instruction in &function.instructions {
        let jvm_instructions = convert_gaia_instruction_to_jvm(instruction, ctx)?;
        for ji in jvm_instructions {
            method = method.with_instruction(ji);
        }
    }

    // 设置栈和局部变量大小（简化处理）
    method = method.with_max_stack(10).with_max_locals(10);

    Ok(method)
}

/// Convert GaiaGlobal to JvmField
fn convert_gaia_global_to_jvm_field(global: &GaiaGlobal) -> Result<JvmField> {
    let descriptor = convert_gaia_type_to_jvm_descriptor(&global.var_type);
    let field = JvmField::new(global.name.clone(), descriptor).with_public().with_static();

    Ok(field)
}

/// Convert GaiaInstruction to JvmInstruction
fn convert_gaia_instruction_to_jvm(instruction: &GaiaInstruction, ctx: &JvmContext) -> Result<Vec<JvmInstruction>> {
    match instruction {
        GaiaInstruction::LoadConstant(constant) => match constant {
            GaiaConstant::Integer64(value) => match *value {
                0 => Ok(vec![JvmInstruction::Iconst0]),
                1 => Ok(vec![JvmInstruction::Iconst1]),
                2 => Ok(vec![JvmInstruction::Iconst2]),
                3 => Ok(vec![JvmInstruction::Iconst3]),
                4 => Ok(vec![JvmInstruction::Iconst4]),
                5 => Ok(vec![JvmInstruction::Iconst5]),
                -1 => Ok(vec![JvmInstruction::IconstM1]),
                _ if *value >= -128 && *value <= 127 => Ok(vec![JvmInstruction::Bipush { value: *value as i8 }]),
                _ if *value >= -32768 && *value <= 32767 => Ok(vec![JvmInstruction::Sipush { value: *value as i16 }]),
                _ => Ok(vec![JvmInstruction::Ldc { symbol: value.to_string() }]),
            },
            GaiaConstant::Float64(value) => match *value {
                0.0 => Ok(vec![JvmInstruction::Fconst0]),
                1.0 => Ok(vec![JvmInstruction::Fconst1]),
                2.0 => Ok(vec![JvmInstruction::Fconst2]),
                _ => Ok(vec![JvmInstruction::Ldc { symbol: value.to_string() }]),
            },
            GaiaConstant::String(value) => Ok(vec![JvmInstruction::Ldc { symbol: value.clone() }]),
            _ => Err(GaiaError::custom_error("Unsupported constant type for JVM")),
        },
        GaiaInstruction::LoadLocal(index) => match *index {
            0 => Ok(vec![JvmInstruction::Iload0]),
            1 => Ok(vec![JvmInstruction::Iload1]),
            2 => Ok(vec![JvmInstruction::Iload2]),
            3 => Ok(vec![JvmInstruction::Iload3]),
            _ => Ok(vec![JvmInstruction::Iload { index: *index as u16 }]),
        },
        GaiaInstruction::StoreLocal(index) => match *index {
            0 => Ok(vec![JvmInstruction::Istore0]),
            1 => Ok(vec![JvmInstruction::Istore1]),
            2 => Ok(vec![JvmInstruction::Istore2]),
            3 => Ok(vec![JvmInstruction::Istore3]),
            _ => Ok(vec![JvmInstruction::Istore { index: *index as u16 }]),
        },
        // 当前 GaiaInstruction 无独立 LoadArgument 变体；参数作为局部变量处理
        GaiaInstruction::Add => Ok(vec![JvmInstruction::Iadd]),
        GaiaInstruction::Subtract => Ok(vec![JvmInstruction::Isub]),
        GaiaInstruction::Multiply => Ok(vec![JvmInstruction::Imul]),
        GaiaInstruction::Divide => Ok(vec![JvmInstruction::Idiv]),
        GaiaInstruction::Remainder => Ok(vec![JvmInstruction::Irem]),
        GaiaInstruction::BitwiseAnd => Ok(vec![JvmInstruction::Iand]),
        GaiaInstruction::BitwiseOr => Ok(vec![JvmInstruction::Ior]),
        GaiaInstruction::BitwiseXor => Ok(vec![JvmInstruction::Ixor]),
        GaiaInstruction::BitwiseNot => {
            // JVM 没有直接的按位取反指令，使用 -1 异或实现
            Err(GaiaError::custom_error("BitwiseNot not directly supported in JVM"))
        }
        GaiaInstruction::ShiftLeft => Ok(vec![JvmInstruction::Ishl]),
        GaiaInstruction::ShiftRight => Ok(vec![JvmInstruction::Ishr]),
        GaiaInstruction::Equal => {
            // JVM 需要使用条件跳转指令实现比较
            Err(GaiaError::custom_error("Equal comparison requires conditional branching in JVM"))
        }
        GaiaInstruction::NotEqual => Err(GaiaError::custom_error("NotEqual comparison requires conditional branching in JVM")),
        GaiaInstruction::LessThan => Err(GaiaError::custom_error("LessThan comparison requires conditional branching in JVM")),
        GaiaInstruction::GreaterThan => {
            Err(GaiaError::custom_error("GreaterThan comparison requires conditional branching in JVM"))
        }
        GaiaInstruction::LessThanOrEqual => {
            Err(GaiaError::custom_error("LessThanOrEqual comparison requires conditional branching in JVM"))
        }
        GaiaInstruction::GreaterThanOrEqual => {
            Err(GaiaError::custom_error("GreaterThanOrEqual comparison requires conditional branching in JVM"))
        }
        GaiaInstruction::Jump(label) => Ok(vec![JvmInstruction::Goto { target: label.clone() }]),
        GaiaInstruction::JumpIfTrue(label) => Ok(vec![JvmInstruction::Ifne { target: label.clone() }]),
        GaiaInstruction::JumpIfFalse(label) => Ok(vec![JvmInstruction::Ifeq { target: label.clone() }]),
        GaiaInstruction::Call(function_name, _arg_count) => {
            // 使用 FunctionMapper 进行统一函数名解析
            let jvm_target = CompilationTarget { build: Architecture::JVM, host: AbiCompatible::JavaAssembly, target: ApiCompatible::JvmRuntime(8) };
            let mapped = ctx
                .function_mapper
                .map_function(&jvm_target, function_name)
                .unwrap_or(function_name.as_str())
                .to_string();

            // 特判常见的 println 映射：System.out.println
            if mapped == "java.lang.System.out.println" {
                return Ok(vec![
                    JvmInstruction::Getstatic { class_name: "java/lang/System".to_string(), field_name: "out".to_string(), descriptor: "Ljava/io/PrintStream;".to_string() },
                    JvmInstruction::Invokevirtual { class_name: "java/io/PrintStream".to_string(), method_name: "println".to_string(), descriptor: "()V".to_string() },
                ]);
            }

            // 其他情况保留原先的静态方法调用约定（假定由前端或运行时提供）
            Ok(vec![JvmInstruction::Invokestatic {
                class_name: "Main".to_string(),
                method_name: function_name.clone(),
                descriptor: "()V".to_string(),
            }])
        }
        GaiaInstruction::Return => Ok(vec![JvmInstruction::Return]),
        GaiaInstruction::Label(_name) => {
            // JVM 指令中标签不是独立的指令，需要在汇编时处理
            Err(GaiaError::custom_error("Labels are handled during assembly, not as instructions"))
        }
        GaiaInstruction::Duplicate => Ok(vec![JvmInstruction::Dup]),
        GaiaInstruction::Pop => Ok(vec![JvmInstruction::Pop]),
        _ => Err(GaiaError::custom_error(&format!("Unsupported instruction for JVM: {:?}", instruction))),
    }
}

/// Build JVM method descriptor from parameters and return type
fn build_method_descriptor(parameters: &[GaiaType], return_type: &Option<GaiaType>) -> String {
    let mut descriptor = String::from("(");

    // 添加参数类型
    for param in parameters {
        descriptor.push_str(&convert_gaia_type_to_jvm_descriptor(param));
    }

    descriptor.push(')');

    // 添加返回类型
    match return_type {
        Some(ret_type) => descriptor.push_str(&convert_gaia_type_to_jvm_descriptor(ret_type)),
        None => descriptor.push('V'), // void
    }

    descriptor
}

/// Convert GaiaType to JVM type descriptor
fn convert_gaia_type_to_jvm_descriptor(gaia_type: &GaiaType) -> String {
    match gaia_type {
        GaiaType::Integer => "I".to_string(),
        GaiaType::Float => "F".to_string(),
        GaiaType::Double => "D".to_string(),
        GaiaType::Boolean => "Z".to_string(),
        GaiaType::String => "Ljava/lang/String;".to_string(),
        GaiaType::Void => "V".to_string(),
        _ => "Ljava/lang/Object;".to_string(), // 默认为 Object
    }
}
