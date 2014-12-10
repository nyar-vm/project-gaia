//! WASI (WebAssembly System Interface) backend compiler

use super::{Backend, GeneratedFiles};
use crate::{
    config::GaiaConfig,
    instruction::{CmpCondition, CoreInstruction, GaiaInstruction, ManagedInstruction},
    program::{GaiaConstant, GaiaFunction, GaiaModule},
    types::GaiaType,
};
use gaia_types::{
    helpers::{AbiCompatible, ApiCompatible, Architecture, CompilationTarget},
    Result,
};
use std::collections::HashMap;
use wasi_assembler::{WasiFunctionType, WasiInstruction, WasiProgram, WasmValueType};

/// WASI Backend implementation
#[derive(Default)]
pub struct WasiBackend {}

impl Backend for WasiBackend {
    fn name(&self) -> &'static str {
        "WASI"
    }

    fn primary_target(&self) -> CompilationTarget {
        CompilationTarget {
            build: Architecture::WASM32,
            host: AbiCompatible::WebAssemblyTextFormat,
            target: ApiCompatible::WASI,
        }
    }

    fn match_score(&self, target: &CompilationTarget) -> f32 {
        match target.host {
            AbiCompatible::WebAssemblyTextFormat => 10.0,
            AbiCompatible::Unknown => match target.build {
                // wat output, 5% support
                Architecture::WASM32 => 5.0,
                Architecture::WASM64 => 0.0,
                _ => -100.0,
            },
            _ => -100.0,
        }
    }

    fn generate(&self, program: &GaiaModule, _config: &GaiaConfig) -> Result<GeneratedFiles> {
        let mut context = create_wasi_context()?;
        compile_program(&mut context, program)?;

        let wasi_program = context.program;

        let mut files = HashMap::new();

        // 1. 生成 .wasm
        files.insert("main.wasm".to_string(), wasi_program.to_wasm()?);

        // 2. 生成 .wat
        // if let Ok(wat_ast) = wasi_program.to_wat() {
        // use oak_core::source::ToSource;
        // files.insert("main.wat".to_string(), wat_ast.to_source_string().into_bytes());
        // }

        Ok(GeneratedFiles { files, diagnostics: vec![] })
    }
}

impl WasiBackend {
    /// Generate WASI WebAssembly bytecode from Gaia program
    pub fn generate(program: &GaiaModule) -> Result<Vec<u8>> {
        let mut context = create_wasi_context()?;
        compile_program(&mut context, program)?;
        context.program.to_wasm()
    }
}

/// Compile Gaia program to WASI WebAssembly
pub fn compile(program: &GaiaModule) -> Result<Vec<u8>> {
    WasiBackend::generate(program)
}

/// Create WASI assembler context
fn create_wasi_context() -> Result<WasiContext> {
    Ok(WasiContext::new())
}

/// Compile entire program
fn compile_program(context: &mut WasiContext, program: &GaiaModule) -> Result<()> {
    // 1. 添加必要的导入
    // console.log 映射到 WASI fd_write 或类似的
    // fd_write(fd: i32, iovs: i32, iovs_len: i32, nwritten: i32) -> i32
    context.add_import(
        "wasi_snapshot_preview1",
        "fd_write",
        vec![WasmValueType::I32, WasmValueType::I32, WasmValueType::I32, WasmValueType::I32],
        vec![WasmValueType::I32],
    );

    // 2. 编译所有函数
    for function in &program.functions {
        compile_function(context, function)?;
    }

    // 3. 导出 main 函数
    if !context.program.functions.is_empty() {
        let main_index = (context.program.imports.len() + context.program.functions.len() - 1) as u32;
        context.program.add_export(wasi_assembler::WasiExport {
            name: "main".to_string(),
            export_type: wasi_assembler::WasmExportType::Function { function_index: main_index },
        });
    }

    Ok(())
}

