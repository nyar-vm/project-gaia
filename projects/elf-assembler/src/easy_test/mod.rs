//! Easy test module for PE file analysis
//!
//! This module provides utilities for automated testing of PE file analysis,
//! including expectation generation, validation, and test organization.

use crate::{
    types::{PeInfo, PeProgram},
    viewer::PeView,
};
use gaia_types::GaiaError;
use serde::{Deserialize, Serialize};
use serde_json;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// PE 文件期望结构体 - 用于定义测试期望
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PeExpected {
    /// 期望的文件名
    pub file_name: String,
    /// 期望的架构
    pub target_arch: String,
    /// 期望的子系统类型
    pub subsystem: String,
    /// 期望的导出函数数量
    pub export_count: usize,
    /// 期望的导入函数数量
    pub import_count: usize,
    /// 期望的节数量
    pub section_count: usize,
    /// 期望包含的特定导出函数
    pub expected_exports: Vec<String>,
    /// 期望包含的特定导入函数
    pub expected_imports: Vec<String>,
    /// 期望的节名称
    pub expected_sections: Vec<String>,
    /// 期望的入口点（可选）
    pub entry_point: Option<u32>,
    /// 期望的镜像基址（可选）
    pub image_base: Option<u64>,
    /// 文件路径（用于调试）
    #[serde(skip)]
    pub file_path: PathBuf,
}

impl PeExpected {
    /// 从 PE 程序自动生成期望对象
    pub fn from_pe_program(program: &PeProgram, info: &PeInfo, file_path: &Path) -> Self {
        let mut expected = Self {
            file_name: file_path.file_name().unwrap_or_default().to_string_lossy().to_string(),
            target_arch: format!("{:?}", info.target_arch),
            subsystem: format!("{:?}", info.subsystem),
            export_count: program.exports.functions.len(),
            import_count: program.imports.functions.len(),
            section_count: program.sections.len(),
            expected_exports: program.exports.functions.iter().take(20).cloned().collect(),
            expected_imports: program.imports.functions.iter().take(20).cloned().collect(),
            expected_sections: program.sections.iter().map(|s| s.name.clone()).collect(),
            entry_point: Some(info.entry_point),
            image_base: Some(info.image_base),
            file_path: file_path.to_path_buf(),
        };

        // 自动检测特征
        expected.auto_detect_features(program);
        expected
    }

    /// 创建新的空期望对象
    pub fn new() -> Self {
        Self {
            file_name: String::new(),
            target_arch: String::new(),
            subsystem: String::new(),
            export_count: 0,
            import_count: 0,
            section_count: 0,
            expected_exports: Vec::new(),
            expected_imports: Vec::new(),
            expected_sections: Vec::new(),
            entry_point: None,
            image_base: None,
            file_path: PathBuf::new(),
        }
    }

    /// 保存期望到 JSON 文件
    pub fn save_to_json(&self, json_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json_content = serde_json::to_string_pretty(self)?;
        std::fs::write(json_path, json_content)?;
        println!("已保存期望文件: {}", json_path);
        Ok(())
    }

    /// 从 JSON 文件加载期望
    pub fn load_from_json(json_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let json_content = std::fs::read_to_string(json_path)?;
        let expected: Self = serde_json::from_str(&json_content)?;
        Ok(expected)
    }

    /// 验证当前 PE 程序是否符合期望
    pub fn validate_pe_program(&self, program: &PeProgram, info: &PeInfo) -> Result<(), String> {
        // 验证架构
        let actual_arch = format!("{:?}", info.target_arch);
        if actual_arch != self.target_arch {
            return Err(format!("架构不匹配: 期望 '{}', 实际 '{}'", self.target_arch, actual_arch));
        }

        // 验证子系统
        let actual_subsystem = format!("{:?}", info.subsystem);
        if actual_subsystem != self.subsystem {
            return Err(format!("子系统不匹配: 期望 '{}', 实际 '{}'", self.subsystem, actual_subsystem));
        }

        // 验证导出函数数量
        if program.exports.functions.len() != self.export_count {
            return Err(format!("导出函数数量不匹配: 期望 {}, 实际 {}", self.export_count, program.exports.functions.len()));
        }

        // 验证导入函数数量
        if program.imports.functions.len() != self.import_count {
            return Err(format!("导入函数数量不匹配: 期望 {}, 实际 {}", self.import_count, program.imports.functions.len()));
        }

        // 验证节数量
        if program.sections.len() != self.section_count {
            return Err(format!("节数量不匹配: 期望 {}, 实际 {}", self.section_count, program.sections.len()));
        }

        // 验证特定导出函数
        for expected_export in &self.expected_exports {
            if !program.exports.functions.iter().any(|f| f == expected_export) {
                return Err(format!("缺少期望的导出函数: {}", expected_export));
            }
        }

        // 验证特定导入函数
        for expected_import in &self.expected_imports {
            if !program.imports.functions.iter().any(|f| f == expected_import) {
                return Err(format!("缺少期望的导入函数: {}", expected_import));
            }
        }

        // 验证节名称
        for expected_section in &self.expected_sections {
            if !program.sections.iter().any(|s| s.name == *expected_section) {
                return Err(format!("缺少期望的节: {}", expected_section));
            }
        }

        // 验证入口点（如果指定）
        if let Some(expected_entry) = self.entry_point {
            if info.entry_point != expected_entry {
                return Err(format!("入口点不匹配: 期望 0x{:08X}, 实际 0x{:08X}", expected_entry, info.entry_point));
            }
        }

        // 验证镜像基址（如果指定）
        if let Some(expected_base) = self.image_base {
            if info.image_base != expected_base {
                return Err(format!("镜像基址不匹配: 期望 0x{:016X}, 实际 0x{:016X}", expected_base, info.image_base));
            }
        }

        Ok(())
    }

