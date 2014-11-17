//! 统一的适配器接口定义
//!
//! 本模块定义了导入和导出适配器的统一接口，以及相关的配置和管理结构。
//! 这些接口旨在抽象不同平台之间的差异，提供一致的API。

use gaia_types::{
    helpers::CompilationTarget,
    instruction::{GaiaInstruction, GaiaProgram},
    GaiaError, Result,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 适配器配置信息
///
/// 包含适配器运行所需的配置参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterConfig {
    /// 适配器名称
    pub name: String,
    /// 编译目标
    pub compilation_target: CompilationTarget,
    /// 配置参数
    pub parameters: HashMap<String, String>,
    /// 是否启用
    pub enabled: bool,
}

/// 适配器元数据
///
/// 描述适配器的基本信息和能力
#[derive(Debug, Clone)]
pub struct AdapterMetadata {
    /// 适配器名称
    pub name: String,
    /// 适配器版本
    pub version: String,
    /// 支持的编译目标
    pub compilation_target: CompilationTarget,
    /// 适配器描述
    pub description: String,
    /// 支持的指令集
    pub supported_instructions: Vec<String>,
}

/// 统一的导出适配器接口
///
/// 定义了将Gaia指令和程序导出到特定平台格式的标准接口
pub trait ExportAdapter: Send + Sync {
    /// 获取适配器元数据
    fn metadata(&self) -> &AdapterMetadata;

    /// 配置适配器
    ///
    /// # 参数
    /// * `config` - 适配器配置
    ///
    /// # 返回值
    /// 配置成功返回Ok(())，失败返回错误信息
    fn configure(&mut self, config: AdapterConfig) -> Result<()>;

    /// 导出单个指令
    ///
    /// # 参数
    /// * `instruction` - 要导出的Gaia指令
    ///
    /// # 返回值
    /// 导出成功返回平台特定的指令数据，失败返回错误信息
    fn export_instruction(&self, instruction: &GaiaInstruction) -> Result<Vec<u8>>;

    /// 导出完整程序
    ///
    /// # 参数
    /// * `program` - 要导出的Gaia程序
    ///
    /// # 返回值
    /// 导出成功返回平台特定的程序数据，失败返回错误信息
    fn export_program(&self, program: &GaiaProgram) -> Result<Vec<u8>>;

    /// 验证指令是否支持
    ///
    /// # 参数
    /// * `instruction` - 要验证的指令
    ///
    /// # 返回值
    /// 支持返回true，不支持返回false
    fn supports_instruction(&self, instruction: &GaiaInstruction) -> bool;

    /// 获取输出文件扩展名
    ///
    /// # 返回值
    /// 平台特定的文件扩展名
    fn file_extension(&self) -> &str;

    /// 清理资源
    ///
    /// 在适配器不再使用时调用，用于清理相关资源
    fn cleanup(&mut self) -> Result<()> {
        Ok(())
    }
}

/// 统一的导入适配器接口
///
/// 定义了从特定平台格式导入到Gaia指令和程序的标准接口
pub trait ImportAdapter: Send + Sync {
    /// 获取适配器元数据
    fn metadata(&self) -> &AdapterMetadata;

    /// 配置适配器
    ///
    /// # 参数
    /// * `config` - 适配器配置
    ///
    /// # 返回值
    /// 配置成功返回Ok(())，失败返回错误信息
    fn configure(&mut self, config: AdapterConfig) -> Result<()>;

    /// 导入单个指令
    ///
    /// # 参数
    /// * `data` - 平台特定的指令数据
    ///
    /// # 返回值
    /// 导入成功返回Gaia指令，失败返回错误信息
    fn import_instruction(&self, data: &[u8]) -> Result<GaiaInstruction>;

    /// 导入完整程序
    ///
    /// # 参数
    /// * `data` - 平台特定的程序数据
    ///
    /// # 返回值
    /// 导入成功返回Gaia程序，失败返回错误信息
    fn import_program(&self, data: &[u8]) -> Result<GaiaProgram>;

    /// 验证数据格式
    ///
    /// # 参数
    /// * `data` - 要验证的数据
    ///
    /// # 返回值
    /// 格式正确返回true，错误返回false
    fn validate_format(&self, data: &[u8]) -> bool;

    /// 获取支持的文件扩展名
    ///
    /// # 返回值
    /// 支持的文件扩展名列表
    fn supported_extensions(&self) -> Vec<&str>;

    /// 清理资源
    ///
    /// 在适配器不再使用时调用，用于清理相关资源
    fn cleanup(&mut self) -> Result<()> {
        Ok(())
    }
}

/// 适配器管理器
pub struct AdapterManager {
    /// 导出适配器注册表
    export_adapters: HashMap<CompilationTarget, Box<dyn ExportAdapter>>,
    /// 导入适配器注册表
    import_adapters: HashMap<CompilationTarget, Box<dyn ImportAdapter>>,
}

