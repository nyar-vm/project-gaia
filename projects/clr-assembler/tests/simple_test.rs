//! 简单的 DotNetWriter 测试

use clr_assembler::{
    formats::dll::writer::DotNetWriter,
    program::{ClrInstruction, ClrMethod, ClrOpcode, ClrProgram, ClrTypeReference},
};
use std::io::Cursor;

#[test]
fn test_dot_net_writer_basic() {
    // 创建一个简单的 CLR 程序
    let mut program = ClrProgram::new("TestAssembly");

    // 创建 void 返回类型
    let void_type = ClrTypeReference {
        name: "Void".to_string(),
        namespace: Some("System".to_string()),
        assembly: Some("mscorlib".to_string()),
        is_value_type: true,
        is_reference_type: false,
        generic_parameters: Vec::new(),
    };

    // 添加一个简单的方法
    let mut method = ClrMethod::new("Main".to_string(), void_type);
    method.is_entry_point = true;

    // 添加一些简单的指令
    method.instructions.push(ClrInstruction::Simple { opcode: ClrOpcode::Nop });
    method.instructions.push(ClrInstruction::Simple { opcode: ClrOpcode::Ret });

    program.global_methods.push(method);

    // 使用 DotNetWriter 生成 PE 文件
    let buffer = Cursor::new(Vec::new());
    let writer = DotNetWriter::new(buffer);
    let result = writer.write(&program);

    // 验证结果
    assert!(result.result.is_ok(), "DotNetWriter 应该能够成功生成 PE 文件");

    let pe_writer = result.result.unwrap();
    let pe_bytes = pe_writer.into_inner();
    assert!(!pe_bytes.is_empty(), "生成的 PE 文件不应该为空");

    // 验证 DOS 头
    assert_eq!(&pe_bytes[0..2], b"MZ", "应该有正确的 DOS 签名");

    println!("✓ DotNetWriter 基本功能测试通过");
    println!("生成的 PE 文件大小: {} 字节", pe_bytes.len());
}

#[test]
fn test_dot_net_writer_with_string() {
    // 创建包含字符串操作的程序
    let mut program = ClrProgram::new("StringTestAssembly");

    // 创建 void 返回类型
    let void_type = ClrTypeReference {
        name: "Void".to_string(),
        namespace: Some("System".to_string()),
        assembly: Some("mscorlib".to_string()),
        is_value_type: true,
        is_reference_type: false,
        generic_parameters: Vec::new(),
    };

    let mut method = ClrMethod::new("PrintHello".to_string(), void_type);
    method.is_entry_point = true;

    // 添加字符串加载指令
    method.instructions.push(ClrInstruction::WithString { opcode: ClrOpcode::Ldstr, value: "Hello, World!".to_string() });
    method.instructions.push(ClrInstruction::Simple { opcode: ClrOpcode::Ret });

    program.global_methods.push(method);

    let buffer = Cursor::new(Vec::new());
    let writer = DotNetWriter::new(buffer);
    let result = writer.write(&program);

    assert!(result.result.is_ok(), "包含字符串的程序应该能够成功生成");

    let pe_writer = result.result.unwrap();
    let pe_bytes = pe_writer.into_inner();
    assert!(!pe_bytes.is_empty(), "生成的 PE 文件不应该为空");

    println!("✓ DotNetWriter 字符串处理测试通过");
}
