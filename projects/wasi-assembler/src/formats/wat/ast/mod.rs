#![doc = include_str!("readme.md")]

mod to_program;
mod to_wat;

use gaia_types::SourceLocation;

/// WAT 程序的根节点
///
/// 表示一个完整的 WAT 程序，通常是一个组件或核心模块。
///
/// # 示例
///
/// ```rust
/// use wasi_wat::ast::{WatItem, WatRoot};
///
/// let root = WatRoot { items: vec![WatItem::Component(component)] };
/// ```
#[derive(Clone, Debug, Default)]
pub struct WatRoot {
    /// 程序中的所有顶级项目
    pub items: Vec<WatItem>,
    /// 源代码位置信息，用于 DWARF 调试信息生成
    pub location: SourceLocation,
}

/// WAT 顶级项目枚举
///
/// 表示 WAT 程序中的各种顶级构造。
#[derive(Clone, Debug)]
pub enum WatItem {
    /// 组件定义
    Component(WatComponent),
    /// 核心模块定义
    CoreModule(WatCoreModule),
    /// 模块定义（简化形式）
    Module(WatModule),
    /// 自定义段
    CustomSection(WatCustomSection),
}

/// WAT 组件定义
///
/// 表示一个 WebAssembly 组件，包含导入、导出、类型定义等。
#[derive(Clone, Debug)]
pub struct WatComponent {
    /// 组件名称（可选）
    pub name: Option<String>,
    /// 组件内的项目列表
    pub items: Vec<WatComponentItem>,
    /// 自定义段列表
    pub custom_sections: Vec<WatCustomSection>,
    /// 源代码位置信息
    pub location: SourceLocation,
}

/// 组件内的项目枚举
#[derive(Clone, Debug)]
pub enum WatComponentItem {
    /// 导入声明
    Import(WatImport),
    /// 导出声明
    Export(WatExport),
    /// 类型定义
    Type(WatTypeDefinition),
    /// 别名定义
    Alias(WatAlias),
    /// 实例定义
    Instance(WatInstance),
    /// 核心模块定义
    CoreModule(WatCoreModule),
    /// 核心函数定义
    CoreFunc(WatCoreFunc),
    /// 核心实例定义
    CoreInstance(WatCoreInstance),
    /// 自定义段
    CustomSection(WatCustomSection),
    /// 全局变量定义
    Global(WatCoreGlobal),
    /// 表定义
    Table(WatTable),
}

/// 导入声明
#[derive(Clone, Debug)]
pub struct WatImport {
    /// 导入的模块名
    pub module: String,
    /// 导入的项目名
    pub name: String,
    /// 导入的类型
    pub import_type: WatImportType,
    /// 本地绑定名称（可选）
    pub local_name: Option<String>,
    /// 源代码位置信息
    pub location: SourceLocation,
}

/// 导入类型枚举
#[derive(Clone, Debug)]
pub enum WatImportType {
    /// 函数类型
    Func(WatFuncType),
    /// 接口类型
    Interface(String),
    /// 实例类型
    Instance(String),
}

/// 导出声明
#[derive(Clone, Debug)]
pub struct WatExport {
    /// 导出的名称
    pub name: String,
    /// 导出的项目
    pub export_item: WatExportItem,
    /// 源代码位置信息
    pub location: SourceLocation,
}

/// 导出项目枚举
#[derive(Clone, Debug)]
pub enum WatExportItem {
    /// 函数引用
    Func(String),
    /// 实例引用
    Instance(String),
    /// 类型引用
    Type(String),
}

/// 类型定义
#[derive(Clone, Debug)]
pub struct WatTypeDefinition {
    /// 类型名称（可选）
    pub name: Option<String>,
    /// 类型内容
    pub type_content: WatType,
    /// 源代码位置信息
    pub location: SourceLocation,
}

