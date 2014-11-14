use gaia_types::{GaiaError, Result, SourceLocation};
use std::collections::HashMap;

/// CLR 程序的高层语义信息结构
///
/// 该结构体表示一个完整的 .NET 程序集，包含了运行时所需的所有信息。
/// 采用与 wasi-assembler 和 jvm-assembler 相同的架构设计。
#[derive(Debug, Clone)]
pub struct ClrProgram {
    /// 程序集名称
    pub name: String,
    /// 程序集版本信息
    pub version: ClrVersion,
    /// 访问标志
    pub access_flags: ClrAccessFlags,
    /// 外部程序集引用
    pub external_assemblies: Vec<ClrExternalAssembly>,
    /// 模块信息
    pub module: Option<ClrModule>,
    /// 类型定义列表
    pub types: Vec<ClrType>,
    /// 全局方法列表
    pub global_methods: Vec<ClrMethod>,
    /// 全局字段列表
    pub global_fields: Vec<ClrField>,
    /// 属性列表
    pub attributes: Vec<ClrAttribute>,
    /// 常量池（字符串、GUID、Blob 等）
    pub constant_pool: ClrConstantPool,
    /// 源文件信息
    pub source_file: Option<String>,
}

/// CLR 版本信息
#[derive(Debug, Clone, Copy)]
pub struct ClrVersion {
    /// 主版本号
    pub major: u16,
    /// 次版本号
    pub minor: u16,
    /// 构建号
    pub build: u16,
    /// 修订号
    pub revision: u16,
}

/// CLR 访问标志
#[derive(Debug, Clone, Copy)]
pub struct ClrAccessFlags {
    /// 是否为公共程序集
    pub is_public: bool,
    /// 是否为私有程序集
    pub is_private: bool,
    /// 是否启用安全透明
    pub is_security_transparent: bool,
    /// 是否为可重定位程序集
    pub is_retargetable: bool,
}

/// 外部程序集引用
#[derive(Debug, Clone)]
pub struct ClrExternalAssembly {
    /// 程序集名称
    pub name: String,
    /// 版本信息
    pub version: ClrVersion,
    /// 公钥标记
    pub public_key_token: Option<Vec<u8>>,
    /// 文化信息
    pub culture: Option<String>,
    /// 哈希算法
    pub hash_algorithm: Option<u32>,
}

/// 模块信息
#[derive(Debug, Clone)]
pub struct ClrModule {
    /// 模块名称
    pub name: String,
    /// 模块版本 ID
    pub mvid: Option<Vec<u8>>,
}

/// CLR 类型定义
#[derive(Debug, Clone)]
pub struct ClrType {
    /// 类型名称
    pub name: String,
    /// 命名空间
    pub namespace: Option<String>,
    /// 访问标志
    pub access_flags: ClrAccessFlags,
    /// 基类型
    pub base_type: Option<String>,
    /// 实现的接口
    pub interfaces: Vec<String>,
    /// 字段列表
    pub fields: Vec<ClrField>,
    /// 方法列表
    pub methods: Vec<ClrMethod>,
    /// 属性列表
    pub properties: Vec<ClrProperty>,
    /// 事件列表
    pub events: Vec<ClrEvent>,
    /// 嵌套类型
    pub nested_types: Vec<ClrType>,
    /// 属性
    pub attributes: Vec<ClrAttribute>,
}

/// CLR 方法定义
#[derive(Debug, Clone)]
pub struct ClrMethod {
    /// 方法名称
    pub name: String,
    /// 返回类型
    pub return_type: ClrTypeReference,
    /// 参数列表
    pub parameters: Vec<ClrParameter>,
    /// 访问标志
    pub access_flags: ClrAccessFlags,
    /// 方法实现标志
    pub impl_flags: ClrMethodImplFlags,
    /// 指令列表
    pub instructions: Vec<ClrInstruction>,
    /// 最大栈深度
    pub max_stack: u16,
    /// 局部变量
    pub locals: Vec<ClrLocalVariable>,
    /// 异常处理表
    pub exception_handlers: Vec<ClrExceptionHandler>,
    /// 属性
    pub attributes: Vec<ClrAttribute>,
    /// 是否为入口点
    pub is_entry_point: bool,
}

