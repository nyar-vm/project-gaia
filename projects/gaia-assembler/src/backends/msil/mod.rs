//! IL (Intermediate Language) backend compiler

#[cfg(feature = "clr")]
use crate::{
    adapters::FunctionMapper,
    config::{GaiaConfig, GaiaSettings},
    instruction::{CmpCondition, CoreInstruction, GaiaInstruction},
    program::{GaiaConstant, GaiaFunction, GaiaModule},
    types::GaiaType,
    Backend, GeneratedFiles,
};
#[cfg(feature = "clr")]
use clr_assembler::program::*;
#[cfg(feature = "clr")]
#[allow(unused_imports)]
use gaia_types::{
    helpers::{AbiCompatible, ApiCompatible, Architecture, CompilationTarget},
    GaiaError, Result,
};
#[cfg(feature = "clr")]
use std::collections::HashMap;

/// IL Backend implementation
#[derive(Default)]
#[cfg(feature = "clr")]
pub struct ClrBackend {}

#[cfg(feature = "clr")]
impl Backend for ClrBackend {
    fn name(&self) -> &'static str {
        "MSIL"
    }

    fn primary_target(&self) -> CompilationTarget {
        CompilationTarget {
            build: Architecture::CLR,
            host: AbiCompatible::MicrosoftIntermediateLanguage,
            target: ApiCompatible::ClrRuntime(4),
        }
    }

    fn match_score(&self, target: &CompilationTarget) -> f32 {
        match target.build {
            Architecture::CLR => match target.host {
                // dll, exe output, 30% support (now real binary!)
                AbiCompatible::Unknown => 30.0,
                // msil output, 30% support
                AbiCompatible::MicrosoftIntermediateLanguage => 30.0,
                _ => -100.0,
            },
            _ => -100.0,
        }
    }

    fn generate(&self, program: &GaiaModule, _config: &GaiaConfig) -> Result<GeneratedFiles> {
        let mut files = HashMap::new();
        match _config.target.host {
            AbiCompatible::Unknown => {
                // 生成真实的 CLR 二进制 (DLL/EXE)
                let clr_program = convert_to_clr_program(program, Some(&_config.setting))?;
                let buffer = std::io::Cursor::new(Vec::new());
                let writer = clr_assembler::formats::dll::writer::DllWriter::new(buffer);
                let result = writer.write(&clr_program);
                if let Some(error) = result.diagnostics.into_iter().next() {
                    return Err(error);
                }
                files.insert("main.dll".to_string(), result.result.expect("Failed to get DLL buffer").into_inner());
            }
            AbiCompatible::MicrosoftIntermediateLanguage => {
                // 生成 IL 文本
                files.insert("main.il".to_string(), compile_with_settings(program, Some(&_config.setting))?);
            }
            _ => Err(GaiaError::invalid_data("Unsupported host ABI for CLR backend"))?,
        }

        Ok(GeneratedFiles { files, diagnostics: vec![] })
    }
}

#[cfg(feature = "clr")]
impl ClrBackend {
    /// Generate IL bytecode from Gaia program
    pub fn generate(program: &GaiaModule) -> Result<Vec<u8>> {
        // 使用文本模式生成 IL，然后返回字节
        compile_with_settings(program, None)
    }

    /// Generate IL bytecode with given GaiaSettings
    pub fn generate_with_settings(program: &GaiaModule, settings: &GaiaSettings) -> Result<Vec<u8>> {
        // Note: compile_with_settings still returns Result<Vec<u8>>
        compile_with_settings(program, Some(settings))
    }
}

/// Compile Gaia program to IL bytecode
#[cfg(feature = "clr")]
pub fn compile(program: &GaiaModule) -> Result<Vec<u8>> {
    ClrBackend::generate(program)
}

/// Compile with optional GaiaSettings for function mapping
#[cfg(feature = "clr")]
fn compile_with_settings(program: &GaiaModule, settings: Option<&GaiaSettings>) -> Result<Vec<u8>> {
    let mut context = IlContext::new_text();
    if let Some(s) = settings {
        context.function_mapper = FunctionMapper::from_config(s).unwrap_or_default();
    }
    compile_program(&mut context, program)?;
    context.generate_bytecode()
}

/// Compile entire program
#[cfg(feature = "clr")]
fn compile_program(context: &mut IlContext, program: &GaiaModule) -> Result<()> {
    // 预填充字段类型映射
    for s in &program.structs {
        for (name, ty) in &s.fields {
            context.field_types.insert((s.name.clone(), name.clone()), ty.clone());
        }
    }
    for c in &program.classes {
        for field in &c.fields {
            context.field_types.insert((c.name.clone(), field.name.clone()), field.ty.clone());
        }
    }
    for g in &program.globals {
        context.field_types.insert(("Main".to_string(), g.name.clone()), g.ty.clone());
    }

    // 添加程序集声明
    context.emit_assembly_declaration(&program.name)?;

    // 处理导入项
    for import in &program.imports {
        context.emit_external_assembly(&import.library)?;
    }

    // Compile all structs
    for s in &program.structs {
        compile_struct(context, s)?;
    }

    // Compile all classes
    for c in &program.classes {
        compile_class(context, c)?;
    }

    // Compile all functions
    for function in &program.functions {
        compile_function(context, function)?;
    }

    Ok(())
}

/// Compile struct definition
#[cfg(feature = "clr")]
fn compile_struct(context: &mut IlContext, s: &crate::program::GaiaStruct) -> Result<()> {
    context.start_class(&s.name)?;

    for (name, ty) in &s.fields {
        context.write_field(name, ty)?;
    }

    // 添加默认构造函数 (仅文本模式)
    if let IlMode::Text(writer) = &mut context.mode {
        writer.write_default_constructor()?;
    }

    context.end_class()?;
    Ok(())
}

/// Compile class definition
#[cfg(feature = "clr")]
fn compile_class(context: &mut IlContext, c: &crate::program::GaiaClass) -> Result<()> {
    context.start_class(&c.name)?;

    for field in &c.fields {
        context.write_field(&field.name, &field.ty)?;
    }

    for method in &c.methods {
        compile_function(context, method)?;
    }

    context.end_class()?;
    Ok(())
}

/// Compile single function
#[cfg(feature = "clr")]
fn compile_function(context: &mut IlContext, function: &GaiaFunction) -> Result<()> {
    // Start function definition
    start_function(
        context,
        &function.name,
        &function.signature.params,
        &Some(function.signature.return_type.clone()),
        function.name == "main",
    )?;

    // Compile blocks
    for block in &function.blocks {
        context.define_label(&block.label)?;

        for instruction in &block.instructions {
            compile_instruction(context, instruction)?;
        }

        // Compile terminator
        match &block.terminator {
            crate::program::GaiaTerminator::Jump(label) => context.emit_br(label)?,
            crate::program::GaiaTerminator::Branch { true_label, false_label } => {
                context.emit_brtrue(true_label)?;
                context.emit_br(false_label)?;
            }
            crate::program::GaiaTerminator::Return => context.emit_ret()?,
            crate::program::GaiaTerminator::Call { callee, args_count: _, next_block } => {
                context.emit_call(callee)?;
                context.emit_br(next_block)?;
            }
            crate::program::GaiaTerminator::Halt => {
                context.emit_ldc_i4(0)?;
                context.emit_call("ExitProcess")?;
            }
        }
    }

    // End function definition
    end_function(context)?;

    Ok(())
}