/// WAT 类型枚举
#[derive(Clone, Debug)]
pub enum WatType {
    /// 函数类型
    Func(WatFuncType),
    /// 资源类型
    Resource(WatResourceType),
    /// 记录类型
    Record(Vec<WatRecordField>),
    /// 变体类型
    Variant(Vec<WatVariantCase>),
    /// 枚举类型
    Enum(Vec<String>),
    /// 联合类型
    Union(Vec<WatType>),
    /// 选项类型
    Option(Box<WatType>),
    /// 列表类型
    List(Box<WatType>),
    /// 元组类型
    Tuple(Vec<WatType>),
    /// 标志类型
    Flags(Vec<String>),
    /// 基本类型
    Primitive(WatPrimitiveType),
}

impl fmt::Display for WatType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WatType::Func(func_type) => write!(f, "{}", func_type),
            WatType::Primitive(primitive_type) => write!(f, "{}", primitive_type),
            WatType::Resource(resource_type) => write!(f, "{}", resource_type),
            WatType::Record(record_fields) => {
                write!(f, "(record")?;
                for field in record_fields {
                    write!(f, " {}", field)?;
                }
                write!(f, ")")
            }
            WatType::Variant(variant_cases) => {
                write!(f, "(variant")?;
                for case in variant_cases {
                    write!(f, " {}", case)?;
                }
                write!(f, ")")
            }
            WatType::Enum(enum_cases) => {
                write!(f, "(enum")?;
                for case in enum_cases {
                    write!(f, " ${}", case)?;
                }
                write!(f, ")")
            }
            WatType::Union(union_types) => {
                write!(f, "(union")?;
                for union_type in union_types {
                    write!(f, " {}", union_type)?;
                }
                write!(f, ")")
            }
            WatType::Option(option_type) => write!(f, "(option {})", option_type),
            WatType::List(list_type) => write!(f, "(list {})", list_type),
            WatType::Tuple(tuple_types) => {
                write!(f, "(tuple")?;
                for tuple_type in tuple_types {
                    write!(f, " {}", tuple_type)?;
                }
                write!(f, ")")
            }
            WatType::Flags(flags) => {
                write!(f, "(flags")?;
                for flag in flags {
                    write!(f, " ${}", flag)?;
                }
                write!(f, ")")
            }
        }
    }
}

/// 函数类型
#[derive(Clone, Debug)]
pub struct WatFuncType {
    /// 参数列表
    pub params: Vec<WatParam>,
    /// 返回值列表
    pub results: Vec<WatResult>,
    /// 源代码位置信息
    pub location: SourceLocation,
}

impl fmt::Display for WatFuncType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(func")?;
        if !self.params.is_empty() {
            write!(f, " (param")?;
            for param in &self.params {
                write!(f, " {}", param)?;
            }
            write!(f, ")")?;
        }
        if !self.results.is_empty() {
            write!(f, " (result")?;
            for result in &self.results {
                write!(f, " {}", result)?;
            }
            write!(f, ")")?;
        }
        write!(f, ")")
    }
}

/// 函数参数
#[derive(Clone, Debug)]
pub struct WatParam {
    /// 参数名称（可选）
    pub name: Option<String>,
    /// 参数类型
    pub param_type: WatType,
    /// 源代码位置信息
    pub location: SourceLocation,
}

impl fmt::Display for WatParam {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(name) = &self.name {
            write!(f, "${} {}", name, self.param_type)
        }
        else {
            write!(f, "{}", self.param_type)
        }
    }
}

/// 函数返回值
#[derive(Clone, Debug)]
pub struct WatResult {
    /// 返回值名称（可选）
    pub name: Option<String>,
    /// 返回值类型
    pub result_type: WatType,
    /// 源代码位置信息
    pub location: SourceLocation,
}

impl fmt::Display for WatResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(name) = &self.name {
            write!(f, "${} {}", name, self.result_type)
        }
        else {
            write!(f, "{}", self.result_type)
        }
    }
}

