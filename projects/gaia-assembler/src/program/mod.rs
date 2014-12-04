use crate::{instruction::GaiaInstruction, types::GaiaType};
use serde::{Deserialize, Serialize};

/// Gaia 程序
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GaiaProgram {
    /// 程序名
    pub name: String,
    /// 函数列表
    pub functions: Vec<GaiaFunction>,
    /// 常量池（名称，值）
    pub constants: Vec<(String, GaiaConstant)>,
    /// 全局变量列表（可选）
    pub globals: Option<Vec<GaiaGlobal>>,
}

impl Default for GaiaProgram {
    fn default() -> Self {
        Self { name: "untitled".to_string(), functions: Vec::new(), constants: Vec::new(), globals: None }
    }
}

impl GaiaProgram {
    /// 创建新的空程序
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), functions: Vec::new(), constants: Vec::new(), globals: None }
    }

    /// 添加函数
    pub fn add_function(&mut self, function: GaiaFunction) {
        self.functions.push(function);
    }

    /// 添加常量
    pub fn add_constant(&mut self, name: impl Into<String>, value: GaiaConstant) {
        self.constants.push((name.into(), value));
    }

    /// 添加全局变量
    pub fn add_global(&mut self, global: GaiaGlobal) {
        if let Some(ref mut globals) = self.globals {
            globals.push(global);
        }
        else {
            self.globals = Some(vec![global]);
        }
    }
}

/// Gaia 常量值
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GaiaConstant {
    /// 8位整数常量
    Integer8(i8),
    /// 16位整数常量
    Integer16(i16),
    /// 32位整数常量
    Integer32(i32),
    /// 64位整数常量
    Integer64(i64),
    /// 32位浮点常量
    Float32(f32),
    /// 64位浮点常量
    Float64(f64),
    /// 布尔常量
    Boolean(bool),
    /// 字符串常量
    String(String),
    /// 空值
    Null,
}

/// Gaia 全局变量
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GaiaGlobal {
    /// 变量名
    pub name: String,
    /// 变量类型
    pub var_type: GaiaType,
    /// 初始值（可选）
    pub initial_value: Option<GaiaConstant>,
    /// 是否为常量
    pub is_constant: bool,
}

/// Gaia 函数定义
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GaiaFunction {
    /// 函数名
    pub name: String,
    /// 参数类型列表
    pub parameters: Vec<GaiaType>,
    /// 返回类型
    pub return_type: Option<GaiaType>,
    /// 指令序列
    pub instructions: Vec<GaiaInstruction>,
    /// 局部变量类型列表
    pub locals: Vec<GaiaType>,
}

impl GaiaFunction {
    /// 创建新函数
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), parameters: Vec::new(), return_type: None, instructions: Vec::new(), locals: Vec::new() }
    }

    /// 添加参数
    pub fn add_parameter(&mut self, param_type: GaiaType) {
        self.parameters.push(param_type);
    }

    /// 设置返回类型
    pub fn set_return_type(&mut self, return_type: GaiaType) {
        self.return_type = Some(return_type);
    }

    /// 添加指令
    pub fn add_instruction(&mut self, instruction: GaiaInstruction) {
        self.instructions.push(instruction);
    }

    /// 添加局部变量
    pub fn add_local(&mut self, local_type: GaiaType) {
        self.locals.push(local_type);
    }
}
