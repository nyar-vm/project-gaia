//! PE 汇编器测试套件
//!
//! 这个文件包含基本的测试入口，具体的测试分布在不同的模块中：
//! - hello_world_tests: Hello World PE 文件生成测试
//! - exit_code_tests: Exit Code PE 文件生成测试  
//! - execution_tests: PE 文件实际执行测试

use std::{path::Path, process::Command};

// 测试模块
mod execution_tests;
mod exit_code_tests;
mod hello_world_tests;

#[cfg(target_os = "windows")]
mod windows;

// PE 文件分析函数
fn analyze_pe_file(path: &Path) -> Result<(String, String), Box<dyn std::error::Error>> {
    let output = Command::new("dumpbin").args(["/headers", path.to_str().unwrap()]).output()?;

    let info = String::from_utf8_lossy(&output.stdout).to_string();
    let program = format!("PE file: {}", path.display());

    Ok((info, program))
}

// 打印 PE 文件摘要信息
fn print_pe_summary(program: &str, info: &str) {
    println!("=== {} ===", program);
    println!("{}", info);
}

#[test]
fn ready() {
    println!("PE Assembler test suite ready!");
}
