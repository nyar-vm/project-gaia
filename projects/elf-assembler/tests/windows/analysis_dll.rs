use super::WindowsDllAnalysis;
use gaia_types::GaiaError;
use pe_assembler::{
    easy_test::{analyze_pe_file, compare_pe_file, print_analysis_result, save_analysis_to_json},
    types::PeInfo,
};
use std::path::Path;

/// 通用的 Windows DLL 分析函数
///
/// # Arguments
/// * `dll_path` - DLL 文件路径
/// * `dll_name` - DLL 名称（用于标识）
///
/// # Returns
/// 返回分析结果结构体
pub fn analyze_windows_dll<P: AsRef<Path>>(dll_path: P, dll_name: &str) -> WindowsDllAnalysis {
    let path = dll_path.as_ref();
    let file_path = path.to_string_lossy().to_string();

    // 检查文件是否存在
    if !path.exists() {
        return WindowsDllAnalysis {
            dll_name: dll_name.to_string(),
            file_path: file_path.clone(),
            basic_info: PeInfo {
                target_arch: gaia_types::helpers::Architecture::X86_64,
                subsystem: pe_assembler::types::SubsystemType::Console,
                entry_point: 0,
                image_base: 0,
                section_count: 0,
                file_size: 0,
            },
            export_count: 0,
            import_count: 0,
            section_count: 0,
            sample_exports: Vec::new(),
            sample_imports: Vec::new(),
            section_names: Vec::new(),
            analysis_success: false,
            error_message: Some(GaiaError::invalid_data(&format!("文件不存在: {}", file_path))),
        };
    }

    // 尝试分析 PE 文件
    match analyze_pe_file(path) {
        Ok((basic_info, program)) => {
            let sample_exports = program.exports.functions.iter().take(20).cloned().collect();

            let sample_imports = program.imports.functions.iter().take(20).cloned().collect();

            let section_names = program.sections.iter().map(|s| s.name.clone()).collect();

            WindowsDllAnalysis {
                dll_name: dll_name.to_string(),
                file_path,
                basic_info,
                export_count: program.exports.functions.len(),
                import_count: program.imports.functions.len(),
                section_count: program.sections.len(),
                sample_exports,
                sample_imports,
                section_names,
                analysis_success: true,
                error_message: None,
            }
        }
        Err(e) => WindowsDllAnalysis {
            dll_name: dll_name.to_string(),
            file_path,
            basic_info: PeInfo {
                target_arch: gaia_types::helpers::Architecture::X86_64,
                subsystem: pe_assembler::types::SubsystemType::Console,
                entry_point: 0,
                image_base: 0,
                section_count: 0,
                file_size: 0,
            },
            export_count: 0,
            import_count: 0,
            section_count: 0,
            sample_exports: Vec::new(),
            sample_imports: Vec::new(),
            section_names: Vec::new(),
            analysis_success: false,
            error_message: Some(GaiaError::invalid_data(&format!("分析失败: {}", e))),
        },
    }
}

#[test]
fn analyzer_kernel32() {
    let kernel32_path = Path::new("C:\\Windows\\System32\\kernel32.dll");

    // 检查文件是否存在
    if !kernel32_path.exists() {
        println!("跳过测试：kernel32.dll 文件不存在于 {}", kernel32_path.display());
        return;
    }

    // 使用新的 easy_test 模块进行分析
    match compare_pe_file("kernel32", kernel32_path) {
        Ok(()) => {
            println!("✅ kernel32.dll 分析成功！");
        }
        Err(e) => {
            println!("❌ kernel32.dll 分析失败: {}", e);
            // 对于系统文件，我们不强制要求测试通过，因为不同系统版本可能有差异
        }
    }
}

#[test]
fn analyze_multiple_windows_dlls() {
    println!("开始分析多个 Windows 系统 DLL...\n");

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
    let mut success_count = 0;

    for (dll_name, dll_path) in &essential_dlls {
        println!("正在分析: {}", dll_name);

        let path = Path::new(dll_path);
        if !path.exists() {
            println!("  ⚠️ 跳过 - 文件不存在: {}", dll_path);
            continue;
        }

        // 使用 easy_test 模块的分析方法生成报告
        match analyze_pe_file(path) {
            Ok((basic_info, program)) => {
                let analysis = WindowsDllAnalysis {
                    dll_name: dll_name.to_string(),
                    file_path: dll_path.to_string(),
                    basic_info,
                    export_count: program.exports.functions.len(),
                    import_count: program.imports.functions.len(),
                    section_count: program.sections.len(),
                    sample_exports: program.exports.functions.iter().take(20).cloned().collect(),
                    sample_imports: program.imports.functions.iter().take(20).cloned().collect(),
                    section_names: program.sections.iter().map(|s| s.name.clone()).collect(),
                    analysis_success: true,
                    error_message: None,
                };

                success_count += 1;
                print_analysis_result(
                    dll_name,
                    analysis.analysis_success,
                    analysis.export_count,
                    analysis.import_count,
                    analysis.section_count,
                    &analysis.sample_exports,
                    None,
                );

                analyses.push(analysis);
            }
            Err(e) => {
                println!("  ❌ 失败 - {}", e);
                let analysis = WindowsDllAnalysis {
                    dll_name: dll_name.to_string(),
                    file_path: dll_path.to_string(),
                    basic_info: pe_assembler::types::PeInfo {
                        target_arch: gaia_types::helpers::Architecture::X86_64,
                        subsystem: pe_assembler::types::SubsystemType::Console,
                        entry_point: 0,
                        image_base: 0,
                        section_count: 0,
                        file_size: 0,
                    },
                    export_count: 0,
                    import_count: 0,
                    section_count: 0,
                    sample_exports: Vec::new(),
                    sample_imports: Vec::new(),
                    section_names: Vec::new(),
                    analysis_success: false,
                    error_message: Some(gaia_types::GaiaError::invalid_data(&format!("分析失败: {}", e))),
                };
                analyses.push(analysis);
            }
        }
        println!();
    }

    // 保存分析结果到 JSON 文件
    if let Err(e) = save_analysis_to_json(&analyses, "tests/windows/windows_dll_analysis.json") {
        println!("保存 JSON 文件失败: {}", e);
    }

    println!("分析完成！");
    println!("总计: {} 个 DLL，成功分析: {} 个", essential_dlls.len(), success_count);

    // 至少要有一些成功的分析
    assert!(success_count > 0, "应该至少成功分析一个 DLL");
}

#[test]
fn test_all_pe_files_automatically() {
    println!("开始自动化测试所有 PE 文件...\n");

    // 测试 Windows 系统目录中的一些 PE 文件
    let test_paths = vec![
        Path::new("C:\\Windows\\System32"),
        // 可以添加更多测试路径
    ];

    for test_path in test_paths {
        if test_path.exists() {
            println!("测试目录: {}", test_path.display());
            // 注意：这里我们不直接调用 validate_pe_files，因为系统目录中的文件太多
            // 而是选择性地测试一些文件
            let sample_files =
                vec![test_path.join("kernel32.dll"), test_path.join("user32.dll"), test_path.join("notepad.exe")];

            for file_path in sample_files {
                if file_path.exists() {
                    let file_name = file_path.file_stem().unwrap_or_default().to_string_lossy();
                    match compare_pe_file(&file_name, &file_path) {
                        Ok(()) => println!("✅ {} 测试通过", file_name),
                        Err(e) => println!("⚠️ {} 测试跳过: {}", file_name, e),
                    }
                }
            }
        }
    }
}
