use crate::test_tools::test_path;
use gaia_types::{
    helpers::{open_file, save_json},
    GaiaError,
};
use wasi_assembler::formats::wasm::WasmReadConfig;
use serde::{Deserialize, Serialize};
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WasmExpected {
    /// 魔数头
    magic_head: [u8; 4],
    /// 程序类型
    program_type: String,
    /// 函数类型数量
    function_type_count: usize,
    /// 函数数量
    function_count: usize,
    /// 导出数量
    export_count: usize,
    /// 导入数量
    import_count: usize,
    /// 内存数量
    memory_count: usize,
    /// 表数量
    table_count: usize,
    /// 全局变量数量
    global_count: usize,
    /// 自定义段数量
    custom_section_count: usize,
    /// 起始函数索引
    start_function: Option<u32>,
    /// 导出详细信息
    exports: Vec<ExportInfo>,
    /// 导入详细信息
    imports: Vec<ImportInfo>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportInfo {
    /// 导出名称
    name: String,
    /// 导出类型
    export_type: String,
    /// 索引
    index: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImportInfo {
    /// 模块名
    module: String,
    /// 字段名
    field: String,
    /// 导入类型
    import_type: String,
}

fn generate_wasm() -> Result<(), GaiaError> {
    let base_dir = test_path("wasm_read");
    let generated_dir = base_dir.join("generated");
    std::fs::create_dir_all(&generated_dir)?;

    // 创建一个简单的 WAT 文件并编译为 WASM
    let wat_content = r#"
(module
  (func $add (param $lhs i32) (param $rhs i32) (result i32)
    local.get $lhs
    local.get $rhs
    i32.add)
  (export "add" (func $add))
)
"#;

    let wat_path = generated_dir.join("simple.wat");
    std::fs::write(&wat_path, wat_content)?;

    // 使用内部 wat 模块编译 WAT 文件为 WASM
    let wasm_path = generated_dir.join("simple.wasm");
    compile_wat_to_wasm(wat_content, &wasm_path)?;

    Ok(())
}

/// 使用内部 wat 模块将 WAT 编译为 WASM
fn compile_wat_to_wasm(wat_content: &str, output_path: &Path) -> Result<(), GaiaError> {
    // 由于 WAT 到 Program 的转换功能尚未完全实现，
    // 我们暂时创建一个简单的 WASM 文件来替代外部 wat2wasm 工具
    eprintln!("注意：WAT 到 Program 转换功能尚未完全实现，使用简化的 WASM 生成");
    create_simple_wasm(output_path)
}

fn create_simple_wasm(path: &Path) -> Result<(), GaiaError> {
    // 创建一个最简单的 WASM 文件：只有魔数和版本
    let wasm_bytes = vec![
        0x00, 0x61, 0x73, 0x6D, // 魔数 "\0asm"
        0x01, 0x00, 0x00, 0x00, // 版本 1
        // Type section
        0x01, // section id
        0x07, // section size
        0x01, // 1 type
        0x60, // function type
        0x02, 0x7F, 0x7F, // 2 params: i32, i32
        0x01, 0x7F, // 1 result: i32
        // Function section
        0x03, // section id
        0x02, // section size
        0x01, // 1 function
        0x00, // type index 0
        // Export section
        0x07, // section id
        0x07, // section size
        0x01, // 1 export
        0x03, 0x61, 0x64, 0x64, // name "add"
        0x00, // function export
        0x00, // function index 0
        // Code section
        0x0A, // section id
        0x09, // section size
        0x01, // 1 function body
        0x07, // body size
        0x00, // 0 locals
        0x20, 0x00, // local.get 0
        0x20, 0x01, // local.get 1
        0x6A, // i32.add
        0x0B, // end
    ];
    
    std::fs::write(path, wasm_bytes)?;
    Ok(())
}

#[test]
fn assert_wasm_info() -> Result<(), GaiaError> {
    if let Err(e) = generate_wasm() {
        // 即便生成失败，也继续测试已缓存的 WASM 文件
        eprintln!("{}", e);
    }
    
    let mut is_success = true;
    let test_dir = test_path("wasm_read");
    
    for entry in WalkDir::new(test_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "wasm"))
    {
        let wasm_path = entry.path();
        let wasm_name = wasm_path.file_stem().unwrap_or_default().to_string_lossy().to_string();

        println!("解析 WASM 文件: {}", wasm_path.display());

        // 读取 WASM 文件
        match read_wasm(wasm_path) {
            Ok(actual) => {
                // 生成对应的 JSON 文件路径
                let json_path = wasm_path.with_extension("expected.json");

                // 检查 JSON 文件是否存在
                if json_path.exists() {
                    // 如果存在，比较结果
                    let expected_content = std::fs::read_to_string(&json_path)?;
                    let expected: WasmExpected = serde_json::from_str(&expected_content)
                        .map_err(|e| GaiaError::invalid_data(&format!("Failed to parse expected JSON: {}", e)))?;

                    if actual == expected {
                        println!("✓ {} 测试通过", wasm_name);
                    } else {
                        println!("✗ {} 测试失败 - 结果不匹配", wasm_name);
                        println!("期望: {:#?}", expected);
                        println!("实际: {:#?}", actual);
                        is_success = false;
                    }
                } else {
                    save_json(&actual, &json_path)?;
                    println!("✓ {} 生成期望结果文件", wasm_name);
                }
            }
            Err(e) => {
                println!("✗ {} 读取失败: {}", wasm_name, e);
                return Err(e);
            }
        }
    }
    
    assert!(is_success);
    Ok(())
}

fn read_wasm(path: &Path) -> Result<WasmExpected, GaiaError> {
    let (file, _url) = open_file(path)?;
    let config = WasmReadConfig { check_magic_head: true };
    let reader = config.as_reader(file);
    let info = reader.get_view()?;
    let prog = reader.get_program()?;

    // 提取导出信息
    let exports: Vec<ExportInfo> = prog.exports.iter().map(|export| {
        let (export_type, index) = match export.export_type {
            wasi_assembler::program::WasmExportType::Function { function_index } => ("Function".to_string(), function_index),
            wasi_assembler::program::WasmExportType::Table { table_index } => ("Table".to_string(), table_index),
            wasi_assembler::program::WasmExportType::Memory { memory_index } => ("Memory".to_string(), memory_index),
            wasi_assembler::program::WasmExportType::Global { global_index } => ("Global".to_string(), global_index),
        };
        ExportInfo {
            name: export.name.clone(),
            export_type,
            index,
        }
    }).collect();

    // 提取导入信息
    let imports: Vec<ImportInfo> = prog.imports.iter().map(|import| {
        let import_type = match import.import_type {
            wasi_assembler::program::WasmImportType::Function { .. } => "Function".to_string(),
            wasi_assembler::program::WasmImportType::Table { .. } => "Table".to_string(),
            wasi_assembler::program::WasmImportType::Memory { .. } => "Memory".to_string(),
            wasi_assembler::program::WasmImportType::Global { .. } => "Global".to_string(),
        };
        ImportInfo {
            module: import.module.clone(),
            field: import.field.clone(),
            import_type,
        }
    }).collect();

    // 创建 WasmExpected 结构体
    let wasm_expected = WasmExpected {
        magic_head: info.magic_head,
        program_type: match prog.program_type {
            wasi_assembler::program::WasiProgramType::Component => "Component".to_string(),
            wasi_assembler::program::WasiProgramType::CoreModule => "CoreModule".to_string(),
        },
        function_type_count: prog.function_types.len(),
        function_count: prog.functions.len(),
        export_count: prog.exports.len(),
        import_count: prog.imports.len(),
        memory_count: prog.memories.len(),
        table_count: prog.tables.len(),
        global_count: prog.globals.len(),
        custom_section_count: prog.custom_sections.len(),
        start_function: prog.start_function,
        exports,
        imports,
    };
    
    Ok(wasm_expected)
}