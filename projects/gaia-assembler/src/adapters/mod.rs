//! 统一的适配器接口定义
//!
//! 本模块定义了导入和导出适配器的统一接口，以及相关的配置和管理结构。
//! 这些接口旨在抽象不同平台之间的差异，提供一致的API。

use crate::instruction::GaiaInstruction;
use gaia_types::{helpers::CompilationTarget, GaiaError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use gaia_types::helpers::{AbiCompatible, ApiCompatible, Architecture};
use crate::config::GaiaSettings;
use crate::program::GaiaProgram;

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

/// 函数映射器
#[derive(Debug)]
pub struct FunctionMapper {
    /// 函数映射表 (平台 -> 源函数 -> 目标函数)
    mappings: HashMap<CompilationTarget, HashMap<String, String>>,
}

impl FunctionMapper {
    /// Create a new function mapper with default mappings
    pub fn new() -> Self {
        // 使用默认 GaiaSettings 统一初始化，避免硬编码重复
        // 默认映射包含 __builtin_print / malloc / free 等通用函数
        Self::from_settings(&GaiaSettings::default())
    }

    /// 从配置创建函数映射器
    pub fn from_config(config: &GaiaSettings) -> Result<Self> {
        Ok(Self::from_settings(config))
    }

    /// 基于 GaiaSettings 生成统一的函数映射（覆盖默认值）
    fn from_settings(settings: &GaiaSettings) -> Self {
        let mut mapper = Self { mappings: HashMap::new() };

        // 预定义的平台目标（统一键：IL/JVM/PE/WASI）
        let il_target = CompilationTarget {
            build: Architecture::CLR,
            host: AbiCompatible::MicrosoftIntermediateLanguage,
            target: ApiCompatible::ClrRuntime(4),
        };
        let jvm_target = CompilationTarget {
            build: Architecture::JVM,
            host: AbiCompatible::JavaAssembly,
            target: ApiCompatible::JvmRuntime(8),
        };
        let pe_target = CompilationTarget {
            build: Architecture::X86_64,
            host: AbiCompatible::PE,
            target: ApiCompatible::MicrosoftVisualC,
        };
        let wasi_target = CompilationTarget {
            build: Architecture::WASM32,
            host: AbiCompatible::WebAssemblyTextFormat,
            target: ApiCompatible::WASI,
        };

        let mut platform_index: HashMap<String, CompilationTarget> = HashMap::new();
        platform_index.insert("IL".to_string(), il_target.clone());
        platform_index.insert("JVM".to_string(), jvm_target.clone());
        platform_index.insert("PE".to_string(), pe_target.clone());
        platform_index.insert("WASI".to_string(), wasi_target.clone());

        // 1) 基线：提供合理的默认别名，避免因设置缺失导致不可用
        // IL 默认：带签名的 WriteLine，适配当前 MsilWriter 的 call 语法
        mapper.add_mapping(&il_target, "console.log", "void [mscorlib]System.Console::WriteLine(string)");
        mapper.add_mapping(&il_target, "console.write", "void [mscorlib]System.Console::WriteLine(string)");
        mapper.add_mapping(&il_target, "conosole.read", "string [mscorlib]System.Console::ReadLine()");
        mapper.add_mapping(&il_target, "malloc", "System.Runtime.InteropServices.Marshal.AllocHGlobal");
        mapper.add_mapping(&il_target, "free", "System.Runtime.InteropServices.Marshal.FreeHGlobal");
        mapper.add_mapping(&il_target, "print", "void [mscorlib]System.Console::WriteLine(string)");
        mapper.add_mapping(&il_target, "println", "void [mscorlib]System.Console::WriteLine(string)");

        // JVM 默认
        mapper.add_mapping(&jvm_target, "console.log", "java.lang.System.out.println");
        mapper.add_mapping(&jvm_target, "console.write", "java.lang.System.out.println");
        mapper.add_mapping(&jvm_target, "console.read", "java.util.Scanner.nextLine");
        mapper.add_mapping(&jvm_target, "malloc", "java.nio.ByteBuffer.allocateDirect");
        mapper.add_mapping(&jvm_target, "free", "System.gc");
        mapper.add_mapping(&jvm_target, "print", "java.lang.System.out.println");
        mapper.add_mapping(&jvm_target, "println", "java.lang.System.out.println");

        // PE 默认
        mapper.add_mapping(&pe_target, "console.log", "puts");
        mapper.add_mapping(&pe_target, "console.write", "printf");
        mapper.add_mapping(&pe_target, "console.read", "gets_s");
        mapper.add_mapping(&pe_target, "malloc", "HeapAlloc");
        mapper.add_mapping(&pe_target, "free", "HeapFree");
        mapper.add_mapping(&pe_target, "print", "puts");
        mapper.add_mapping(&pe_target, "println", "puts");

        // WASI 默认
        mapper.add_mapping(&wasi_target, "console.log", "wasi_println");
        mapper.add_mapping(&wasi_target, "console.write", "wasi_print");
        mapper.add_mapping(&wasi_target, "console.read", "wasi_read");
        mapper.add_mapping(&wasi_target, "malloc", "malloc");
        mapper.add_mapping(&wasi_target, "free", "free");
        mapper.add_mapping(&wasi_target, "print", "wasi_println");
        mapper.add_mapping(&wasi_target, "println", "wasi_println");

        // 2) 覆盖：使用 settings.function_mappings 覆盖/扩展默认映射
        for fm in &settings.function_mappings {
            for (platform_name, target_func) in &fm.platform_mappings {
                let key = platform_name.to_ascii_uppercase();
                if let Some(platform_target) = platform_index.get(&key) {
                    // 直接映射通用名
                    mapper.add_mapping(platform_target, &fm.common_name, target_func);

                    // 为常见别名提供联动（减少前端重复工作）
                    if fm.common_name == "__builtin_print" {
                        let is_il = matches!(platform_target.host, AbiCompatible::MicrosoftIntermediateLanguage);
                        // IL 平台保留带签名的默认 print 映射，避免生成不完整的 MSIL 调用操作数
                        if !is_il {
                            mapper.add_mapping(platform_target, "print", target_func);
                        }
                        mapper.add_mapping(platform_target, "console.log", target_func);
                        mapper.add_mapping(platform_target, "console.write", target_func);
                    }
                }
            }
        }

        mapper
    }

    /// 添加函数映射
    pub fn add_mapping(&mut self, target: &CompilationTarget, source_func: &str, target_func: &str) {
        self.mappings
            .entry(target.clone())
            .or_insert_with(HashMap::new)
            .insert(source_func.to_string(), target_func.to_string());
    }

    /// 映射函数名
    pub fn map_function(&self, target: &CompilationTarget, function_name: &str) -> Option<&str> {
        self.mappings.get(target).and_then(|platform_mappings| platform_mappings.get(function_name)).map(|s| s.as_str())
    }
}

impl Default for FunctionMapper {
    fn default() -> Self {
        Self::new()
    }
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
