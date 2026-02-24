use super::types::{
    ClrAccessFlags, ClrAttribute, ClrExternalAssembly, ClrField, ClrMethodImplFlags,
    ClrModule, ClrParameter, ClrVersion, ClrTypeReference, ClrLocalVariable, ClrExceptionHandler
};
use super::instructions::ClrInstruction;
use super::pool::ClrConstantPool;
use gaia_types::{GaiaError, Result, SourceLocation};

/// CLR 程序集结构
#[derive(Debug, Clone)]
pub struct ClrProgram {
    /// 程序集名称
    pub name: String,
    /// 程序集版本
    pub version: ClrVersion,
    /// 访问标志
    pub access_flags: ClrAccessFlags,
    /// 外部程序集引用
    pub external_assemblies: Vec<ClrExternalAssembly>,
    /// 模块定义
    pub module: Option<ClrModule>,
    /// 程序集中的类型
    pub types: Vec<ClrType>,
    /// 全局方法
    pub global_methods: Vec<ClrMethod>,
    /// 全局字段
    pub global_fields: Vec<ClrField>,
    /// 程序集特性
    pub attributes: Vec<ClrAttribute>,
    /// 常量池
    pub constant_pool: ClrConstantPool,
    /// 源文件路径
    pub source_file: Option<String>,
}

impl ClrProgram {
    /// 创建新的程序集
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

    /// 添加类型到程序集
    pub fn add_type(&mut self, clr_type: ClrType) {
        self.types.push(clr_type);
    }

    /// 添加全局方法
    pub fn add_global_method(&mut self, method: ClrMethod) {
        self.global_methods.push(method);
    }

    /// 添加全局字段
    pub fn add_global_field(&mut self, field: ClrField) {
        self.global_fields.push(field);
    }

    /// 添加外部程序集引用
    pub fn add_external_assembly(&mut self, assembly: ClrExternalAssembly) {
        self.external_assemblies.push(assembly);
    }

    /// 验证程序集定义
    pub fn validate(&self) -> Result<()> {
        // 验证程序集名称
        if self.name.is_empty() {
            return Err(GaiaError::syntax_error("程序集名称不能为空".to_string(), SourceLocation::default()));
        }

        // 验证所有类型
        for clr_type in &self.types {
            clr_type.validate()?;
        }

        // 验证全局方法
        for method in &self.global_methods {
            method.validate()?;
        }

        Ok(())
    }
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
    /// 基类
    pub base_type: Option<ClrTypeReference>,
    /// 实现的接口
    pub interfaces: Vec<ClrTypeReference>,
    /// 类型中的字段
    pub fields: Vec<ClrField>,
    /// 类型中的方法
    pub methods: Vec<ClrMethod>,
    /// 类型中的属性
    pub properties: Vec<String>, // 简化处理
    /// 类型中的事件
    pub events: Vec<String>, // 简化处理
    /// 嵌套类型
    pub nested_types: Vec<ClrType>,
    /// 类型特性
    pub attributes: Vec<ClrAttribute>,
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

/// CLR 方法定义
#[derive(Debug, Clone)]
pub struct ClrMethod {
    /// 方法名称
    pub name: String,
    /// 返回值类型
    pub return_type: ClrTypeReference,
    /// 方法参数
    pub parameters: Vec<ClrParameter>,
    /// 访问标志
    pub access_flags: ClrAccessFlags,
    /// 实现标志
    pub impl_flags: ClrMethodImplFlags,
    /// 指令列表
    pub instructions: Vec<ClrInstruction>,
    /// 最大栈深度
    pub max_stack: u32,
    /// 局部变量
    pub locals: Vec<ClrLocalVariable>,
    /// 异常处理器
    pub exception_handlers: Vec<ClrExceptionHandler>,
    /// 方法特性
    pub attributes: Vec<ClrAttribute>,
    /// 是否为入口点
    pub is_entry_point: bool,
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
