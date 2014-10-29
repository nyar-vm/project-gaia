//! IL (Intermediate Language) backend compiler
use super::{Backend, FunctionMapper, TargetPlatform};
use crate::instruction::*;
use clr_msil::writer::MsilWriter;
use gaia_types::*;
use std::io::Cursor;

/// IL Backend implementation
pub struct MsilBackend {}

impl Backend for MsilBackend {
    const IS_BINARY: bool = false;
    fn compile(program: &GaiaProgram) -> Result<Vec<u8>> {
        // Create IL assembler context
        let mut il_context = create_il_context()?;

        // Compile program
        compile_program(&mut il_context, program)?;

        // Generate IL bytecode
        generate_il_bytecode(il_context)
    }

    fn name() -> &'static str {
        "MSIL"
    }

    fn file_extension() -> &'static str {
        "msil"
    }
}

/// Compile Gaia program to IL bytecode
pub fn compile(program: &GaiaProgram) -> Result<Vec<u8>> {
    MsilBackend::compile(program)
}

/// Create IL assembler context
fn create_il_context() -> Result<IlContext> {
    Ok(IlContext::new())
}

/// Compile entire program
fn compile_program(context: &mut IlContext, program: &GaiaProgram) -> Result<()> {
    // 添加程序集声明
    context.emit_assembly_declaration(&program.name)?;

    // Compile all functions
    for function in &program.functions {
        compile_function(context, function)?;
    }

    Ok(())
}

/// Compile single function
fn compile_function(context: &mut IlContext, function: &GaiaFunction) -> Result<()> {
    // 为 main 函数进行特殊处理：如果有返回指令且没有显式返回类型，推断为 int32
    let inferred_return_type = if function.name == "main" && function.return_type.is_none() {
        // 检查是否有返回指令
        let has_return = function.instructions.iter().any(|inst| matches!(inst, GaiaInstruction::Return));
        if has_return {
            Some(GaiaType::Integer32)
        }
        else {
            function.return_type.clone()
        }
    }
    else {
        function.return_type.clone()
    };

    // Start function definition
    start_function(context, &function.name, &function.parameters, &inferred_return_type)?;

    // Compile instructions
    for instruction in &function.instructions {
        compile_instruction(context, instruction)?;
    }

    // End function definition
    end_function(context)?;

    Ok(())
}

/// Compile single instruction
fn compile_instruction(context: &mut IlContext, instruction: &GaiaInstruction) -> Result<()> {
    match instruction {
        GaiaInstruction::LoadConstant(constant) => compile_load_constant(context, constant),
        GaiaInstruction::LoadLocal(index) => compile_load_local(context, *index),
        GaiaInstruction::StoreLocal(index) => compile_store_local(context, *index),
        GaiaInstruction::LoadArgument(index) => compile_load_argument(context, *index),
        GaiaInstruction::Add => compile_add(context),
        GaiaInstruction::Subtract => compile_subtract(context),
        GaiaInstruction::Multiply => compile_multiply(context),
        GaiaInstruction::Divide => compile_divide(context),
        GaiaInstruction::CompareEqual => compile_equal(context),
        GaiaInstruction::CompareNotEqual => compile_not_equal(context),
        GaiaInstruction::CompareLessThan => compile_less_than(context),
        GaiaInstruction::CompareGreaterThan => compile_greater_than(context),
        GaiaInstruction::Branch(label) => compile_branch(context, label),
        GaiaInstruction::BranchIfTrue(label) => compile_branch_if_true(context, label),
        GaiaInstruction::BranchIfFalse(label) => compile_branch_if_false(context, label),
        GaiaInstruction::Call(function_name) => compile_call(context, function_name),
        GaiaInstruction::Return => compile_return(context),
        GaiaInstruction::Label(name) => compile_label(context, name),
        GaiaInstruction::Duplicate => compile_duplicate(context),
        GaiaInstruction::Pop => compile_pop(context),
        GaiaInstruction::LoadAddress(addr) => compile_load_address(context, *addr),
        GaiaInstruction::LoadIndirect(gaia_type) => compile_load_indirect(context, gaia_type),
        GaiaInstruction::StoreIndirect(gaia_type) => compile_store_indirect(context, gaia_type),
        GaiaInstruction::Convert(from_type, to_type) => compile_convert(context, from_type, to_type),
        GaiaInstruction::Box(gaia_type) => compile_box(context, gaia_type),
        GaiaInstruction::Unbox(gaia_type) => compile_unbox(context, gaia_type),
        _ => Ok(()), // 忽略其他指令
    }
}

