//! ELF 汇编器测试套件
//!
//! 这个文件包含基本的测试入口，具体的测试分布在不同的模块中：
//! - hello_world_tests: Hello World ELF 文件生成测试
//! - exit_code_tests: Exit Code ELF 文件生成测试  
//! - execution_tests: ELF 文件实际执行测试

use std::{path::Path, process::Command};

// 测试模块
mod execution_tests;
mod exit_code_tests;
mod hello_world_tests;

#[cfg(target_os = "linux")]
mod linux;

// ELF 文件分析函数
fn analyze_elf_file(path: &Path) -> Result<(String, String), Box<dyn std::error::Error>> {
    let output = Command::new("readelf").args(["-h", path.to_str().unwrap()]).output()?;

    let info = String::from_utf8_lossy(&output.stdout).to_string();
    let program = format!("ELF file: {}", path.display());

    Ok((info, program))
}

// 打印 ELF 文件摘要信息
fn print_elf_summary(program: &str, info: &str) {
    println!("=== {} ===", program);
    println!("{}", info);
}

#[test]
fn ready() {
    println!("ELF Assembler test suite ready!");
}
