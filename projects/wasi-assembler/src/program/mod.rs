#![doc = include_str!("readme.md")]

use gaia_types::{GaiaError, Result};
use std::collections::HashMap;
use std::fmt;

/// WASI 程序的高层次表示
///
/// 这个结构体可以表示一个完整的 WebAssembly Component 或者一个传统的核心模块
#[derive(Debug, Clone)]
pub struct WasiProgram {
    /// 程序类型：组件或核心模块
    pub program_type: WasiProgramType,
    /// 程序名称（可选）
    pub name: Option<String>,
    /// 函数类型定义
    pub function_types: Vec<WasiFunctionType>,
    /// 函数定义
    pub functions: Vec<WasiFunction>,
    /// 导出定义
    pub exports: Vec<WasiExport>,
    /// 导入定义
    pub imports: Vec<WasiImport>,
    /// 内存定义
    pub memories: Vec<WasiMemory>,
    /// 表定义
    pub tables: Vec<WasiTable>,
    /// 全局变量定义
    pub globals: Vec<WasiGlobal>,
    /// 自定义段
    pub custom_sections: Vec<WasiCustomSection>,
    /// 起始函数索引
    pub start_function: Option<u32>,
    /// 组件特有的项目（仅当 program_type 为 Component 时使用）
    pub component_items: Vec<WasiComponentItem>,
    /// 核心模块列表（用于组件中的嵌套模块）
    pub core_modules: Vec<WasiCoreModule>,
    /// 实例列表
    pub instances: Vec<WasiInstance>,
    /// 别名定义
    pub aliases: Vec<WasiAlias>,
    /// 符号表，用于名称到索引的映射
    pub symbol_table: HashMap<String, WasiSymbol>,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct WasmInfo {
    pub magic_head: [u8; 4],
}

/// 程序类型枚举
#[derive(Debug, Clone, PartialEq)]
pub enum WasiProgramType {
    /// WebAssembly Component Model 组件
    Component,
    /// 传统的 WebAssembly 核心模块
    CoreModule,
}

/// 组件项目枚举
#[derive(Debug, Clone)]
pub enum WasiComponentItem {
    /// 类型定义
    Type(WasiTypeDefinition),
    /// 别名定义
    Alias(WasiAlias),
    /// 实例定义
    Instance(WasiInstance),
    /// 核心函数定义
    CoreFunc(WasiCoreFunc),
    /// 核心实例定义
    CoreInstance(WasiCoreInstance),
}

/// 核心模块定义
#[derive(Debug, Clone)]
pub struct WasiCoreModule {
    /// 模块名称（可选）
    pub name: Option<String>,
    /// 模块索引
    pub index: u32,
    /// 函数类型定义
    pub function_types: Vec<WasiFunctionType>,
    /// 函数定义
    pub functions: Vec<WasiFunction>,
    /// 导出定义
    pub exports: Vec<WasiExport>,
    /// 导入定义
    pub imports: Vec<WasiImport>,
    /// 内存定义
    pub memories: Vec<WasiMemory>,
    /// 表定义
    pub tables: Vec<WasiTable>,
    /// 全局变量定义
    pub globals: Vec<WasiGlobal>,
    /// 数据段
    pub data_segments: Vec<WasiDataSegment>,
    /// 元素段
    pub element_segments: Vec<WasiElementSegment>,
    /// 起始函数索引
    pub start_function: Option<u32>,
}

/// 实例定义
#[derive(Debug, Clone)]
pub struct WasiInstance {
    /// 实例名称（可选）
    pub name: Option<String>,
    /// 实例索引
    pub index: u32,
    /// 实例化的模块或组件名称
    pub instantiate_target: String,
    /// 实例化参数
    pub args: Vec<WasiInstanceArg>,
    /// 实例类型
    pub instance_type: WasiInstanceType,
}

/// 实例类型枚举
#[derive(Debug, Clone)]
pub enum WasiInstanceType {
    /// 核心模块实例
    CoreModule,
    /// 组件实例
    Component,
}

/// 实例化参数
#[derive(Debug, Clone)]
pub struct WasiInstanceArg {
    /// 参数名称
    pub name: String,
    /// 参数值（引用的符号名称）
    pub value: String,
}

/// 别名定义
#[derive(Debug, Clone)]
pub struct WasiAlias {
    /// 别名名称（可选）
    pub name: Option<String>,
    /// 别名目标
    pub target: WasiAliasTarget,
}

/// 别名目标枚举
#[derive(Debug, Clone)]
pub enum WasiAliasTarget {
    /// 外部别名
    Outer {
        /// 外部索引
        outer_index: u32,
        /// 项目索引
        item_index: u32,
    },
    /// 核心别名
    Core {
        /// 核心类型
        core_type: WasiCoreType,
        /// 项目索引
        item_index: u32,
    },
    /// 导出别名
    Export {
        /// 实例名称
        instance: String,
        /// 导出名称
        name: String,
    },
    /// 核心导出别名
    CoreExport {
        /// 实例名称
        instance: String,
        /// 导出名称
        name: String,
    },
}

/// 核心类型枚举
#[derive(Debug, Clone, Copy)]
pub enum WasiCoreType {
    Module,
    Func,
    Table,
    Memory,
    Global,
}

/// 类型定义
#[derive(Debug, Clone)]
pub struct WasiTypeDefinition {
    /// 类型名称（可选）
    pub name: Option<String>,
    /// 类型索引
    pub index: u32,
    /// 类型内容
    pub type_content: WasiType,
}

/// 类型枚举
#[derive(Debug, Clone)]
pub enum WasiType {
    /// 函数类型
    Func(WasiFunctionType),
    /// 接口类型
    Interface(String),
    /// 实例类型
    Instance(String),
    /// 资源类型
    Resource(WasiResourceType),
    /// 记录类型
    Record(Vec<WasiRecordField>),
    /// 变体类型
    Variant(Vec<WasiVariantCase>),
    /// 枚举类型
    Enum(Vec<String>),
    /// 联合类型
    Union(Vec<WasiType>),
    /// 选项类型
    Option(Box<WasiType>),
    /// 列表类型
    List(Box<WasiType>),
    /// 元组类型
    Tuple(Vec<WasiType>),
    /// 标志类型
    Flags(Vec<String>),
    /// 原始类型
    Primitive(WasiPrimitiveType),
}

/// 资源类型
#[derive(Debug, Clone)]
pub struct WasiResourceType {
    /// 资源名称
    pub name: String,
    /// 资源方法
    pub methods: Vec<WasiResourceMethod>,
}

/// 资源方法
#[derive(Debug, Clone)]
pub struct WasiResourceMethod {
    /// 方法名称
    pub name: String,
    /// 方法类型
    pub method_type: WasiFunctionType,
}

/// 记录字段
#[derive(Debug, Clone)]
pub struct WasiRecordField {
    /// 字段名称
    pub name: String,
    /// 字段类型
    pub field_type: WasiType,
}

/// 变体情况
#[derive(Debug, Clone)]
pub struct WasiVariantCase {
    /// 情况名称
    pub name: String,
    /// 情况类型（可选）
    pub case_type: Option<WasiType>,
}

/// 原始类型枚举
#[derive(Debug, Clone, Copy)]
pub enum WasiPrimitiveType {
    Bool,
    S8,
    S16,
    S32,
    S64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    Char,
    String,
}

/// 核心函数定义
#[derive(Debug, Clone)]
pub struct WasiCoreFunc {
    /// 函数名称（可选）
    pub name: Option<String>,
    /// 函数索引
    pub index: u32,
    /// 函数类型
    pub func_type: WasiFunctionType,
    /// 规范化操作（可选）
    pub canon: Option<WasiCanonicalOperation>,
}

/// 规范化操作枚举
#[derive(Debug, Clone)]
pub enum WasiCanonicalOperation {
    /// Lower 操作
    Lower {
        /// 函数名称
        func: String,
        /// 选项
        options: Vec<WasiCanonOption>,
    },
    /// Lift 操作
    Lift {
        /// 函数名称
        func: String,
        /// 选项
        options: Vec<WasiCanonOption>,
    },
    /// 资源创建
    ResourceNew(String),
    /// 资源销毁
    ResourceDrop(String),
    /// 资源表示
    ResourceRep(String),
}

/// 规范化选项枚举
#[derive(Debug, Clone)]
pub enum WasiCanonOption {
    /// 字符串编码
    StringEncoding(String),
    /// 内存
    Memory(String),
    /// 重新分配函数
    Realloc(String),
}

/// 核心实例定义
#[derive(Debug, Clone)]
pub struct WasiCoreInstance {
    /// 实例名称（可选）
    pub name: Option<String>,
    /// 实例索引
    pub index: u32,
    /// 实例化的模块名称
    pub instantiate_target: String,
    /// 实例化参数
    pub args: Vec<WasiInstanceArg>,
}

/// 数据段定义
#[derive(Debug, Clone)]
pub struct WasiDataSegment {
    /// 数据段名称（可选）
    pub name: Option<String>,
    /// 内存索引（可选，默认为 0）
    pub memory_index: Option<u32>,
    /// 偏移表达式
    pub offset: Vec<WasiInstruction>,
    /// 数据内容
    pub data: Vec<u8>,
}

/// 元素段定义
#[derive(Debug, Clone)]
pub struct WasiElementSegment {
    /// 元素段名称（可选）
    pub name: Option<String>,
    /// 表索引（可选，默认为 0）
    pub table_index: Option<u32>,
    /// 偏移表达式
    pub offset: Vec<WasiInstruction>,
    /// 元素列表（函数索引）
    pub elements: Vec<u32>,
}

/// 符号定义
#[derive(Debug, Clone)]
pub struct WasiSymbol {
    /// 符号名称
    pub name: String,
    /// 符号类型
    pub symbol_type: WasiSymbolType,
    /// 符号索引
    pub index: u32,
}

/// 符号类型枚举
#[derive(Debug, Clone)]
pub enum WasiSymbolType {
    Function,
    Type,
    Memory,
    Table,
    Global,
    Instance,
    Module,
    Component,
}

/// WASM 函数类型
#[derive(Debug, Clone, PartialEq)]
pub struct WasiFunctionType {
    /// 参数类型
    pub params: Vec<WasmValueType>,
    /// 返回值类型
    pub results: Vec<WasmValueType>,
}

/// WASM 值类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WasmValueType {
    I32,
    I64,
    F32,
    F64,
    V128,
    Funcref,
    Externref,
}