    /// 自动检测 PE 文件特征
    fn auto_detect_features(&mut self, program: &PeProgram) {
        // 检测常见的 Windows API 函数
        let common_apis = vec![
            "CreateFileA",
            "CreateFileW",
            "ReadFile",
            "WriteFile",
            "CloseHandle",
            "LoadLibraryA",
            "LoadLibraryW",
            "GetProcAddress",
            "ExitProcess",
            "MessageBoxA",
            "MessageBoxW",
            "GetCurrentProcess",
            "GetCurrentThread",
        ];

        for api in &common_apis {
            if program.exports.functions.iter().any(|f| f.contains(api))
                || program.imports.functions.iter().any(|f| f.contains(api))
            {
                // 可以在这里添加特征标记，类似于 JASM 的语法糖特征
            }
        }
    }
}

/// 获取 PE 文件对应的 JSON 期望文件路径
pub fn get_expected_json_path(path: &Path) -> String {
    let parent = path.parent().unwrap_or(Path::new("."));
    let stem = path.file_stem().unwrap().to_string_lossy();
    parent.join(format!("{}.expected.json", stem)).to_string_lossy().to_string()
}

/// 验证指定文件夹中的所有 PE 文件
pub fn validate_pe_files(folder: &Path) {
    let mut test_count = 0;
    let mut success_count = 0;
    let mut failed_tests = Vec::new();

    for entry in WalkDir::new(folder).into_iter().filter_map(|e| e.ok()).filter(|e| {
        if let Some(ext) = e.path().extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            ext_str == "exe" || ext_str == "dll"
        }
        else {
            false
        }
    }) {
        let pe_path = entry.path();
        let test_name = entry.path().file_stem().unwrap_or_default().to_string_lossy().to_string();

        test_count += 1;
        println!("\n--- 测试文件: {} ---", pe_path.display());

        match compare_pe_file(&test_name, &pe_path) {
            Ok(()) => {
                success_count += 1;
                println!("✓ 测试通过: {}", test_name);
            }
            Err(e) => {
                failed_tests.push((test_name.clone(), e.to_string()));
                println!("✗ 测试失败: {} - {}", test_name, e);
            }
        }
    }

    println!("\n=== 测试总结 ===");
    println!("总测试数: {}", test_count);
    println!("成功数: {}", success_count);
    println!("失败数: {}", failed_tests.len());

    if !failed_tests.is_empty() {
        println!("\n失败的测试:");
        for (name, error) in &failed_tests {
            println!("  - {}: {}", name, error);
        }
    }

    // 如果有失败的测试，让测试失败
    if !failed_tests.is_empty() {
        panic!("有 {} 个测试失败", failed_tests.len());
    }
}

/// 比较单个 PE 文件与其期望
pub fn compare_pe_file(test_name: &str, file_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("正在分析: {}", test_name);

    // 检查文件是否存在
    if !file_path.exists() {
        return Err(format!("文件不存在: {}", file_path.display()).into());
    }

    // 分析 PE 文件
    let pe_view = PeView::view_file(file_path)?;
    let info = pe_view.info().clone();
    let program = pe_view.to_program()?;

    println!(
        "PE 分析成功: {} 个导出函数, {} 个导入函数, {} 个节",
        program.exports.functions.len(),
        program.imports.functions.len(),
        program.sections.len()
    );

    let json_path = get_expected_json_path(file_path);

    // 检查是否存在期望文件
    if !Path::new(&json_path).exists() {
        // 首次运行，创建期望文件
        let expected = PeExpected::from_pe_program(&program, &info, file_path);
        expected.save_to_json(&json_path)?;
        println!("✓ 已生成期望文件: {}", json_path);
        print_pe_summary(&program, &info);
        return Ok(());
    }

    // 加载现有期望文件并验证
    let expected = PeExpected::load_from_json(&json_path)?;
    println!("✓ 已加载期望文件: {}", json_path);

    match expected.validate_pe_program(&program, &info) {
        Ok(()) => {
            println!("✓ 验证通过: 分析结果符合期望");
            print_pe_summary(&program, &info);
        }
        Err(e) => {
            println!("✗ 验证失败: {}", e);
            println!("\n当前分析结果:");
            print_pe_summary(&program, &info);
            println!("\n期望结果:");
            println!("{:#?}", expected);
            return Err(e.into());
        }
    }

    Ok(())
}

