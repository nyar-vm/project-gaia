//! 配置管理模块
//!
//! 本模块负责加载和管理Gaia汇编器的配置，包括平台映射、适配器配置等。
//! 支持从TOML配置文件动态加载配置信息。

use gaia_types::{
    GaiaError,
    helpers::CompilationTarget,
    Result,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// 函数映射配置
///
/// 定义了通用函数名到平台特定函数名的映射关系
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionMapping {
    /// 通用函数名
    pub common_name: String,
    /// 平台特定映射
    pub platform_mappings: HashMap<String, String>,
    /// 函数描述
    pub description: Option<String>,
}

/// 平台配置
///
/// 包含特定平台的配置信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformConfig {
    /// 编译目标
    pub target: CompilationTarget,
    /// 平台描述
    pub description: Option<String>,
    /// 支持的架构
    pub supported_architectures: Vec<String>,
    /// 默认文件扩展名
    pub default_extension: String,
    /// 平台特定参数
    pub parameters: HashMap<String, String>,
}

/// 适配器配置
///
/// 包含适配器的配置信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterConfigEntry {
    /// 适配器名称
    pub name: String,
    /// 适配器类型 (export/import)
    pub adapter_type: String,
    /// 编译目标
    pub compilation_target: CompilationTarget,
    /// 是否启用
    pub enabled: bool,
    /// 适配器参数
    pub parameters: HashMap<String, String>,
}

/// 全局配置
///
/// 包含全局设置参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    /// 默认输出目录
    pub default_output_dir: String,
    /// 是否启用调试模式
    pub debug_mode: bool,
    /// 全局参数
    pub parameters: HashMap<String, String>,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            default_output_dir: "./output".to_string(),
            debug_mode: false,
            parameters: HashMap::new(),
        }
    }
}

/// Gaia汇编器主配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GaiaConfig {
    /// 配置版本
    pub version: String,
    /// 全局设置
    pub global: GlobalConfig,
    /// 平台配置映射
    pub platforms: HashMap<CompilationTarget, PlatformConfig>,
    /// 函数映射列表
    pub function_mappings: Vec<FunctionMapping>,
    /// 适配器配置列表
    pub adapters: Vec<AdapterConfigEntry>,
}

impl Default for GaiaConfig {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            global: GlobalConfig::default(),
            platforms: HashMap::new(),
            function_mappings: vec![
                FunctionMapping {
                    common_name: "__builtin_print".to_string(),
                    platform_mappings: [
                        ("PE".to_string(), "printf".to_string()),
                        ("IL".to_string(), "System.Console.WriteLine".to_string()),
                        ("JVM".to_string(), "java.lang.System.out.println".to_string()),
                        ("WASI".to_string(), "fd_write".to_string()),
                    ].into_iter().collect(),
                    description: Some("标准输出函数".to_string()),
                },
                FunctionMapping {
                    common_name: "malloc".to_string(),
                    platform_mappings: [
                        ("PE".to_string(), "malloc".to_string()),
                        ("IL".to_string(), "System.Runtime.InteropServices.Marshal.AllocHGlobal".to_string()),
                        ("JVM".to_string(), "java.nio.ByteBuffer.allocate".to_string()),
                        ("WASI".to_string(), "memory.grow".to_string()),
                    ].into_iter().collect(),
                    description: Some("内存分配函数".to_string()),
                },
                FunctionMapping {
                    common_name: "free".to_string(),
                    platform_mappings: [
                        ("PE".to_string(), "free".to_string()),
                        ("IL".to_string(), "System.Runtime.InteropServices.Marshal.FreeHGlobal".to_string()),
                        ("JVM".to_string(), "System.gc".to_string()),
                        ("WASI".to_string(), "memory.shrink".to_string()),
                    ].into_iter().collect(),
                    description: Some("内存释放函数".to_string()),
                },
            ],
            adapters: vec![],
        }
    }
}

/// 配置管理器
///
/// 负责加载、保存和管理配置信息
pub struct ConfigManager {
    /// 当前配置
    config: GaiaConfig,
    /// 配置文件路径
    config_path: Option<String>,
}

impl ConfigManager {
    /// 创建新的配置管理器
    pub fn new() -> Self {
        Self {
            config: GaiaConfig::default(),
            config_path: None,
        }
    }

