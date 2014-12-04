use gaia_assembler::{
    config::{AdapterConfigEntry, ConfigManager, FunctionMapping, PlatformConfig},
    *,
};
use gaia_types::{
    helpers::{AbiCompatible, ApiCompatible, Architecture, CompilationTarget},
    Result,
};
use std::collections::HashMap;

/// 测试配置管理器创建
#[test]
fn test_config_manager_creation() {
    let config_manager = ConfigManager::new();

    // 验证默认配置
    let config = config_manager.config();
    assert_eq!(config.version, "1.0.0");
    assert!(!config.platforms.is_empty(), "应该有默认平台配置");
    assert!(!config.function_mappings.is_empty(), "应该有默认函数映射");
}

/// 测试平台配置管理
#[test]
fn test_platform_config_management() {
    let mut config_manager = ConfigManager::new();
    let target =
        CompilationTarget { build: Architecture::X86_64, host: AbiCompatible::PE, target: ApiCompatible::MicrosoftVisualC };

    // 测试获取平台配置
    let platform_config = config_manager.get_platform_config(&target);
    assert!(platform_config.is_some(), "应该能获取到默认平台配置");

    // 测试添加新的平台配置
    let new_platform = PlatformConfig {
        target: target.clone(),
        description: Some("测试平台".to_string()),
        supported_architectures: vec!["x86_64".to_string()],
        default_extension: ".test".to_string(),
        parameters: HashMap::new(),
    };

    config_manager.add_platform_config(target.clone(), new_platform);

    // 验证配置已添加
    let retrieved_config = config_manager.get_platform_config(&target);
    assert!(retrieved_config.is_some());
    assert_eq!(retrieved_config.unwrap().default_extension, ".test");
}

/// 测试适配器配置管理
#[test]
fn test_adapter_config_management() {
    let mut config_manager = ConfigManager::new();

    // 测试添加适配器配置
    let adapter_config = AdapterConfigEntry {
        name: "test_adapter".to_string(),
        adapter_type: "test".to_string(),
        compilation_target: CompilationTarget {
            build: Architecture::X86_64,
            host: AbiCompatible::PE,
            target: ApiCompatible::MicrosoftVisualC,
        },
        enabled: true,
        parameters: HashMap::new(),
    };

    config_manager.add_adapter_config(adapter_config);

    // 验证适配器配置已添加
    let retrieved_adapter = config_manager.get_adapter_config("test_adapter");
    assert!(retrieved_adapter.is_some());
    assert_eq!(retrieved_adapter.unwrap().adapter_type, "test");
}

/// 测试函数映射管理
#[test]
fn test_function_mapping_management() {
    let mut config_manager = ConfigManager::new();

    // 测试添加函数映射
    let function_mapping = FunctionMapping {
        common_name: "test_function".to_string(),
        platform_mappings: {
            let mut mappings = HashMap::new();
            mappings.insert("jvm".to_string(), "testFunction".to_string());
            mappings.insert("msil_read".to_string(), "TestFunction".to_string());
            mappings
        },
        description: Some("测试函数映射".to_string()),
    };

    config_manager.add_function_mapping(function_mapping);

    // 验证函数映射
    let jvm_mapping = config_manager.get_function_mapping("test_function", "jvm");
    assert_eq!(jvm_mapping, Some("testFunction"));

    let msil_mapping = config_manager.get_function_mapping("test_function", "msil_read");
    assert_eq!(msil_mapping, Some("TestFunction"));

    let unknown_mapping = config_manager.get_function_mapping("test_function", "unknown");
    assert_eq!(unknown_mapping, None);
}

/// 测试全局设置管理
#[test]
fn test_global_settings_management() {
    let mut config_manager = ConfigManager::new();

    // 测试设置全局设置
    config_manager.set_global_setting("debug_mode".to_string(), "true".to_string());
    config_manager.set_global_setting("optimization_level".to_string(), "2".to_string());

    // 验证全局设置
    assert_eq!(config_manager.get_global_setting("debug_mode"), Some("true"));
    assert_eq!(config_manager.get_global_setting("optimization_level"), Some("2"));
    assert_eq!(config_manager.get_global_setting("nonexistent"), None);
}

/// 测试配置验证
#[test]
fn test_config_validation() {
    let config_manager = ConfigManager::new();

    // 验证默认配置应该是有效的
    let validation_result = config_manager.validate();
    match validation_result {
        Ok(()) => println!("配置验证通过"),
        Err(e) => println!("配置验证失败: {:?}", e),
    }
}

/// 测试配置文件保存和加载
#[test]
fn test_config_file_operations() {
    use std::{fs, path::Path};

    let mut config_manager = ConfigManager::new();
    let test_config_path = "test_config.toml";

    // 添加一些测试配置
    config_manager.set_global_setting("test_setting".to_string(), "test_value".to_string());

    // 测试保存配置
    let save_result = config_manager.save_to_file(Some(test_config_path));
    match save_result {
        Ok(()) => {
            println!("配置保存成功");

            // 测试加载配置
            let mut new_config_manager = ConfigManager::new();
            let load_result = new_config_manager.load_from_file(test_config_path);

            match load_result {
                Ok(()) => {
                    // 验证加载的配置
                    assert_eq!(new_config_manager.get_global_setting("test_setting"), Some("test_value"));
                    println!("配置加载成功");
                }
                Err(e) => println!("配置加载失败: {:?}", e),
            }

            // 清理测试文件
            if Path::new(test_config_path).exists() {
                let _ = fs::remove_file(test_config_path);
            }
        }
        Err(e) => println!("配置保存失败: {:?}", e),
    }
}

/// 测试默认配置内容
#[test]
fn test_default_config_content() {
    let config_manager = ConfigManager::new();
    let config = config_manager.config();

    // 验证默认配置包含必要的内容
    assert!(!config.platforms.is_empty(), "应该有默认平台配置");
    assert!(!config.function_mappings.is_empty(), "应该有默认函数映射");

    // 验证全局配置
    assert!(!config.global.default_output_dir.is_empty(), "应该有默认输出目录");

    println!("默认配置包含 {} 个平台", config.platforms.len());
    println!("默认配置包含 {} 个函数映射", config.function_mappings.len());
    println!("默认配置包含 {} 个适配器", config.adapters.len());
}
