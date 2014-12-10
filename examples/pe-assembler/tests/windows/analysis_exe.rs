use crate::test_tools::test_path;
use gaia_types::{
    helpers::{open_file, save_json, Architecture},
    GaiaError,
};
use pe_assembler::{
    formats::exe::reader::ExeReader,
    helpers::PeReader,
    types::{PeInfo, SubsystemType},
};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Windows 可执行文件分析结果
#[derive(Debug, Serialize, Deserialize)]
pub struct WindowsExeAnalysis {
    /// 可执行文件名称
    pub exe_name: String,
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
    /// 链接器版本信息
    pub linker_version: String,
    /// 操作系统版本要求
    pub os_version: String,
    /// 映像版本
    pub image_version: String,
    /// 子系统版本要求
    pub subsystem_version: String,
    /// 校验和
    pub checksum: u32,
    /// DLL特征标志
    pub dll_characteristics: u16,
    /// 栈保留大小
    pub stack_reserve_size: u64,
    /// 栈提交大小
    pub stack_commit_size: u64,
    /// 堆保留大小
    pub heap_reserve_size: u64,
    /// 堆提交大小
    pub heap_commit_size: u64,
    /// 代码节大小
    pub code_size: u32,
    /// 初始化数据大小
    pub initialized_data_size: u32,
    /// 未初始化数据大小
    pub uninitialized_data_size: u32,
    /// 分析是否成功
    pub analysis_success: bool,
    /// 错误详情（如果分析失败）
    #[serde(skip)]
    pub error_details: Option<GaiaError>,
}

#[test]
fn analyze_essential_windows_executables() -> Result<(), GaiaError> {
    println!("开始分析 Windows 系统可执行文件...\n");

    // 常见的 Windows 系统可执行文件
    let essential_exes = vec![
        r"C:\Windows\System32\notepad.exe",
        r"C:\Windows\System32\calc.exe",
        r"C:\Windows\System32\cmd.exe",
        r"C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe",
        r"C:\Windows\explorer.exe",
        r"C:\Windows\System32\taskmgr.exe",
        r"C:\Windows\regedit.exe",
    ];

    let mut analyses = Vec::new();

    for exe_path in essential_exes {
        let exe_name = Path::new(exe_path).file_stem().unwrap_or_default().to_string_lossy().to_string();
        println!("分析 EXE: {}", exe_name);
        let path = Path::new(exe_path);

        if path.exists() {
            match analyze_exe_file(path) {
                Ok(analysis) => {
                    println!(
                        "  ✓ 成功分析 {} - 导出函数: {}, 导入函数: {}, 节数: {}",
                        exe_name, analysis.export_count, analysis.import_count, analysis.section_count
                    );
                    analyses.push(analysis);
                }
                Err(e) => {
                    println!("  ✗ 分析失败 {}: {:?}", exe_name, e);
                }
            }
        }
        else {
            println!("  ⚠ 文件不存在: {}", exe_path);
        }
    }

    let output_path = test_path("windows/windows_exe_analysis.json");
    save_json(&analyses, &output_path)?;
    Ok(())
}

pub fn analyze_exe_file(path: &Path) -> Result<WindowsExeAnalysis, GaiaError> {
    let (file, _url) = open_file(path)?;
    let file_size = file.metadata()?.len();
    let mut reader = ExeReader::new(file);

    // 使用 lazy reader 模式：先检查是否已读取，如果没有则强制读取
    match reader.get_program() {
        Ok(program) => {
            // 获取基本信息
            let basic_info = PeInfo {
                target_arch: program.header.coff_header.get_architecture(),
                subsystem: program.header.optional_header.subsystem,
                entry_point: program.header.optional_header.address_of_entry_point,
                image_base: program.header.optional_header.image_base,
                section_count: program.header.coff_header.number_of_sections,
                file_size,
            };

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

            // 获取详细的头部信息
            let opt_header = &program.header.optional_header;

            Ok(WindowsExeAnalysis {
                exe_name: path.file_stem().unwrap_or_default().to_string_lossy().to_string(),
                basic_info,
                export_count,
                import_count,
                section_count,
                sample_exports,
                sample_imports,
                section_names,
                linker_version: format!("{}.{}", opt_header.major_linker_version, opt_header.minor_linker_version),
                os_version: format!(
                    "{}.{}",
                    opt_header.major_operating_system_version, opt_header.minor_operating_system_version
                ),
                image_version: format!("{}.{}", opt_header.major_image_version, opt_header.minor_image_version),
                subsystem_version: format!("{}.{}", opt_header.major_subsystem_version, opt_header.minor_subsystem_version),
                checksum: opt_header.checksum,
                dll_characteristics: opt_header.dll_characteristics,
                stack_reserve_size: opt_header.size_of_stack_reserve,
                stack_commit_size: opt_header.size_of_stack_commit,
                heap_reserve_size: opt_header.size_of_heap_reserve,
                heap_commit_size: opt_header.size_of_heap_commit,
                code_size: opt_header.size_of_code,
                initialized_data_size: opt_header.size_of_initialized_data,
                uninitialized_data_size: opt_header.size_of_uninitialized_data,
                analysis_success: true,
                error_details: None,
            })
        }
        Err(error) => {
            // 分析失败，返回基本信息
            Ok(WindowsExeAnalysis {
                exe_name: path.file_stem().unwrap_or_default().to_string_lossy().to_string(),
                basic_info: PeInfo {
                    target_arch: Architecture::Unknown,
                    subsystem: SubsystemType::Console,
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
                linker_version: "0.0".to_string(),
                os_version: "0.0".to_string(),
                image_version: "0.0".to_string(),
                subsystem_version: "0.0".to_string(),
                checksum: 0,
                dll_characteristics: 0,
                stack_reserve_size: 0,
                stack_commit_size: 0,
                heap_reserve_size: 0,
                heap_commit_size: 0,
                code_size: 0,
                initialized_data_size: 0,
                uninitialized_data_size: 0,
                analysis_success: false,
                error_details: Some(error),
            })
        }
    }
}