impl TryFrom<u8> for WasmValueType {
    type Error = GaiaError;

    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        match value {
            0x7F => Ok(WasmValueType::I32),
            0x7E => Ok(WasmValueType::I64),
            0x7D => Ok(WasmValueType::F32),
            0x7C => Ok(WasmValueType::F64),
            0x7B => Ok(WasmValueType::V128),
            0x70 => Ok(WasmValueType::Funcref),
            0x6F => Ok(WasmValueType::Externref),
            _ => Err(GaiaError::invalid_data(&format!("Unknown value type: 0x{:02X}", value))),
        }
    }
}

impl fmt::Display for WasmValueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WasmValueType::I32 => write!(f, "i32"),
            WasmValueType::I64 => write!(f, "i64"),
            WasmValueType::F32 => write!(f, "f32"),
            WasmValueType::F64 => write!(f, "f64"),
            WasmValueType::V128 => write!(f, "v128"),
            WasmValueType::Funcref => write!(f, "funcref"),
            WasmValueType::Externref => write!(f, "externref"),
        }
    }
}

/// WASM 函数定义
#[derive(Debug, Clone)]
pub struct WasiFunction {
    /// 函数类型索引
    pub type_index: u32,
    /// 局部变量
    pub locals: Vec<WasmLocal>,
    /// 函数体指令
    pub body: Vec<WasiInstruction>,
}