/// CLR 字段定义
#[derive(Debug, Clone)]
pub struct ClrField {
    /// 字段名称
    pub name: String,
    /// 字段类型
    pub field_type: ClrTypeReference,
    /// 访问标志
    pub access_flags: ClrAccessFlags,
    /// 默认值
    pub default_value: Option<ClrConstantValue>,
    /// 属性
    pub attributes: Vec<ClrAttribute>,
}

/// CLR 属性定义
#[derive(Debug, Clone)]
pub struct ClrProperty {
    /// 属性名称
    pub name: String,
    /// 属性类型
    pub property_type: ClrTypeReference,
    /// Getter 方法
    pub getter: Option<String>,
    /// Setter 方法
    pub setter: Option<String>,
    /// 属性
    pub attributes: Vec<ClrAttribute>,
}

/// CLR 事件定义
#[derive(Debug, Clone)]
pub struct ClrEvent {
    /// 事件名称
    pub name: String,
    /// 事件类型
    pub event_type: ClrTypeReference,
    /// 添加方法
    pub add_method: Option<String>,
    /// 移除方法
    pub remove_method: Option<String>,
    /// 触发方法
    pub raise_method: Option<String>,
    /// 属性
    pub attributes: Vec<ClrAttribute>,
}

/// CLR 指令
#[derive(Debug, Clone, PartialEq)]
pub enum ClrInstruction {
    /// 简单指令（无操作数）
    Simple { opcode: ClrOpcode },
    /// 带立即数的指令
    WithImmediate { opcode: ClrOpcode, value: i32 },
    /// 带局部变量索引的指令
    WithLocalVar { opcode: ClrOpcode, index: u16 },
    /// 带参数索引的指令
    WithParameter { opcode: ClrOpcode, index: u16 },
    /// 带字段引用的指令
    WithField { opcode: ClrOpcode, field_ref: String },
    /// 带方法引用的指令
    WithMethod { opcode: ClrOpcode, method_ref: String },
    /// 带类型引用的指令
    WithType { opcode: ClrOpcode, type_ref: String },
    /// 带字符串的指令
    WithString { opcode: ClrOpcode, value: String },
    /// 带分支标签的指令
    WithLabel { opcode: ClrOpcode, label: String },
    /// 带 switch 表的指令
    WithSwitch { opcode: ClrOpcode, labels: Vec<String> },
}

