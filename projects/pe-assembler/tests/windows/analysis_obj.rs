use crate::test_tools::test_path;
use gaia_types::{helpers::save_json, GaiaError};
use pe_assembler::formats::obj::reader::ObjReader;
use serde::{Deserialize, Serialize};
use std::{
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

/// Windows OBJ 文件分析结果
#[derive(Debug, Serialize, Deserialize)]
pub struct WindowsObjAnalysis {
    /// 分析是否成功
    pub analysis_success: bool,
    /// 错误信息（如果分析失败）- 跳过序列化
    #[serde(skip)]
    pub error_details: Option<GaiaError>,
    /// 文件大小
    pub file_size: u64,
    /// 机器类型
    pub machine_type: Option<u16>,
    /// 节数量
    pub number_of_sections: u32,
    /// 符号数量
    pub number_of_symbols: u32,
    /// 时间戳（使用SystemTime）
    pub timestamp: SystemTime,
    /// 特征标志
    pub characteristics: u16,
    /// 节名称列表
    pub section_names: Vec<String>,
    /// 符号名称列表（样本）- 修复编码问题
    pub symbol_names: Vec<String>,
}

/// 分析单个 OBJ 文件
fn analyze_obj_file<P: AsRef<Path>>(path: P) -> WindowsObjAnalysis {
    let path_ref = path.as_ref();

    // 检查文件是否存在
    if !path_ref.exists() {
        return WindowsObjAnalysis {
            analysis_success: false,
            error_details: Some(GaiaError::invalid_data("文件不存在")),
            file_size: 0,
            machine_type: None,
            number_of_sections: 0,
            number_of_symbols: 0,
            timestamp: UNIX_EPOCH,
            characteristics: 0,
            section_names: vec![],
            symbol_names: vec![],
        };
    }

    // 获取文件大小
    let file_size = match std::fs::metadata(&path_ref) {
        Ok(metadata) => metadata.len(),
        Err(_) => 0,
    };

    // 尝试读取 OBJ 文件
    match ObjReader::from_file(&path_ref) {
        Ok(mut reader) => {
            match reader.view() {
                Ok(coff_info) => {
                    // 尝试读取完整对象以获取更多信息
                    let obj_result = reader.read_object();
                    let coff_object = obj_result.result.ok();

                    // 收集节名称
                    let section_names: Vec<String> = coff_object
                        .as_ref()
                        .map(|obj| {
                            obj.sections
                                .iter()
                                .map(|section| {
                                    // 修复编码问题：正确处理节名称
                                    let name_bytes = &section.header.name;
                                    let name_str = match std::str::from_utf8(name_bytes) {
                                        Ok(s) => s,
                                        Err(_) => &String::from_utf8_lossy(name_bytes),
                                    };
                                    name_str.trim_end_matches('\0').trim().to_string()
                                })
                                .filter(|name| !name.is_empty())
                                .collect()
                        })
                        .unwrap_or_default();

                    // 收集符号名称（取前10个作为样本）- 修复编码问题
                    let symbol_names: Vec<String> = coff_object
                        .as_ref()
                        .map(|obj| {
                            obj.symbols
                                .iter()
                                .take(10)
                                .filter_map(|symbol| {
                                    if !symbol.name.is_empty() {
                                        // 修复编码问题：确保符号名称是有效的UTF-8
                                        let clean_name = symbol
                                            .name
                                            .chars()
                                            .filter(|c| c.is_ascii_graphic() || c.is_ascii_whitespace())
                                            .collect::<String>()
                                            .trim()
                                            .to_string();
                                        if !clean_name.is_empty() {
                                            Some(clean_name)
                                        }
                                        else {
                                            Some(format!("<binary_symbol_{}>", symbol.name.len()))
                                        }
                                    }
                                    else {
                                        None
                                    }
                                })
                                .collect()
                        })
                        .unwrap_or_default();

                    // 将u32时间戳转换为SystemTime
                    let timestamp = UNIX_EPOCH + std::time::Duration::from_secs(coff_info.timestamp as u64);

                    WindowsObjAnalysis {
                        analysis_success: true,
                        error_details: None,
                        file_size,
                        machine_type: Some(match coff_info.target_arch {
                            gaia_types::helpers::Architecture::X86 => 0x014c,
                            gaia_types::helpers::Architecture::X86_64 => 0x8664,
                            gaia_types::helpers::Architecture::ARM32 => 0x01c0,
                            gaia_types::helpers::Architecture::ARM64 => 0xaa64,
                            _ => 0,
                        }),
                        number_of_sections: coff_info.section_count as u32,
                        number_of_symbols: coff_info.symbol_count as u32,
                        timestamp,
                        characteristics: 0, // CoffInfo doesn't have characteristics
                        section_names,
                        symbol_names,
                    }
                }
                Err(e) => WindowsObjAnalysis {
                    analysis_success: false,
                    error_details: Some(e),
                    file_size,
                    machine_type: None,
                    number_of_sections: 0,
                    number_of_symbols: 0,
                    timestamp: UNIX_EPOCH,
                    characteristics: 0,
                    section_names: vec![],
                    symbol_names: vec![],
                },
            }
        }
        Err(e) => WindowsObjAnalysis {
            analysis_success: false,
            error_details: Some(e),
            file_size,
            machine_type: None,
            number_of_sections: 0,
            number_of_symbols: 0,
            timestamp: UNIX_EPOCH,
            characteristics: 0,
            section_names: vec![],
            symbol_names: vec![],
        },
    }
}

/// 测试解析 Windows 系统中的 obj 文件
#[test]
fn test_parse_system_obj_files() -> Result<(), GaiaError> {
    let obj_files = vec![
        r"C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Tools\MSVC\14.44.35207\lib\x64\binmode.obj",
        r"C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Tools\MSVC\14.44.35207\lib\x64\commode.obj",
        r"C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Tools\MSVC\14.44.35207\lib\x64\chkstk.obj",
        r"C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Tools\MSVC\14.44.35207\lib\x64\invalidcontinue.obj",
        r"C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Tools\MSVC\14.44.35207\lib\x64\legacy_stdio_float_rounding.obj",
        r"C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Tools\MSVC\14.44.35207\lib\x64\loosefpmath.obj",
        r"C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Tools\MSVC\14.44.35207\lib\x64\newmode.obj",
        r"C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Tools\MSVC\14.44.35207\lib\x64\noarg.obj",
    ];

    let mut analyses = Vec::new();

    for obj_file in &obj_files {
        println!("分析 OBJ 文件: {}", obj_file);

        let analysis = analyze_obj_file(obj_file);

        if analysis.analysis_success {
            println!(
                "  ✓ 成功 - 节数: {}, 符号数: {}, 文件大小: {} bytes",
                analysis.number_of_sections, analysis.number_of_symbols, analysis.file_size
            );
        }
        else {
            println!("  ✗ 失败: {}", analysis.error_details.as_ref().map(|e| e.to_string()).as_deref().unwrap_or("未知错误"));
        }

        analyses.push(analysis);
    }

    let output_path = test_path("windows/windows_obj_analysis.json");
    save_json(&analyses, &output_path)?;

    Ok(())
}
