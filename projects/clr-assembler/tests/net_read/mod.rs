use crate::test_tools::test_path;
use clr_assembler::formats::dll::reader::{read_dotnet_assembly, DotNetReader, DotNetReaderOptions};
use gaia_types::helpers::save_json;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// .NET DLL 分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetDllAnalysis {
    pub dll_name: String,
    pub assembly_info: String,
    pub version: String,
    pub type_count: usize,
    pub method_count: usize,
    pub field_count: usize,
    pub sample_type: String,
    pub sample_method: String,
    pub referenced_assemblies: Vec<String>,
    pub analysis_success: bool,
    pub error_message: Option<String>,
}

impl Default for NetDllAnalysis {
    fn default() -> Self {
        Self {
            dll_name: String::new(),
            assembly_info: "Unknown".to_string(),
            version: "0.0.0.0".to_string(),
            type_count: 0,
            method_count: 0,
            field_count: 0,
            sample_type: String::new(),
            sample_method: String::new(),
            referenced_assemblies: Vec::new(),
            analysis_success: false,
            error_message: None,
        }
    }
}

/// 分析单个.NET DLL文件
pub fn analyze_net_dll_file(
    dll_path: &str,
    options: &DotNetReaderOptions,
) -> Result<NetDllAnalysis, Box<dyn std::error::Error>> {
    let mut analysis = NetDllAnalysis::default();
    analysis.dll_name = Path::new(dll_path).file_name().and_then(|n| n.to_str()).unwrap_or("unknown").to_string();

    // 检查文件是否存在
    if !Path::new(dll_path).exists() {
        analysis.error_message = Some("File not found".to_string());
        return Ok(analysis);
    }

    // 尝试读取.NET程序集
    match read_dotnet_assembly(dll_path, options).result {
        Ok(clr_program) => {
            // 程序集名称与版本
            analysis.assembly_info = clr_program.name.clone();
            analysis.version = format!(
                "{}.{}.{}.{}",
                clr_program.version.major, clr_program.version.minor, clr_program.version.build, clr_program.version.revision
            );

            // 统计类型、方法和字段数量（类型仅统计导出的 public/nested public）
            analysis.type_count = clr_program.types.iter().filter(|t| t.access_flags.is_public).count();
            analysis.method_count = clr_program.get_method_count();
            analysis.field_count = clr_program.get_field_count();

            // 获取示例类型和方法名称
            if let Some(sample_type) = clr_program.get_sample_type_name() {
                analysis.sample_type = sample_type;
            }
            if let Some(sample_method) = clr_program.get_sample_method_name() {
                analysis.sample_method = sample_method;
            }

            // 获取引用的程序集
            analysis.referenced_assemblies = clr_program.get_referenced_assemblies();

            analysis.analysis_success = true;
        }
        Err(e) => {
            analysis.error_message = Some(format!("Failed to read assembly: {}", e));
        }
    }

    Ok(analysis)
}

/// 分析常见的.NET DLL并生成报告
#[test]
fn test_analyze_common_net_dlls() {
    let common_dlls = vec![
        "mscorlib.dll",
        "System.dll",
        "System.Core.dll",
        "System.Data.dll",
        "System.Drawing.dll",
        "System.Windows.Forms.dll",
        "System.Xml.dll",
        "System.Web.dll",
        "Microsoft.CSharp.dll",
        "System.Runtime.dll",
    ];

    let mut results = Vec::new();

    let fallback_names_str = std::env::var("GAIA_CLR_ASMREF_FALLBACK_NAMES").unwrap_or_default();
    let fallback_names: Vec<String> =
        fallback_names_str.split(',').filter(|s| !s.trim().is_empty()).map(|s| s.trim().to_string()).collect();

    let reader_options = DotNetReaderOptions { assembly_ref_fallback_names: fallback_names };

    for dll_name in common_dlls {
        // 尝试在多个可能的位置查找DLL
        let possible_paths = vec![
            format!("C:\\Windows\\Microsoft.NET\\Framework64\\v4.0.30319\\{}", dll_name),
            format!("C:\\Windows\\Microsoft.NET\\Framework\\v4.0.30319\\{}", dll_name),
            format!("C:\\Program Files\\dotnet\\shared\\Microsoft.NETCore.App\\6.0.0\\{}", dll_name),
            format!("C:\\Program Files (x86)\\Reference Assemblies\\Microsoft\\Framework\\.NETFramework\\v4.8\\{}", dll_name),
        ];

        let mut analysis_result = None;
        for path in possible_paths {
            if let Ok(result) = analyze_net_dll_file(&path, &reader_options) {
                analysis_result = Some(result);
                break;
            }
        }

        if let Some(result) = analysis_result {
            println!(
                "✓ 成功分析 {} - 类型: {}, 方法: {}, 字段: {}",
                result.dll_name, result.type_count, result.method_count, result.field_count
            );
            results.push(result);
        }
        else {
            // 如果找不到文件，创建一个失败的分析结果
            let mut failed_result = NetDllAnalysis::default();
            failed_result.dll_name = dll_name.to_string();
            failed_result.error_message = Some("DLL file not found".to_string());
            println!("⚠ 未找到 {}", dll_name);
            results.push(failed_result);
        }
    }

    // 将结果保存到JSON文件
    let output_path = test_path("pyc_read/net_dll_analysis.json");
    save_json(&results, &output_path).expect("Failed to save analysis results");
    println!("分析结果已保存到: {:?}", output_path);

    // 打印统计信息
    let successful_analyses = results.iter().filter(|r| r.analysis_success).count();
    println!("成功分析: {}/{}", successful_analyses, results.len());
}
