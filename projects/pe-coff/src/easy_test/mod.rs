//! Easy test module for COFF file analysis
//!
//! This module provides utilities for automated testing of COFF file analysis,
//! including convenience functions for analyzing COFF objects and static libraries.

use crate::{
    reader::{read_coff_from_file, read_lib_from_file, CoffReader},
    types::{CoffFileType, CoffObject, StaticLibrary},
};
use std::path::Path;
use walkdir::WalkDir;

// ============================================================================
// COFF 文件分析便利函数
// ============================================================================

/// 分析单个 COFF 对象文件并打印详细信息
pub fn analyze_coff_file<P: AsRef<Path>>(path: P) -> Result<(), Box<dyn std::error::Error>> {
    let path = path.as_ref();
    println!("分析 COFF 文件: {}", path.display());

    let coff_object = read_coff_from_file(path)?;
    print_coff_summary(&coff_object);

    Ok(())
}

/// 分析单个静态库文件并打印详细信息
pub fn analyze_lib_file<P: AsRef<Path>>(path: P) -> Result<(), Box<dyn std::error::Error>> {
    let path = path.as_ref();
    println!("分析静态库文件: {}", path.display());

    let library = read_lib_from_file(path)?;
    print_lib_summary(&library);

    Ok(())
}

/// 打印 COFF 对象文件的摘要信息
pub fn print_coff_summary(coff_object: &CoffObject) {
    println!("COFF 对象文件信息:");
    println!("  机器类型: 0x{:04x}", coff_object.header.machine);
    println!("  节数量: {}", coff_object.header.number_of_sections);
    println!("  符号数量: {}", coff_object.header.number_of_symbols);
    println!("  时间戳: {}", coff_object.header.time_date_stamp);
    println!("  特征: 0x{:04x}", coff_object.header.characteristics);

    if !coff_object.sections.is_empty() {
        println!("  节信息:");
        for (i, section) in coff_object.sections.iter().enumerate() {
            let name_raw = String::from_utf8_lossy(&section.header.name);
            let name = name_raw.trim_end_matches('\0');
            println!(
                "    节 {}: {} (大小: {} 字节, 特征: 0x{:08x})",
                i + 1,
                name,
                section.header.size_of_raw_data,
                section.header.characteristics
            );
        }
    }

    if !coff_object.symbols.is_empty() {
        println!("  符号信息 (前5个):");
        for (i, symbol) in coff_object.symbols.iter().take(5).enumerate() {
            println!(
                "    符号 {}: {} (值: 0x{:08x}, 节: {}, 类型: 0x{:04x})",
                i + 1,
                symbol.name,
                symbol.value,
                symbol.section_number,
                symbol.symbol_type
            );
        }
        if coff_object.symbols.len() > 5 {
            println!("    ... 还有 {} 个符号", coff_object.symbols.len() - 5);
        }
    }
}

/// 打印静态库文件的摘要信息
pub fn print_lib_summary(library: &StaticLibrary) {
    println!("静态库文件信息:");
    println!("  签名: {}", library.signature);
    println!("  成员数量: {}", library.members.len());
    println!("  符号索引数量: {}", library.symbol_index.len());

    if !library.members.is_empty() {
        println!("  成员信息 (前5个):");
        for (i, member) in library.members.iter().take(5).enumerate() {
            println!(
                "    成员 {}: {} (大小: {} 字节, 时间戳: {})",
                i + 1,
                member.header.name,
                member.header.size,
                member.header.timestamp
            );

            if let Some(ref coff_obj) = member.coff_object {
                println!(
                    "      COFF 对象: {} 节, {} 符号",
                    coff_obj.header.number_of_sections, coff_obj.header.number_of_symbols
                );
            }
        }
        if library.members.len() > 5 {
            println!("    ... 还有 {} 个成员", library.members.len() - 5);
        }
    }

    if !library.symbol_index.is_empty() {
        println!("  符号索引 (前10个):");
        for (i, (symbol_name, member_index)) in library.symbol_index.iter().take(10).enumerate() {
            println!("    符号 {}: {} -> 成员 {}", i + 1, symbol_name, member_index);
        }
        if library.symbol_index.len() > 10 {
            println!("    ... 还有 {} 个符号", library.symbol_index.len() - 10);
        }
    }
}