/// Compile single instruction
#[cfg(feature = "clr")]
fn compile_instruction(context: &mut IlContext, instruction: &GaiaInstruction) -> Result<()> {
    match instruction {
        GaiaInstruction::Core(core) => match core {
            CoreInstruction::PushConstant(constant) => compile_load_constant(context, constant),
            CoreInstruction::Load(gaia_type) => compile_load_indirect(context, gaia_type),
            CoreInstruction::Store(gaia_type) => compile_store_indirect(context, gaia_type),
            CoreInstruction::Add(_) => compile_add(context),
            CoreInstruction::Sub(_) => compile_subtract(context),
            CoreInstruction::Mul(_) => compile_multiply(context),
            CoreInstruction::Div(_) => compile_divide(context),
            CoreInstruction::Pop => compile_pop(context),
            CoreInstruction::Dup => compile_duplicate(context),
            CoreInstruction::Cmp(cond, _) => match cond {
                CmpCondition::Eq => compile_equal(context),
                CmpCondition::Ne => compile_not_equal(context),
                CmpCondition::Lt => compile_less_than(context),
                CmpCondition::Le => compile_less_than_or_equal(context),
                CmpCondition::Gt => compile_greater_than(context),
                CmpCondition::Ge => compile_greater_than_or_equal(context),
            },
            CoreInstruction::Ret => compile_return(context),
            CoreInstruction::Br(label) => compile_branch(context, label),
            CoreInstruction::BrTrue(label) => compile_branch_true(context, label),
            CoreInstruction::BrFalse(label) => compile_branch_false(context, label),
            CoreInstruction::Label(name) => compile_label(context, name),
            CoreInstruction::Call(name, arg_count) => compile_call(context, name, *arg_count),
            CoreInstruction::LoadLocal(index, _) => compile_load_local(context, *index),
            CoreInstruction::StoreLocal(index, _) => compile_store_local(context, *index),
            CoreInstruction::LoadArg(index, _) => compile_load_argument(context, *index),
            CoreInstruction::StoreArg(index, _) => compile_store_argument(context, *index),
            CoreInstruction::New(type_name) => compile_new_object(context, type_name),
            CoreInstruction::NewArray(elem_type, _) => compile_new_array(context, elem_type),
            CoreInstruction::LoadField(type_name, field_name) => {
                let gaia_type = context
                    .field_types
                    .get(&(type_name.to_string(), field_name.to_string()))
                    .cloned()
                    .unwrap_or(GaiaType::Object);
                compile_load_field(context, type_name, field_name, &gaia_type)
            }
            CoreInstruction::StoreField(type_name, field_name) => {
                let gaia_type = context
                    .field_types
                    .get(&(type_name.to_string(), field_name.to_string()))
                    .cloned()
                    .unwrap_or(GaiaType::Object);
                compile_store_field(context, type_name, field_name, &gaia_type)
            }
            CoreInstruction::LoadElement(elem_type) => compile_load_element(context, elem_type),
            CoreInstruction::StoreElement(elem_type) => compile_store_element(context, elem_type),
            CoreInstruction::ArrayLength => compile_array_length(context),
            _ => Ok(()),
        },
        GaiaInstruction::Managed(managed) => match managed {
            _ => Ok(()),
        },
        _ => Ok(()),
    }
}

#[cfg(feature = "clr")]
fn compile_new_object(context: &mut IlContext, type_name: &str) -> Result<()> {
    // 构造构造函数引用，默认为 .ctor()
    let ctor_ref = format!("instance void {}::.ctor()", type_name);
    context.emit_newobj(&ctor_ref)
}

#[cfg(feature = "clr")]
fn compile_load_field(context: &mut IlContext, type_name: &str, field_name: &str, gaia_type: &GaiaType) -> Result<()> {
    let field_ref = format!("{} {}::{}", gaia_type_to_msil_name(gaia_type), type_name, field_name);
    context.emit_ldfld(&field_ref)
}

#[cfg(feature = "clr")]
fn compile_store_field(context: &mut IlContext, type_name: &str, field_name: &str, gaia_type: &GaiaType) -> Result<()> {
    let field_ref = format!("{} {}::{}", gaia_type_to_msil_name(gaia_type), type_name, field_name);
    context.emit_stfld(&field_ref)
}

#[cfg(feature = "clr")]
fn compile_new_array(context: &mut IlContext, elem_type: &GaiaType) -> Result<()> {
    context.emit_newarr(&gaia_type_to_msil_name(elem_type))
}

#[cfg(feature = "clr")]
fn compile_load_element(context: &mut IlContext, elem_type: &GaiaType) -> Result<()> {
    context.emit_ldelem(&gaia_type_to_msil_name(elem_type))
}

#[cfg(feature = "clr")]
fn compile_store_element(context: &mut IlContext, elem_type: &GaiaType) -> Result<()> {
    context.emit_stelem(&gaia_type_to_msil_name(elem_type))
}

#[cfg(feature = "clr")]
fn compile_array_length(context: &mut IlContext) -> Result<()> {
    context.emit_ldlen()
}

#[cfg(feature = "clr")]
fn compile_return(context: &mut IlContext) -> Result<()> {
    context.emit_ret()
}

#[cfg(feature = "clr")]
fn compile_branch(context: &mut IlContext, label: &str) -> Result<()> {
    context.emit_br(label)
}

#[cfg(feature = "clr")]
fn compile_branch_true(context: &mut IlContext, label: &str) -> Result<()> {
    context.emit_brtrue(label)
}

#[cfg(feature = "clr")]
fn compile_branch_false(context: &mut IlContext, label: &str) -> Result<()> {
    context.emit_brfalse(label)
}

#[cfg(feature = "clr")]
fn compile_label(context: &mut IlContext, name: &str) -> Result<()> {
    context.define_label(name)
}

#[cfg(feature = "clr")]
fn compile_call(context: &mut IlContext, name: &str, _arg_count: usize) -> Result<()> {
    // 这是一个简化实现，实际需要根据函数名查找签名
    // 目前假设由 context.emit_call 处理映射
    let mapped_name = context.map_function(name);
    context.emit_call(&mapped_name)
}

#[cfg(feature = "clr")]
fn compile_load_constant(context: &mut IlContext, constant: &GaiaConstant) -> Result<()> {
    match constant {
        GaiaConstant::I8(value) => context.emit_ldc_i4(*value as i32),
        GaiaConstant::U8(value) => context.emit_ldc_i4(*value as i32),
        GaiaConstant::I16(value) => context.emit_ldc_i4(*value as i32),
        GaiaConstant::U16(value) => context.emit_ldc_i4(*value as i32),
        GaiaConstant::I32(value) => context.emit_ldc_i4(*value),
        GaiaConstant::U32(value) => context.emit_ldc_i4(*value as i32),
        GaiaConstant::I64(value) => context.emit_ldc_i8(*value),
        GaiaConstant::U64(value) => context.emit_ldc_i8(*value as i64),
        GaiaConstant::F32(value) => context.emit_ldc_r4(*value),
        GaiaConstant::F64(value) => context.emit_ldc_r8(*value),
        GaiaConstant::String(value) => context.emit_ldstr(value),
        GaiaConstant::Bool(value) => context.emit_ldc_i4(if *value { 1 } else { 0 }),
        GaiaConstant::Null => context.emit_ldnull(),
        _ => Err(GaiaError::custom_error("Unsupported constant type for CLR")),
    }
}

#[cfg(feature = "clr")]
fn compile_load_local(context: &mut IlContext, index: u32) -> Result<()> {
    context.emit_ldloc(index)
}

