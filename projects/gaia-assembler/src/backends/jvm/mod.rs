//! JVM (Java Virtual Machine) backend compiler

use crate::{
    adapters::FunctionMapper,
    backends::{Backend, GeneratedFiles},
    config::GaiaConfig,
    instruction::{CoreInstruction, GaiaInstruction},
    program::{GaiaConstant, GaiaFunction, GaiaGlobal, GaiaModule},
    types::GaiaType,
};
use gaia_types::{
    helpers::{AbiCompatible, ApiCompatible, Architecture, CompilationTarget},
    GaiaError, Result,
};
use jvm_assembler::{
    formats::{class::writer::ClassWriter /* , jasm::ast::to_jasm::JvmToJasmConverter */},
    program::{JvmAccessFlags, JvmField, JvmInstruction, JvmMethod, JvmProgram, JvmVersion},
};
use std::collections::HashMap;

/// JVM Backend implementation
#[derive(Default)]
pub struct JvmBackend {}

impl Backend for JvmBackend {
    fn name(&self) -> &'static str {
        "JVM"
    }

    fn primary_target(&self) -> CompilationTarget {
        CompilationTarget { build: Architecture::JVM, host: AbiCompatible::Unknown, target: ApiCompatible::JvmRuntime(8) }
    }

    fn match_score(&self, target: &CompilationTarget) -> f32 {
        match target.build {
            Architecture::JVM => match target.host {
                // bytecode output, 80% support (primary)
                AbiCompatible::Unknown => 80.0,
                // jasm output, 5% support (disabled)
                AbiCompatible::JavaAssembly => 5.0,
                _ => -100.0,
            },
            _ => -100.0,
        }
    }

    fn generate(&self, program: &GaiaModule, config: &GaiaConfig) -> Result<GeneratedFiles> {
        let mut files = HashMap::new();

        // 将 GaiaModule 转换为 JvmProgram（带配置与函数映射）
        let jvm_program = convert_gaia_to_jvm(program, config)?;

        match config.target.host {
            AbiCompatible::Unknown => {
                // 生成 .class 字节码文件
                let buffer = Vec::new();
                let class_writer = ClassWriter::new(buffer);
                let class_bytes = class_writer.write(&jvm_program).result?;
                files.insert("main.class".to_string(), class_bytes);
            }
            AbiCompatible::JavaAssembly => {
                // 生成 .jasm 汇编文件
                // let mut converter = JvmToJasmConverter::new();
                // let jasm_result = converter.convert(jvm_program);
                // match jasm_result.result {
                // Ok(jasm_root) => {
                // let jasm_string = format!("{:#?}", jasm_root);
                // files.insert("main.jasm".to_string(), jasm_string.into_bytes());
                // }
                // Err(error) => return Err(error),
                // }
                return Err(GaiaError::custom_error("JASM output is currently disabled"));
            }
            _ => return Err(GaiaError::custom_error(&format!("Unsupported host ABI: {:?}", config.target.host))),
        }

        Ok(GeneratedFiles { files, diagnostics: vec![] })
    }
}

impl JvmBackend {
    /// Generate JVM program from Gaia program
    pub fn generate_program(program: &GaiaModule) -> Result<JvmProgram> {
        // 使用默认配置生成（保持向后兼容）
        let default_config = GaiaConfig::default();
        convert_gaia_to_jvm(program, &default_config)
    }
}

/// JVM 编译上下文，携带函数映射与目标信息
struct JvmContext {
    function_mapper: FunctionMapper,
    /// 字段类型映射 (类名, 字段名) -> 描述符
    field_types: HashMap<(String, String), String>,
}

