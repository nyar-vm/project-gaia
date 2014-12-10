use python_assembler::{
    builder::PythonBuilder,
    formats::pyc::{writer::PycWriter, PycWriteConfig},
    program::{PycHeader, PythonVersion},
};
use std::{fs::File, io::BufWriter, path::Path, process::Command};

#[test]
fn test_build_and_write_pyc() {
    // 1. 使用 PythonBuilder 构建一个简单的 Python 程序
    let builder = PythonBuilder::new();
    let program = builder.print_str("Hello World").build(create_pyc_header());

    // 验证程序结构
    assert_eq!(program.code_object.source_name, "<string>");
    assert_eq!(program.code_object.co_consts.len(), 2); // [None, "Hello World"]

    // 2. 使用 PycWriter 将程序写入 .pyc 文件
    let output_path = "test_output.pyc";
    let file = File::create(output_path).expect("Failed to create output file");
    let mut writer = PycWriter::new(BufWriter::new(file), PycWriteConfig::default());

    let bytes_written = writer.write(&program).expect("Failed to write pyc file");
    println!("Successfully wrote {} bytes to {}", bytes_written, output_path);

    // 验证文件存在
    assert!(Path::new(output_path).exists(), "Output file should exist");

    // 3. 使用本机 Python 运行生成的 .pyc 文件
    let output = Command::new("python").arg(output_path).output();

    match output {
        Ok(result) => {
            let stdout = String::from_utf8_lossy(&result.stdout);
            let stderr = String::from_utf8_lossy(&result.stderr);

            if result.status.success() {
                println!("Python execution output: {}", stdout);
                assert_eq!(stdout.trim(), "Hello World");
            }
            else {
                println!("Python execution failed: {}", stderr);
                panic!("Generated .pyc failed to run: {}", stderr);
            }
        }
        Err(e) => {
            println!("Failed to execute python: {}. Skipping execution test.", e);
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
    let program = builder.print_str("Test Message").build(create_pyc_header());

    // 验证程序结构
    assert_eq!(program.version, PythonVersion::Python3_12);
    assert_eq!(program.code_object.source_name, "<string>");
    assert_eq!(program.code_object.first_line, 1);
    assert_eq!(program.code_object.last_line, 1);

    // 验证常量
    assert_eq!(program.code_object.co_consts.len(), 2);
    match &program.code_object.co_consts[1] {
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

    // 验证写入了预期的字节数（头部 16 字节 + Marshal 数据）
    assert!(bytes_written > 16);
    assert_eq!(buffer.len(), bytes_written);

    // 验证头部内容
    let expected_magic = PythonVersion::Python3_12.as_magic();
    assert_eq!(&buffer[0..4], &expected_magic);
}

/// 创建一个用于测试的 PycHeader
fn create_pyc_header() -> PycHeader {
    let version = PythonVersion::Python3_12;
    PycHeader {
        magic: version.as_magic(),
        flags: 0,
        timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as u32,
        size: 0, // 将在写入时计算
    }
}
