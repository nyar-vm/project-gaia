#![doc = include_str!("readme.md")]

use crate::{formats::wat::ast::*, program::*};
use gaia_types::{GaiaDiagnostics, GaiaError, Result};
use std::collections::HashMap;

/// WAT 编译器，将 AST 转换为 Program
pub struct WatCompiler {
    /// 当前正在编译的程序
    program: WasiProgram,
    /// 符号表，用于名称解析
    symbol_table: HashMap<String, u32>,
    /// 当前函数索引计数器
    function_index: u32,
    /// 当前类型索引计数器
    type_index: u32,
    /// 当前内存索引计数器
    memory_index: u32,
    /// 当前表索引计数器
    table_index: u32,
    /// 当前全局变量索引计数器
    global_index: u32,
    gaia_diagnostics: Vec<GaiaError>,
}

impl WatCompiler {
    /// 创建新的编译器实例
    pub fn new() -> Self {
        Self {
            program: WasiProgram::new_component(),
            symbol_table: HashMap::new(),
            function_index: 0,
            type_index: 0,
            memory_index: 0,
            table_index: 0,
            global_index: 0,
            gaia_diagnostics: vec![],
        }
    }

    /// 编译 WAT AST 为 Program
    pub fn compile(mut self, ast: WatRoot) -> GaiaDiagnostics<WasiProgram> {
        for item in ast.items {
            match self.compile_item(item) {
                Ok(o) => {}
                Err(e) => self.gaia_diagnostics.push(e),
            }
        }

        // 验证程序完整性
        match self.program.validate() {
            Ok(o) => GaiaDiagnostics { result: Ok(self.program), diagnostics: vec![] },
            Err(e) => GaiaDiagnostics { result: Err(e), diagnostics: vec![] },
        }
    }

    /// 编译顶级项目
    fn compile_item(&mut self, item: WatItem) -> Result<()> {
        match item {
            WatItem::Component(component) => {
                self.compile_component(component)?;
            }
            WatItem::CoreModule(core_module) => {
                self.compile_core_module(core_module)?;
            }
            WatItem::Module(module) => {
                self.compile_module(module)?;
            }
            WatItem::CustomSection(custom_section) => {
                self.compile_custom_section(custom_section)?;
            }
        }
        Ok(())
    }

    /// 编译组件
    fn compile_component(&mut self, component: WatComponent) -> Result<()> {
        // 设置程序类型为组件
        self.program.program_type = WasiProgramType::Component;
        self.program.name = component.name;

        // 编译组件项目
        for item in component.items {
            self.compile_component_item(item)?;
        }

        // 编译自定义段
        for custom_section in component.custom_sections {
            self.compile_custom_section(custom_section)?;
        }

        Ok(())
    }

    /// 编译组件项目
    fn compile_component_item(&mut self, item: WatComponentItem) -> Result<()> {
        match item {
            WatComponentItem::Import(import) => {
                self.compile_import(import)?;
            }
            WatComponentItem::Export(export) => {
                self.compile_export(export)?;
            }
            WatComponentItem::Type(type_def) => {
                self.compile_type_definition(type_def)?;
            }
            WatComponentItem::Alias(alias) => {
                self.compile_alias(alias)?;
            }
            WatComponentItem::Instance(instance) => {
                self.compile_instance(instance)?;
            }
            WatComponentItem::CoreModule(core_module) => {
                self.compile_core_module(core_module)?;
            }
            WatComponentItem::CoreFunc(core_func) => {
                self.compile_core_func(core_func)?;
            }
            WatComponentItem::CoreInstance(core_instance) => {
                self.compile_core_instance(core_instance)?;
            }
        }
        Ok(())
    }

