use crate::test_tools::test_path;
use gaia_types::{
    helpers::{open_file, save_json},
    GaiaError,
};
use pe_assembler::{formats::dll::reader::DllReader, helpers::PeReader, types::PeInfo};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Windows DLL 分析结果
#[derive(Debug, Serialize, Deserialize)]
pub struct WindowsDllAnalysis {
    /// DLL 文件名
    pub dll_name: String,
    /// PE 基本信息
    pub basic_info: PeInfo,
    /// 导出函数数量
    pub export_count: usize,
    /// 导入函数数量
    pub import_count: usize,
    /// 节数量
    pub section_count: usize,
    /// 前20个导出函数（用于展示）
    pub sample_exports: Vec<String>,
    /// 前20个导入函数（用于展示）
    pub sample_imports: Vec<String>,
    /// 节名称列表
    pub section_names: Vec<String>,
    /// 分析是否成功
    pub analysis_success: bool,
    /// 错误信息（如果有）
    #[serde(skip)]
    pub error_message: Option<GaiaError>,
}

#[test]
fn dump_common_windows_dlls() -> Result<(), GaiaError> {
    // 常见的 Windows 系统 DLL
    let essential_dlls = vec![
        ("kernel32", r"C:\Windows\System32\kernel32.dll"),
        ("user32", r"C:\Windows\System32\user32.dll"),
        ("ntdll", r"C:\Windows\System32\ntdll.dll"),
        ("advapi32", r"C:\Windows\System32\advapi32.dll"),
        ("ole32", r"C:\Windows\System32\ole32.dll"),
        ("shell32", r"C:\Windows\System32\shell32.dll"),
        ("comctl32", r"C:\Windows\System32\comctl32.dll"),
        ("gdi32", r"C:\Windows\System32\gdi32.dll"),
        ("ws2_32", r"C:\Windows\System32\ws2_32.dll"),
        ("msvcrt", r"C:\Windows\System32\msvcrt.dll"),
    ];

    let mut analyses = Vec::new();

    for (dll_name, dll_path) in essential_dlls {
        println!("分析 DLL: {}", dll_name);
        let path = Path::new(dll_path);

        if path.exists() {
            match analyze_dll_file(path) {
                Ok(analysis) => {
                    println!(
                        "  ✓ 成功分析 {} - 导出函数: {}, 导入函数: {}, 节数: {}",
                        dll_name, analysis.export_count, analysis.import_count, analysis.section_count
                    );
                    analyses.push(analysis);
                }
                Err(e) => {
                    println!("  ✗ 分析失败 {}: {:?}", dll_name, e);
                }
            }
        }
        else {
            println!("  ⚠ 文件不存在: {}", dll_path);
        }
    }

    let output_path = test_path("windows/windows_dll_analysis.json");
    save_json(&analyses, &output_path)?;
    Ok(())
}

pub fn analyze_dll_file(path: &Path) -> Result<WindowsDllAnalysis, GaiaError> {
    let (file, _url) = open_file(path)?;
    let mut reader = DllReader::new(file);

    let basic_info = reader.create_pe_info()?;
    let program = reader.get_program()?;

    // 获取导出函数
    let export_count = program.exports.functions.len();
    let sample_exports = program.exports.functions.iter().take(20).cloned().collect();
    // 获取导入函数
    let import_count = program.imports.entries.iter().map(|e| e.functions.len()).sum::<usize>();
    let sample_imports: Vec<String> =
        program.imports.entries.iter().flat_map(|e| e.functions.iter()).take(20).cloned().collect();
    // 获取节信息
    let section_count = program.sections.len();
    let section_names = program.sections.iter().map(|s| s.name.clone()).collect();
    Ok(WindowsDllAnalysis {
        dll_name: path.file_stem().unwrap_or_default().to_string_lossy().to_string(),
        basic_info,
        export_count,
        import_count,
        section_count,
        sample_exports,
        sample_imports,
        section_names,
        analysis_success: true,
        error_message: None,
    })
}
