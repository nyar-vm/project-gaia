//! 配置管理模块
//!
//! 本模块负责加载和管理Gaia汇编器的配置，包括平台映射、适配器配置等。
//! 支持从TOML配置文件动态加载配置信息。

use gaia_types::{
    helpers::{AbiCompatible, ApiCompatible, Architecture, CompilationTarget},
    GaiaError, Result,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::Path};

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
        Self { default_output_dir: "./target".to_string(), debug_mode: false, parameters: HashMap::new() }
    }
}

pub struct GaiaConfig {
    /// static part when load
    pub setting: GaiaSettings,

    pub target: CompilationTarget,
}

impl Default for GaiaConfig {
    fn default() -> Self {
        Self {
            setting: GaiaSettings::default(),
            target: CompilationTarget {
                build: Architecture::Unknown,
                host: AbiCompatible::Unknown,
                target: ApiCompatible::Unknown,
            },
        }
    }
}

/// Gaia汇编器主配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GaiaSettings {
    /// 配置版本
    pub version: String,
    /// 全局设置
    pub global: GlobalConfig,
    /// 平台配置映射
    #[serde(with = "target_map_serde")]
    pub platforms: HashMap<CompilationTarget, PlatformConfig>,
    /// 函数映射列表
    pub function_mappings: Vec<FunctionMapping>,
    /// 适配器配置列表
    pub adapters: Vec<AdapterConfigEntry>,
}

mod target_map_serde {
    use super::*;
    use serde::{Deserializer, Serializer};
    use std::str::FromStr;

    pub fn serialize<S>(map: &HashMap<CompilationTarget, PlatformConfig>, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let string_map: HashMap<String, PlatformConfig> = map.iter().map(|(k, v)| (k.to_string(), v.clone())).collect();
        string_map.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> std::result::Result<HashMap<CompilationTarget, PlatformConfig>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let string_map: HashMap<String, PlatformConfig> = HashMap::deserialize(deserializer)?;
        let mut map = HashMap::new();
        for (k, v) in string_map {
            // NOTE: This assumes CompilationTarget::from_str is implemented or we have a way to parse it.
            // CompilationTarget has Display, but does it have FromStr?
            // Let's check gaia_types.
            let target = parse_target(&k).map_err(serde::de::Error::custom)?;
            map.insert(target, v);
        }
        Ok(map)
    }

    fn parse_target(s: &str) -> std::result::Result<CompilationTarget, String> {
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() != 3 {
            return Err(format!("Invalid compilation target string: {}", s));
        }

        let build = Architecture::from_str(parts[0])?;
        let host = AbiCompatible::from_str(parts[1])?;
        let target = ApiCompatible::from_str(parts[2])?;

        Ok(CompilationTarget { build, host, target })
    }
}