#[cfg(feature = "clr")]
fn compile_store_local(context: &mut IlContext, index: u32) -> Result<()> {
    context.emit_stloc(index)
}

#[cfg(feature = "clr")]
fn compile_load_argument(context: &mut IlContext, index: u32) -> Result<()> {
    context.emit_ldarg(index)
}

#[cfg(feature = "clr")]
fn compile_store_argument(context: &mut IlContext, index: u32) -> Result<()> {
    context.emit_starg(index)
}

#[cfg(feature = "clr")]
fn compile_add(context: &mut IlContext) -> Result<()> {
    context.emit_add()
}

#[cfg(feature = "clr")]
fn compile_subtract(context: &mut IlContext) -> Result<()> {
    context.emit_sub()
}

#[cfg(feature = "clr")]
fn compile_multiply(context: &mut IlContext) -> Result<()> {
    context.emit_mul()
}

#[cfg(feature = "clr")]
fn compile_divide(context: &mut IlContext) -> Result<()> {
    context.emit_div()
}

#[cfg(feature = "clr")]
fn compile_equal(context: &mut IlContext) -> Result<()> {
    context.emit_ceq()
}

#[cfg(feature = "clr")]
fn compile_not_equal(context: &mut IlContext) -> Result<()> {
    context.emit_ceq()?;
    context.emit_ldc_i4(0)?;
    context.emit_ceq()
}

#[cfg(feature = "clr")]
fn compile_less_than(context: &mut IlContext) -> Result<()> {
    context.emit_clt()
}

#[cfg(feature = "clr")]
fn compile_greater_than(context: &mut IlContext) -> Result<()> {
    context.emit_cgt()
}

#[cfg(feature = "clr")]
fn compile_less_than_or_equal(context: &mut IlContext) -> Result<()> {
    // a <= b  <=>  !(a > b)  <=>  (a > b) == 0
    context.emit_cgt()?;
    context.emit_ldc_i4(0)?;
    context.emit_ceq()
}

#[cfg(feature = "clr")]
fn compile_greater_than_or_equal(context: &mut IlContext) -> Result<()> {
    // a >= b  <=>  !(a < b)  <=>  (a < b) == 0
    context.emit_clt()?;
    context.emit_ldc_i4(0)?;
    context.emit_ceq()
}

#[cfg(feature = "clr")]
fn compile_logical_not(context: &mut IlContext) -> Result<()> {
    // !a  <=>  a == 0
    context.emit_ldc_i4(0)?;
    context.emit_ceq()
}

#[cfg(feature = "clr")]
fn compile_duplicate(context: &mut IlContext) -> Result<()> {
    context.emit_dup()
}

#[cfg(feature = "clr")]
fn compile_pop(context: &mut IlContext) -> Result<()> {
    context.emit_pop()
}

#[cfg(feature = "clr")]
fn compile_logical_and(context: &mut IlContext) -> Result<()> {
    context.emit_and()
}

#[cfg(feature = "clr")]
fn compile_logical_or(context: &mut IlContext) -> Result<()> {
    context.emit_or()
}

#[cfg(feature = "clr")]
fn compile_load_address(context: &mut IlContext, addr: u32) -> Result<()> {
    context.emit_ldloca(addr)
}

#[cfg(feature = "clr")]
fn compile_load_indirect(context: &mut IlContext, gaia_type: &GaiaType) -> Result<()> {
    match gaia_type {
        GaiaType::I8 => context.emit_ldind_i4(),  // 8位整数加载为32位
        GaiaType::I16 => context.emit_ldind_i4(), // 16位整数加载为32位
        GaiaType::I32 => context.emit_ldind_i4(),
        GaiaType::I64 => context.emit_ldind_i8(),
        GaiaType::F32 => context.emit_ldind_r4(),
        GaiaType::F64 => context.emit_ldind_r8(),
        _ => context.emit_ldind_ref(),
    }
}

#[cfg(feature = "clr")]
fn compile_store_indirect(context: &mut IlContext, gaia_type: &GaiaType) -> Result<()> {
    match gaia_type {
        GaiaType::I8 => context.emit_stind_i4(),  // 8位整数存储为32位
        GaiaType::I16 => context.emit_stind_i4(), // 16位整数存储为32位
        GaiaType::I32 => context.emit_stind_i4(),
        GaiaType::I64 => context.emit_stind_i8(),
        GaiaType::F32 => context.emit_stind_r4(),
        GaiaType::F64 => context.emit_stind_r8(),
        _ => context.emit_stind_ref(),
    }
}

#[cfg(feature = "clr")]
fn compile_convert(_context: &mut IlContext, _from_type: &GaiaType, _to_type: &GaiaType) -> Result<()> {
    match _to_type {
        GaiaType::I8 => _context.emit_conv_i4(),  // 8位整数转换为32位
        GaiaType::I16 => _context.emit_conv_i4(), // 16位整数转换为32位
        GaiaType::I32 => _context.emit_conv_i4(),
        GaiaType::I64 => _context.emit_conv_i8(),
        GaiaType::F32 => _context.emit_conv_r4(),
        GaiaType::F64 => _context.emit_conv_r8(),
        _ => Ok(()),
    }
}

#[cfg(feature = "clr")]
fn compile_box(context: &mut IlContext, gaia_type: &GaiaType) -> Result<()> {
    context.emit_box(gaia_type)
}

#[cfg(feature = "clr")]
fn compile_unbox(context: &mut IlContext, gaia_type: &GaiaType) -> Result<()> {
    context.emit_unbox(gaia_type)
}

/// Start function definition
#[cfg(feature = "clr")]
fn start_function(
    context: &mut IlContext,
    name: &str,
    parameters: &[GaiaType],
    return_type: &Option<GaiaType>,
    is_entry: bool,
) -> Result<()> {
    context.start_method(name, parameters, return_type, is_entry)
}

#[cfg(feature = "clr")]
fn end_function(context: &mut IlContext) -> Result<()> {
    context.end_method()
}

/// 将 GaiaType 映射为 MSIL 类型名称
#[cfg(feature = "clr")]
fn gaia_type_to_msil_name(gaia_type: &GaiaType) -> String {
    match gaia_type {
        GaiaType::I8 => "int8".to_string(),
        GaiaType::U8 => "uint8".to_string(),
        GaiaType::I16 => "int16".to_string(),
        GaiaType::U16 => "uint16".to_string(),
        GaiaType::I32 => "int32".to_string(),
        GaiaType::U32 => "uint32".to_string(),
        GaiaType::I64 => "int64".to_string(),
        GaiaType::U64 => "uint64".to_string(),
        GaiaType::F32 => "float32".to_string(),
        GaiaType::F64 => "float64".to_string(),
        GaiaType::Bool => "bool".to_string(),
        GaiaType::String => "string".to_string(),
        GaiaType::Object => "object".to_string(),
        GaiaType::Class(name) => name.clone(),
        GaiaType::Struct(name) => name.clone(),
        GaiaType::Array(elem, _) => format!("{}[]", gaia_type_to_msil_name(elem)),
        GaiaType::Pointer(_, _) => "native int".to_string(),
        GaiaType::Void => "void".to_string(),
        _ => "object".to_string(),
    }
}

/// MSIL Text Writer
#[cfg(feature = "clr")]
struct MsilWriter<W: std::fmt::Write> {
    writer: gaia_types::writer::TextWriter<W>,
}

