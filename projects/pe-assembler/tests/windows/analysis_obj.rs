use crate::test_tools::test_path;
use gaia_types::{helpers::save_json, GaiaError};
use pe_assembler::formats::obj::reader::ObjReader;
use serde::{Deserialize, Serialize};
use std::{
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};
use gaia_types::helpers::open_file;

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
    let (file, _url) = match open_file(&path_ref) {
        Ok(result) => result,
        Err(_) => return WindowsObjAnalysis {
            analysis_success: false,
            error_details: Some(GaiaError::invalid_data("无法打开文件")),
            file_size,
            machine_type: None,
            number_of_sections: 0,
            number_of_symbols: 0,
            timestamp: UNIX_EPOCH,
            characteristics: 0,
            section_names: vec![],
            symbol_names: vec![],
        },
    };
    
    match ObjReader::new(file).read_object().result {
        Ok(coff_object) => {
            // 收集节名称
            let section_names: Vec<String> = coff_object
                .sections
                .iter()
                .map(|section| {
                    section.header.get_name().to_string()
                })
                .collect();

            // 收集符号名称
            let symbol_names: Vec<String> = coff_object
                .symbols
                .iter()
                .map(|symbol| symbol.name.clone())
                .collect();

            WindowsObjAnalysis {
                analysis_success: true,
                error_details: None,
                file_size,
                machine_type: Some(coff_object.header.machine),
                number_of_sections: coff_object.header.number_of_sections as u32,
                number_of_symbols: coff_object.header.number_of_symbols,
                timestamp: UNIX_EPOCH + std::time::Duration::from_secs(coff_object.header.time_date_stamp as u64),
                characteristics: coff_object.header.characteristics,
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