/// Convert GaiaModule to JvmProgram
fn convert_gaia_to_jvm(program: &GaiaModule, config: &GaiaConfig) -> Result<JvmProgram> {
    let mut jvm_program = JvmProgram::new(program.name.clone());

    // Set version information
    jvm_program.version = JvmVersion { major: 52, minor: 0 }; // Java 8

    // Set access flags
    jvm_program.access_flags = JvmAccessFlags::PUBLIC;

    // 构建字段类型映射
    let mut field_types = HashMap::new();
    for class in &program.classes {
        for field in &class.fields {
            field_types.insert((class.name.clone(), field.name.clone()), convert_gaia_type_to_jvm_descriptor(&field.ty));
        }
    }
    for global in &program.globals {
        field_types.insert(("Main".to_string(), global.name.clone()), convert_gaia_type_to_jvm_descriptor(&global.ty));
    }

    // 构建上下文（从配置初始化函数映射）
    let ctx = JvmContext { function_mapper: FunctionMapper::from_config(&config.setting)?, field_types };

    // 处理导入 (JVM 中通常通过外部类引用实现，不需要显式导入表，但可以记录)
    // for import in &program.imports { ... }

    // Convert functions（带上下文）
    for function in &program.functions {
        let jvm_method = convert_gaia_function_to_jvm(function, &ctx)?;
        jvm_program.add_method(jvm_method);
    }

    // Convert classes
    for class in &program.classes {
        // 在 JVM 中，每个类通常是一个单独的文件。
        // 这里简化处理，如果是 GaiaModule 中的类，可能需要生成多个 JvmProgram
        // 或者将它们作为内部类处理。
        // 目前我们只处理主类中的字段和方法。
        for field in &class.fields {
            let jvm_field = convert_gaia_field_to_jvm_field(field)?;
            jvm_program.add_field(jvm_field);
        }
        for method in &class.methods {
            let jvm_method = convert_gaia_function_to_jvm(method, &ctx)?;
            jvm_program.add_method(jvm_method);
        }
    }

    // Convert global variables to fields
    for global in &program.globals {
        let jvm_field = convert_gaia_global_to_jvm_field(global)?;
        jvm_program.add_field(jvm_field);
    }

    Ok(jvm_program)
}

/// Convert GaiaFunction to JvmMethod
fn convert_gaia_function_to_jvm(function: &GaiaFunction, ctx: &JvmContext) -> Result<JvmMethod> {
    // 构建方法描述符
    let descriptor = build_method_descriptor(&function.signature.params, &Some(function.signature.return_type.clone()));

    let mut method = JvmMethod::new(function.name.clone(), descriptor);

    // 设置访问标志
    method = method.with_public().with_static();

    // 转换基本块
    for block in &function.blocks {
        // 在 JVM 中，标签通常是指令流的一部分或由汇编器处理
        // 这里我们简化处理，假设 jvm-assembler 能处理标签指令
        // method = method.with_instruction(JvmInstruction::Label { name: block.label.clone() });

        for instruction in &block.instructions {
            let jvm_instructions = convert_gaia_instruction_to_jvm(instruction, ctx)?;
            for ji in jvm_instructions {
                method = method.with_instruction(ji);
            }
        }

        // 处理终止符
        match &block.terminator {
            crate::program::GaiaTerminator::Jump(label) => {
                method = method.with_instruction(JvmInstruction::Goto { target: label.clone() });
            }
            crate::program::GaiaTerminator::Branch { true_label, false_label } => {
                // JVM 通常先判断 false 情况跳转，或者 true 跳转。这里采用 Ifne (if not zero)
                method = method.with_instruction(JvmInstruction::Ifne { target: true_label.clone() });
                method = method.with_instruction(JvmInstruction::Goto { target: false_label.clone() });
            }
            crate::program::GaiaTerminator::Return => {
                method = method.with_instruction(JvmInstruction::Return);
            }
            crate::program::GaiaTerminator::Call { callee, args_count: _, next_block } => {
                // JVM call 映射
                let jvm_target = CompilationTarget {
                    build: Architecture::JVM,
                    host: AbiCompatible::JavaAssembly,
                    target: ApiCompatible::JvmRuntime(8),
                };
                let mapped = ctx.function_mapper.map_function(&jvm_target, callee).unwrap_or(callee.as_str()).to_string();

                method = method.with_instruction(JvmInstruction::Invokestatic {
                    class_name: "Main".to_string(),
                    method_name: mapped,
                    descriptor: "()V".to_string(), // 简化处理，实际需要根据函数签名确定
                });
                method = method.with_instruction(JvmInstruction::Goto { target: next_block.clone() });
            }
            crate::program::GaiaTerminator::Halt => {
                // JVM 中 Halt 可以映射为 System.exit(0)
                method = method.with_instruction(JvmInstruction::Iconst0);
                method = method.with_instruction(JvmInstruction::Invokestatic {
                    class_name: "java/lang/System".to_string(),
                    method_name: "exit".to_string(),
                    descriptor: "(I)V".to_string(),
                });
            }
        }
    }

    // 设置栈和局部变量大小（简化处理）
    method = method.with_max_stack(10).with_max_locals(10);

    Ok(method)
}