#[cfg(feature = "clr")]
impl<W: std::fmt::Write> MsilWriter<W> {
    fn new(writer: W) -> Self {
        Self { writer: gaia_types::writer::TextWriter::new(writer) }
    }

    fn finish(self) -> W {
        self.writer.into_inner()
    }

    fn write_assembly(&mut self, name: &str) -> Result<()> {
        self.writer.write_line(&format!(".assembly {} {{}}", name))?;
        Ok(())
    }

    fn write_external_assembly(&mut self, name: &str) -> Result<()> {
        self.writer.write_line(&format!(".assembly extern {} {{}}", name))?;
        Ok(())
    }

    fn start_class(&mut self, name: &str) -> Result<()> {
        self.writer
            .write_line(&format!(".class public auto ansi beforefieldinit {} extends [mscorlib]System.Object {{", name))?;
        self.writer.indent("")?;
        Ok(())
    }

    fn end_class(&mut self) -> Result<()> {
        self.writer.dedent("}")?;
        self.writer.write_line("")?;
        Ok(())
    }

    fn write_field(&mut self, name: &str, msil_type: &str) -> Result<()> {
        self.writer.write_line(&format!(".field public {} {}", msil_type, name))?;
        Ok(())
    }

    fn write_default_constructor(&mut self) -> Result<()> {
        self.writer.write_line(".method public hidebysig specialname rtspecialname instance void .ctor() cil managed {")?;
        self.writer.indent("")?;
        self.writer.write_line("ldarg.0")?;
        self.writer.write_line("call instance void [mscorlib]System.Object::.ctor()")?;
        self.writer.write_line("ret")?;
        self.writer.dedent("}")?;
        Ok(())
    }

    fn start_method(&mut self, name: &str, parameters: &[&str], return_type: Option<&str>, is_entry: bool) -> Result<()> {
        let ret = return_type.unwrap_or("void");
        let params = parameters.join(", ");
        self.writer.write_line(&format!(".method public hidebysig {} {}({}) cil managed {{", ret, name, params))?;
        self.writer.indent("")?;
        if is_entry {
            self.writer.write_line(".entrypoint")?;
        }
        self.writer.write_line(".maxstack 8")?;
        Ok(())
    }

    fn end_method(&mut self) -> Result<()> {
        self.writer.dedent("}")?;
        self.writer.write_line("")?;
        Ok(())
    }

    fn define_label(&mut self, name: &str) -> Result<()> {
        self.writer.write_line(&format!("{}:", name))?;
        Ok(())
    }

    fn emit_ldc_i4(&mut self, value: i32) -> Result<()> {
        self.writer.write_line(&format!("ldc.i4 {}", value))?;
        Ok(())
    }

    fn emit_ldc_i8(&mut self, value: i64) -> Result<()> {
        self.writer.write_line(&format!("ldc.i8 {}", value))?;
        Ok(())
    }

    fn emit_ldc_r4(&mut self, value: f32) -> Result<()> {
        self.writer.write_line(&format!("ldc.r4 {}", value))?;
        Ok(())
    }

    fn emit_ldc_r8(&mut self, value: f64) -> Result<()> {
        self.writer.write_line(&format!("ldc.r8 {}", value))?;
        Ok(())
    }

    fn emit_ldstr(&mut self, value: &str) -> Result<()> {
        self.writer.write_line(&format!("ldstr \"{}\"", value))?;
        Ok(())
    }

    fn emit_ldnull(&mut self) -> Result<()> {
        self.writer.write_line("ldnull")?;
        Ok(())
    }

    fn emit_ldloc(&mut self, index: u32) -> Result<()> {
        self.writer.write_line(&format!("ldloc {}", index))?;
        Ok(())
    }

    fn emit_stloc(&mut self, index: u32) -> Result<()> {
        self.writer.write_line(&format!("stloc {}", index))?;
        Ok(())
    }

    fn emit_ldarg(&mut self, index: u32) -> Result<()> {
        self.writer.write_line(&format!("ldarg {}", index))?;
        Ok(())
    }

    fn emit_starg(&mut self, index: u32) -> Result<()> {
        self.writer.write_line(&format!("starg {}", index))?;
        Ok(())
    }

    fn emit_add(&mut self) -> Result<()> {
        self.writer.write_line("add")?;
        Ok(())
    }

    fn emit_sub(&mut self) -> Result<()> {
        self.writer.write_line("sub")?;
        Ok(())
    }

    fn emit_mul(&mut self) -> Result<()> {
        self.writer.write_line("mul")?;
        Ok(())
    }

    fn emit_div(&mut self) -> Result<()> {
        self.writer.write_line("div")?;
        Ok(())
    }

    fn emit_ceq(&mut self) -> Result<()> {
        self.writer.write_line("ceq")?;
        Ok(())
    }

    fn emit_cgt(&mut self) -> Result<()> {
        self.writer.write_line("cgt")?;
        Ok(())
    }

    fn emit_clt(&mut self) -> Result<()> {
        self.writer.write_line("clt")?;
        Ok(())
    }

    fn emit_br(&mut self, label: &str) -> Result<()> {
        self.writer.write_line(&format!("br {}", label))?;
        Ok(())
    }

    fn emit_brtrue(&mut self, label: &str) -> Result<()> {
        self.writer.write_line(&format!("brtrue {}", label))?;
        Ok(())
    }

    fn emit_brfalse(&mut self, label: &str) -> Result<()> {
        self.writer.write_line(&format!("brfalse {}", label))?;
        Ok(())
    }

    fn emit_call(&mut self, method_ref: &str) -> Result<()> {
        self.writer.write_line(&format!("call {}", method_ref))?;
        Ok(())
    }

    fn emit_ret(&mut self) -> Result<()> {
        self.writer.write_line("ret")?;
        Ok(())
    }

    fn emit_dup(&mut self) -> Result<()> {
        self.writer.write_line("dup")?;
        Ok(())
    }

    fn emit_pop(&mut self) -> Result<()> {
        self.writer.write_line("pop")?;
        Ok(())
    }

    fn emit_newarr(&mut self, type_name: &str) -> Result<()> {
        self.writer.write_line(&format!("newarr {}", type_name))?;
        Ok(())
    }

    fn emit_ldelem(&mut self, type_name: &str) -> Result<()> {
        self.writer.write_line(&format!("ldelem {}", type_name))?;
        Ok(())
    }

    fn emit_stelem(&mut self, type_name: &str) -> Result<()> {
        self.writer.write_line(&format!("stelem {}", type_name))?;
        Ok(())
    }

    fn emit_ldlen(&mut self) -> Result<()> {
        self.writer.write_line("ldlen")?;
        Ok(())
    }

    fn emit_and(&mut self) -> Result<()> {
        self.writer.write_line("and")?;
        Ok(())
    }

    fn emit_or(&mut self) -> Result<()> {
        self.writer.write_line("or")?;
        Ok(())
    }

    fn emit_ldloca(&mut self, index: u32) -> Result<()> {
        self.writer.write_line(&format!("ldloca {}", index))?;
        Ok(())
    }

    fn emit_ldind_i4(&mut self) -> Result<()> {
        self.writer.write_line("ldind.i4")?;
        Ok(())
    }

    fn emit_ldind_i8(&mut self) -> Result<()> {
        self.writer.write_line("ldind.i8")?;
        Ok(())
    }

    fn emit_ldind_r4(&mut self) -> Result<()> {
        self.writer.write_line("ldind.r4")?;
        Ok(())
    }

    fn emit_ldind_r8(&mut self) -> Result<()> {
        self.writer.write_line("ldind.r8")?;
        Ok(())
    }

