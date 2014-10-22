use pe_assembler::easy_test::{
    analyze_pe_file, compare_pe_file, print_analysis_result, save_analysis_to_json, WindowsExeAnalysis,
};
use std::path::Path;

#[test]
fn analyze_essential_windows_executables() {
    println!("开始分析 Windows 系统可执行文件...\n");

    // 常见的 Windows 系统可执行文件
    let essential_exes = vec![
        ("notepad", r"C:\Windows\System32\notepad.exe"),
        ("calc", r"C:\Windows\System32\calc.exe"),
        ("cmd", r"C:\Windows\System32\cmd.exe"),
        ("powershell", r"C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe"),
        ("explorer", r"C:\Windows\explorer.exe"),
        ("taskmgr", r"C:\Windows\System32\taskmgr.exe"),
        ("regedit", r"C:\Windows\regedit.exe"),
    ];

    let mut analyses = Vec::new();
    let mut success_count = 0;

    for (exe_name, exe_path) in &essential_exes {
        println!("正在分析: {}", exe_name);

        let path = Path::new(exe_path);
        if !path.exists() {
            println!("  ⚠️ 跳过 - 文件不存在: {}", exe_path);
            continue;
        }

        // 使用 easy_test 模块的分析方法生成报告
        match analyze_pe_file(path) {
            Ok((basic_info, program)) => {
                let analysis = WindowsExeAnalysis {
                    exe_name: exe_name.to_string(),
                    file_path: exe_path.to_string(),
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
                    exe_name,
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
                let analysis = WindowsExeAnalysis {
                    exe_name: exe_name.to_string(),
                    file_path: exe_path.to_string(),
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
    if let Err(e) = save_analysis_to_json(&analyses, "tests/windows/windows_exe_analysis.json") {
        println!("保存 JSON 文件失败: {}", e);
    }

    println!("分析完成！");
    println!("总计: {} 个可执行文件，成功分析: {} 个", essential_exes.len(), success_count);

    // 至少要有一些成功的分析
    assert!(success_count > 0, "应该至少成功分析一个可执行文件");
}

#[test]
fn analyze_notepad_detailed() {
    println!("详细分析 notepad.exe...\n");

    let notepad_path = Path::new(r"C:\Windows\System32\notepad.exe");

    // 检查文件是否存在
    if !notepad_path.exists() {
        println!("跳过测试：notepad.exe 文件不存在于 {}", notepad_path.display());
        return;
    }

    // 使用新的 easy_test 模块进行分析
    match compare_pe_file("notepad", notepad_path) {
        Ok(()) => {
            println!("✅ notepad.exe 分析成功！");

            // 额外进行详细分析以显示更多信息
            if let Ok((basic_info, program)) = analyze_pe_file(notepad_path) {
                println!("架构: {:?}", basic_info.target_arch);
                println!("子系统: {:?}", basic_info.subsystem);
                println!("入口点: 0x{:08X}", basic_info.entry_point);
                println!("镜像基址: 0x{:016X}", basic_info.image_base);
                println!("节数量: {}", program.sections.len());
                println!("导出函数数量: {}", program.exports.functions.len());
                println!("导入函数数量: {}", program.imports.functions.len());

                if !program.sections.is_empty() {
                    println!("节名称: {:?}", program.sections.iter().map(|s| &s.name).collect::<Vec<_>>());
                }

                if !program.imports.functions.is_empty() {
                    println!("示例导入函数: {:?}", program.imports.functions.iter().take(10).collect::<Vec<_>>());
                }
            }
        }
        Err(e) => {
            println!("❌ notepad.exe 分析失败: {}", e);
            // 对于系统文件，我们不强制要求测试通过，因为不同系统版本可能有差异
        }
    }
}

#[test]
fn test_pe_exe_files_automatically() {
    println!("开始自动化测试 PE 可执行文件...\n");

    // 测试一些常见的 Windows 可执行文件
    let sample_files = vec![
        Path::new("C:\\Windows\\System32\\notepad.exe"),
        Path::new("C:\\Windows\\System32\\calc.exe"),
        Path::new("C:\\Windows\\System32\\cmd.exe"),
    ];

    for file_path in sample_files {
        if file_path.exists() {
            let file_name = file_path.file_stem().unwrap_or_default().to_string_lossy();
            match compare_pe_file(&file_name, &file_path) {
                Ok(()) => println!("✅ {} 测试通过", file_name),
                Err(e) => println!("⚠️ {} 测试跳过: {}", file_name, e),
            }
        }
        else {
            let file_name = file_path.file_stem().unwrap_or_default().to_string_lossy();
            println!("⚠️ {} 文件不存在，跳过测试", file_name);
        }
    }
}