/// 检测文件类型并进行相应的分析
pub fn analyze_file_auto<P: AsRef<Path>>(path: P) -> Result<(), Box<dyn std::error::Error>> {
    let path = path.as_ref();

    // 根据文件扩展名判断类型
    match path.extension().and_then(|s| s.to_str()) {
        Some("obj") => analyze_coff_file(path),
        Some("lib") => analyze_lib_file(path),
        Some("a") => analyze_lib_file(path), // Unix 静态库
        _ => {
            // 尝试检测文件类型
            match CoffReader::detect_file_type(path) {
                Ok(CoffFileType::Object) => analyze_coff_file(path),
                Ok(CoffFileType::StaticLibrary) => analyze_lib_file(path),
                Ok(file_type) => {
                    println!("检测到文件类型: {:?}，但暂不支持分析", file_type);
                    Ok(())
                }
                Err(e) => {
                    println!("无法检测文件类型: {}", e);
                    Err(Box::new(e))
                }
            }
        }
    }
}

/// 批量分析目录中的 COFF 和库文件
pub fn analyze_directory<P: AsRef<Path>>(dir_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let dir_path = dir_path.as_ref();
    println!("分析目录: {}", dir_path.display());

    let mut analyzed_count = 0;
    let mut error_count = 0;

    for entry in WalkDir::new(dir_path).max_depth(1) {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            match path.extension().and_then(|s| s.to_str()) {
                Some("obj") | Some("lib") | Some("a") => {
                    println!("\n{}", "=".repeat(60));
                    match analyze_file_auto(path) {
                        Ok(_) => analyzed_count += 1,
                        Err(e) => {
                            println!("分析失败: {}", e);
                            error_count += 1;
                        }
                    }
                }
                _ => {} // 跳过其他文件类型
            }
        }
    }

    println!("\n{}", "=".repeat(60));
    println!("分析完成: 成功 {} 个文件, 失败 {} 个文件", analyzed_count, error_count);

    Ok(())
}

/// 创建最小的 COFF 测试数据
pub fn create_minimal_coff_data() -> Vec<u8> {
    let mut data = Vec::new();

    // COFF 头部 (20 字节)
    data.extend_from_slice(&0x014cu16.to_le_bytes()); // machine (x86)
    data.extend_from_slice(&0u16.to_le_bytes()); // number_of_sections
    data.extend_from_slice(&0u32.to_le_bytes()); // time_date_stamp
    data.extend_from_slice(&0u32.to_le_bytes()); // pointer_to_symbol_table
    data.extend_from_slice(&0u32.to_le_bytes()); // number_of_symbols
    data.extend_from_slice(&0u16.to_le_bytes()); // size_of_optional_header
    data.extend_from_slice(&0u16.to_le_bytes()); // characteristics

    data
}

/// 创建最小的静态库测试数据
pub fn create_minimal_lib_data() -> Vec<u8> {
    let mut data = Vec::new();

    // 静态库签名
    data.extend_from_slice(b"!<arch>\n");

    // 第一个成员头 (60 字节)
    let mut member_header = [b' '; 60];
    member_header[58] = b'`';
    member_header[59] = b'\n';

    // 成员名称
    let name = b"test.obj/";
    member_header[..name.len()].copy_from_slice(name);

    data.extend_from_slice(&member_header);

    data
}

/// 打印分析结果的通用函数
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
        println!("✓ 成功解析");
        println!("  导出函数: {}", export_count);
        println!("  导入函数: {}", import_count);
        println!("  节数量: {}", section_count);
        if !sample_exports.is_empty() {
            println!("  示例导出: {:?}", sample_exports);
        }
    }
    else {
        println!("✗ 解析失败");
        if let Some(error) = error_message {
            println!("  错误: {}", error);
        }
    }
}