/// Compile single function
fn compile_function(context: &mut WasiContext, function: &GaiaFunction) -> Result<()> {
    let mut instrs = Vec::new();

    // 编译代码块
    for block in &function.blocks {
        // TODO: 处理标签

        for instruction in &block.instructions {
            compile_instruction(context, &mut instrs, instruction)?;
        }

        // 编译终结符
        match &block.terminator {
            crate::program::GaiaTerminator::Jump(_label) => instrs.push(WasiInstruction::Br { label_index: 0 }), // FIXME
            crate::program::GaiaTerminator::Branch { .. } => {
                instrs.push(WasiInstruction::BrIf { label_index: 0 }); // FIXME
            }
            crate::program::GaiaTerminator::Return => instrs.push(WasiInstruction::Return),
            crate::program::GaiaTerminator::Call { .. } => {
                // TODO: 查找函数索引
                instrs.push(WasiInstruction::Call { function_index: 0 });
            }
            crate::program::GaiaTerminator::Halt => {
                // WASI: exit(0)
                instrs.push(WasiInstruction::I32Const { value: 0 });
                // TODO: 查找 proc_exit 索引
                instrs.push(WasiInstruction::Call { function_index: 0 });
            }
        }
    }

    instrs.push(WasiInstruction::End);

    // 映射参数和返回值类型
    let params = function.signature.params.iter().map(|p| map_type(p)).collect();
    let results = match function.signature.return_type {
        GaiaType::Void => vec![],
        _ => vec![map_type(&function.signature.return_type)],
    };

    // 添加函数到程序
    context.add_function(instrs, params, results);

    Ok(())
}

fn map_type(ty: &GaiaType) -> WasmValueType {
    match ty {
        GaiaType::I32 => WasmValueType::I32,
        GaiaType::I64 => WasmValueType::I64,
        GaiaType::F32 => WasmValueType::F32,
        GaiaType::F64 => WasmValueType::F64,
        _ => WasmValueType::I32, // Fallback
    }
}

/// Compile single instruction
fn compile_instruction(
    _context: &mut WasiContext,
    instrs: &mut Vec<WasiInstruction>,
    instruction: &GaiaInstruction,
) -> Result<()> {
    match instruction {
        GaiaInstruction::Core(core) => match core {
            CoreInstruction::PushConstant(constant) => compile_load_constant(instrs, constant),
            CoreInstruction::Load(gaia_type) => compile_load_indirect(instrs, gaia_type),
            CoreInstruction::Store(gaia_type) => compile_store_indirect(instrs, gaia_type),
            CoreInstruction::Add(_) => {
                instrs.push(WasiInstruction::I32Add);
                Ok(())
            }
            CoreInstruction::Sub(_) => {
                instrs.push(WasiInstruction::I32Sub);
                Ok(())
            }
            CoreInstruction::Mul(_) => {
                instrs.push(WasiInstruction::I32Mul);
                Ok(())
            }
            CoreInstruction::Div(_) => {
                instrs.push(WasiInstruction::I32DivS);
                Ok(())
            }
            CoreInstruction::Rem(_) => {
                instrs.push(WasiInstruction::I32RemS);
                Ok(())
            }
            CoreInstruction::And(_) => {
                instrs.push(WasiInstruction::I32And);
                Ok(())
            }
            CoreInstruction::Or(_) => {
                instrs.push(WasiInstruction::I32Or);
                Ok(())
            }
            CoreInstruction::Xor(_) => {
                instrs.push(WasiInstruction::I32Xor);
                Ok(())
            }
            CoreInstruction::Shl(_) => {
                instrs.push(WasiInstruction::I32Shl);
                Ok(())
            }
            CoreInstruction::Shr(_) => {
                instrs.push(WasiInstruction::I32ShrS);
                Ok(())
            }
            CoreInstruction::Pop => {
                instrs.push(WasiInstruction::Drop);
                Ok(())
            }
            CoreInstruction::LoadLocal(index, _) => {
                instrs.push(WasiInstruction::LocalGet { local_index: *index });
                Ok(())
            }
            CoreInstruction::StoreLocal(index, _) => {
                instrs.push(WasiInstruction::LocalSet { local_index: *index });
                Ok(())
            }
            CoreInstruction::LoadArg(index, _) => {
                instrs.push(WasiInstruction::LocalGet { local_index: *index });
                Ok(())
            }
            CoreInstruction::StoreArg(index, _) => {
                instrs.push(WasiInstruction::LocalSet { local_index: *index });
                Ok(())
            }
            CoreInstruction::Ret => {
                instrs.push(WasiInstruction::Return);
                Ok(())
            }
            CoreInstruction::Call(_, _) => {
                // TODO: 查找函数索引
                instrs.push(WasiInstruction::Call { function_index: 0 });
                Ok(())
            }
            CoreInstruction::Cmp(cond, _) => compile_compare(instrs, cond),
            CoreInstruction::StructNew(_) => {
                // TODO: 查找类型索引
                instrs.push(WasiInstruction::StructNew { type_index: 0 });
                Ok(())
            }
            CoreInstruction::StructGet { field_index, .. } => {
                // TODO: 查找类型索引
                instrs.push(WasiInstruction::StructGet { type_index: 0, field_index: *field_index });
                Ok(())
            }
            CoreInstruction::StructSet { field_index, .. } => {
                // TODO: 查找类型索引
                instrs.push(WasiInstruction::StructSet { type_index: 0, field_index: *field_index });
                Ok(())
            }
            _ => Ok(()),
        },
        GaiaInstruction::Managed(managed) => match managed {
            ManagedInstruction::CallStatic { .. } => {
                // AOT 模式下静态方法调用映射为普通 call
                instrs.push(WasiInstruction::Call { function_index: 0 });
                Ok(())
            }
            _ => Ok(()),
        },
        _ => Ok(()),
    }
}