impl Default for GaiaSettings {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            global: GlobalConfig::default(),
            platforms: {
                let mut platforms = HashMap::new();
                let x64_target = CompilationTarget {
                    build: Architecture::X86_64,
                    host: AbiCompatible::PE,
                    target: ApiCompatible::MicrosoftVisualC,
                };
                platforms.insert(
                    x64_target.clone(),
                    PlatformConfig {
                        target: x64_target,
                        description: Some("Windows x64 Native".to_string()),
                        supported_architectures: vec!["x86_64".to_string()],
                        default_extension: ".exe".to_string(),
                        parameters: HashMap::new(),
                    },
                );

                let clr_target = CompilationTarget {
                    build: Architecture::CLR,
                    host: AbiCompatible::PE,
                    target: ApiCompatible::ClrRuntime(4),
                };
                platforms.insert(
                    clr_target.clone(),
                    PlatformConfig {
                        target: clr_target,
                        description: Some(".NET CLR v4.0".to_string()),
                        supported_architectures: vec!["msil".to_string()],
                        default_extension: ".exe".to_string(),
                        parameters: HashMap::new(),
                    },
                );
                platforms
            },
            function_mappings: vec![
                FunctionMapping {
                    common_name: "__builtin_print".to_string(),
                    platform_mappings: [
                        ("PE".to_string(), "printf".to_string()),
                        ("IL".to_string(), "System.Console.WriteLine".to_string()),
                        ("JVM".to_string(), "java.lang.System.out.println".to_string()),
                        ("WASI".to_string(), "fd_write".to_string()),
                    ]
                    .into_iter()
                    .collect(),
                    description: Some("标准输出函数".to_string()),
                },
                FunctionMapping {
                    common_name: "malloc".to_string(),
                    platform_mappings: [
                        ("PE".to_string(), "malloc".to_string()),
                        ("IL".to_string(), "System.Runtime.InteropServices.Marshal.AllocHGlobal".to_string()),
                        ("JVM".to_string(), "java.nio.ByteBuffer.allocate".to_string()),
                        ("WASI".to_string(), "memory.grow".to_string()),
                    ]
                    .into_iter()
                    .collect(),
                    description: Some("内存分配函数".to_string()),
                },
                FunctionMapping {
                    common_name: "free".to_string(),
                    platform_mappings: [
                        ("PE".to_string(), "free".to_string()),
                        ("IL".to_string(), "System.Runtime.InteropServices.Marshal.FreeHGlobal".to_string()),
                        ("JVM".to_string(), "System.gc".to_string()),
                        ("WASI".to_string(), "memory.shrink".to_string()),
                    ]
                    .into_iter()
                    .collect(),
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
    config: GaiaSettings,
    /// 配置文件路径
    config_path: Option<String>,
}

impl ConfigManager {
    /// 创建新的配置管理器
    pub fn new() -> Self {
        Self { config: GaiaSettings::default(), config_path: None }
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
            .map_err(|e| GaiaError::config_error(Some(path.to_string_lossy()), format!("读取配置文件失败: {}", e)))?;

        self.config = toml::from_str(&content)
            .map_err(|e| GaiaError::config_error(Some(path.to_string_lossy()), format!("解析配置文件失败: {}", e)))?;

        self.config_path = Some(path.to_string_lossy().to_string());
        Ok(())
    }

    /// 保存配置到文件
    ///
    /// # 参数
    /// * `path` - 配置文件路径
    ///
    /// # 返回值
    /// 保存成功返回Ok(())，失败返回错误信息
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        let content = toml::to_string_pretty(&self.config)
            .map_err(|e| GaiaError::config_error(Some(path.to_string_lossy()), format!("序列化配置失败: {}", e)))?;

        fs::write(path, content)
            .map_err(|e| GaiaError::config_error(Some(path.to_string_lossy()), format!("写入配置文件失败: {}", e)))?;

        Ok(())
    }

    /// 获取当前设置
    pub fn settings(&self) -> &GaiaSettings {
        &self.config
    }

    /// 获取当前设置 (兼容旧代码)
    pub fn config(&self) -> &GaiaSettings {
        &self.config
    }

    /// 获取全局设置
    pub fn get_global_setting(&self, key: &str) -> Option<&str> {
        self.config.global.parameters.get(key).map(|s| s.as_str())
    }

    /// 设置全局设置
    pub fn set_global_setting(&mut self, key: String, value: String) {
        self.config.global.parameters.insert(key, value);
    }

    /// 获取平台配置
    pub fn get_platform_config(&self, target: &CompilationTarget) -> Option<&PlatformConfig> {
        self.config.platforms.get(target)
    }

    /// 添加平台配置
    pub fn add_platform_config(&mut self, target: CompilationTarget, config: PlatformConfig) {
        self.config.platforms.insert(target, config);
    }

    /// 添加适配器配置
    pub fn add_adapter_config(&mut self, config: AdapterConfigEntry) {
        self.config.adapters.push(config);
    }

    /// 获取适配器配置
    pub fn get_adapter_config(&self, name: &str) -> Option<&AdapterConfigEntry> {
        self.config.adapters.iter().find(|a| a.name == name)
    }

    /// 添加函数映射
    pub fn add_function_mapping(&mut self, mapping: FunctionMapping) {
        self.config.function_mappings.push(mapping);
    }

    /// 获取函数映射
    pub fn get_function_mapping(&self, common_name: &str, platform: &str) -> Option<&str> {
        self.config
            .function_mappings
            .iter()
            .find(|m| m.common_name == common_name)
            .and_then(|m| m.platform_mappings.get(platform).map(|s| s.as_str()))
    }

    /// 验证配置
    pub fn validate(&self) -> Result<()> {
        Ok(())
    }

    /// 获取当前配置文件的路径
    pub fn config_path(&self) -> Option<&str> {
        self.config_path.as_deref()
    }
}