    fn emit_ldind_ref(&mut self) -> Result<()> {
        self.writer.write_line("ldind.ref")?;
        Ok(())
    }

    fn emit_stind_i4(&mut self) -> Result<()> {
        self.writer.write_line("stind.i4")?;
        Ok(())
    }

    fn emit_stind_i8(&mut self) -> Result<()> {
        self.writer.write_line("stind.i8")?;
        Ok(())
    }

    fn emit_stind_r4(&mut self) -> Result<()> {
        self.writer.write_line("stind.r4")?;
        Ok(())
    }

    fn emit_stind_r8(&mut self) -> Result<()> {
        self.writer.write_line("stind.r8")?;
        Ok(())
    }

    fn emit_stind_ref(&mut self) -> Result<()> {
        self.writer.write_line("stind.ref")?;
        Ok(())
    }

    fn emit_newobj(&mut self, method_ref: &str) -> Result<()> {
        self.writer.write_line(&format!("newobj {}", method_ref))?;
        Ok(())
    }

    fn emit_ldfld(&mut self, field_ref: &str) -> Result<()> {
        self.writer.write_line(&format!("ldfld {}", field_ref))?;
        Ok(())
    }

    fn emit_stfld(&mut self, field_ref: &str) -> Result<()> {
        self.writer.write_line(&format!("stfld {}", field_ref))?;
        Ok(())
    }

    fn emit_conv_i4(&mut self) -> Result<()> {
        self.writer.write_line("conv.i4")?;
        Ok(())
    }

    fn emit_conv_i8(&mut self) -> Result<()> {
        self.writer.write_line("conv.i8")?;
        Ok(())
    }

    fn emit_conv_r4(&mut self) -> Result<()> {
        self.writer.write_line("conv.r4")?;
        Ok(())
    }

    fn emit_conv_r8(&mut self) -> Result<()> {
        self.writer.write_line("conv.r8")?;
        Ok(())
    }

    fn emit_box(&mut self, type_name: &str) -> Result<()> {
        self.writer.write_line(&format!("box {}", type_name))?;
        Ok(())
    }

    fn emit_unbox(&mut self, type_name: &str) -> Result<()> {
        self.writer.write_line(&format!("unbox {}", type_name))?;
        Ok(())
    }
}

/// IL Context for code generation
#[cfg(feature = "clr")]
struct IlContext {
    mode: IlMode,
    function_mapper: FunctionMapper,
    /// 字段类型映射 (类名, 字段名) -> GaiaType
    field_types: HashMap<(String, String), GaiaType>,
}

#[cfg(feature = "clr")]
enum IlMode {
    Text(MsilWriter<String>),
    Binary { program: ClrProgram, current_type: Option<ClrType>, current_method: Option<ClrMethod> },
}

#[cfg(feature = "clr")]
impl IlContext {
    fn new_text() -> Self {
        Self {
            mode: IlMode::Text(MsilWriter::new(String::new())),
            function_mapper: FunctionMapper::new(),
            field_types: HashMap::new(),
        }
    }

    fn new_binary(name: String) -> Self {
        let program = ClrProgram {
            name: name.clone(),
            version: ClrVersion { major: 1, minor: 0, build: 0, revision: 0 },
            access_flags: ClrAccessFlags {
                is_public: true,
                is_private: false,
                is_security_transparent: false,
                is_retargetable: false,
            },
            external_assemblies: vec![ClrExternalAssembly {
                name: "mscorlib".to_string(),
                version: ClrVersion { major: 4, minor: 0, build: 0, revision: 0 },
                public_key_token: Some(vec![0xB7, 0x7A, 0x5C, 0x56, 0x19, 0x34, 0xE0, 0x89]),
                culture: None,
                hash_algorithm: None,
            }],
            module: Some(ClrModule { name: format!("{}.dll", name), mvid: None }),
            types: vec![],
            global_methods: vec![],
            global_fields: vec![],
            attributes: vec![],
            constant_pool: ClrConstantPool::new(),
            source_file: None,
        };
        Self {
            mode: IlMode::Binary { program, current_type: None, current_method: None },
            function_mapper: FunctionMapper::new(),
            field_types: HashMap::new(),
        }
    }

    fn start_class(&mut self, name: &str) -> Result<()> {
        match &mut self.mode {
            IlMode::Text(writer) => writer.start_class(name),
            IlMode::Binary { current_type, .. } => {
                let clr_type = ClrType {
                    name: name.to_string(),
                    namespace: None,
                    access_flags: ClrAccessFlags {
                        is_public: true,
                        is_private: false,
                        is_security_transparent: false,
                        is_retargetable: false,
                    },
                    base_type: Some("System.Object".to_string()),
                    interfaces: vec![],
                    fields: vec![],
                    methods: vec![],
                    properties: vec![],
                    events: vec![],
                    nested_types: vec![],
                    attributes: vec![],
                };
                *current_type = Some(clr_type);
                Ok(())
            }
        }
    }

    fn end_class(&mut self) -> Result<()> {
        match &mut self.mode {
            IlMode::Text(writer) => writer.end_class(),
            IlMode::Binary { program, current_type, .. } => {
                if let Some(t) = current_type.take() {
                    program.types.push(t);
                }
                Ok(())
            }
        }
    }

    fn write_field(&mut self, name: &str, gaia_type: &GaiaType) -> Result<()> {
        match &mut self.mode {
            IlMode::Text(writer) => writer.write_field(name, &gaia_type_to_msil_name(gaia_type)),
            IlMode::Binary { current_type, .. } => {
                if let Some(t) = current_type {
                    t.fields.push(ClrField {
                        name: name.to_string(),
                        field_type: gaia_type_to_clr_type(gaia_type),
                        access_flags: ClrAccessFlags {
                            is_public: true,
                            is_private: false,
                            is_security_transparent: false,
                            is_retargetable: false,
                        },
                        default_value: None,
                        attributes: vec![],
                    });
                }
                Ok(())
            }
        }
    }

    fn emit_assembly_declaration(&mut self, name: &str) -> Result<()> {
        match &mut self.mode {
            IlMode::Text(writer) => writer.write_assembly(name),
            IlMode::Binary { .. } => Ok(()), // Already set in constructor
        }
    }

    fn emit_external_assembly(&mut self, name: &str) -> Result<()> {
        match &mut self.mode {
            IlMode::Text(writer) => writer.write_external_assembly(name),
            IlMode::Binary { program, .. } => {
                // 检查是否已经存在
                if !program.external_assemblies.iter().any(|a| a.name == name) {
                    program.external_assemblies.push(ClrExternalAssembly {
                        name: name.to_string(),
                        version: ClrVersion { major: 0, minor: 0, build: 0, revision: 0 },
                        public_key_token: None,
                        culture: None,
                        hash_algorithm: None,
                    });
                }
                Ok(())
            }
        }
    }