fn compile_compare(instrs: &mut Vec<WasiInstruction>, cond: &CmpCondition) -> Result<()> {
    match cond {
        CmpCondition::Eq => instrs.push(WasiInstruction::I32Eq),
        CmpCondition::Ne => instrs.push(WasiInstruction::I32Ne),
        CmpCondition::Lt => instrs.push(WasiInstruction::I32LtS),
        CmpCondition::Le => instrs.push(WasiInstruction::I32LeS),
        CmpCondition::Gt => instrs.push(WasiInstruction::I32GtS),
        CmpCondition::Ge => instrs.push(WasiInstruction::I32GeS),
    }
    Ok(())
}

fn compile_load_indirect(instrs: &mut Vec<WasiInstruction>, gaia_type: &GaiaType) -> Result<()> {
    // 假设内存偏移已经在栈顶
    match gaia_type {
        GaiaType::I32 => instrs.push(WasiInstruction::I32Load { offset: 0, align: 0 }),
        GaiaType::I64 => instrs.push(WasiInstruction::I64Load { offset: 0, align: 0 }),
        GaiaType::F32 => instrs.push(WasiInstruction::F32Load { offset: 0, align: 0 }),
        GaiaType::F64 => instrs.push(WasiInstruction::F64Load { offset: 0, align: 0 }),
        _ => {}
    }
    Ok(())
}

fn compile_store_indirect(instrs: &mut Vec<WasiInstruction>, gaia_type: &GaiaType) -> Result<()> {
    // 假设 [ptr, value] 已经在栈上
    match gaia_type {
        GaiaType::I32 => instrs.push(WasiInstruction::I32Store { offset: 0, align: 0 }),
        GaiaType::I64 => instrs.push(WasiInstruction::I64Store { offset: 0, align: 0 }),
        GaiaType::F32 => instrs.push(WasiInstruction::F32Store { offset: 0, align: 0 }),
        GaiaType::F64 => instrs.push(WasiInstruction::F64Store { offset: 0, align: 0 }),
        _ => {}
    }
    Ok(())
}

fn compile_load_constant(instrs: &mut Vec<WasiInstruction>, constant: &GaiaConstant) -> Result<()> {
    match constant {
        GaiaConstant::I32(value) => instrs.push(WasiInstruction::I32Const { value: *value }),
        GaiaConstant::I64(value) => instrs.push(WasiInstruction::I64Const { value: *value }),
        GaiaConstant::F32(value) => instrs.push(WasiInstruction::F32Const { value: *value }),
        GaiaConstant::F64(value) => instrs.push(WasiInstruction::F64Const { value: *value }),
        _ => {}
    }
    Ok(())
}

/// WASI assembler context
struct WasiContext {
    program: WasiProgram,
    /// 导入映射
    import_map: HashMap<String, u32>,
}

impl WasiContext {
    fn new() -> Self {
        WasiContext { program: WasiProgram::new_core_module(), import_map: HashMap::new() }
    }

    fn add_import(&mut self, module: &str, field: &str, params: Vec<WasmValueType>, results: Vec<WasmValueType>) -> u32 {
        let key = format!("{}.{}", module, field);
        if let Some(&index) = self.import_map.get(&key) {
            index
        }
        else {
            let index = self.import_map.len() as u32;
            let func_type = WasiFunctionType { params, results };
            let type_index = self.program.add_function_type(func_type);

            self.program.add_import(wasi_assembler::WasiImport {
                module: module.to_string(),
                field: field.to_string(),
                import_type: wasi_assembler::WasmImportType::Function { type_index },
            });

            self.import_map.insert(key, index);
            index
        }
    }

    fn add_function(&mut self, body: Vec<WasiInstruction>, params: Vec<WasmValueType>, results: Vec<WasmValueType>) {
        let func_type = WasiFunctionType { params, results };
        let type_index = self.program.add_function_type(func_type);
        let func = wasi_assembler::WasiFunction { type_index, locals: vec![], body };
        self.program.add_function(func);
    }
}