/// 资源类型
#[derive(Clone, Debug)]
pub struct WatResourceType {
    /// 资源名称
    pub name: String,
    /// 资源方法（可选）
    pub methods: Vec<WatResourceMethod>,
    /// 源代码位置信息
    pub location: SourceLocation,
}

impl fmt::Display for WatResourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(resource ${}", self.name)?;
        for method in &self.methods {
            write!(f, " {}", method)?;
        }
        write!(f, ")")
    }
}

/// 资源方法
#[derive(Clone, Debug)]
pub struct WatResourceMethod {
    /// 方法名称
    pub name: String,
    /// 方法类型
    pub method_type: WatFuncType,
    /// 源代码位置信息
    pub location: SourceLocation,
}

impl fmt::Display for WatResourceMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(method ${} (func {}", self.name, self.method_type)
    }
}

/// 记录字段
#[derive(Clone, Debug)]
pub struct WatRecordField {
    /// 字段名称
    pub name: String,
    /// 字段类型
    pub field_type: WatValueType,
    /// 源代码位置信息
    pub location: SourceLocation,
}

impl fmt::Display for WatRecordField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(field ${} {})", self.name, self.field_type)
    }
}

/// 变体情况
#[derive(Clone, Debug)]
pub struct WatVariantCase {
    /// 情况名称
    pub name: String,
    /// 情况类型（可选）
    pub case_type: Option<WatType>,
    /// 源代码位置信息
    pub location: SourceLocation,
}

impl fmt::Display for WatVariantCase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(case ${}", self.name)?;
        if let Some(case_type) = &self.case_type {
            write!(f, " {}", case_type)?;
        }
        write!(f, ")")
    }
}

/// 基本类型枚举
#[derive(Clone, Debug, Copy)]
pub enum WatPrimitiveType {
    /// 布尔类型
    Bool,
    /// 8位有符号整数
    S8,
    /// 16位有符号整数
    S16,
    /// 32位有符号整数
    S32,
    /// 64位有符号整数
    S64,
    /// 8位无符号整数
    U8,
    /// 16位无符号整数
    U16,
    /// 32位无符号整数
    U32,
    /// 64位无符号整数
    U64,
    /// 32位浮点数
    F32,
    /// 64位浮点数
    F64,
    /// 字符类型
    Char,
    /// 字符串类型
    String,
}

impl fmt::Display for WatPrimitiveType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WatPrimitiveType::Bool => write!(f, "bool"),
            WatPrimitiveType::S8 => write!(f, "s8"),
            WatPrimitiveType::S16 => write!(f, "s16"),
            WatPrimitiveType::S32 => write!(f, "s32"),
            WatPrimitiveType::S64 => write!(f, "s64"),
            WatPrimitiveType::U8 => write!(f, "u8"),
            WatPrimitiveType::U16 => write!(f, "u16"),
            WatPrimitiveType::U32 => write!(f, "u32"),
            WatPrimitiveType::U64 => write!(f, "u64"),
            WatPrimitiveType::F32 => write!(f, "f32"),
            WatPrimitiveType::F64 => write!(f, "f64"),
            WatPrimitiveType::Char => write!(f, "char"),
            WatPrimitiveType::String => write!(f, "string"),
        }
    }
}

/// 别名定义
#[derive(Clone, Debug)]
pub struct WatAlias {
    /// 别名名称（可选）
    pub name: Option<String>,
    /// 别名目标
    pub target: WatAliasTarget,
    /// 源代码位置信息
    pub location: SourceLocation,
}