/// CLR 操作码
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClrOpcode {
    // 常量加载指令
    Nop,
    LdcI4,
    LdcI4S,
    LdcI4M1,
    LdcI40,
    LdcI41,
    LdcI42,
    LdcI43,
    LdcI44,
    LdcI45,
    LdcI46,
    LdcI47,
    LdcI48,
    LdcI8,
    LdcR4,
    LdcR8,
    Ldnull,
    Ldstr,

    // 参数和局部变量指令
    Ldarg,
    LdargS,
    Ldarg0,
    Ldarg1,
    Ldarg2,
    Ldarg3,
    Ldloc,
    LdlocS,
    Ldloc0,
    Ldloc1,
    Ldloc2,
    Ldloc3,
    Starg,
    StargS,
    Stloc,
    StlocS,
    Stloc0,
    Stloc1,
    Stloc2,
    Stloc3,
    Ldarga,
    LdargaS,
    Ldloca,
    LdlocaS,

    // 数组指令
    Ldelem,
    LdelemI1,
    LdelemU1,
    LdelemI2,
    LdelemU2,
    LdelemI4,
    LdelemU4,
    LdelemI8,
    LdelemI,
    LdelemR4,
    LdelemR8,
    LdelemRef,
    Stelem,
    StelemI1,
    StelemI2,
    StelemI4,
    StelemI8,
    StelemI,
    StelemR4,
    StelemR8,
    StelemRef,
    Ldlen,
    Newarr,

    // 字段指令
    Ldfld,
    Ldflda,
    Stfld,
    Ldsfld,
    Ldsflda,
    Stsfld,

    // 方法调用指令
    Call,
    Callvirt,
    Calli,
    Ret,

    // 对象指令
    Newobj,
    Castclass,
    Isinst,
    Unbox,
    UnboxAny,
    Box,

    // 控制流指令
    Br,
    BrS,
    Brtrue,
    BrtrueS,
    Brfalse,
    BrfalseS,
    Beq,
    BeqS,
    Bne,
    BneS,
    Blt,
    BltS,
    BltUn,
    BltUnS,
    Ble,
    BleS,
    BleUn,
    BleUnS,
    Bgt,
    BgtS,
    BgtUn,
    BgtUnS,
    Bge,
    BgeS,
    BgeUn,
    BgeUnS,
    Switch,

    // 算术指令
    Add,
    AddOvf,
    AddOvfUn,
    Sub,
    SubOvf,
    SubOvfUn,
    Mul,
    MulOvf,
    MulOvfUn,
    Div,
    DivUn,
    Rem,
    RemUn,
    And,
    Or,
    Xor,
    Not,
    Shl,
    Shr,
    ShrUn,
    Neg,

    // 比较指令
    Ceq,
    Cgt,
    CgtUn,
    Clt,
    CltUn,

    // 转换指令
    ConvI1,
    ConvI2,
    ConvI4,
    ConvI8,
    ConvR4,
    ConvR8,
    ConvU4,
    ConvU8,
    ConvOvfI1,
    ConvOvfI2,
    ConvOvfI4,
    ConvOvfI8,
    ConvOvfU1,
    ConvOvfU2,
    ConvOvfU4,
    ConvOvfU8,
    ConvOvfI1Un,
    ConvOvfI2Un,
    ConvOvfI4Un,
    ConvOvfI8Un,
    ConvOvfU1Un,
    ConvOvfU2Un,
    ConvOvfU4Un,
    ConvOvfU8Un,
    ConvRUn,
    ConvOvfIUn,
    ConvOvfUUn,

    // 栈操作指令
    Dup,
    Pop,

    // 异常处理指令
    Throw,
    Rethrow,
    Leave,
    LeaveS,
    Endfinally,
    Endfilter,

    // 其他指令
    Sizeof,
    Refanytype,
    Refanyval,
    Mkrefany,
    Arglist,
    Localloc,
    Jmp,
    Calli2,
    Tail,
    Volatile,
    Unaligned,
    Constrained,
    Readonly,
}

/// CLR 类型引用
#[derive(Debug, Clone, PartialEq)]
pub struct ClrTypeReference {
    /// 类型名称
    pub name: String,
    /// 命名空间
    pub namespace: Option<String>,
    /// 程序集引用
    pub assembly: Option<String>,
    /// 是否为值类型
    pub is_value_type: bool,
    /// 是否为引用类型
    pub is_reference_type: bool,
    /// 泛型参数
    pub generic_parameters: Vec<ClrTypeReference>,
}

/// CLR 参数
#[derive(Debug, Clone)]
pub struct ClrParameter {
    /// 参数名称
    pub name: String,
    /// 参数类型
    pub parameter_type: ClrTypeReference,
    /// 是否为输入参数
    pub is_in: bool,
    /// 是否为输出参数
    pub is_out: bool,
    /// 是否为可选参数
    pub is_optional: bool,
    /// 默认值
    pub default_value: Option<ClrConstantValue>,
    /// 属性
    pub attributes: Vec<ClrAttribute>,
}

/// CLR 局部变量
#[derive(Debug, Clone)]
pub struct ClrLocalVariable {
    /// 变量名称（可选）
    pub name: Option<String>,
    /// 变量类型
    pub variable_type: ClrTypeReference,
    /// 是否为固定变量
    pub is_pinned: bool,
}

/// CLR 异常处理器
#[derive(Debug, Clone)]
pub struct ClrExceptionHandler {
    /// 处理器类型
    pub handler_type: ClrExceptionHandlerType,
    /// 尝试块开始偏移
    pub try_start: u32,
    /// 尝试块长度
    pub try_length: u32,
    /// 处理器开始偏移
    pub handler_start: u32,
    /// 处理器长度
    pub handler_length: u32,
    /// 异常类型（对于 catch 处理器）
    pub catch_type: Option<ClrTypeReference>,
    /// 过滤器开始偏移（对于 filter 处理器）
    pub filter_start: Option<u32>,
}