fn compile_load_constant(context: &mut IlContext, constant: &GaiaConstant) -> Result<()> {
    match constant {
        GaiaConstant::Integer8(value) => context.emit_ldc_i4(*value as i32),
        GaiaConstant::Integer16(value) => context.emit_ldc_i4(*value as i32),
        GaiaConstant::Integer32(value) => context.emit_ldc_i4(*value),
        GaiaConstant::Integer64(value) => context.emit_ldc_i8(*value),
        GaiaConstant::Float32(value) => context.emit_ldc_r4(*value),
        GaiaConstant::Float64(value) => context.emit_ldc_r8(*value),
        GaiaConstant::String(value) => context.emit_ldstr(value),
        GaiaConstant::Boolean(value) => context.emit_ldc_i4(if *value { 1 } else { 0 }),
        GaiaConstant::Null => context.emit_ldnull(),
    }
}

fn compile_load_local(context: &mut IlContext, index: u32) -> Result<()> {
    context.emit_ldloc(index)
}

fn compile_store_local(context: &mut IlContext, index: u32) -> Result<()> {
    context.emit_stloc(index)
}

fn compile_load_argument(context: &mut IlContext, index: u32) -> Result<()> {
    context.emit_ldarg(index)
}

fn compile_add(context: &mut IlContext) -> Result<()> {
    context.emit_add()
}

fn compile_subtract(context: &mut IlContext) -> Result<()> {
    context.emit_sub()
}

fn compile_multiply(context: &mut IlContext) -> Result<()> {
    context.emit_mul()
}

fn compile_divide(context: &mut IlContext) -> Result<()> {
    context.emit_div()
}

fn compile_equal(context: &mut IlContext) -> Result<()> {
    context.emit_ceq()
}

fn compile_not_equal(context: &mut IlContext) -> Result<()> {
    context.emit_ceq()?;
    context.emit_ldc_i4(0)?;
    context.emit_ceq()
}

fn compile_less_than(context: &mut IlContext) -> Result<()> {
    context.emit_clt()
}

fn compile_greater_than(context: &mut IlContext) -> Result<()> {
    context.emit_cgt()
}

fn compile_branch(context: &mut IlContext, label: &str) -> Result<()> {
    context.emit_br(label)
}

fn compile_branch_if_true(context: &mut IlContext, label: &str) -> Result<()> {
    context.emit_brtrue(label)
}

fn compile_branch_if_false(context: &mut IlContext, label: &str) -> Result<()> {
    context.emit_brfalse(label)
}

fn compile_call(context: &mut IlContext, function_name: &str) -> Result<()> {
    let mapper = FunctionMapper::new();
    let mapped_name = mapper.map_function(function_name, TargetPlatform::IL);
    context.emit_call(&mapped_name)
}

fn compile_return(context: &mut IlContext) -> Result<()> {
    context.emit_ret()
}

fn compile_label(context: &mut IlContext, name: &str) -> Result<()> {
    context.define_label(name)
}

fn compile_duplicate(context: &mut IlContext) -> Result<()> {
    context.emit_dup()
}

fn compile_pop(context: &mut IlContext) -> Result<()> {
    context.emit_pop()
}

fn compile_load_address(context: &mut IlContext, addr: u32) -> Result<()> {
    context.emit_ldloca(addr)
}

fn compile_load_indirect(context: &mut IlContext, gaia_type: &GaiaType) -> Result<()> {
    match gaia_type {
        GaiaType::Integer8 => context.emit_ldind_i4(),  // 8位整数加载为32位
        GaiaType::Integer16 => context.emit_ldind_i4(), // 16位整数加载为32位
        GaiaType::Integer32 => context.emit_ldind_i4(),
        GaiaType::Integer64 => context.emit_ldind_i8(),
        GaiaType::Float32 => context.emit_ldind_r4(),
        GaiaType::Float64 => context.emit_ldind_r8(),
        _ => context.emit_ldind_ref(),
    }
}