/// 别名目标枚举
#[derive(Clone, Debug)] // Removed Copy trait
pub enum WatAliasTarget {
    /// 外部别名
    Outer {
        /// 外部索引
        outer_index: u32,
        /// 项目索引
        item_index: u32,
    },
    /// 核心别名
    Core {
        /// 核心项目类型
        core_type: WatCoreType,
        /// 项目索引
        item_index: u32,
    },
    /// 导出别名 (Component Model)
    Export {
        /// 实例名称
        instance: String,
        /// 导出名称
        name: String,
    },
    /// 核心导出别名 (Core Module)
    CoreExport {
        /// 实例名称
        instance: String,
        /// 导出名称
        name: String,
    },
}

/// 核心类型枚举
#[derive(Clone, Debug, Copy)]
pub enum WatCoreType {
    /// 核心模块
    Module,
    /// 核心函数
    Func,
    /// 核心表
    Table,
    /// 核心内存
    Memory,
    /// 核心全局变量
    Global,
}

/// 实例定义
#[derive(Clone, Debug)]
pub struct WatInstance {
    /// 实例名称（可选）
    pub name: Option<String>,
    /// 实例化目标
    pub instantiate: WatInstantiate,
    /// 实例化参数
    pub args: Vec<WatInstanceArg>,
    /// 源代码位置信息
    pub location: SourceLocation,
}

/// 实例化目标
#[derive(Clone, Debug)]
pub enum WatInstantiate {
    /// 模块引用
    Module(String),
    /// 组件引用
    Component(String),
}

/// 实例化参数
#[derive(Clone, Debug)]
pub struct WatInstanceArg {
    /// 参数名称
    pub name: String,
    /// 参数值
    pub value: String,
    /// 源代码位置信息
    pub location: SourceLocation,
}

/// 核心模块定义
#[derive(Clone, Debug)]
pub struct WatCoreModule {
    /// 模块名称（可选）
    pub name: Option<String>,
    /// 模块内容
    pub items: Vec<WatCoreModuleItem>,
    /// 自定义段列表
    pub custom_sections: Vec<WatCustomSection>,
    /// 源代码位置信息
    pub location: SourceLocation,
}

/// 核心模块项目枚举
#[derive(Clone, Debug)]
pub enum WatCoreModuleItem {
    /// 导入声明
    Import(WatCoreImport),
    /// 导出声明
    Export(WatCoreExport),
    /// 函数定义
    Func(WatCoreFunc),
    /// 表定义
    Table(WatCoreTable),
    /// 内存定义
    Memory(WatCoreMemory),
    /// 全局变量定义
    Global(WatCoreGlobal),
    /// 启动函数
    Start(String),
    /// 数据段
    Data(WatCoreData),
    /// 元素段
    Elem(WatCoreElem),
}

/// 核心导入声明
#[derive(Clone, Debug)]
pub struct WatCoreImport {
    /// 导入模块名
    pub module: String,
    /// 导入项目名
    pub name: String,
    /// 导入类型
    pub import_type: WatCoreImportType,
    /// 本地名称（可选）
    pub local_name: Option<String>,
    /// 源代码位置信息
    pub location: SourceLocation,
}

/// 核心导入类型枚举
#[derive(Clone, Debug)]
pub enum WatCoreImportType {
    /// 函数类型
    Func(WatCoreFuncType),
    /// 表类型
    Table(WatCoreTableType),
    /// 内存类型
    Memory(WatCoreMemoryType),
    /// 全局变量类型
    Global(WatCoreGlobalType),
}

/// 核心导出声明
#[derive(Clone, Debug)]
pub struct WatCoreExport {
    /// 导出名称
    pub name: String,
    /// 导出项目
    pub export_item: WatCoreExportItem,
    /// 源代码位置信息
    pub location: SourceLocation,
}

/// 核心导出项目枚举
#[derive(Clone, Debug)]
pub enum WatCoreExportItem {
    /// 函数引用
    Func(String),
    /// 表引用
    Table(String),
    /// 内存引用
    Memory(String),
    /// 全局变量引用
    Global(String),
}