/// CLR 异常处理器类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClrExceptionHandlerType {
    /// Catch 处理器
    Catch,
    /// Filter 处理器
    Filter,
    /// Finally 处理器
    Finally,
    /// Fault 处理器
    Fault,
}

/// CLR 方法实现标志
#[derive(Debug, Clone, Copy)]
pub struct ClrMethodImplFlags {
    /// 是否为托管代码
    pub is_managed: bool,
    /// 是否为本机代码
    pub is_native: bool,
    /// 是否为运行时代码
    pub is_runtime: bool,
    /// 是否为内联
    pub is_inline: bool,
    /// 是否不内联
    pub is_no_inline: bool,
    /// 是否同步
    pub is_synchronized: bool,
}

/// CLR 属性
#[derive(Debug, Clone)]
pub struct ClrAttribute {
    /// 属性类型
    pub attribute_type: ClrTypeReference,
    /// 构造函数参数
    pub constructor_args: Vec<ClrConstantValue>,
    /// 命名参数
    pub named_args: Vec<(String, ClrConstantValue)>,
}

/// CLR 常量值
#[derive(Debug, Clone, PartialEq)]
pub enum ClrConstantValue {
    /// 布尔值
    Boolean(bool),
    /// 8 位有符号整数
    I1(i8),
    /// 8 位无符号整数
    U1(u8),
    /// 16 位有符号整数
    I2(i16),
    /// 16 位无符号整数
    U2(u16),
    /// 32 位有符号整数
    I4(i32),
    /// 32 位无符号整数
    U4(u32),
    /// 64 位有符号整数
    I8(i64),
    /// 64 位无符号整数
    U8(u64),
    /// 32 位浮点数
    R4(f32),
    /// 64 位浮点数
    R8(f64),
    /// 字符串
    String(String),
    /// 空值
    Null,
    /// 类型引用
    Type(ClrTypeReference),
    /// 枚举值
    Enum(ClrTypeReference, Box<ClrConstantValue>),
    /// 数组
    Array(Vec<ClrConstantValue>),
}

/// CLR 常量池
#[derive(Debug, Clone)]
pub struct ClrConstantPool {
    /// 字符串表
    pub strings: HashMap<String, u32>,
    /// GUID 表
    pub guids: HashMap<Vec<u8>, u32>,
    /// Blob 表
    pub blobs: HashMap<Vec<u8>, u32>,
    /// 用户字符串表
    pub user_strings: HashMap<String, u32>,
}

impl ClrProgram {
    /// 创建新的 CLR 程序
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: ClrVersion::default(),
            access_flags: ClrAccessFlags::default(),
            external_assemblies: Vec::new(),
            module: None,
            types: Vec::new(),
            global_methods: Vec::new(),
            global_fields: Vec::new(),
            attributes: Vec::new(),
            constant_pool: ClrConstantPool::new(),
            source_file: None,
        }
    }

    /// 添加类型
    pub fn add_type(&mut self, clr_type: ClrType) {
        self.types.push(clr_type);
    }

    /// 添加外部程序集引用
    pub fn add_external_assembly(&mut self, assembly: ClrExternalAssembly) {
        self.external_assemblies.push(assembly);
    }

    /// 设置模块信息
    pub fn set_module(&mut self, module: ClrModule) {
        self.module = Some(module);
    }

    /// 设置源文件
    pub fn set_source_file(&mut self, filename: String) {
        self.source_file = Some(filename);
    }

    /// 验证程序的完整性
    pub fn validate(&self) -> Result<()> {
        // 验证程序集名称
        if self.name.is_empty() {
            return Err(GaiaError::syntax_error("程序集名称不能为空".to_string(), SourceLocation::default()));
        }

        // 验证类型定义
        for clr_type in &self.types {
            clr_type.validate()?;
        }

        Ok(())
    }

    /// 获取类型数量
    pub fn get_type_count(&self) -> usize {
        self.types.len()
    }

    /// 获取方法数量（包括全局方法和类型中的方法）
    pub fn get_method_count(&self) -> usize {
        let type_methods: usize = self.types.iter().map(|t| t.methods.len()).sum();
        type_methods + self.global_methods.len()
    }

    /// 获取字段数量（包括全局字段和类型中的字段）
    pub fn get_field_count(&self) -> usize {
        let type_fields: usize = self.types.iter().map(|t| t.fields.len()).sum();
        type_fields + self.global_fields.len()
    }

    /// 获取示例类型名称
    pub fn get_sample_type_name(&self) -> Option<String> {
        self.types.first().map(|t| {
            if let Some(namespace) = &t.namespace {
                format!("{}.{}", namespace, t.name)
            }
            else {
                t.name.clone()
            }
        })
    }

    /// 获取示例方法名称
    pub fn get_sample_method_name(&self) -> Option<String> {
        // 首先尝试从类型中获取方法
        for clr_type in &self.types {
            if let Some(method) = clr_type.methods.first() {
                return Some(format!("{}.{}", clr_type.name, method.name));
            }
        }
        // 如果没有类型方法，尝试全局方法
        self.global_methods.first().map(|m| m.name.clone())
    }

    /// 获取引用的程序集列表
    pub fn get_referenced_assemblies(&self) -> Vec<String> {
        self.external_assemblies.iter().map(|a| a.name.clone()).collect()
    }
}