impl AdapterManager {
    /// 创建新的适配器管理器
    pub fn new() -> Self {
        Self { export_adapters: HashMap::new(), import_adapters: HashMap::new() }
    }

    /// 注册导出适配器
    ///
    /// # 参数
    /// * `target` - 编译目标
    /// * `adapter` - 要注册的导出适配器
    ///
    /// # 返回值
    /// 注册成功返回Ok(())，失败返回错误信息
    pub fn register_export_adapter(&mut self, target: CompilationTarget, adapter: Box<dyn ExportAdapter>) -> Result<()> {
        if self.export_adapters.contains_key(&target) {
            return Err(GaiaError::adapter_error(&format!("{:?}", target), "导出适配器已存在", None));
        }
        self.export_adapters.insert(target, adapter);
        Ok(())
    }

    /// 注册导入适配器
    ///
    /// # 参数
    /// * `target` - 编译目标
    /// * `adapter` - 要注册的导入适配器
    ///
    /// # 返回值
    /// 注册成功返回Ok(())，失败返回错误信息
    pub fn register_import_adapter(&mut self, target: CompilationTarget, adapter: Box<dyn ImportAdapter>) -> Result<()> {
        if self.import_adapters.contains_key(&target) {
            return Err(GaiaError::adapter_error(&format!("{:?}", target), "导入适配器已存在", None));
        }
        self.import_adapters.insert(target, adapter);
        Ok(())
    }

    /// 获取导出适配器
    ///
    /// # 参数
    /// * `target` - 编译目标
    ///
    /// # 返回值
    /// 找到返回适配器引用，未找到返回错误
    pub fn get_export_adapter(&self, target: &CompilationTarget) -> Result<&dyn ExportAdapter> {
        self.export_adapters
            .get(target)
            .map(|adapter| adapter.as_ref())
            .ok_or_else(|| GaiaError::adapter_error(&format!("{:?}", target), "导出适配器未找到", None))
    }

    /// 获取可变导出适配器
    ///
    /// # 参数
    /// * `target` - 编译目标
    ///
    /// # 返回值
    /// 找到返回适配器可变引用，未找到返回错误
    pub fn get_export_adapter_mut(&mut self, target: &CompilationTarget) -> Result<&mut (dyn ExportAdapter + '_)> {
        match self.export_adapters.get_mut(target) {
            Some(adapter) => Ok(adapter.as_mut()),
            None => Err(GaiaError::adapter_error(&format!("{:?}", target), "导出适配器未找到", None)),
        }
    }

    /// 获取导入适配器
    ///
    /// # 参数
    /// * `target` - 编译目标
    ///
    /// # 返回值
    /// 找到返回适配器引用，未找到返回错误
    pub fn get_import_adapter(&self, target: &CompilationTarget) -> Result<&dyn ImportAdapter> {
        self.import_adapters
            .get(target)
            .map(|adapter| adapter.as_ref())
            .ok_or_else(|| GaiaError::adapter_error(&format!("{:?}", target), "导入适配器未找到", None))
    }

    /// 获取可变导入适配器
    ///
    /// # 参数
    /// * `target` - 编译目标
    ///
    /// # 返回值
    /// 找到返回适配器可变引用，未找到返回错误
    pub fn get_import_adapter_mut(&mut self, target: &CompilationTarget) -> Result<&mut (dyn ImportAdapter + '_)> {
        match self.import_adapters.get_mut(target) {
            Some(adapter) => Ok(adapter.as_mut()),
            None => Err(GaiaError::adapter_error(&format!("{:?}", target), "导入适配器未找到", None)),
        }
    }

    /// 列出所有支持的编译目标
    ///
    /// # 返回值
    /// 编译目标列表
    pub fn list_supported_targets(&self) -> Vec<CompilationTarget> {
        let mut targets = Vec::new();
        targets.extend(self.export_adapters.keys().cloned());
        targets.extend(self.import_adapters.keys().cloned());
        targets.sort_by_key(|t| format!("{:?}", t));
        targets.dedup();
        targets
    }

    /// 清理所有适配器资源
    ///
    /// # 返回值
    /// 清理成功返回Ok(())，失败返回错误信息
    pub fn cleanup_all(&mut self) -> Result<()> {
        for (target, adapter) in &mut self.export_adapters {
            if let Err(e) = adapter.cleanup() {
                return Err(GaiaError::adapter_error(&format!("{:?}", target), "清理导出适配器失败", Some(Box::new(e))));
            }
        }

        for (target, adapter) in &mut self.import_adapters {
            if let Err(e) = adapter.cleanup() {
                return Err(GaiaError::adapter_error(&format!("{:?}", target), "清理导入适配器失败", Some(Box::new(e))));
            }
        }

        Ok(())
    }
}

impl Default for AdapterManager {
    fn default() -> Self {
        Self::new()
    }
}