/// WASM 局部变量
#[derive(Copy, Debug, Clone)]
pub struct WasmLocal {
    /// 变量数量
    pub count: u32,
    /// 变量类型
    pub value_type: WasmValueType,
}

/// WASM 指令
#[derive(Copy, Debug, Clone)]
pub enum WasiInstruction {
    /// 无操作
    Nop,
    /// 不可达
    Unreachable,
    /// 块开始
    Block {
        block_type: Option<WasmValueType>,
    },
    /// 循环开始
    Loop {
        block_type: Option<WasmValueType>,
    },
    /// 条件分支
    If {
        block_type: Option<WasmValueType>,
    },
    /// Else 分支
    Else,
    /// 块结束
    End,
    /// 分支
    Br {
        label_index: u32,
    },
    /// 条件分支
    BrIf {
        label_index: u32,
    },
    /// 返回
    Return,
    /// 函数调用
    Call {
        function_index: u32,
    },
    /// 丢弃栈顶值
    Drop,
    /// 选择
    Select,
    /// 加载局部变量
    LocalGet {
        local_index: u32,
    },
    /// 设置局部变量
    LocalSet {
        local_index: u32,
    },
    /// 加载常量
    I32Const {
        value: i32,
    },
    I64Const {
        value: i64,
    },
    F32Const {
        value: f32,
    },
    F64Const {
        value: f64,
    },
    /// 算术运算
    I32Add,
    I32Sub,
    I32Mul,
    I32DivS,
    I32DivU,
    I32RemS,
    I32RemU,
    I32And,
    I32Or,
    I32Xor,
    I32Shl,
    I32ShrS,
    I32ShrU,
    I32Rotl,
    I32Rotr,
    /// 比较运算
    I32Eqz,
    I32Eq,
    I32Ne,
    I32LtS,
    I32LtU,
    I32GtS,
    I32GtU,
    I32LeS,
    I32LeU,
    I32GeS,
    I32GeU,
}