impl ClrConstantPool {
    /// 创建新的常量池
    pub fn new() -> Self {
        Self { strings: HashMap::new(), guids: HashMap::new(), blobs: HashMap::new(), user_strings: HashMap::new() }
    }

    /// 添加字符串到常量池
    pub fn add_string(&mut self, s: String) -> u32 {
        let next_index = self.strings.len() as u32;
        *self.strings.entry(s).or_insert(next_index)
    }

    /// 添加 GUID 到常量池
    pub fn add_guid(&mut self, guid: Vec<u8>) -> u32 {
        let next_index = self.guids.len() as u32;
        *self.guids.entry(guid).or_insert(next_index)
    }

    /// 添加 Blob 到常量池
    pub fn add_blob(&mut self, blob: Vec<u8>) -> u32 {
        let next_index = self.blobs.len() as u32;
        *self.blobs.entry(blob).or_insert(next_index)
    }

    /// 添加用户字符串到常量池
    pub fn add_user_string(&mut self, s: String) -> u32 {
        let next_index = self.user_strings.len() as u32;
        *self.user_strings.entry(s).or_insert(next_index)
    }
}

impl ClrType {
    /// 创建新的类型
    pub fn new(name: String, namespace: Option<String>) -> Self {
        Self {
            name,
            namespace,
            access_flags: ClrAccessFlags::default(),
            base_type: None,
            interfaces: Vec::new(),
            fields: Vec::new(),
            methods: Vec::new(),
            properties: Vec::new(),
            events: Vec::new(),
            nested_types: Vec::new(),
            attributes: Vec::new(),
        }
    }

    /// 添加方法
    pub fn add_method(&mut self, method: ClrMethod) {
        self.methods.push(method);
    }

    /// 添加字段
    pub fn add_field(&mut self, field: ClrField) {
        self.fields.push(field);
    }

    /// 验证类型定义
    pub fn validate(&self) -> Result<()> {
        // 验证类型名称
        if self.name.is_empty() {
            return Err(GaiaError::syntax_error("类型名称不能为空".to_string(), SourceLocation::default()));
        }

        // 验证方法
        for method in &self.methods {
            method.validate()?;
        }

        Ok(())
    }
}

impl ClrMethod {
    /// 创建新的方法
    pub fn new(name: String, return_type: ClrTypeReference) -> Self {
        Self {
            name,
            return_type,
            parameters: Vec::new(),
            access_flags: ClrAccessFlags::default(),
            impl_flags: ClrMethodImplFlags::default(),
            instructions: Vec::new(),
            max_stack: 8,
            locals: Vec::new(),
            exception_handlers: Vec::new(),
            attributes: Vec::new(),
            is_entry_point: false,
        }
    }