    fn start_method(
        &mut self,
        name: &str,
        parameters: &[GaiaType],
        return_type: &Option<GaiaType>,
        is_entry: bool,
    ) -> Result<()> {
        match &mut self.mode {
            IlMode::Text(writer) => {
                let param_names: Vec<String> = parameters.iter().map(|t| gaia_type_to_msil_name(t)).collect();
                let param_refs: Vec<&str> = param_names.iter().map(|s| s.as_str()).collect();
                let ret_name_owned = return_type.as_ref().map(|t| gaia_type_to_msil_name(t));
                let ret_name = ret_name_owned.as_deref();
                writer.start_method(name, &param_refs, ret_name, is_entry)
            }
            IlMode::Binary { current_method, .. } => {
                let clr_params = parameters
                    .iter()
                    .enumerate()
                    .map(|(i, t)| ClrParameter {
                        name: format!("arg{}", i),
                        parameter_type: gaia_type_to_clr_type(t),
                        is_in: true,
                        is_out: false,
                        is_optional: false,
                        default_value: None,
                        attributes: vec![],
                    })
                    .collect();

                let method = ClrMethod {
                    name: name.to_string(),
                    return_type: return_type.as_ref().map(gaia_type_to_clr_type).unwrap_or(ClrTypeReference {
                        name: "void".to_string(),
                        namespace: None,
                        assembly: None,
                        is_value_type: false,
                        is_reference_type: false,
                        generic_parameters: vec![],
                    }),
                    parameters: clr_params,
                    access_flags: ClrAccessFlags {
                        is_public: true,
                        is_private: false,
                        is_security_transparent: false,
                        is_retargetable: false,
                    },
                    impl_flags: ClrMethodImplFlags { is_managed: true, ..Default::default() },
                    instructions: vec![],
                    max_stack: 8u16,
                    locals: vec![],
                    exception_handlers: vec![],
                    attributes: vec![],
                    is_entry_point: is_entry,
                };
                *current_method = Some(method);
                Ok(())
            }
        }
    }

    fn end_method(&mut self) -> Result<()> {
        match &mut self.mode {
            IlMode::Text(writer) => writer.end_method(),
            IlMode::Binary { program, current_type, current_method } => {
                if let Some(method) = current_method.take() {
                    if let Some(t) = current_type {
                        t.methods.push(method);
                    }
                    else {
                        program.global_methods.push(method);
                    }
                }
                Ok(())
            }
        }
    }

    fn generate_bytecode(self) -> Result<Vec<u8>> {
        match self.mode {
            IlMode::Text(writer) => Ok(writer.finish().into_bytes()),
            IlMode::Binary { .. } => Err(GaiaError::custom_error("Use convert_to_clr_program for binary mode")),
        }
    }

    fn finish_binary(self) -> Result<ClrProgram> {
        match self.mode {
            IlMode::Binary { program, .. } => Ok(program),
            _ => Err(GaiaError::custom_error("Use generate_bytecode for text mode")),
        }
    }

    /// 根据当前上下文映射函数名（IL 目标）
    fn map_function(&self, raw_name: &str) -> String {
        let il_target = CompilationTarget {
            build: Architecture::CLR,
            host: AbiCompatible::MicrosoftIntermediateLanguage,
            target: ApiCompatible::ClrRuntime(4),
        };
        self.function_mapper.map_function(&il_target, raw_name).unwrap_or(raw_name).to_string()
    }

    fn emit_ldc_i4(&mut self, value: i32) -> Result<()> {
        self.emit_with_immediate(ClrOpcode::LdcI4, value)
    }

    fn emit_ldc_i8(&mut self, value: i64) -> Result<()> {
        match &mut self.mode {
            IlMode::Text(writer) => writer.emit_ldc_i8(value),
            IlMode::Binary { current_method, .. } => {
                if let Some(method) = current_method {
                    method.instructions.push(ClrInstruction::WithImmediate64 { opcode: ClrOpcode::LdcI8, value });
                    Ok(())
                }
                else {
                    Err(GaiaError::custom_error("No current method"))
                }
            }
        }
    }

    fn emit_ldc_r4(&mut self, value: f32) -> Result<()> {
        match &mut self.mode {
            IlMode::Text(writer) => writer.emit_ldc_r4(value),
            IlMode::Binary { current_method, .. } => {
                if let Some(method) = current_method {
                    method.instructions.push(ClrInstruction::WithFloat32 { opcode: ClrOpcode::LdcR4, value });
                    Ok(())
                }
                else {
                    Err(GaiaError::custom_error("No current method"))
                }
            }
        }
    }

    fn emit_ldc_r8(&mut self, value: f64) -> Result<()> {
        match &mut self.mode {
            IlMode::Text(writer) => writer.emit_ldc_r8(value),
            IlMode::Binary { current_method, .. } => {
                if let Some(method) = current_method {
                    method.instructions.push(ClrInstruction::WithFloat64 { opcode: ClrOpcode::LdcR8, value });
                    Ok(())
                }
                else {
                    Err(GaiaError::custom_error("No current method"))
                }
            }
        }
    }

    fn emit_ldstr(&mut self, value: &str) -> Result<()> {
        match &mut self.mode {
            IlMode::Text(writer) => writer.emit_ldstr(value),
            IlMode::Binary { current_method, .. } => {
                if let Some(method) = current_method {
                    method.instructions.push(ClrInstruction::WithString { opcode: ClrOpcode::Ldstr, value: value.to_string() });
                    Ok(())
                }
                else {
                    Err(GaiaError::invalid_data("No current method context for binary IL generation"))
                }
            }
        }
    }

    fn emit_ldnull(&mut self) -> Result<()> {
        self.emit_simple(ClrOpcode::Ldnull)
    }

    fn emit_ldloc(&mut self, index: u32) -> Result<()> {
        match &mut self.mode {
            IlMode::Text(writer) => writer.emit_ldloc(index),
            IlMode::Binary { current_method, .. } => {
                if let Some(method) = current_method {
                    method.instructions.push(ClrInstruction::WithLocalVar { opcode: ClrOpcode::Ldloc, index: index as u16 });
                    Ok(())
                }
                else {
                    Err(GaiaError::custom_error("No current method"))
                }
            }
        }
    }

    fn emit_stloc(&mut self, index: u32) -> Result<()> {
        match &mut self.mode {
            IlMode::Text(writer) => writer.emit_stloc(index),
            IlMode::Binary { current_method, .. } => {
                if let Some(method) = current_method {
                    method.instructions.push(ClrInstruction::WithLocalVar { opcode: ClrOpcode::Stloc, index: index as u16 });
                    Ok(())
                }
                else {
                    Err(GaiaError::custom_error("No current method"))
                }
            }
        }
    }

    fn emit_ldarg(&mut self, index: u32) -> Result<()> {
        match &mut self.mode {
            IlMode::Text(writer) => writer.emit_ldarg(index),
            IlMode::Binary { current_method, .. } => {
                if let Some(method) = current_method {
                    method.instructions.push(ClrInstruction::WithParameter { opcode: ClrOpcode::Ldarg, index: index as u16 });
                    Ok(())
                }
                else {
                    Err(GaiaError::custom_error("No current method"))
                }
            }
        }
    }

    fn emit_starg(&mut self, index: u32) -> Result<()> {
        match &mut self.mode {
            IlMode::Text(writer) => writer.emit_starg(index),
            IlMode::Binary { current_method, .. } => {
                if let Some(method) = current_method {
                    method.instructions.push(ClrInstruction::WithParameter { opcode: ClrOpcode::Starg, index: index as u16 });
                    Ok(())
                }
                else {
                    Err(GaiaError::custom_error("No current method"))
                }
            }
        }
    }

    fn emit_add(&mut self) -> Result<()> {
        self.emit_simple(ClrOpcode::Add)
    }

    fn emit_sub(&mut self) -> Result<()> {
        self.emit_simple(ClrOpcode::Sub)
    }

    fn emit_mul(&mut self) -> Result<()> {
        self.emit_simple(ClrOpcode::Mul)
    }