/// 核心函数定义
#[derive(Clone, Debug)]
pub struct WatCoreFunc {
    /// 函数名称（可选）
    pub name: Option<String>,
    /// 函数类型
    pub func_type: WatCoreFuncType,
    /// 函数体（可选，导入函数没有函数体）
    pub body: Option<Vec<WatInstruction>>,
    /// 是否为 canon lower/lift
    pub canon: Option<WatCanonicalOperation>,
    /// 源代码位置信息
    pub location: SourceLocation,
}

/// 规范操作枚举
#[derive(Clone, Debug)]
pub enum WatCanonicalOperation {
    /// Lower 操作
    Lower {
        /// 目标函数
        func: String,
        /// 选项
        options: Vec<WatCanonOption>,
    },
    /// Lift 操作
    Lift {
        /// 目标函数
        func: String,
        /// 选项
        options: Vec<WatCanonOption>,
    },
    /// Resource.new 操作
    ResourceNew(String),
    /// Resource.drop 操作
    ResourceDrop(String),
    /// Resource.rep 操作
    ResourceRep(String),
}

/// 规范选项
#[derive(Clone, Debug)]
pub enum WatCanonOption {
    /// 字符串编码
    StringEncoding(String),
    /// 内存选项
    Memory(String),
    /// 重新分配器
    Realloc(String),
}

/// 核心函数类型
#[derive(Clone, Debug)]
pub struct WatCoreFuncType {
    /// 参数类型列表
    pub params: Vec<WatValueType>,
    /// 返回值类型列表
    pub results: Vec<WatValueType>,
    /// 源代码位置信息
    pub location: SourceLocation,
}

/// WebAssembly 值类型枚举
#[derive(Clone, Debug, Copy)]
pub enum WatValueType {
    /// 32位整数
    I32,
    /// 64位整数
    I64,
    /// 32位浮点数
    F32,
    /// 64位浮点数
    F64,
    /// 128位向量
    V128,
    /// 函数引用
    Funcref,
    /// 外部引用
    Externref,
}

impl From<crate::program::WasmValueType> for WatValueType {
    fn from(value: crate::program::WasmValueType) -> Self {
        match value {
            crate::program::WasmValueType::I32 => WatValueType::I32,
            crate::program::WasmValueType::I64 => WatValueType::I64,
            crate::program::WasmValueType::F32 => WatValueType::F32,
            crate::program::WasmValueType::F64 => WatValueType::F64,
            crate::program::WasmValueType::V128 => WatValueType::V128,
            crate::program::WasmValueType::Funcref => WatValueType::Funcref,
            crate::program::WasmValueType::Externref => WatValueType::Externref,
        }
    }
}

impl fmt::Display for WatValueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WatValueType::I32 => write!(f, "i32"),
            WatValueType::I64 => write!(f, "i64"),
            WatValueType::F32 => write!(f, "f32"),
            WatValueType::F64 => write!(f, "f64"),
            WatValueType::V128 => write!(f, "v128"),
            WatValueType::Funcref => write!(f, "funcref"),
            WatValueType::Externref => write!(f, "externref"),
        }
    }
}

/// 核心表定义
#[derive(Clone, Debug)]
pub struct WatCoreTable {
    /// 表名称（可选）
    pub name: Option<String>,
    /// 表类型
    pub table_type: WatCoreTableType,
    /// 源代码位置信息
    pub location: SourceLocation,
}

/// 核心表类型
#[derive(Clone, Debug)]
pub struct WatCoreTableType {
    /// 最小大小
    pub min: u32,
    /// 最大大小（可选）
    pub max: Option<u32>,
    /// 元素类型
    pub element_type: WatValueType,
    /// 源代码位置信息
    pub location: SourceLocation,
}

/// 核心内存定义
#[derive(Clone, Debug)]
pub struct WatCoreMemory {
    /// 内存名称（可选）
    pub name: Option<String>,
    /// 内存类型
    pub memory_type: WatCoreMemoryType,
    /// 源代码位置信息
    pub location: SourceLocation,
}