    /// 添加指令
    pub fn add_instruction(&mut self, instruction: ClrInstruction) {
        self.instructions.push(instruction);
    }

    /// 添加参数
    pub fn add_parameter(&mut self, parameter: ClrParameter) {
        self.parameters.push(parameter);
    }

    /// 验证方法定义
    pub fn validate(&self) -> Result<()> {
        // 验证方法名称
        if self.name.is_empty() {
            return Err(GaiaError::syntax_error("方法名称不能为空".to_string(), SourceLocation::default()));
        }

        Ok(())
    }
}

impl Default for ClrVersion {
    fn default() -> Self {
        Self { major: 0, minor: 0, build: 0, revision: 0 }
    }
}

impl Default for ClrAccessFlags {
    fn default() -> Self {
        Self { is_public: false, is_private: true, is_security_transparent: false, is_retargetable: false }
    }
}

impl Default for ClrMethodImplFlags {
    fn default() -> Self {
        Self {
            is_managed: true,
            is_native: false,
            is_runtime: false,
            is_inline: false,
            is_no_inline: false,
            is_synchronized: false,
        }
    }
}

impl ClrOpcode {
    /// 将操作码转换为字节
    pub fn to_byte(&self) -> u8 {
        match self {
            ClrOpcode::Nop => 0x00,
            ClrOpcode::LdcI4M1 => 0x15,
            ClrOpcode::LdcI40 => 0x16,
            ClrOpcode::LdcI41 => 0x17,
            ClrOpcode::LdcI42 => 0x18,
            ClrOpcode::LdcI43 => 0x19,
            ClrOpcode::LdcI44 => 0x1A,
            ClrOpcode::LdcI45 => 0x1B,
            ClrOpcode::LdcI46 => 0x1C,
            ClrOpcode::LdcI47 => 0x1D,
            ClrOpcode::LdcI48 => 0x1E,
            ClrOpcode::LdcI4S => 0x1F,
            ClrOpcode::LdcI4 => 0x20,
            ClrOpcode::LdcI8 => 0x21,
            ClrOpcode::LdcR4 => 0x22,
            ClrOpcode::LdcR8 => 0x23,
            ClrOpcode::Ldnull => 0x14,
            ClrOpcode::Ldstr => 0x72,
            ClrOpcode::Ldarg0 => 0x02,
            ClrOpcode::Ldarg1 => 0x03,
            ClrOpcode::Ldarg2 => 0x04,
            ClrOpcode::Ldarg3 => 0x05,
            ClrOpcode::Ldloc0 => 0x06,
            ClrOpcode::Ldloc1 => 0x07,
            ClrOpcode::Ldloc2 => 0x08,
            ClrOpcode::Ldloc3 => 0x09,
            ClrOpcode::Stloc0 => 0x0A,
            ClrOpcode::Stloc1 => 0x0B,
            ClrOpcode::Stloc2 => 0x0C,
            ClrOpcode::Stloc3 => 0x0D,
            ClrOpcode::Call => 0x28,
            ClrOpcode::Callvirt => 0x6F,
            ClrOpcode::Ret => 0x2A,
            ClrOpcode::Newobj => 0x73,
            ClrOpcode::Pop => 0x26,
            ClrOpcode::Dup => 0x25,
            _ => 0x00, // 默认值，实际使用时需要完善所有操作码
        }
    }