/// WASM 导出
#[derive(Debug, Clone)]
pub struct WasiExport {
    /// 导出名称
    pub name: String,
    /// 导出类型
    pub export_type: WasmExportType,
}

/// WASM 导出类型
#[derive(Copy, Debug, Clone)]
pub enum WasmExportType {
    Function { function_index: u32 },
    Table { table_index: u32 },
    Memory { memory_index: u32 },
    Global { global_index: u32 },
}

impl WasmExportType {
    pub fn function_index(&self) -> Option<u32> {
        match self {
            WasmExportType::Function { function_index } => Some(*function_index),
            _ => None,
        }
    }
}

/// WASM 导入
#[derive(Debug, Clone)]
pub struct WasiImport {
    /// 模块名
    pub module: String,
    /// 字段名
    pub field: String,
    /// 导入类型
    pub import_type: WasmImportType,
}

/// WASM 导入类型
#[derive(Copy, Debug, Clone)]
pub enum WasmImportType {
    Function { type_index: u32 },
    Table { table_type: WasmTableType },
    Memory { memory_type: WasmMemoryType },
    Global { global_type: WasmGlobalType },
}

/// WASM 内存定义
#[derive(Copy, Debug, Clone)]
pub struct WasiMemory {
    pub memory_type: WasmMemoryType,
}

/// WASM 内存类型
#[derive(Copy, Debug, Clone)]
pub struct WasmMemoryType {
    /// 最小页数
    pub min: u32,
    /// 最大页数（可选）
    pub max: Option<u32>,
}

/// WASM 表定义
#[derive(Copy, Debug, Clone)]
pub struct WasiTable {
    pub table_type: WasmTableType,
}

/// WASM 表类型
#[derive(Copy, Debug, Clone)]
pub struct WasmTableType {
    /// 元素类型
    pub element_type: WasmReferenceType,
    /// 最小大小
    pub min: u32,
    /// 最大大小（可选）
    pub max: Option<u32>,
}

/// WASM 引用类型
#[derive(Copy, Debug, Clone)]
pub enum WasmReferenceType {
    FuncRef,
    ExternRef,
}