    /// 编译核心模块
    fn compile_core_module(&mut self, core_module: WatCoreModule) -> Result<()> {
        let mut wasi_core_module = WasiCoreModule {
            name: core_module.name.clone(),
            index: self.program.core_modules.len() as u32,
            function_types: Vec::new(),
            functions: Vec::new(),
            exports: Vec::new(),
            imports: Vec::new(),
            memories: Vec::new(),
            tables: Vec::new(),
            globals: Vec::new(),
            data_segments: Vec::new(),
            element_segments: Vec::new(),
            start_function: None,
        };

        // 编译核心模块项目
        for item in core_module.items {
            self.compile_core_module_item(item, &mut wasi_core_module)?;
        }

        // 添加到程序中
        let module_index = self.program.add_core_module(wasi_core_module);

        // 如果有名称，添加到符号表
        if let Some(name) = core_module.name {
            self.program.add_symbol(name, WasiSymbolType::Module, module_index);
        }

        Ok(())
    }

    /// 编译核心模块项目
    fn compile_core_module_item(&mut self, item: WatCoreModuleItem, core_module: &mut WasiCoreModule) -> Result<()> {
        match item {
            WatCoreModuleItem::Import(import) => {
                let wasi_import = self.compile_core_import(import)?;
                core_module.imports.push(wasi_import);
            }
            WatCoreModuleItem::Export(export) => {
                let wasi_export = self.compile_core_export(export)?;
                core_module.exports.push(wasi_export);
            }
            WatCoreModuleItem::Func(func) => {
                let wasi_func = self.compile_core_func_to_wasi(func)?;
                core_module.functions.push(wasi_func);
            }
            WatCoreModuleItem::Memory(memory) => {
                let wasi_memory = self.compile_core_memory(memory)?;
                core_module.memories.push(wasi_memory);
            }
            WatCoreModuleItem::Data(data) => {
                let wasi_data = self.compile_core_data(data)?;
                core_module.data_segments.push(wasi_data);
            }
            WatCoreModuleItem::Start(start_func) => {
                // 查找起始函数索引
                if let Some(symbol) = self.program.find_symbol(&start_func) {
                    core_module.start_function = Some(symbol.index);
                }
            }
            _ => {
                // 其他项目暂时跳过
            }
        }
        Ok(())
    }

    /// 编译导入
    fn compile_import(&mut self, import: WatImport) -> Result<()> {
        let import_type = match import.import_type {
            WatImportType::Func(func_type) => {
                let wasi_func_type = self.compile_func_type(func_type)?;
                let type_index = self.program.add_function_type(wasi_func_type);
                WasmImportType::Function { type_index }
            }
            WatImportType::Interface(_) => {
                // 接口导入暂时跳过
                return Ok(());
            }
            WatImportType::Instance(_) => {
                // 实例导入暂时跳过
                return Ok(());
            }
        };

        let wasi_import = WasiImport { module: import.module, field: import.name, import_type };

        self.program.add_import(wasi_import);
        Ok(())
    }

    /// 编译导出
    fn compile_export(&mut self, export: WatExport) -> Result<()> {
        let export_type = match export.export_item {
            WatExportItem::Func(func_name) => {
                if let Some(symbol) = self.program.find_symbol(&func_name) {
                    WasmExportType::Function { function_index: symbol.index }
                }
                else {
                    return Err(GaiaError::custom_error(format!("Unresolved symbol: {}", func_name)));
                }
            }
            WatExportItem::Instance(_) => {
                // 实例导出暂时跳过
                return Ok(());
            }
            WatExportItem::Type(_) => {
                // 类型导出暂时跳过
                return Ok(());
            }
        };

        let wasi_export = WasiExport { name: export.name, export_type };

        self.program.add_export(wasi_export);
        Ok(())
    }

    /// 编译类型定义
    fn compile_type_definition(&mut self, type_def: WatTypeDefinition) -> Result<()> {
        let wasi_type_def = WasiTypeDefinition {
            name: type_def.name.clone(),
            index: self.type_index,
            type_content: self.compile_type(type_def.type_content)?,
        };

        self.program.component_items.push(WasiComponentItem::Type(wasi_type_def));

        // 如果有名称，添加到符号表
        if let Some(name) = type_def.name {
            self.program.add_symbol(name, WasiSymbolType::Type, self.type_index);
        }

        self.type_index += 1;
        Ok(())
    }

