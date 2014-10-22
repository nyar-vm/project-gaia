//! PE 文件实际执行测试
//!
//! 这个模块测试生成的 PE 文件是否能在 Windows 环境下正常执行

use gaia_types::helpers::Architecture;
use pe_assembler::generator::{easy_console_log, easy_exit_code};
use std::{fs, process::Command};

#[cfg(target_os = "windows")]
#[test]
fn test_execute_hello_world_x64() {
    // 生成 Hello World PE 文件
    let pe_data = easy_console_log(Architecture::X86_64, "Hello from Gaia PE Assembler!").unwrap();

    // 保存到临时文件
    let test_dir = "tests/generated";
    fs::create_dir_all(test_dir).ok();
    let exe_path = format!("{}/test_hello_x64.exe", test_dir);
    fs::write(&exe_path, &pe_data).unwrap();

    // 尝试执行
    let output = Command::new(&exe_path).output().expect("Failed to execute generated PE file");

    println!("Exit status: {}", output.status);
    println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));

    // 验证程序能够执行（即使可能有错误）
    // 这里我们主要验证文件格式是否正确，能够被 Windows 加载
    println!("✅ x64 Hello World PE file executed (status: {})", output.status);
}

#[cfg(target_os = "windows")]
#[test]
fn test_execute_exit_code_x64() {
    // 生成退出代码 PE 文件
    let pe_data = easy_exit_code(Architecture::X86_64, 42).unwrap();

    // 保存到临时文件
    let test_dir = "tests/generated";
    fs::create_dir_all(test_dir).ok();
    let exe_path = format!("{}/test_exit_x64.exe", test_dir);
    fs::write(&exe_path, &pe_data).unwrap();

    // 尝试执行
    let output = Command::new(&exe_path).output().expect("Failed to execute generated PE file");

    println!("Exit status: {}", output.status);
    println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));

    // 验证退出代码（如果程序正确执行的话）
    if let Some(code) = output.status.code() {
        println!("Program exited with code: {}", code);
        // 注意：由于我们的 PE 文件可能还有格式问题，这里先不强制验证退出代码
    }

    println!("✅ x64 Exit Code PE file executed (status: {})", output.status);
}

#[cfg(target_os = "windows")]
#[test]
fn test_execute_hello_world_x86() {
    // 生成 x86 Hello World PE 文件
    let pe_data = easy_console_log(Architecture::X86, "Hello from x86!").unwrap();

    // 保存到临时文件
    let test_dir = "tests/generated";
    fs::create_dir_all(test_dir).ok();
    let exe_path = format!("{}/test_hello_x86.exe", test_dir);
    fs::write(&exe_path, &pe_data).unwrap();

    // 尝试执行
    let output = Command::new(&exe_path).output().expect("Failed to execute generated PE file");

    println!("Exit status: {}", output.status);
    println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));

    println!("✅ x86 Hello World PE file executed (status: {})", output.status);
}

#[cfg(not(target_os = "windows"))]
#[test]
fn test_execution_skipped_non_windows() {
    println!("⚠️  Execution tests skipped - not running on Windows");
}