/// 核心内存类型
#[derive(Clone, Debug)]
pub struct WatCoreMemoryType {
    /// 最小页数
    pub min: u32,
    /// 最大页数（可选）
    pub max: Option<u32>,
    /// 源代码位置信息
    pub location: SourceLocation,
}

/// 核心全局变量定义
#[derive(Clone, Debug)]
pub struct WatCoreGlobal {
    /// 全局变量名称（可选）
    pub name: Option<String>,
    /// 全局变量类型
    pub global_type: WatCoreGlobalType,
    /// 初始值表达式
    pub init: Vec<WatInstruction>,
    /// 源代码位置信息
    pub location: SourceLocation,
}

/// 核心全局变量类型
#[derive(Clone, Debug)]
pub struct WatCoreGlobalType {
    /// 值类型
    pub value_type: WatValueType,
    /// 是否可变
    pub mutable: bool,
    /// 源代码位置信息
    pub location: SourceLocation,
}

/// 核心数据段
#[derive(Clone, Debug)]
pub struct WatCoreData {
    /// 数据段名称（可选）
    pub name: Option<String>,
    /// 内存索引
    pub memory: Option<String>,
    /// 偏移表达式
    pub offset: Vec<WatInstruction>,
    /// 数据内容
    pub data: Vec<u8>,
    /// 源代码位置信息
    pub location: SourceLocation,
}

/// 核心元素段
#[derive(Clone, Debug)]
pub struct WatCoreElem {
    /// 元素段名称（可选）
    pub name: Option<String>,
    /// 表索引
    pub table: Option<String>,
    /// 偏移表达式
    pub offset: Vec<WatInstruction>,
    /// 元素列表
    pub elements: Vec<String>,
    /// 源代码位置信息
    pub location: SourceLocation,
}

/// 核心实例定义
#[derive(Clone, Debug)]
pub struct WatCoreInstance {
    /// 实例名称（可选）
    pub name: Option<String>,
    /// 实例化目标
    pub instantiate: String,
    /// 实例化参数
    pub args: Vec<WatInstanceArg>,
    /// 源代码位置信息
    pub location: SourceLocation,
}

/// 模块定义（简化形式）
#[derive(Clone, Debug)]
pub struct WatModule {
    /// 模块名称（可选）
    pub name: Option<String>,
    /// 模块内容
    pub items: Vec<WatCoreModuleItem>,
    /// 源代码位置信息
    pub location: SourceLocation,
}

/// WebAssembly 指令
#[derive(Clone, Debug)]
pub struct WatInstruction {
    /// 指令操作码
    pub opcode: String,
    /// 指令操作数
    pub operands: Vec<WatOperand>,
    /// 源代码位置信息
    pub location: SourceLocation,
}