/// Convert GaiaField to JvmField
fn convert_gaia_field_to_jvm_field(field: &crate::program::GaiaField) -> Result<JvmField> {
    let descriptor = convert_gaia_type_to_jvm_descriptor(&field.ty);
    let mut jvm_field = JvmField::new(field.name.clone(), descriptor);

    if field.is_static {
        jvm_field = jvm_field.with_static();
    }

    match field.visibility {
        crate::program::Visibility::Public => jvm_field = jvm_field.with_public(),
        crate::program::Visibility::Private => jvm_field = jvm_field.with_private(),
        crate::program::Visibility::Protected => jvm_field = jvm_field.with_protected(),
        _ => {}
    }

    Ok(jvm_field)
}

/// Convert GaiaGlobal to JvmField
fn convert_gaia_global_to_jvm_field(global: &GaiaGlobal) -> Result<JvmField> {
    let descriptor = convert_gaia_type_to_jvm_descriptor(&global.ty);
    let field = JvmField::new(global.name.clone(), descriptor).with_public().with_static();

    Ok(field)
}

/// Convert GaiaInstruction to JvmInstruction
fn convert_gaia_instruction_to_jvm(instruction: &GaiaInstruction, ctx: &JvmContext) -> Result<Vec<JvmInstruction>> {
    match instruction {
        GaiaInstruction::Core(core) => match core {
            CoreInstruction::PushConstant(constant) => match constant {
                GaiaConstant::I8(value) => Ok(vec![JvmInstruction::Bipush { value: *value }]),
                GaiaConstant::U8(value) => Ok(vec![JvmInstruction::Bipush { value: *value as i8 }]),
                GaiaConstant::I16(value) => Ok(vec![JvmInstruction::Sipush { value: *value }]),
                GaiaConstant::U16(value) => Ok(vec![JvmInstruction::Sipush { value: *value as i16 }]),
                GaiaConstant::I32(value) => match *value {
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
                GaiaConstant::U32(value) => Ok(vec![JvmInstruction::Ldc { symbol: value.to_string() }]),
                GaiaConstant::I64(value) => Ok(vec![JvmInstruction::Ldc2W { symbol: value.to_string() }]),
                GaiaConstant::U64(value) => Ok(vec![JvmInstruction::Ldc2W { symbol: value.to_string() }]),
                GaiaConstant::F32(value) => match *value {
                    0.0 => Ok(vec![JvmInstruction::Fconst0]),
                    1.0 => Ok(vec![JvmInstruction::Fconst1]),
                    2.0 => Ok(vec![JvmInstruction::Fconst2]),
                    _ => Ok(vec![JvmInstruction::Ldc { symbol: value.to_string() }]),
                },
                GaiaConstant::F64(value) => match *value {
                    0.0 => Ok(vec![JvmInstruction::Dconst0]),
                    1.0 => Ok(vec![JvmInstruction::Dconst1]),
                    _ => Ok(vec![JvmInstruction::Ldc2W { symbol: value.to_string() }]),
                },
                GaiaConstant::String(value) => Ok(vec![JvmInstruction::Ldc { symbol: value.clone() }]),
                GaiaConstant::Bool(value) => Ok(vec![if *value { JvmInstruction::Iconst1 } else { JvmInstruction::Iconst0 }]),
                GaiaConstant::Null => Ok(vec![JvmInstruction::AconstNull]),
                _ => Err(GaiaError::custom_error("Unsupported constant type for JVM")),
            },
            CoreInstruction::Load(_gaia_type) => {
                // JVM 不直接支持内存间接加载，这里可能需要映射到数组访问或抛出未实现
                Err(GaiaError::not_implemented("JVM indirect load"))
            }
            CoreInstruction::Store(_gaia_type) => {
                // JVM 不直接支持内存间接存储
                Err(GaiaError::not_implemented("JVM indirect store"))
            }
            CoreInstruction::Add(_) => Ok(vec![JvmInstruction::Iadd]),
            CoreInstruction::Sub(_) => Ok(vec![JvmInstruction::Isub]),
            CoreInstruction::Mul(_) => Ok(vec![JvmInstruction::Imul]),
            CoreInstruction::Div(_) => Ok(vec![JvmInstruction::Idiv]),
            CoreInstruction::Pop => Ok(vec![JvmInstruction::Pop]),
            CoreInstruction::Dup => Ok(vec![JvmInstruction::Dup]),
            CoreInstruction::Ret => Ok(vec![JvmInstruction::Return]),
            CoreInstruction::Br(label) => Ok(vec![JvmInstruction::Goto { target: label.clone() }]),
            CoreInstruction::BrTrue(label) => Ok(vec![JvmInstruction::Ifne { target: label.clone() }]),
            CoreInstruction::BrFalse(label) => Ok(vec![JvmInstruction::Ifeq { target: label.clone() }]),
            CoreInstruction::Label(name) => Ok(vec![JvmInstruction::Label { name: name.clone() }]),
            CoreInstruction::Call(name, arg_count) => {
                // 简化处理，由适配器完成映射
                let jvm_target = gaia_types::helpers::CompilationTarget {
                    build: gaia_types::helpers::Architecture::JVM,
                    host: gaia_types::helpers::AbiCompatible::JavaAssembly,
                    target: gaia_types::helpers::ApiCompatible::JvmRuntime(8),
                };
                let mapped = ctx.function_mapper.map_function(&jvm_target, name).unwrap_or(name);
                Ok(vec![JvmInstruction::Invokestatic {
                    class_name: "Main".to_string(), // 假设在 Main 类中
                    method_name: mapped.to_string(),
                    descriptor: format!("({})V", "I".repeat(*arg_count)), // 简化：假设全为 I，返回 Void
                }])
            }
            CoreInstruction::LoadLocal(index, ty) => Ok(vec![match ty {
                GaiaType::I32
                | GaiaType::U32
                | GaiaType::Bool
                | GaiaType::I8
                | GaiaType::U8
                | GaiaType::I16
                | GaiaType::U16 => JvmInstruction::Iload { index: *index as u16 },
                GaiaType::I64 | GaiaType::U64 => JvmInstruction::Lload { index: *index as u16 },
                GaiaType::F32 => JvmInstruction::Fload { index: *index as u16 },
                GaiaType::F64 => JvmInstruction::Dload { index: *index as u16 },
                _ => JvmInstruction::Aload { index: *index as u16 },
            }]),
            CoreInstruction::StoreLocal(index, ty) => Ok(vec![match ty {
                GaiaType::I32
                | GaiaType::U32
                | GaiaType::Bool
                | GaiaType::I8
                | GaiaType::U8
                | GaiaType::I16
                | GaiaType::U16 => JvmInstruction::Istore { index: *index as u16 },
                GaiaType::I64 | GaiaType::U64 => JvmInstruction::Lstore { index: *index as u16 },
                GaiaType::F32 => JvmInstruction::Fstore { index: *index as u16 },
                GaiaType::F64 => JvmInstruction::Dstore { index: *index as u16 },
                _ => JvmInstruction::Astore { index: *index as u16 },
            }]),
            CoreInstruction::LoadArg(index, ty) => Ok(vec![match ty {
                GaiaType::I32
                | GaiaType::U32
                | GaiaType::Bool
                | GaiaType::I8
                | GaiaType::U8
                | GaiaType::I16
                | GaiaType::U16 => JvmInstruction::Iload { index: *index as u16 },
                GaiaType::I64 | GaiaType::U64 => JvmInstruction::Lload { index: *index as u16 },
                GaiaType::F32 => JvmInstruction::Fload { index: *index as u16 },
                GaiaType::F64 => JvmInstruction::Dload { index: *index as u16 },
                _ => JvmInstruction::Aload { index: *index as u16 },
            }]),
            CoreInstruction::StoreArg(index, ty) => Ok(vec![match ty {
                GaiaType::I32
                | GaiaType::U32
                | GaiaType::Bool
                | GaiaType::I8
                | GaiaType::U8
                | GaiaType::I16
                | GaiaType::U16 => JvmInstruction::Istore { index: *index as u16 },
                GaiaType::I64 | GaiaType::U64 => JvmInstruction::Lstore { index: *index as u16 },
                GaiaType::F32 => JvmInstruction::Fstore { index: *index as u16 },
                GaiaType::F64 => JvmInstruction::Dstore { index: *index as u16 },
                _ => JvmInstruction::Astore { index: *index as u16 },
            }]),
            CoreInstruction::New(type_name) => Ok(vec![
                JvmInstruction::New { class_name: type_name.replace('.', "/") },
                JvmInstruction::Dup,
                JvmInstruction::Invokespecial {
                    class_name: type_name.replace('.', "/"),
                    method_name: "<init>".to_string(),
                    descriptor: "()V".to_string(),
                },
            ]),
            CoreInstruction::LoadField(type_name, field_name) => {
                let descriptor = ctx
                    .field_types
                    .get(&(type_name.clone(), field_name.clone()))
                    .cloned()
                    .unwrap_or_else(|| "Ljava/lang/Object;".to_string());
                Ok(vec![JvmInstruction::Getfield {
                    class_name: type_name.replace('.', "/"),
                    field_name: field_name.to_string(),
                    descriptor,
                }])
            }
            CoreInstruction::StoreField(type_name, field_name) => {
                let descriptor = ctx
                    .field_types
                    .get(&(type_name.clone(), field_name.clone()))
                    .cloned()
                    .unwrap_or_else(|| "Ljava/lang/Object;".to_string());
                Ok(vec![JvmInstruction::Putfield {
                    class_name: type_name.replace('.', "/"),
                    field_name: field_name.to_string(),
                    descriptor,
                }])
            }
            CoreInstruction::LoadElement(ty) => Ok(vec![match ty {
                GaiaType::I32 | GaiaType::U32 => JvmInstruction::Iaload,
                GaiaType::I64 | GaiaType::U64 => JvmInstruction::Laload,
                GaiaType::F32 => JvmInstruction::Faload,
                GaiaType::F64 => JvmInstruction::Daload,
                GaiaType::I8 | GaiaType::U8 | GaiaType::Bool => JvmInstruction::Baload,
                GaiaType::I16 | GaiaType::U16 => JvmInstruction::Saload,
                _ => JvmInstruction::Aaload,
            }]),
            CoreInstruction::StoreElement(ty) => Ok(vec![match ty {
                GaiaType::I32 | GaiaType::U32 => JvmInstruction::Iastore,
                GaiaType::I64 | GaiaType::U64 => JvmInstruction::Lastore,
                GaiaType::F32 => JvmInstruction::Fastore,
                GaiaType::F64 => JvmInstruction::Dastore,
                GaiaType::I8 | GaiaType::U8 | GaiaType::Bool => JvmInstruction::Bastore,
                GaiaType::I16 | GaiaType::U16 => JvmInstruction::Sastore,
                _ => JvmInstruction::Aastore,
            }]),
            CoreInstruction::ArrayLength => Ok(vec![JvmInstruction::Arraylength]),
            CoreInstruction::NewArray(elem_type, _) => {
                Ok(vec![match elem_type {
                    GaiaType::I32 | GaiaType::U32 => JvmInstruction::Newarray { atype: 10 }, // T_INT
                    GaiaType::I64 | GaiaType::U64 => JvmInstruction::Newarray { atype: 11 }, // T_LONG
                    GaiaType::F32 => JvmInstruction::Newarray { atype: 6 },                  // T_FLOAT
                    GaiaType::F64 => JvmInstruction::Newarray { atype: 7 },                  // T_DOUBLE
                    GaiaType::I8 | GaiaType::U8 => JvmInstruction::Newarray { atype: 8 },    // T_BYTE
                    GaiaType::Bool => JvmInstruction::Newarray { atype: 4 },                 // T_BOOLEAN
                    GaiaType::I16 | GaiaType::U16 => JvmInstruction::Newarray { atype: 9 },  // T_SHORT
                    _ => JvmInstruction::Anewarray { class_name: convert_gaia_type_to_jvm_descriptor(elem_type) },
                }])
            }
            _ => Ok(vec![]),
        },
        GaiaInstruction::Managed(managed) => match managed {
            _ => Ok(vec![]),
        },
        _ => Ok(vec![]),
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
        GaiaType::Bool => "Z".to_string(),
        GaiaType::I8 | GaiaType::U8 => "B".to_string(),
        GaiaType::I16 | GaiaType::U16 => "S".to_string(),
        GaiaType::I32 | GaiaType::U32 => "I".to_string(),
        GaiaType::I64 | GaiaType::U64 => "J".to_string(),
        GaiaType::F32 | GaiaType::F16 => "F".to_string(),
        GaiaType::F64 => "D".to_string(),
        GaiaType::String => "Ljava/lang/String;".to_string(),
        GaiaType::Class(name) => format!("L{};", name.replace('.', "/")),
        GaiaType::Struct(name) => format!("L{};", name.replace('.', "/")),
        GaiaType::Array(elem, _) => format!("[{}", convert_gaia_type_to_jvm_descriptor(elem)),
        GaiaType::Void => "V".to_string(),
        GaiaType::Object => "Ljava/lang/Object;".to_string(),
        _ => "Ljava/lang/Object;".to_string(),
    }
}
