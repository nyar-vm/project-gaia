use gaia_types::{helpers::open_file, GaiaError};
use std::path::Path;
use wasmtime::{
    component::{Component, Linker},
    Config, Engine, Store,
};
use wasmtime_wasi::WasiCtxBuilder;

/// 使用 Wasmtime 运行 WIT component（wasip2）
pub fn wasi_run(path: &Path) -> Result<(), GaiaError> {
    let (file, url) = open_file(path)?;

    // 创建 Wasmtime 配置
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_multi_memory(true);

    // 创建引擎
    let engine =
        Engine::new(&config).map_err(|e| GaiaError::invalid_data(&format!("Failed to create Wasmtime engine: {}", e)))?;

    // 读取组件字节码
    let mut bytes = Vec::new();
    use std::io::Read;
    let mut file = file;
    file.read_to_end(&mut bytes).map_err(|e| GaiaError::io_error(e, url.clone()))?;

    // 创建组件
    let component =
        Component::new(&engine, &bytes).map_err(|e| GaiaError::invalid_data(&format!("Failed to create component: {}", e)))?;

    // 创建存储和上下文
    let wasi = WasiCtx::new();
    let mut store = Store::new(&engine, WasiHostState::new(wasi));

    // 创建链接器并添加WASI
    let mut linker = Linker::new(&engine);
    wasmtime_wasi::add_to_linker(&mut linker)
        .map_err(|e| GaiaError::invalid_data(&format!("Failed to add WASI to linker: {}", e)))?;

    // 实例化组件
    let instance = linker
        .instantiate(&mut store, &component)
        .map_err(|e| GaiaError::invalid_data(&format!("Failed to instantiate component: {}", e)))?;

    // 尝试调用导出的函数（如果有）
    if let Ok(export) = instance.get_func(&mut store, "run") {
        println!("找到 'run' 函数，正在执行...");
        let result: WasmtimeResult<()> = export.call(&mut store, &[]);
        match result {
            Ok(_) => println!("组件执行成功"),
            Err(e) => println!("组件执行失败: {}", e),
        }
    }
    else if let Ok(export) = instance.get_func(&mut store, "main") {
        println!("找到 'main' 函数，正在执行...");
        let result: WasmtimeResult<()> = export.call(&mut store, &[]);
        match result {
            Ok(_) => println!("组件执行成功"),
            Err(e) => println!("组件执行失败: {}", e),
        }
    }
    else {
        println!("未找到 'run' 或 'main' 函数，列出所有导出:");
        for (name, _) in instance.exports(&mut store) {
            println!("  - {}", name);
        }
    }

    Ok(())
}

/// WASI 主机状态
struct WasiHostState {
    ctx: WasiCtx,
    table: ResourceTable,
}

impl WasiHostState {
    fn new(ctx: WasiCtx) -> Self {
        Self { ctx, table: ResourceTable::new() }
    }
}

impl WasiView for WasiHostState {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }

    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.ctx
    }
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

/// 列出组件的所有导出
pub fn list_component_exports(path: &Path) -> Result<Vec<String>, GaiaError> {
    let (file, url) = open_file(path)?;

    // 创建 Wasmtime 配置
    let mut config = Config::new();
    config.wasm_component_model(true);

    // 创建引擎
    let engine =
        Engine::new(&config).map_err(|e| GaiaError::invalid_data(&format!("Failed to create Wasmtime engine: {}", e)))?;

    // 读取组件字节码
    let mut bytes = Vec::new();
    use std::io::Read;
    let mut file = file;
    file.read_to_end(&mut bytes).map_err(|e| GaiaError::io_error(e, url.clone()))?;

    // 创建组件
    let component =
        Component::new(&engine, &bytes).map_err(|e| GaiaError::invalid_data(&format!("Failed to create component: {}", e)))?;

    // 创建存储和上下文
    let wasi = WasiCtxBuilder::new().build();
    let mut store = Store::new(&engine, WasiHostState::new(wasi));

    // 创建链接器并添加WASI
    let mut linker = Linker::new(&engine);
    wasmtime_wasi::add_to_linker(&mut linker)
        .map_err(|e| GaiaError::invalid_data(&format!("Failed to add WASI to linker: {}", e)))?;

    // 实例化组件
    let instance = linker
        .instantiate(&mut store, &component)
        .map_err(|e| GaiaError::invalid_data(&format!("Failed to instantiate component: {}", e)))?;

    // 收集所有导出
    let mut exports = Vec::new();
    for (name, _) in instance.exports(&mut store) {
        exports.push(name.to_string());
    }

    Ok(exports)
}