/// WASM 全局变量
#[derive(Debug, Clone)]
pub struct WasiGlobal {
    pub global_type: WasmGlobalType,
    pub init_expr: Vec<WasiInstruction>,
}

/// WASM 全局变量类型
#[derive(Copy, Debug, Clone)]
pub struct WasmGlobalType {
    pub value_type: WasmValueType,
    pub mutable: bool,
}

/// WASM 自定义段
#[derive(Debug, Clone)]
pub struct WasiCustomSection {
    /// 段名称
    pub name: String,
    /// 段数据
    pub data: Vec<u8>,
}

impl WasiProgram {
    /// 创建新的 WASI 程序
    pub fn new(program_type: WasiProgramType) -> Self {
        Self {
            program_type,
            name: None,
            function_types: Vec::new(),
            functions: Vec::new(),
            exports: Vec::new(),
            imports: Vec::new(),
            memories: Vec::new(),
            tables: Vec::new(),
            globals: Vec::new(),
            custom_sections: Vec::new(),
            start_function: None,
            component_items: Vec::new(),
            core_modules: Vec::new(),
            instances: Vec::new(),
            aliases: Vec::new(),
            symbol_table: HashMap::new(),
        }
    }

    /// 创建新的组件程序
    pub fn new_component() -> Self {
        Self::new(WasiProgramType::Component)
    }

    /// 创建新的核心模块程序
    pub fn new_core_module() -> Self {
        Self::new(WasiProgramType::CoreModule)
    }

    /// 添加函数类型定义，返回类型索引
    pub fn add_function_type(&mut self, func_type: WasiFunctionType) -> u32 {
        // 检查是否已存在相同的函数类型
        for (index, existing_type) in self.function_types.iter().enumerate() {
            if existing_type == &func_type {
                return index as u32;
            }
        }

        let index = self.function_types.len() as u32;
        self.function_types.push(func_type);
        index
    }

    /// 添加函数定义，返回函数索引
    pub fn add_function(&mut self, function: WasiFunction) -> u32 {
        let index = self.functions.len() as u32;
        self.functions.push(function);
        index
    }

    /// 添加导出定义
    pub fn add_export(&mut self, export: WasiExport) {
        self.exports.push(export);
    }

    /// 添加导入定义
    pub fn add_import(&mut self, import: WasiImport) {
        self.imports.push(import);
    }

    /// 添加内存定义
    pub fn add_memory(&mut self, memory: WasiMemory) -> u32 {
        let index = self.memories.len() as u32;
        self.memories.push(memory);
        index
    }

    /// 添加表定义
    pub fn add_table(&mut self, table: WasiTable) -> u32 {
        let index = self.tables.len() as u32;
        self.tables.push(table);
        index
    }

    /// 添加全局变量定义
    pub fn add_global(&mut self, global: WasiGlobal) -> u32 {
        let index = self.globals.len() as u32;
        self.globals.push(global);
        index
    }

    /// 添加核心模块
    pub fn add_core_module(&mut self, core_module: WasiCoreModule) -> u32 {
        let index = self.core_modules.len() as u32;
        self.core_modules.push(core_module);
        index
    }

    /// 添加实例
    pub fn add_instance(&mut self, instance: WasiInstance) -> u32 {
        let index = self.instances.len() as u32;
        self.instances.push(instance);
        index
    }

    /// 添加别名
    pub fn add_alias(&mut self, alias: WasiAlias) {
        self.aliases.push(alias);
    }

    /// 添加符号到符号表
    pub fn add_symbol(&mut self, name: String, symbol_type: WasiSymbolType, index: u32) {
        let symbol = WasiSymbol { name: name.clone(), symbol_type, index };
        self.symbol_table.insert(name, symbol);
    }

    /// 根据名称查找符号
    pub fn find_symbol(&self, name: &str) -> Option<&WasiSymbol> {
        self.symbol_table.get(name)
    }