fn compile_store_indirect(context: &mut IlContext, gaia_type: &GaiaType) -> Result<()> {
    match gaia_type {
        GaiaType::Integer8 => context.emit_stind_i4(),  // 8位整数存储为32位
        GaiaType::Integer16 => context.emit_stind_i4(), // 16位整数存储为32位
        GaiaType::Integer32 => context.emit_stind_i4(),
        GaiaType::Integer64 => context.emit_stind_i8(),
        GaiaType::Float32 => context.emit_stind_r4(),
        GaiaType::Float64 => context.emit_stind_r8(),
        _ => context.emit_stind_ref(),
    }
}

fn compile_convert(context: &mut IlContext, from_type: &GaiaType, to_type: &GaiaType) -> Result<()> {
    match to_type {
        GaiaType::Integer8 => context.emit_conv_i4(),  // 8位整数转换为32位
        GaiaType::Integer16 => context.emit_conv_i4(), // 16位整数转换为32位
        GaiaType::Integer32 => context.emit_conv_i4(),
        GaiaType::Integer64 => context.emit_conv_i8(),
        GaiaType::Float32 => context.emit_conv_r4(),
        GaiaType::Float64 => context.emit_conv_r8(),
        _ => Ok(()),
    }
}

fn compile_box(context: &mut IlContext, gaia_type: &GaiaType) -> Result<()> {
    context.emit_box(gaia_type)
}

fn compile_unbox(context: &mut IlContext, gaia_type: &GaiaType) -> Result<()> {
    context.emit_unbox(gaia_type)
}

/// Start function definition
fn start_function(context: &mut IlContext, name: &str, parameters: &[GaiaType], return_type: &Option<GaiaType>) -> Result<()> {
    context.start_method(name, parameters, return_type)
}

fn end_function(context: &mut IlContext) -> Result<()> {
    context.end_method()
}

/// Generate IL bytecode
fn generate_il_bytecode(context: IlContext) -> Result<Vec<u8>> {
    context.generate_bytecode()
}

/// IL Context for code generation
struct IlContext {
    writer: MsilWriter<Cursor<Vec<u8>>>,
}

impl IlContext {
    fn new() -> Self {
        let buffer = Cursor::new(Vec::new());
        Self { writer: MsilWriter::new(buffer) }
    }

    fn emit_assembly_declaration(&mut self, name: &str) -> Result<()> {
        self.writer.write_assembly(name)
    }

    fn start_method(&mut self, name: &str, parameters: &[GaiaType], return_type: &Option<GaiaType>) -> Result<()> {
        self.writer.start_method(name, parameters, return_type)
    }

    fn end_method(&mut self) -> Result<()> {
        self.writer.end_method()
    }

    fn generate_bytecode(self) -> Result<Vec<u8>> {
        // 获取写入器中的内容
        let buffer = self.writer.finish().into_inner();
        Ok(buffer)
    }

    fn emit_ldc_i4(&mut self, value: i32) -> Result<()> {
        self.writer.emit_ldc_i4(value)
    }

    fn emit_ldc_i8(&mut self, value: i64) -> Result<()> {
        self.writer.emit_ldc_i8(value)
    }

    fn emit_ldc_r4(&mut self, value: f32) -> Result<()> {
        self.writer.emit_ldc_r4(value)
    }

    fn emit_ldc_r8(&mut self, value: f64) -> Result<()> {
        self.writer.emit_ldc_r8(value)
    }

    fn emit_ldstr(&mut self, value: &str) -> Result<()> {
        self.writer.emit_ldstr(value)
    }

    fn emit_ldnull(&mut self) -> Result<()> {
        self.writer.emit_ldnull()
    }

    fn emit_ldloc(&mut self, index: u32) -> Result<()> {
        self.writer.emit_ldloc(index)
    }

    fn emit_stloc(&mut self, index: u32) -> Result<()> {
        self.writer.emit_stloc(index)
    }

    fn emit_ldarg(&mut self, index: u32) -> Result<()> {
        self.writer.emit_ldarg(index)
    }