/// 打印 PE 程序摘要信息
pub fn print_pe_summary(program: &PeProgram, info: &PeInfo) {
    println!("PE 分析结果摘要:");
    println!("  架构: {:?}", info.target_arch);
    println!("  子系统: {:?}", info.subsystem);
    println!("  入口点: 0x{:08X}", info.entry_point);
    println!("  镜像基址: 0x{:016X}", info.image_base);
    println!("  文件大小: {} 字节", info.file_size);

    println!("  导出函数数: {}", program.exports.functions.len());
    if !program.exports.functions.is_empty() {
        println!("    示例导出函数:");
        for (i, func) in program.exports.functions.iter().take(5).enumerate() {
            println!("      {}: {}", i + 1, func);
        }
        if program.exports.functions.len() > 5 {
            println!("      ... 还有 {} 个函数", program.exports.functions.len() - 5);
        }
    }

    println!("  导入函数数: {}", program.imports.functions.len());
    if !program.imports.functions.is_empty() {
        println!("    示例导入函数:");
        for (i, func) in program.imports.functions.iter().take(5).enumerate() {
            println!("      {}: {}", i + 1, func);
        }
        if program.imports.functions.len() > 5 {
            println!("      ... 还有 {} 个函数", program.imports.functions.len() - 5);
        }
    }

    println!("  节数: {}", program.sections.len());
    for (i, section) in program.sections.iter().enumerate() {
        println!(
            "    {}: {} (虚拟地址: 0x{:08X}, 大小: 0x{:08X})",
            i + 1,
            section.name,
            section.virtual_address,
            section.virtual_size
        );
    }
}

/// Windows DLL 分析结果
#[derive(Debug, Serialize, Deserialize)]
pub struct WindowsDllAnalysis {
    /// DLL 文件名
    pub dll_name: String,
    /// 文件路径
    pub file_path: String,
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

/// Windows 可执行文件分析结果
#[derive(Debug, Serialize, Deserialize)]
pub struct WindowsExeAnalysis {
    /// 可执行文件名称
    pub exe_name: String,
    /// 文件路径
    pub file_path: String,
    /// PE 基本信息
    pub basic_info: PeInfo,
    /// 导出函数数量
    pub export_count: usize,
    /// 导入函数数量
    pub import_count: usize,
    /// 节数量
    pub section_count: usize,
    /// 示例导出函数（前20个）
    pub sample_exports: Vec<String>,
    /// 示例导入函数（前20个）
    pub sample_imports: Vec<String>,
    /// 节名称列表
    pub section_names: Vec<String>,
    /// 分析是否成功
    pub analysis_success: bool,
    /// 错误信息（如果分析失败）
    #[serde(skip)]
    pub error_message: Option<GaiaError>,
}

/// 通用的 PE 文件分析函数
pub fn analyze_pe_file<P: AsRef<Path>>(path: P) -> Result<(PeInfo, PeProgram), Box<dyn std::error::Error>> {
    let pe_view = PeView::view_file(path.as_ref())?;
    let basic_info = pe_view.info().clone();
    let program = pe_view.to_program()?;
    Ok((basic_info, program))
}

/// 通用的 JSON 保存函数
pub fn save_analysis_to_json<T: Serialize>(analyses: &[T], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // 使用自定义格式化器，设置 4 空格缩进
    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let mut buf = Vec::new();
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
    analyses.serialize(&mut ser)?;

    std::fs::write(output_path, buf)?;
    println!("分析结果已保存到: {}", output_path);
    Ok(())
}

/// 通用的分析结果打印函数
pub fn print_analysis_result(
    _name: &str,
    success: bool,
    export_count: usize,
    import_count: usize,
    section_count: usize,
    sample_exports: &[String],
    error_message: Option<&str>,
) {
    if success {
        println!("  ✅ 成功 - 导出: {}, 导入: {}, 节: {}", export_count, import_count, section_count);

        // 显示一些示例导出函数
        if !sample_exports.is_empty() {
            println!("  示例导出函数: {:?}", sample_exports.iter().take(3).collect::<Vec<_>>());
        }
    }
    else {
        println!("  ❌ 失败 - {}", error_message.unwrap_or("未知错误"));
    }
}