    /// 从文件加载配置
    ///
    /// # 参数
    /// * `path` - 配置文件路径
    ///
    /// # 返回值
    /// 加载成功返回Ok(())，失败返回错误信息
    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)
            .map_err(|e| GaiaError::config_error(
                Some(path.to_string_lossy()),
                format!("读取配置文件失败: {}", e)
            ))?;

        self.config = toml::from_str(&content)
            .map_err(|e| GaiaError::config_error(
                Some(path.to_string_lossy()),
                format!("解析配置文件失败: {}", e)
            ))?;

        self.config_path = Some(path.to_string_lossy().to_string());
        Ok(())
    }

    /// 保存配置到文件
    ///
    /// # 参数
    /// * `path` - 配置文件路径，如果为None则使用当前路径
    ///
    /// # 返回值
    /// 保存成功返回Ok(())，失败返回错误信息
    pub fn save_to_file<P: AsRef<Path>>(&self, path: Option<P>) -> Result<()> {
        let target_path = if let Some(path) = path {
            path.as_ref().to_string_lossy().to_string()
        } else if let Some(ref current_path) = self.config_path {
            current_path.clone()
        } else {
            return Err(GaiaError::config_error(
                None::<String>,
                "未指定配置文件路径"
            ));
        };

        let content = toml::to_string_pretty(&self.config)
            .map_err(|e| GaiaError::config_error(
                Some(&target_path),
                format!("序列化配置失败: {}", e)
            ))?;

        fs::write(&target_path, content)
            .map_err(|e| GaiaError::config_error(
                Some(&target_path),
                format!("写入配置文件失败: {}", e)
            ))?;

        Ok(())
    }

    /// 获取当前配置
    pub fn config(&self) -> &GaiaConfig {
        &self.config
    }

    /// 获取可变配置
    pub fn config_mut(&mut self) -> &mut GaiaConfig {
        &mut self.config
    }

    /// 获取函数映射
    ///
    /// # 参数
    /// * `common_name` - 通用函数名
    /// * `platform` - 目标平台
    ///
    /// # 返回值
    /// 找到返回平台特定函数名，未找到返回None
    pub fn get_function_mapping(&self, common_name: &str, platform: &str) -> Option<&str> {
        self.config
            .function_mappings
            .iter()
            .find(|mapping| mapping.common_name == common_name)
            .and_then(|mapping| mapping.platform_mappings.get(platform))
            .map(|s| s.as_str())
    }

    /// 添加函数映射
    ///
    /// # 参数
    /// * `mapping` - 函数映射配置
    pub fn add_function_mapping(&mut self, mapping: FunctionMapping) {
        // 检查是否已存在同名映射
        if let Some(existing) = self.config
            .function_mappings
            .iter_mut()
            .find(|m| m.common_name == mapping.common_name) {
            // 合并平台映射
            existing.platform_mappings.extend(mapping.platform_mappings);
            if mapping.description.is_some() {
                existing.description = mapping.description;
            }
        } else {
            self.config.function_mappings.push(mapping);
        }
    }

    /// 获取平台配置
    ///
    /// # 参数
    /// * `platform_name` - 平台名称
    ///
    /// # 返回值
    /// 找到返回平台配置，未找到返回None
    pub fn get_platform_config(&self, target: &CompilationTarget) -> Option<&PlatformConfig> {
        self.config.platforms.get(target)
    }

    /// 添加平台配置
    ///
    /// # 参数
    /// * `target` - 编译目标
    /// * `platform` - 平台配置
    pub fn add_platform_config(&mut self, target: CompilationTarget, platform: PlatformConfig) {
        self.config.platforms.insert(target, platform);
    }

    /// 获取适配器配置
    ///
    /// # 参数
    /// * `adapter_name` - 适配器名称
    ///
    /// # 返回值
    /// 找到返回适配器配置，未找到返回None
    pub fn get_adapter_config(&self, adapter_name: &str) -> Option<&AdapterConfigEntry> {
        self.config
            .adapters
            .iter()
            .find(|adapter| adapter.name == adapter_name)
    }

    /// 添加适配器配置
    ///
    /// # 参数
    /// * `adapter` - 适配器配置
    pub fn add_adapter_config(&mut self, adapter: AdapterConfigEntry) {
        // 检查是否已存在同名适配器
        if let Some(existing) = self.config
            .adapters
            .iter_mut()
            .find(|a| a.name == adapter.name) {
            *existing = adapter;
        } else {
            self.config.adapters.push(adapter);
        }
    }

    /// 获取全局设置
    ///
    /// # 参数
    /// * `key` - 设置键
    ///
    /// # 返回值
    /// 找到返回设置值，未找到返回None
    pub fn get_global_setting(&self, key: &str) -> Option<&str> {
        self.config.global.parameters.get(key).map(|s| s.as_str())
    }

    /// 设置全局设置
    ///
    /// # 参数
    /// * `key` - 设置键
    /// * `value` - 设置值
    pub fn set_global_setting(&mut self, key: String, value: String) {
        self.config.global.parameters.insert(key, value);
    }

    /// 验证配置
    ///
    /// # 返回值
    /// 配置有效返回Ok(())，无效返回错误信息
    pub fn validate(&self) -> Result<()> {
        // 验证平台配置
        for (target, platform) in &self.config.platforms {
            if platform.supported_architectures.is_empty() {
                return Err(GaiaError::config_error(
                    self.config_path.as_ref(),
                    format!("平台 '{:?}' 必须支持至少一种架构", target)
                ));
            }
        }

        // 验证适配器配置
        for adapter in &self.config.adapters {
            if adapter.name.is_empty() {
                return Err(GaiaError::config_error(
                    self.config_path.as_ref(),
                    "适配器名称不能为空"
                ));
            }
            if !["export", "import"].contains(&adapter.adapter_type.as_str()) {
                return Err(GaiaError::config_error(
                    self.config_path.as_ref(),
                    format!("适配器 '{}' 的类型必须是 'export' 或 'import'", adapter.name)
                ));
            }
            // 验证目标平台是否存在
            if !self.config.platforms.contains_key(&adapter.compilation_target) {
                return Err(GaiaError::config_error(
                    self.config_path.as_ref(),
                    format!("适配器 '{}' 的目标平台 '{:?}' 不存在", adapter.name, adapter.compilation_target)
                ));
            }
        }

        Ok(())
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}