    /// 编译类型
    fn compile_type(&mut self, wat_type: WatType) -> Result<WasiType> {
        match wat_type {
            WatType::Func(func_type) => {
                let wasi_func_type = self.compile_func_type(func_type)?;
                Ok(WasiType::Func(wasi_func_type))
            }
            WatType::Resource(resource_type) => {
                let wasi_resource_type = WasiResourceType {
                    name: resource_type.name,
                    methods: resource_type
                        .methods
                        .into_iter()
                        .map(|method| WasiResourceMethod {
                            name: method.name,
                            method_type: self.compile_func_type(method.method_type).unwrap(),
                        })
                        .collect(),
                };
                Ok(WasiType::Resource(wasi_resource_type))
            }
            WatType::Record(fields) => {
                let wasi_fields = fields
                    .into_iter()
                    .map(|field| WasiRecordField { name: field.name, field_type: self.compile_type(field.field_type).unwrap() })
                    .collect();
                Ok(WasiType::Record(wasi_fields))
            }
            WatType::Variant(cases) => {
                let wasi_cases = cases
                    .into_iter()
                    .map(|case| WasiVariantCase {
                        name: case.name,
                        case_type: case.case_type.map(|t| self.compile_type(t).unwrap()),
                    })
                    .collect();
                Ok(WasiType::Variant(wasi_cases))
            }
            WatType::Enum(variants) => Ok(WasiType::Enum(variants)),
            WatType::Union(types) => {
                let wasi_types = types.into_iter().map(|t| self.compile_type(t).unwrap()).collect();
                Ok(WasiType::Union(wasi_types))
            }
            WatType::Option(inner_type) => {
                let wasi_inner_type = self.compile_type(*inner_type)?;
                Ok(WasiType::Option(Box::new(wasi_inner_type)))
            }
            WatType::List(inner_type) => {
                let wasi_inner_type = self.compile_type(*inner_type)?;
                Ok(WasiType::List(Box::new(wasi_inner_type)))
            }
            WatType::Tuple(types) => {
                let wasi_types = types.into_iter().map(|t| self.compile_type(t).unwrap()).collect();
                Ok(WasiType::Tuple(wasi_types))
            }
            WatType::Flags(flags) => Ok(WasiType::Flags(flags)),
            WatType::Primitive(primitive) => {
                let wasi_primitive = match primitive {
                    WatPrimitiveType::Bool => WasiPrimitiveType::Bool,
                    WatPrimitiveType::S8 => WasiPrimitiveType::S8,
                    WatPrimitiveType::S16 => WasiPrimitiveType::S16,
                    WatPrimitiveType::S32 => WasiPrimitiveType::S32,
                    WatPrimitiveType::S64 => WasiPrimitiveType::S64,
                    WatPrimitiveType::U8 => WasiPrimitiveType::U8,
                    WatPrimitiveType::U16 => WasiPrimitiveType::U16,
                    WatPrimitiveType::U32 => WasiPrimitiveType::U32,
                    WatPrimitiveType::U64 => WasiPrimitiveType::U64,
                    WatPrimitiveType::F32 => WasiPrimitiveType::F32,
                    WatPrimitiveType::F64 => WasiPrimitiveType::F64,
                    WatPrimitiveType::Char => WasiPrimitiveType::Char,
                    WatPrimitiveType::String => WasiPrimitiveType::String,
                };
                Ok(WasiType::Primitive(wasi_primitive))
            }
        }
    }

    /// 编译函数类型
    fn compile_func_type(&mut self, func_type: WatFuncType) -> Result<WasiFunctionType> {
        let params =
            func_type.params.into_iter().map(|param| self.compile_value_type(param.param_type)).collect::<Result<Vec<_>>>()?;

        let results = func_type
            .results
            .into_iter()
            .map(|result| self.compile_value_type(result.result_type))
            .collect::<Result<Vec<_>>>()?;

        Ok(WasiFunctionType { params, results })
    }