    /// 从字符串解析操作码
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "nop" => Some(ClrOpcode::Nop),
            "ldc.i4.m1" => Some(ClrOpcode::LdcI4M1),
            "ldc.i4.0" => Some(ClrOpcode::LdcI40),
            "ldc.i4.1" => Some(ClrOpcode::LdcI41),
            "ldc.i4.2" => Some(ClrOpcode::LdcI42),
            "ldc.i4.3" => Some(ClrOpcode::LdcI43),
            "ldc.i4.4" => Some(ClrOpcode::LdcI44),
            "ldc.i4.5" => Some(ClrOpcode::LdcI45),
            "ldc.i4.6" => Some(ClrOpcode::LdcI46),
            "ldc.i4.7" => Some(ClrOpcode::LdcI47),
            "ldc.i4.8" => Some(ClrOpcode::LdcI48),
            "ldc.i4.s" => Some(ClrOpcode::LdcI4S),
            "ldc.i4" => Some(ClrOpcode::LdcI4),
            "ldc.i8" => Some(ClrOpcode::LdcI8),
            "ldc.r4" => Some(ClrOpcode::LdcR4),
            "ldc.r8" => Some(ClrOpcode::LdcR8),
            "ldnull" => Some(ClrOpcode::Ldnull),
            "ldstr" => Some(ClrOpcode::Ldstr),
            "ldarg.0" => Some(ClrOpcode::Ldarg0),
            "ldarg.1" => Some(ClrOpcode::Ldarg1),
            "ldarg.2" => Some(ClrOpcode::Ldarg2),
            "ldarg.3" => Some(ClrOpcode::Ldarg3),
            "ldloc.0" => Some(ClrOpcode::Ldloc0),
            "ldloc.1" => Some(ClrOpcode::Ldloc1),
            "ldloc.2" => Some(ClrOpcode::Ldloc2),
            "ldloc.3" => Some(ClrOpcode::Ldloc3),
            "stloc.0" => Some(ClrOpcode::Stloc0),
            "stloc.1" => Some(ClrOpcode::Stloc1),
            "stloc.2" => Some(ClrOpcode::Stloc2),
            "stloc.3" => Some(ClrOpcode::Stloc3),
            "call" => Some(ClrOpcode::Call),
            "callvirt" => Some(ClrOpcode::Callvirt),
            "ret" => Some(ClrOpcode::Ret),
            "newobj" => Some(ClrOpcode::Newobj),
            "pop" => Some(ClrOpcode::Pop),
            "dup" => Some(ClrOpcode::Dup),
            _ => None,
        }
    }
}

// 为了兼容性，重新导出一些类型
// pub use ClrProgram as DotNetAssemblyInfo; // 注释掉以避免重复定义

/// 旧版本的 CLR 头结构，用于向后兼容
#[derive(Copy, Debug, Clone)]
pub struct ClrHeader {
    /// 头的总大小（字节）
    pub cb: u32,
    /// CLR 运行时主版本号
    pub major_runtime_version: u16,
    /// CLR 运行时次版本号
    pub minor_runtime_version: u16,
    /// 元数据的相对虚拟地址
    pub metadata_rva: u32,
    /// 元数据的大小（字节）
    pub metadata_size: u32,
    /// 程序集的标志位，如是否为纯 IL 代码等
    pub flags: u32,
}

/// 旧版本的元数据头结构，用于向后兼容
#[derive(Debug, Clone)]
pub struct MetadataHeader {
    /// 魔数，通常为 0x424A5342 (BSJB)
    pub signature: u32,
    /// 元数据格式主版本
    pub major_version: u16,
    /// 元数据格式次版本
    pub minor_version: u16,
    /// 保留字段，通常为 0
    pub reserved: u32,
    /// 运行时版本字符串的长度
    pub version_length: u32,
    /// 运行时版本字符串的内容
    pub version_string: String,
    /// 元数据标志位
    pub flags: u16,
    /// 元数据流的数量
    pub streams: u16,
}

/// 旧版本的流头结构，用于向后兼容
#[derive(Debug, Clone)]
pub struct StreamHeader {
    /// 该流在元数据中的偏移量
    pub offset: u32,
    /// 流的大小（字节）
    pub size: u32,
    /// 流的名称，如 "#Strings"、"#US"、"#GUID"、"#Blob" 等
    pub name: String,
}

/// 旧版本的 .NET 程序集信息，用于向后兼容
#[derive(Debug, Clone)]
pub struct DotNetAssemblyInfo {
    /// 程序集名称
    pub name: String,
    /// 版本号，格式为 major.minor.build.revision
    pub version: String,
    /// 文化区域信息，如 "zh-CN"，null 表示中性文化
    pub culture: Option<String>,
    /// 公钥标记，用于强名称验证
    pub public_key_token: Option<String>,
    /// .NET 运行时版本，如 "v4.0.30319"
    pub runtime_version: Option<String>,
}
