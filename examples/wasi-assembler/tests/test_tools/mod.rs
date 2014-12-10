use gaia_types::{helpers::open_file, GaiaError};
use std::path::{Path, PathBuf};
use wasmtime::{Config, Engine, Store};
use wasmtime_wasi::{p1::WasiP1Ctx, WasiCtxBuilder};

/// 获取测试数据路径
pub fn test_path(test_name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("data");
    path.push(test_name);
    path
}

/// 使用 Wasmtime 运行传统 WASM 模块
pub fn wasi_run(path: &Path) -> Result<(), GaiaError> {
    let (file, url) = open_file(path)?;

    // 创建 Wasmtime 配置
    let mut config = Config::new();
    config.wasm_component_model(false); // 使用传统的 WASM 模块
    config.wasm_multi_memory(true);
    // 不启用异步支持，使用同步 API

    // 创建引擎
    let engine =
        Engine::new(&config).map_err(|e| GaiaError::invalid_data(&format!("Failed to create Wasmtime engine: {}", e)))?;

    // 读取模块字节码
    let mut bytes = Vec::new();
    use std::io::Read;
    let mut file = file;
    file.read_to_end(&mut bytes).map_err(|e| GaiaError::io_error(e, url.clone()))?;

    // 创建模块
    let module = wasmtime::Module::new(&engine, &bytes)
        .map_err(|e| GaiaError::invalid_data(&format!("Failed to create module: {}", e)))?;

    // 创建存储和上下文
    let wasi_ctx = WasiCtxBuilder::new().inherit_stdio().inherit_env().build_p1();
    let mut store = Store::new(&engine, wasi_ctx);

    // 创建链接器并添加WASI (使用传统模块 API)
    let mut linker = wasmtime::Linker::new(&engine);
    wasmtime_wasi::p1::add_to_linker_sync(&mut linker, |s: &mut WasiP1Ctx| s)
        .map_err(|e| GaiaError::invalid_data(&format!("Failed to add WASI to linker: {}", e)))?;

    // 实例化模块
    let instance = linker
        .instantiate(&mut store, &module)
        .map_err(|e| GaiaError::invalid_data(&format!("Failed to instantiate module: {}", e)))?;

    // 尝试调用导出的函数（如果有）
    if let Ok(export) = instance.get_typed_func::<(), ()>(&mut store, "_start") {
        println!("找到 '_start' 函数，正在执行...");
        let result = export.call(&mut store, ());
        match result {
            Ok(_) => println!("'_start' 函数执行成功"),
            Err(e) => println!("'_start' 函数执行失败: {}", e),
        }
    }
    else if let Ok(export) = instance.get_typed_func::<(), ()>(&mut store, "main") {
        println!("找到 'main' 函数，正在执行...");
        let result = export.call(&mut store, ());
        match result {
            Ok(_) => println!("'main' 函数执行成功"),
            Err(e) => println!("'main' 函数执行失败: {}", e),
        }
    }
    else {
        println!("未找到 '_start' 或 'main' 函数");
    }

    Ok(())
}

/// 运行一个简单的WAT组件测试
pub fn test_run_wat_component(wat_content: &str) -> Result<(), GaiaError> {
    use std::io::Write;
    use tempfile::NamedTempFile;

    // 创建临时文件
    let mut temp_file = NamedTempFile::new()
        .map_err(|e| GaiaError::io_error(e, gaia_types::helpers::url_from_path(Path::new("temp")).unwrap()))?;

    // 写入WAT内容
    temp_file
        .write_all(wat_content.as_bytes())
        .map_err(|e| GaiaError::io_error(e, gaia_types::helpers::url_from_path(temp_file.path()).unwrap()))?;

    // 运行组件
    wasi_run(temp_file.path())
}