    fn emit_add(&mut self) -> Result<()> {
        self.writer.emit_add()
    }

    fn emit_sub(&mut self) -> Result<()> {
        self.writer.emit_sub()
    }

    fn emit_mul(&mut self) -> Result<()> {
        self.writer.emit_mul()
    }

    fn emit_div(&mut self) -> Result<()> {
        self.writer.emit_div()
    }

    fn emit_ceq(&mut self) -> Result<()> {
        self.writer.emit_ceq()
    }

    fn emit_clt(&mut self) -> Result<()> {
        self.writer.emit_clt()
    }

    fn emit_cgt(&mut self) -> Result<()> {
        self.writer.emit_cgt()
    }

    fn emit_br(&mut self, label: &str) -> Result<()> {
        self.writer.emit_br(label)
    }

    fn emit_brtrue(&mut self, label: &str) -> Result<()> {
        self.writer.emit_brtrue(label)
    }

    fn emit_brfalse(&mut self, label: &str) -> Result<()> {
        self.writer.emit_brfalse(label)
    }

    fn emit_call(&mut self, function_name: &str) -> Result<()> {
        self.writer.emit_call(function_name)
    }

    fn emit_ret(&mut self) -> Result<()> {
        self.writer.emit_ret()
    }

    fn define_label(&mut self, name: &str) -> Result<()> {
        self.writer.define_label(name)
    }

    fn emit_dup(&mut self) -> Result<()> {
        self.writer.emit_dup()
    }

    fn emit_pop(&mut self) -> Result<()> {
        self.writer.emit_pop()
    }

    fn emit_ldloca(&mut self, addr: u32) -> Result<()> {
        self.writer.emit_ldloca(addr)
    }

    fn emit_ldind_i4(&mut self) -> Result<()> {
        self.writer.emit_ldind_i4()
    }

    fn emit_ldind_i8(&mut self) -> Result<()> {
        self.writer.emit_ldind_i8()
    }

    fn emit_ldind_r4(&mut self) -> Result<()> {
        self.writer.emit_ldind_r4()
    }

    fn emit_ldind_r8(&mut self) -> Result<()> {
        self.writer.emit_ldind_r8()
    }

    fn emit_ldind_ref(&mut self) -> Result<()> {
        self.writer.emit_ldind_ref()
    }

    fn emit_stind_i4(&mut self) -> Result<()> {
        self.writer.emit_stind_i4()
    }

    fn emit_stind_i8(&mut self) -> Result<()> {
        self.writer.emit_stind_i8()
    }

    fn emit_stind_r4(&mut self) -> Result<()> {
        self.writer.emit_stind_r4()
    }

    fn emit_stind_r8(&mut self) -> Result<()> {
        self.writer.emit_stind_r8()
    }

    fn emit_stind_ref(&mut self) -> Result<()> {
        self.writer.emit_stind_ref()
    }

    fn emit_conv_i4(&mut self) -> Result<()> {
        self.writer.emit_conv_i4()
    }

    fn emit_conv_i8(&mut self) -> Result<()> {
        self.writer.emit_conv_i8()
    }

    fn emit_conv_r4(&mut self) -> Result<()> {
        self.writer.emit_conv_r4()
    }

    fn emit_conv_r8(&mut self) -> Result<()> {
        self.writer.emit_conv_r8()
    }

    fn emit_box(&mut self, gaia_type: &GaiaType) -> Result<()> {
        let type_name = match gaia_type {
            GaiaType::Integer32 => "int32",
            GaiaType::Integer64 => "int64",
            GaiaType::Float32 => "float32",
            GaiaType::Float64 => "float64",
            GaiaType::Boolean => "bool",
            _ => "object",
        };
        self.writer.emit_box(type_name)
    }

    fn emit_unbox(&mut self, gaia_type: &GaiaType) -> Result<()> {
        let type_name = match gaia_type {
            GaiaType::Integer32 => "int32",
            GaiaType::Integer64 => "int64",
            GaiaType::Float32 => "float32",
            GaiaType::Float64 => "float64",
            GaiaType::Boolean => "bool",
            _ => "object",
        };
        self.writer.emit_unbox(type_name)
    }
}