impl From<crate::program::WasiInstruction> for WatInstruction {
    fn from(value: crate::program::WasiInstruction) -> Self {
        match value {
            crate::program::WasiInstruction::Nop => WatInstruction { opcode: "nop".to_string(), operands: vec![], location: SourceLocation::default() },
            crate::program::WasiInstruction::Unreachable => WatInstruction { opcode: "unreachable".to_string(), operands: vec![], location: SourceLocation::default() },
            crate::program::WasiInstruction::Block { block_type } => WatInstruction { opcode: "block".to_string(), operands: block_type.map(|t| WatOperand::Identifier(t.to_string())).into_iter().collect(), location: SourceLocation::default() },
            crate::program::WasiInstruction::Loop { block_type } => WatInstruction { opcode: "loop".to_string(), operands: block_type.map(|t| WatOperand::Identifier(t.to_string())).into_iter().collect(), location: SourceLocation::default() },
            crate::program::WasiInstruction::If { block_type } => WatInstruction { opcode: "if".to_string(), operands: block_type.map(|t| WatOperand::Identifier(t.to_string())).into_iter().collect(), location: SourceLocation::default() },
            crate::program::WasiInstruction::Else => WatInstruction { opcode: "else".to_string(), operands: vec![], location: SourceLocation::default() },
            crate::program::WasiInstruction::End => WatInstruction { opcode: "end".to_string(), operands: vec![], location: SourceLocation::default() },
            crate::program::WasiInstruction::Br { label_index } => WatInstruction { opcode: "br".to_string(), operands: vec![WatOperand::Integer(label_index as i64)], location: SourceLocation::default() },
            crate::program::WasiInstruction::BrIf { label_index } => WatInstruction { opcode: "br_if".to_string(), operands: vec![WatOperand::Integer(label_index as i64)], location: SourceLocation::default() },
            crate::program::WasiInstruction::Return => WatInstruction { opcode: "return".to_string(), operands: vec![], location: SourceLocation::default() },
            crate::program::WasiInstruction::Call { function_index } => WatInstruction { opcode: "call".to_string(), operands: vec![WatOperand::Integer(function_index as i64)], location: SourceLocation::default() },
            crate::program::WasiInstruction::Drop => WatInstruction { opcode: "drop".to_string(), operands: vec![], location: SourceLocation::default() },
            crate::program::WasiInstruction::Select => WatInstruction { opcode: "select".to_string(), operands: vec![], location: SourceLocation::default() },
            crate::program::WasiInstruction::LocalGet { local_index } => WatInstruction { opcode: "local.get".to_string(), operands: vec![WatOperand::Integer(local_index as i64)], location: SourceLocation::default() },
            crate::program::WasiInstruction::LocalSet { local_index } => WatInstruction { opcode: "local.set".to_string(), operands: vec![WatOperand::Integer(local_index as i64)], location: SourceLocation::default() },
            crate::program::WasiInstruction::I32Const { value } => WatInstruction { opcode: "i32.const".to_string(), operands: vec![WatOperand::Integer(value as i64)], location: SourceLocation::default() },
            crate::program::WasiInstruction::I64Const { value } => WatInstruction { opcode: "i64.const".to_string(), operands: vec![WatOperand::Integer(value as i64)], location: SourceLocation::default() },
            crate::program::WasiInstruction::F32Const { value } => WatInstruction { opcode: "f32.const".to_string(), operands: vec![WatOperand::Float(value as f64)], location: SourceLocation::default() },
            crate::program::WasiInstruction::F64Const { value } => WatInstruction { opcode: "f64.const".to_string(), operands: vec![WatOperand::Float(value)], location: SourceLocation::default() },
            crate::program::WasiInstruction::I32Add => WatInstruction { opcode: "i32.add".to_string(), operands: vec![], location: SourceLocation::default() },
            crate::program::WasiInstruction::I32Sub => WatInstruction { opcode: "i32.sub".to_string(), operands: vec![], location: SourceLocation::default() },
            crate::program::WasiInstruction::I32Mul => WatInstruction { opcode: "i32.mul".to_string(), operands: vec![], location: SourceLocation::default() },
            crate::program::WasiInstruction::I32DivS => WatInstruction { opcode: "i32.div_s".to_string(), operands: vec![], location: SourceLocation::default() },
            crate::program::WasiInstruction::I32DivU => WatInstruction { opcode: "i32.div_u".to_string(), operands: vec![], location: SourceLocation::default() },
            crate::program::WasiInstruction::I32RemS => WatInstruction { opcode: "i32.rem_s".to_string(), operands: vec![], location: SourceLocation::default() },
            crate::program::WasiInstruction::I32RemU => WatInstruction { opcode: "i32.rem_u".to_string(), operands: vec![], location: SourceLocation::default() },
            crate::program::WasiInstruction::I32And => WatInstruction { opcode: "i32.and".to_string(), operands: vec![], location: SourceLocation::default() },
            crate::program::WasiInstruction::I32Or => WatInstruction { opcode: "i32.or".to_string(), operands: vec![], location: SourceLocation::default() },
            crate::program::WasiInstruction::I32Xor => WatInstruction { opcode: "i32.xor".to_string(), operands: vec![], location: SourceLocation::default() },
            crate::program::WasiInstruction::I32Shl => WatInstruction { opcode: "i32.shl".to_string(), operands: vec![], location: SourceLocation::default() },
            crate::program::WasiInstruction::I32ShrS => WatInstruction { opcode: "i32.shr_s".to_string(), operands: vec![], location: SourceLocation::default() },
            crate::program::WasiInstruction::I32ShrU => WatInstruction { opcode: "i32.shr_u".to_string(), operands: vec![], location: SourceLocation::default() },
            crate::program::WasiInstruction::I32Rotl => WatInstruction { opcode: "i32.rotl".to_string(), operands: vec![], location: SourceLocation::default() },
            crate::program::WasiInstruction::I32Rotr => WatInstruction { opcode: "i32.rotr".to_string(), operands: vec![], location: SourceLocation::default() },
            crate::program::WasiInstruction::I32Eqz => WatInstruction { opcode: "i32.eqz".to_string(), operands: vec![], location: SourceLocation::default() },
            crate::program::WasiInstruction::I32Eq => WatInstruction { opcode: "i32.eq".to_string(), operands: vec![], location: SourceLocation::default() },
            crate::program::WasiInstruction::I32Ne => WatInstruction { opcode: "i32.ne".to_string(), operands: vec![], location: SourceLocation::default() },
            crate::program::WasiInstruction::I32LtS => WatInstruction { opcode: "i32.lt_s".to_string(), operands: vec![], location: SourceLocation::default() },
            crate::program::WasiInstruction::I32LtU => WatInstruction { opcode: "i32.lt_u".to_string(), operands: vec![], location: SourceLocation::default() },
            crate::program::WasiInstruction::I32GtS => WatInstruction { opcode: "i32.gt_s".to_string(), operands: vec![], location: SourceLocation::default() },
            crate::program::WasiInstruction::I32GtU => WatInstruction { opcode: "i32.gt_u".to_string(), operands: vec![], location: SourceLocation::default() },
            crate::program::WasiInstruction::I32LeS => WatInstruction { opcode: "i32.le_s".to_string(), operands: vec![], location: SourceLocation::default() },
            crate::program::WasiInstruction::I32LeU => WatInstruction { opcode: "i32.le_u".to_string(), operands: vec![], location: SourceLocation::default() },
            crate::program::WasiInstruction::I32GeS => WatInstruction { opcode: "i32.ge_s".to_string(), operands: vec![], location: SourceLocation::default() },
            crate::program::WasiInstruction::I32GeU => WatInstruction { opcode: "i32.ge_u".to_string(), operands: vec![], location: SourceLocation::default() },
        }
    }
}

use std::fmt;

/// 指令操作数枚举
#[derive(Clone, Debug)]
pub enum WatOperand {
    /// 整数字面量
    Integer(i64),
    /// 浮点数字面量
    Float(f64),
    /// 字符串字面量
    String(String),
    /// 标识符引用
    Identifier(String),
}

impl fmt::Display for WatOperand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WatOperand::Integer(i) => write!(f, "{}", i),
            WatOperand::Float(fl) => write!(f, "{}", fl),
            WatOperand::String(s) => write!(f, "\"{s}\""),
            WatOperand::Identifier(id) => write!(f, "${}", id),
        }
    }
}

/// 自定义段定义
#[derive(Debug, Clone)]
pub struct WatCustomSection {
    /// 段名称
    pub name: String,
    /// 段数据（文本形式）
    pub data: Vec<u8>,
    /// 源代码位置信息
    pub location: SourceLocation,
}

#[derive(Copy, Debug, Clone)]
pub struct WatTable {}
