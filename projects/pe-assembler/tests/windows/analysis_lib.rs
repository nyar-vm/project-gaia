use crate::test_tools::test_path;
use gaia_types::{helpers::save_json, GaiaError};
use pe_assembler::formats::lib::reader::LibReader;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Windows 静态库分析结果
#[derive(Debug, Serialize, Deserialize)]
pub struct WindowsLibAnalysis {
    /// 库文件名称
    pub lib_name: String,
    /// 文件类型（始终为StaticLibrary）
    pub file_type: String,
    /// 成员数量（对象文件数量）
    pub member_count: usize,
    /// 符号数量
    pub symbol_count: usize,
    /// 文件大小
    pub file_size: u64,
    /// 示例成员列表（前10个）
    pub sample_members: Vec<String>,
    /// 示例符号列表（前10个）
    pub sample_symbols: Vec<String>,
    /// 分析是否成功
    pub analysis_success: bool,
    /// 错误信息（如果分析失败）
    #[serde(skip)]
    pub error_message: Option<GaiaError>,
}

#[test]
fn test_parse_system_lib_files() -> Result<(), GaiaError> {
    println!("开始分析 Windows 系统 LIB 文件...\n");

    // 常见的 Windows 系统 lib 文件路径
    let test_paths = vec![
        // kernel32
        r"C:\Program Files (x86)\Windows Kits\10\Lib\10.0.26100.0\um\x86\kernel32.Lib",
        r"C:\Program Files (x86)\Windows Kits\10\Lib\10.0.26100.0\um\x64\kernel32.Lib",
        r"C:\Program Files (x86)\Windows Kits\10\Lib\10.0.26100.0\um\amd64\kernel32.Lib",
        // user32
        r"C:\Program Files (x86)\Windows Kits\10\Lib\10.0.26100.0\um\x86\user32.Lib",
        r"C:\Program Files (x86)\Windows Kits\10\Lib\10.0.26100.0\um\x64\user32.Lib",
        r"C:\Program Files (x86)\Windows Kits\10\Lib\10.0.26100.0\um\arm64\user32.Lib",
        // msvcrt
        r"C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Tools\MSVC\14.44.35207\lib\x64\msvcrt.lib",
        r"C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Tools\MSVC\14.44.35207\lib\x86\msvcrt.lib",
        // ddraw
        r"C:\Program Files (x86)\Windows Kits\10\Lib\10.0.26100.0\um\x86\ddraw.lib",
        r"C:\Program Files (x86)\Windows Kits\10\Lib\10.0.26100.0\um\x64\ddraw.lib",
        r"C:\Program Files (x86)\Windows Kits\10\Lib\10.0.26100.0\um\arm64\ddraw.lib",
    ];

    let mut analyses = Vec::new();

    for lib_path in test_paths {
        let lib_name = Path::new(lib_path).file_stem().unwrap_or_default().to_string_lossy().to_string();
        println!("分析 LIB: {}", lib_name);
        let path = Path::new(lib_path);

        if path.exists() {
            match analyze_lib_file(path) {
                Ok(analysis) => {
                    println!(
                        "  ✓ 成功分析 {} - 成员数: {}, 符号数: {}, 文件大小: {} bytes",
                        lib_name, analysis.member_count, analysis.symbol_count, analysis.file_size
                    );
                    analyses.push(analysis);
                }
                Err(e) => {
                    println!("  ✗ 分析失败 {}: {:?}", lib_name, e);
                }
            }
        }
        else {
            println!("  ⚠ 文件不存在: {}", lib_path);
        }
    }

    let output_path = test_path("windows/windows_lib_analysis.json");
    save_json(&analyses, &output_path)?;

    Ok(())
}

pub fn analyze_lib_file(path: &Path) -> Result<WindowsLibAnalysis, GaiaError> {
    let mut reader = LibReader::from_file(path)?;

    // 验证是否为有效的静态库文件
    if !reader.is_valid_lib()? {
        return Ok(WindowsLibAnalysis {
            lib_name: path.file_stem().unwrap_or_default().to_string_lossy().to_string(),
            file_type: "StaticLibrary".to_string(),
            member_count: 0,
            symbol_count: 0,
            file_size: 0,
            sample_members: Vec::new(),
            sample_symbols: Vec::new(),
            analysis_success: false,
            error_message: Some(GaiaError::invalid_data("不是有效的静态库文件")),
        });
    }

    // 读取库信息
    let library_result = reader.read_library();

    match library_result {
        Ok(library) => {
            // 获取成员信息
            let member_count = library.members.len();
            let sample_members = library
                .members
                .iter()
                .take(10)
                .map(|m| format!("{} ({} bytes)", m.header.name.trim_end_matches('\0').trim(), m.header.size))
                .collect();

            // 获取符号信息
            let symbol_count = library.symbol_index.len();
            let sample_symbols = library.symbol_index.iter().take(10).map(|(name, _)| name.clone()).collect();

            // 获取文件大小
            let file_size = reader.get_file_size().unwrap_or(0);

            Ok(WindowsLibAnalysis {
                lib_name: path.file_stem().unwrap_or_default().to_string_lossy().to_string(),
                file_type: "StaticLibrary".to_string(),
                member_count,
                symbol_count,
                file_size,
                sample_members,
                sample_symbols,
                analysis_success: true,
                error_message: None,
            })
        }
        Err(error) => Ok(WindowsLibAnalysis {
            lib_name: path.file_stem().unwrap_or_default().to_string_lossy().to_string(),
            file_type: "StaticLibrary".to_string(),
            member_count: 0,
            symbol_count: 0,
            file_size: 0,
            sample_members: Vec::new(),
            sample_symbols: Vec::new(),
            analysis_success: false,
            error_message: Some(error),
        }),
    }
}