    fn emit_div(&mut self) -> Result<()> {
        self.emit_simple(ClrOpcode::Div)
    }

    fn emit_ceq(&mut self) -> Result<()> {
        self.emit_simple(ClrOpcode::Ceq)
    }

    fn emit_cgt(&mut self) -> Result<()> {
        self.emit_simple(ClrOpcode::Cgt)
    }

    fn emit_clt(&mut self) -> Result<()> {
        self.emit_simple(ClrOpcode::Clt)
    }

    fn emit_br(&mut self, label: &str) -> Result<()> {
        self.emit_branch(ClrOpcode::Br, label)
    }

    fn emit_brtrue(&mut self, label: &str) -> Result<()> {
        self.emit_branch(ClrOpcode::Brtrue, label)
    }

    fn emit_brfalse(&mut self, label: &str) -> Result<()> {
        self.emit_branch(ClrOpcode::Brfalse, label)
    }

    fn emit_call(&mut self, method_ref: &str) -> Result<()> {
        match &mut self.mode {
            IlMode::Text(writer) => writer.emit_call(method_ref),
            IlMode::Binary { current_method, .. } => {
                if let Some(method) = current_method {
                    method
                        .instructions
                        .push(ClrInstruction::WithMethod { opcode: ClrOpcode::Call, method_ref: method_ref.to_string() });
                    Ok(())
                }
                else {
                    Err(GaiaError::custom_error("No current method"))
                }
            }
        }
    }

    fn emit_ret(&mut self) -> Result<()> {
        self.emit_simple(ClrOpcode::Ret)
    }

    fn define_label(&mut self, name: &str) -> Result<()> {
        match &mut self.mode {
            IlMode::Text(writer) => writer.define_label(name),
            IlMode::Binary { .. } => Ok(()), // Binary mode usually resolves labels later
        }
    }

    fn emit_dup(&mut self) -> Result<()> {
        self.emit_simple(ClrOpcode::Dup)
    }

    fn emit_pop(&mut self) -> Result<()> {
        self.emit_simple(ClrOpcode::Pop)
    }

    fn emit_newarr(&mut self, type_name: &str) -> Result<()> {
        match &mut self.mode {
            IlMode::Text(writer) => writer.emit_newarr(type_name),
            IlMode::Binary { current_method, .. } => {
                if let Some(method) = current_method {
                    method
                        .instructions
                        .push(ClrInstruction::WithType { opcode: ClrOpcode::Newarr, type_ref: type_name.to_string() });
                    Ok(())
                }
                else {
                    Err(GaiaError::custom_error("No current method"))
                }
            }
        }
    }

    fn emit_ldelem(&mut self, type_name: &str) -> Result<()> {
        match &mut self.mode {
            IlMode::Text(writer) => writer.emit_ldelem(type_name),
            IlMode::Binary { current_method, .. } => {
                if let Some(method) = current_method {
                    method
                        .instructions
                        .push(ClrInstruction::WithType { opcode: ClrOpcode::Ldelem, type_ref: type_name.to_string() });
                    Ok(())
                }
                else {
                    Err(GaiaError::custom_error("No current method"))
                }
            }
        }
    }

    fn emit_stelem(&mut self, type_name: &str) -> Result<()> {
        match &mut self.mode {
            IlMode::Text(writer) => writer.emit_stelem(type_name),
            IlMode::Binary { current_method, .. } => {
                if let Some(method) = current_method {
                    method
                        .instructions
                        .push(ClrInstruction::WithType { opcode: ClrOpcode::Stelem, type_ref: type_name.to_string() });
                    Ok(())
                }
                else {
                    Err(GaiaError::custom_error("No current method"))
                }
            }
        }
    }

    fn emit_ldlen(&mut self) -> Result<()> {
        self.emit_simple(ClrOpcode::Ldlen)
    }

    fn emit_and(&mut self) -> Result<()> {
        self.emit_simple(ClrOpcode::And)
    }

    fn emit_or(&mut self) -> Result<()> {
        self.emit_simple(ClrOpcode::Or)
    }

    fn emit_ldloca(&mut self, index: u32) -> Result<()> {
        match &mut self.mode {
            IlMode::Text(writer) => writer.emit_ldloca(index),
            IlMode::Binary { current_method, .. } => {
                if let Some(method) = current_method {
                    method.instructions.push(ClrInstruction::WithLocalVar { opcode: ClrOpcode::Ldloca, index: index as u16 });
                    Ok(())
                }
                else {
                    Err(GaiaError::custom_error("No current method"))
                }
            }
        }
    }

    fn emit_ldind_i4(&mut self) -> Result<()> {
        self.emit_simple(ClrOpcode::LdindI4)
    }

    fn emit_ldind_i8(&mut self) -> Result<()> {
        self.emit_simple(ClrOpcode::LdindI8)
    }

    fn emit_ldind_r4(&mut self) -> Result<()> {
        self.emit_simple(ClrOpcode::LdindR4)
    }

    fn emit_ldind_r8(&mut self) -> Result<()> {
        self.emit_simple(ClrOpcode::LdindR8)
    }

    fn emit_ldind_ref(&mut self) -> Result<()> {
        self.emit_simple(ClrOpcode::LdindRef)
    }

    fn emit_stind_i4(&mut self) -> Result<()> {
        self.emit_simple(ClrOpcode::StindI4)
    }

    fn emit_stind_i8(&mut self) -> Result<()> {
        self.emit_simple(ClrOpcode::StindI8)
    }

    fn emit_stind_r4(&mut self) -> Result<()> {
        self.emit_simple(ClrOpcode::StindR4)
    }

    fn emit_stind_r8(&mut self) -> Result<()> {
        self.emit_simple(ClrOpcode::StindR8)
    }

    fn emit_stind_ref(&mut self) -> Result<()> {
        self.emit_simple(ClrOpcode::StindRef)
    }

    fn emit_newobj(&mut self, method_ref: &str) -> Result<()> {
        match &mut self.mode {
            IlMode::Text(writer) => writer.emit_newobj(method_ref),
            IlMode::Binary { current_method, .. } => {
                if let Some(method) = current_method {
                    method
                        .instructions
                        .push(ClrInstruction::WithMethod { opcode: ClrOpcode::Newobj, method_ref: method_ref.to_string() });
                    Ok(())
                }
                else {
                    Err(GaiaError::custom_error("No current method"))
                }
            }
        }
    }

    fn emit_ldfld(&mut self, field_ref: &str) -> Result<()> {
        match &mut self.mode {
            IlMode::Text(writer) => writer.emit_ldfld(field_ref),
            IlMode::Binary { current_method, .. } => {
                if let Some(method) = current_method {
                    method
                        .instructions
                        .push(ClrInstruction::WithField { opcode: ClrOpcode::Ldfld, field_ref: field_ref.to_string() });
                    Ok(())
                }
                else {
                    Err(GaiaError::custom_error("No current method"))
                }
            }
        }
    }

    fn emit_stfld(&mut self, field_ref: &str) -> Result<()> {
        match &mut self.mode {
            IlMode::Text(writer) => writer.emit_stfld(field_ref),
            IlMode::Binary { current_method, .. } => {
                if let Some(method) = current_method {
                    method
                        .instructions
                        .push(ClrInstruction::WithField { opcode: ClrOpcode::Stfld, field_ref: field_ref.to_string() });
                    Ok(())
                }
                else {
                    Err(GaiaError::custom_error("No current method"))
                }
            }
        }
    }