    /// 验证程序的完整性
    pub fn validate(&self) -> Result<()> {
        // 验证函数类型索引
        for function in &self.functions {
            if function.type_index as usize >= self.function_types.len() {
                return Err(GaiaError::custom_error(format!(
                    "函数类型索引 {} 超出范围，最大索引为 {}",
                    function.type_index,
                    self.function_types.len().saturating_sub(1)
                )));
            }
        }

        // 验证导出索引
        for export in &self.exports {
            match &export.export_type {
                WasmExportType::Function { function_index } => {
                    let total_functions =
                        self.imports.iter().filter(|imp| matches!(imp.import_type, WasmImportType::Function { .. })).count()
                            + self.functions.len();

                    if *function_index as usize >= total_functions {
                        return Err(GaiaError::custom_error(format!(
                            "导出函数索引 {} 超出范围，最大索引为 {}",
                            function_index,
                            total_functions.saturating_sub(1)
                        )));
                    }
                }
                _ => {} // 其他类型的验证可以后续添加
            }
        }

        Ok(())
    }
}

impl Default for WasiProgram {
    fn default() -> Self {
        Self::new_core_module()
    }
}

/// WasiProgram 的构建器
pub struct WasiProgramBuilder {
    program: WasiProgram,
}

impl WasiProgramBuilder {
    /// 创建一个新的 WasiProgramBuilder
    pub fn new(program_type: WasiProgramType) -> Self {
        Self { program: WasiProgram::new(program_type) }
    }

    /// 设置程序名称
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.program.name = Some(name.into());
        self
    }

    /// 添加函数类型定义
    pub fn with_function_type(mut self, func_type: WasiFunctionType) -> Self {
        self.program.add_function_type(func_type);
        self
    }

    /// 添加函数定义
    pub fn with_function(mut self, function: WasiFunction) -> Self {
        self.program.add_function(function);
        self
    }

    /// 添加导出定义
    pub fn with_export(mut self, export: WasiExport) -> Self {
        self.program.add_export(export);
        self
    }

    /// 添加导入定义
    pub fn with_import(mut self, import: WasiImport) -> Self {
        self.program.add_import(import);
        self
    }

    /// 添加内存定义
    pub fn with_memory(mut self, memory: WasiMemory) -> Self {
        self.program.add_memory(memory);
        self
    }

    /// 添加表定义
    pub fn with_table(mut self, table: WasiTable) -> Self {
        self.program.add_table(table);
        self
    }

    /// 添加全局变量定义
    pub fn with_global(mut self, global: WasiGlobal) -> Self {
        self.program.add_global(global);
        self
    }

    /// 添加自定义段
    pub fn with_custom_section(mut self, custom_section: WasiCustomSection) -> Self {
        self.program.custom_sections.push(custom_section);
        self
    }

    /// 设置起始函数索引
    pub fn with_start_function(mut self, start_function_index: u32) -> Self {
        self.program.start_function = Some(start_function_index);
        self
    }

    /// 添加组件项目
    pub fn with_component_item(mut self, item: WasiComponentItem) -> Self {
        self.program.component_items.push(item);
        self
    }

    /// 添加核心模块
    pub fn with_core_module(mut self, core_module: WasiCoreModule) -> Self {
        self.program.add_core_module(core_module);
        self
    }

    /// 添加实例
    pub fn with_instance(mut self, instance: WasiInstance) -> Self {
        self.program.add_instance(instance);
        self
    }

    /// 添加别名
    pub fn with_alias(mut self, alias: WasiAlias) -> Self {
        self.program.add_alias(alias);
        self
    }

    /// 添加符号到符号表
    pub fn with_symbol(mut self, name: impl Into<String>, symbol_type: WasiSymbolType, index: u32) -> Self {
        self.program.add_symbol(name.into(), symbol_type, index);
        self
    }

    /// 构建 WasiProgram 实例
    pub fn build(self) -> Result<WasiProgram> {
        self.program.validate().map(|_| self.program)
    }
}

impl WasiProgram {
    /// 创建 WasiProgram 的构建器
    pub fn builder(program_type: WasiProgramType) -> WasiProgramBuilder {
        WasiProgramBuilder::new(program_type)
    }
}