    /// 编译核心函数类型
    fn compile_core_func_type(&mut self, func_type: WatCoreFuncType) -> Result<WasiFunctionType> {
        let params =
            func_type.params.into_iter().map(|param| self.compile_core_value_type(param)).collect::<Result<Vec<_>>>()?;

        let results =
            func_type.results.into_iter().map(|result| self.compile_core_value_type(result)).collect::<Result<Vec<_>>>()?;

        Ok(WasiFunctionType { params, results })
    }

    /// 编译值类型（从 WatType 到 WasmValueType）
    fn compile_value_type(&mut self, wat_type: WatType) -> Result<WasmValueType> {
        match wat_type {
            WatType::Primitive(WatPrimitiveType::S32) | WatType::Primitive(WatPrimitiveType::U32) => Ok(WasmValueType::I32),
            WatType::Primitive(WatPrimitiveType::S64) | WatType::Primitive(WatPrimitiveType::U64) => Ok(WasmValueType::I64),
            WatType::Primitive(WatPrimitiveType::F32) => Ok(WasmValueType::F32),
            WatType::Primitive(WatPrimitiveType::F64) => Ok(WasmValueType::F64),
            _ => {
                // 其他类型暂时映射为 i32
                Ok(WasmValueType::I32)
            }
        }
    }

    /// 编译核心值类型
    fn compile_core_value_type(&mut self, value_type: WatValueType) -> Result<WasmValueType> {
        match value_type {
            WatValueType::I32 => Ok(WasmValueType::I32),
            WatValueType::I64 => Ok(WasmValueType::I64),
            WatValueType::F32 => Ok(WasmValueType::F32),
            WatValueType::F64 => Ok(WasmValueType::F64),
            _ => {
                // 其他类型暂时不支持
                Err(GaiaError::not_implemented(format!("Unsupported value type: {:?}", value_type)))
            }
        }
    }

    // 其他编译方法的占位符实现
    fn compile_module(&mut self, _module: WatModule) -> Result<()> {
        // 简化实现
        Ok(())
    }

    fn compile_custom_section(&mut self, _custom_section: WatCustomSection) -> Result<()> {
        // 简化实现
        Ok(())
    }

    fn compile_alias(&mut self, _alias: WatAlias) -> Result<()> {
        // 简化实现
        Ok(())
    }

    fn compile_instance(&mut self, _instance: WatInstance) -> Result<()> {
        // 简化实现
        Ok(())
    }

    fn compile_core_func(&mut self, _core_func: WatCoreFunc) -> Result<()> {
        // 简化实现
        Ok(())
    }

    fn compile_core_instance(&mut self, _core_instance: WatCoreInstance) -> Result<()> {
        // 简化实现
        Ok(())
    }

    fn compile_core_import(&mut self, _import: WatCoreImport) -> Result<WasiImport> {
        // 简化实现
        Ok(WasiImport {
            module: "placeholder".to_string(),
            field: "placeholder".to_string(),
            import_type: WasmImportType::Function { type_index: 0 },
        })
    }

    fn compile_core_export(&mut self, _export: WatCoreExport) -> Result<WasiExport> {
        // 简化实现
        Ok(WasiExport { name: "placeholder".to_string(), export_type: WasmExportType::Function { function_index: 0 } })
    }

    fn compile_core_func_to_wasi(&mut self, _func: WatCoreFunc) -> Result<WasiFunction> {
        // 简化实现
        Ok(WasiFunction { type_index: 0, locals: Vec::new(), body: Vec::new() })
    }

    fn compile_core_memory(&mut self, _memory: WatCoreMemory) -> Result<WasiMemory> {
        // 简化实现
        Ok(WasiMemory { memory_type: WasmMemoryType { min: 1, max: None } })
    }

    fn compile_core_data(&mut self, _data: WatCoreData) -> Result<WasiDataSegment> {
        // 简化实现
        Ok(WasiDataSegment { name: None, memory_index: Some(0), offset: Vec::new(), data: Vec::new() })
    }
}

impl Default for WatCompiler {
    fn default() -> Self {
        Self::new()
    }
}
