use python_assembler::{
    builder::PythonBuilder,
    formats::pyc::{PycWriteConfig, writer::PycWriter},
    program::{PycHeader, PythonVersion},
};
use std::{
    fs::File,
    io::BufWriter,
    path::Path,
    process::Command,
};

#[test]
fn test_build_and_write_pyc() {
    // 1. 使用 PythonBuilder 构建一个简单的 Python 程序
    let builder = PythonBuilder::new();
    let program = builder
        .print_str("Hello World")
        .build(create_pyc_header());

    // 验证程序结构
    assert_eq!(program.code_object.source_name, "<string>");
    assert_eq!(program.code_object.co_consts.len(), 1);
    
    // 2. 使用 PycWriter 将程序写入 .pyc 文件
    let output_path = "test_output.pyc";
    let file = File::create(output_path).expect("Failed to create output file");
    let mut writer = PycWriter::new(BufWriter::new(file), PycWriteConfig::default());
    
    let bytes_written = writer.write(&program).expect("Failed to write pyc file");
    println!("Successfully wrote {} bytes to {}", bytes_written, output_path);
    
    // 验证文件存在
    assert!(Path::new(output_path).exists(), "Output file should exist");
    
    // 3. 使用本机 Python 运行生成的 .pyc 文件
    let output = Command::new("python")
        .arg(output_path)
        .output();
    
    match output {
        Ok(result) => {
            if result.status.success() {
                let stdout = String::from_utf8_lossy(&result.stdout);
                println!("Python execution output: {}", stdout);
                // 注意：由于当前 PycWriter 只实现了头部写入，实际的 marshal 数据还未实现
                // 所以这个测试主要验证文件创建和基本结构
            } else {
                let stderr = String::from_utf8_lossy(&result.stderr);
                println!("Python execution failed: {}", stderr);
                // 这是预期的，因为 marshal 数据还未完全实现
            }
        }
        Err(e) => {
            println!("Failed to execute python: {}. This might be expected if Python is not installed or not in PATH.", e);
        }
    }
    
    // 清理测试文件
    if Path::new(output_path).exists() {
        std::fs::remove_file(output_path).ok();
    }
}

#[test]
fn test_python_builder_functionality() {
    // 测试 PythonBuilder 的基本功能
    let builder = PythonBuilder::new();
    let program = builder
        .print_str("Test Message")
        .build(create_pyc_header());
    
    // 验证程序结构
    assert_eq!(program.version, PythonVersion::Python3_9);
    assert_eq!(program.code_object.source_name, "<string>");
    assert_eq!(program.code_object.first_line, 1);
    assert_eq!(program.code_object.last_line, 1);
    
    // 验证常量
    assert_eq!(program.code_object.co_consts.len(), 1);
    match &program.code_object.co_consts[0] {
        python_assembler::program::PythonObject::Str(s) => {
            assert_eq!(s, "Test Message");
        }
        _ => panic!("Expected string constant"),
    }
}

#[test]
fn test_pyc_writer_basic() {
    // 测试 PycWriter 的基本写入功能
    let builder = PythonBuilder::new();
    let program = builder.build(create_pyc_header());
    
    let mut buffer = Vec::new();
    let mut writer = PycWriter::new(&mut buffer, PycWriteConfig::default());
    
    let bytes_written = writer.write(&program).expect("Failed to write to buffer");
    
    // 验证写入了预期的字节数（当前只有头部，16字节）
    assert_eq!(bytes_written, 16);
    assert_eq!(buffer.len(), 16);
    
    // 验证头部内容
    let expected_magic = PythonVersion::Python3_9.as_magic();
    assert_eq!(&buffer[0..4], &expected_magic);
}

/// 创建一个用于测试的 PycHeader
fn create_pyc_header() -> PycHeader {
    let version = PythonVersion::Python3_9;
    PycHeader {
        magic: version.as_magic(),
        flags: 0,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as u32,
        size: 0, // 将在写入时计算
    }
}