    fn emit_conv_i4(&mut self) -> Result<()> {
        self.emit_simple(ClrOpcode::ConvI4)
    }

    fn emit_conv_i8(&mut self) -> Result<()> {
        self.emit_simple(ClrOpcode::ConvI8)
    }

    fn emit_conv_r4(&mut self) -> Result<()> {
        self.emit_simple(ClrOpcode::ConvR4)
    }

    fn emit_conv_r8(&mut self) -> Result<()> {
        self.emit_simple(ClrOpcode::ConvR8)
    }

    fn emit_box(&mut self, gaia_type: &GaiaType) -> Result<()> {
        let type_name = gaia_type_to_msil_name(gaia_type);
        match &mut self.mode {
            IlMode::Text(writer) => writer.emit_box(&type_name),
            IlMode::Binary { current_method, .. } => {
                if let Some(method) = current_method {
                    method.instructions.push(ClrInstruction::WithType { opcode: ClrOpcode::Box, type_ref: type_name });
                    Ok(())
                }
                else {
                    Err(GaiaError::custom_error("No current method"))
                }
            }
        }
    }

    fn emit_unbox(&mut self, gaia_type: &GaiaType) -> Result<()> {
        let type_name = gaia_type_to_msil_name(gaia_type);
        match &mut self.mode {
            IlMode::Text(writer) => writer.emit_unbox(&type_name),
            IlMode::Binary { current_method, .. } => {
                if let Some(method) = current_method {
                    method.instructions.push(ClrInstruction::WithType { opcode: ClrOpcode::Unbox, type_ref: type_name });
                    Ok(())
                }
                else {
                    Err(GaiaError::custom_error("No current method"))
                }
            }
        }
    }

    fn emit_simple(&mut self, opcode: ClrOpcode) -> Result<()> {
        match &mut self.mode {
            IlMode::Text(writer) => match opcode {
                ClrOpcode::Add => writer.emit_add(),
                ClrOpcode::Sub => writer.emit_sub(),
                ClrOpcode::Mul => writer.emit_mul(),
                ClrOpcode::Div => writer.emit_div(),
                ClrOpcode::Ceq => writer.emit_ceq(),
                ClrOpcode::Cgt => writer.emit_cgt(),
                ClrOpcode::Clt => writer.emit_clt(),
                ClrOpcode::Ret => writer.emit_ret(),
                ClrOpcode::Dup => writer.emit_dup(),
                ClrOpcode::Pop => writer.emit_pop(),
                ClrOpcode::And => writer.emit_and(),
                ClrOpcode::Or => writer.emit_or(),
                ClrOpcode::Ldnull => writer.emit_ldnull(),
                ClrOpcode::LdindI4 => writer.emit_ldind_i4(),
                ClrOpcode::LdindI8 => writer.emit_ldind_i8(),
                ClrOpcode::LdindR4 => writer.emit_ldind_r4(),
                ClrOpcode::LdindR8 => writer.emit_ldind_r8(),
                ClrOpcode::LdindRef => writer.emit_ldind_ref(),
                ClrOpcode::StindI4 => writer.emit_stind_i4(),
                ClrOpcode::StindI8 => writer.emit_stind_i8(),
                ClrOpcode::StindR4 => writer.emit_stind_r4(),
                ClrOpcode::StindR8 => writer.emit_stind_r8(),
                ClrOpcode::StindRef => writer.emit_stind_ref(),
                ClrOpcode::ConvI4 => writer.emit_conv_i4(),
                ClrOpcode::ConvI8 => writer.emit_conv_i8(),
                ClrOpcode::ConvR4 => writer.emit_conv_r4(),
                ClrOpcode::ConvR8 => writer.emit_conv_r8(),
                ClrOpcode::Ldlen => writer.emit_ldlen(),
                _ => Err(GaiaError::custom_error(format!("Unsupported simple opcode: {:?}", opcode))),
            },
            IlMode::Binary { current_method, .. } => {
                if let Some(method) = current_method {
                    method.instructions.push(ClrInstruction::Simple { opcode });
                    Ok(())
                }
                else {
                    Err(GaiaError::custom_error("No current method"))
                }
            }
        }
    }

    fn emit_with_immediate(&mut self, opcode: ClrOpcode, value: i32) -> Result<()> {
        match &mut self.mode {
            IlMode::Text(writer) => match opcode {
                ClrOpcode::LdcI4 => writer.emit_ldc_i4(value),
                _ => Err(GaiaError::custom_error(format!("Unsupported immediate opcode: {:?}", opcode))),
            },
            IlMode::Binary { current_method, .. } => {
                if let Some(method) = current_method {
                    method.instructions.push(ClrInstruction::WithImmediate { opcode, value });
                    Ok(())
                }
                else {
                    Err(GaiaError::custom_error("No current method"))
                }
            }
        }
    }

    fn emit_branch(&mut self, opcode: ClrOpcode, label: &str) -> Result<()> {
        match &mut self.mode {
            IlMode::Text(writer) => match opcode {
                ClrOpcode::Br => writer.emit_br(label),
                ClrOpcode::Brtrue => writer.emit_brtrue(label),
                ClrOpcode::Brfalse => writer.emit_brfalse(label),
                _ => Err(GaiaError::custom_error(format!("Unsupported branch opcode: {:?}", opcode))),
            },
            IlMode::Binary { current_method, .. } => {
                if let Some(method) = current_method {
                    method.instructions.push(ClrInstruction::WithLabel { opcode, label: label.to_string() });
                    Ok(())
                }
                else {
                    Err(GaiaError::custom_error("No current method"))
                }
            }
        }
    }
}

#[cfg(feature = "clr")]
fn gaia_type_to_clr_type(t: &GaiaType) -> ClrTypeReference {
    match t {
        GaiaType::Array(elem, _) => {
            let mut elem_ref = gaia_type_to_clr_type(elem);
            elem_ref.name = format!("{}[]", elem_ref.name);
            elem_ref.is_value_type = false;
            elem_ref.is_reference_type = true;
            elem_ref
        }
        _ => {
            let name = match t {
                GaiaType::I8 => "int8",
                GaiaType::U8 => "uint8",
                GaiaType::I16 => "int16",
                GaiaType::U16 => "uint16",
                GaiaType::I32 => "int32",
                GaiaType::U32 => "uint32",
                GaiaType::I64 => "int64",
                GaiaType::U64 => "uint64",
                GaiaType::F32 => "float32",
                GaiaType::F64 => "float64",
                GaiaType::String => "string",
                GaiaType::Bool => "bool",
                GaiaType::Void => "void",
                GaiaType::Class(name) => name,
                GaiaType::Struct(name) => name,
                _ => "object",
            };
            ClrTypeReference {
                name: name.to_string(),
                namespace: None,
                assembly: None,
                is_value_type: !matches!(t, GaiaType::String | GaiaType::Object | GaiaType::Array(_, _) | GaiaType::Class(_)),
                is_reference_type: matches!(
                    t,
                    GaiaType::String | GaiaType::Object | GaiaType::Array(_, _) | GaiaType::Class(_)
                ),
                generic_parameters: vec![],
            }
        }
    }
}

#[cfg(feature = "clr")]
fn convert_to_clr_program(program: &GaiaModule, settings: Option<&GaiaSettings>) -> Result<ClrProgram> {
    let mut context = IlContext::new_binary(program.name.clone());
    if let Some(s) = settings {
        context.function_mapper = FunctionMapper::from_config(s).unwrap_or_default();
    }
    compile_program(&mut context, program)?;
    context.finish_binary()
}
