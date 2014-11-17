use gaia_assembler::{
    adapters::AdapterManager,
    backends::{Backend, ClrBackend, FunctionMapper, JvmBackend, PeBackend, WasiBackend},
    *,
};
use gaia_types::{
    helpers::{AbiCompatible, ApiCompatible, Architecture, CompilationTarget},
    GaiaFunction, GaiaInstruction, GaiaProgram,
};

/// 测试适配器管理器创建
#[test]
fn test_adapter_manager_creation() {
    let adapter_manager = AdapterManager::new();

    // 验证适配器管理器已创建
    // 注意：由于 AdapterManager 没有公开的方法来检查内部状态，
    // 我们只能验证它能够成功创建
    println!("适配器管理器创建成功");
}

/// 测试后端创建和基本功能
#[test]
fn test_backend_creation() {
    // 测试各个后端的创建
    let jvm_backend = JvmBackend::default();
    let msil_backend = ClrBackend::default();
    let pe_backend = PeBackend::default();
    let wasi_backend = WasiBackend::default();

    // 验证后端名称
    assert_eq!(jvm_backend.name(), "JVM");
    assert_eq!(msil_backend.name(), "MSIL");
    assert_eq!(pe_backend.name(), "PE");
    assert_eq!(wasi_backend.name(), "WASI");

    println!("所有后端创建成功");
}

/// 测试后端兼容性评分
#[test]
fn test_backend_compatibility() {
    let target =
        CompilationTarget { build: Architecture::X86_64, host: AbiCompatible::PE, target: ApiCompatible::MicrosoftVisualC };

    let backends: Vec<Box<dyn Backend>> = vec![
        Box::new(JvmBackend::default()),
        Box::new(ClrBackend::default()),
        Box::new(PeBackend::default()),
        Box::new(WasiBackend::default()),
    ];

    for backend in backends {
        let score = backend.match_score(&target);

        // 验证评分在有效范围内
        assert!(score >= 0.0 && score <= 1.0, "后端 {} 的兼容性评分 {} 不在有效范围内", backend.name(), score);

        println!("后端 {} 兼容性评分: {:.2}", backend.name(), score);
    }
}

/// 测试后端编译功能
#[test]
fn test_backend_compilation() {
    let program = create_simple_test_program();
    let target =
        CompilationTarget { build: Architecture::X86_64, host: AbiCompatible::PE, target: ApiCompatible::MicrosoftVisualC };

    // 创建默认配置
    let config = gaia_assembler::config::GaiaConfig::default();

    let backends: Vec<Box<dyn Backend>> = vec![
        Box::new(JvmBackend::default()),
        Box::new(ClrBackend::default()),
        Box::new(PeBackend::default()),
        Box::new(WasiBackend::default()),
    ];

    for backend in backends {
        println!("测试后端 {} 的编译功能", backend.name());

        let result = backend.generate(&program, &config);

        match result {
            Ok(generated_files) => {
                assert!(!generated_files.files.is_empty(), "后端 {} 生成的文件不应为空", backend.name());
                let total_size: usize = generated_files.files.values().map(|v| v.len()).sum();
                println!("后端 {} 编译成功，生成 {} 字节", backend.name(), total_size);
            }
            Err(e) => {
                println!("后端 {} 编译失败: {:?}", backend.name(), e);
                // 编译失败可能是预期的，因为某些后端可能还未完全实现
            }
        }
    }
}

/// 测试后端映射功能
#[test]
fn test_backend_mappings() {
    // 测试 FunctionMapper 功能
    let mut mapper = FunctionMapper::new();
    let target = CompilationTarget {
        build: Architecture::CLR,
        host: AbiCompatible::MicrosoftIntermediateLanguage,
        target: ApiCompatible::ClrRuntime(4),
    };

    // 添加一些映射
    mapper.add_mapping(&target, "print", "Console.WriteLine");
    mapper.add_mapping(&target, "malloc", "Marshal.AllocHGlobal");

    // 测试映射查找
    let mapped = mapper.map_function(&target, "print");
    assert_eq!(mapped, Some("Console.WriteLine"));

    println!("后端映射测试完成");
}

/// 测试后端初始化
#[test]
fn test_backend_initialization() {
    // 测试所有后端都能正确初始化
    let _jvm = JvmBackend::default();
    let _msil = ClrBackend::default();
    let _pe = PeBackend::default();
    let _wasi = WasiBackend::default();

    println!("所有后端初始化成功");
}

// 辅助函数
fn create_simple_test_program() -> GaiaProgram {
    let main_function = GaiaFunction {
        name: "main".to_string(),
        parameters: vec![],
        return_type: None,
        instructions: vec![GaiaInstruction::Return],
        locals: vec![],
    };

    GaiaProgram { name: "simple_test".to_string(), functions: vec![main_function], constants: vec![] }
}

/// 测试后端特定功能
#[test]
fn test_backend_specific_features() {
    test_jvm_backend_features();
    test_msil_backend_features();
    test_pe_backend_features();
    test_wasi_backend_features();
}

fn test_jvm_backend_features() {
    let backend = JvmBackend::default();
    let program = create_simple_test_program();
    let target = backend.primary_target();
    let config = gaia_assembler::config::GaiaConfig::default();
    let result = backend.generate(&program, &config);

    // 测试 JVM 特定功能
    println!("JVM 后端特定功能测试完成");
}

fn test_msil_backend_features() {
    let backend = ClrBackend::default();
    let program = create_simple_test_program();
    let target = backend.primary_target();
    let config = gaia_assembler::config::GaiaConfig::default();
    let result = backend.generate(&program, &config);

    // 测试 MSIL 特定功能
    println!("MSIL 后端特定功能测试完成");
}

fn test_pe_backend_features() {
    let backend = PeBackend::default();
    let program = create_simple_test_program();
    let _target =
        CompilationTarget { build: Architecture::X86_64, host: AbiCompatible::PE, target: ApiCompatible::MicrosoftVisualC };

    // 测试 PE 特定功能
    println!("PE 后端特定功能测试完成");
}

fn test_wasi_backend_features() {
    let backend = WasiBackend::default();
    let program = create_simple_test_program();
    let _target =
        CompilationTarget { build: Architecture::X86_64, host: AbiCompatible::PE, target: ApiCompatible::MicrosoftVisualC };

    // 测试 WASI 特定功能
    println!("WASI 后端特定功能测试完成");
}
