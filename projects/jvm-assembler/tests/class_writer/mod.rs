use std::fs::remove_file;
use gaia_types::GaiaError;
use crate::test_tools::test_path;
use jvm_assembler::{
    formats::class::writer::ClassWriter,
    program::{JvmField, JvmInstruction, JvmMethod, JvmProgram},
};

/// 测试使用 JVM builder 生成程序，然后用 class writer 写入，最后用 Java 运行
#[test]
fn test_build_write_and_run_simple_class() -> Result<(), GaiaError>{
    // 1. 使用 JVM builder 创建一个简单的 Hello World 程序
    let program = build_hello_world_program();

    // 2. 使用 class writer 写入 .class 文件// 写入到字节数组
    let class_bytes = write_program_to_bytes(&program)?;

    // 3. 保存到文件
    let output_dir = test_path("class_writer/generated");
    std::fs::create_dir_all(&output_dir)?;
    let class_file_path = output_dir.join("HelloWorld.class");
    std::fs::write(&class_file_path, &class_bytes)?;

    // 4. 使用 Java 运行验证
    let output = std::process::Command::new("java")
        .arg("-cp")
        .arg(test_path("class_writer/generated"))
        .arg("HelloWorld")
        .output()?;

    // 验证输出
    assert!(output.status.success(), "Java execution failed: {}", String::from_utf8_lossy(&output.stderr));

    // 清理文件
    let _ = remove_file(class_file_path)?;

    Ok(())
}

/// 构建一个简单的 Hello World JVM 程序
fn build_hello_world_program() -> JvmProgram {
    let mut program = JvmProgram::new("HelloWorld".to_string());

    // 设置为 public class
    program.access_flags.is_public = true;

    // 创建 main 方法
    let mut main_method = JvmMethod::new("main".to_string(), "([Ljava/lang/String;)V".to_string());
    main_method.access_flags.is_public = true;
    main_method.access_flags.is_static = true;

    // 添加指令
    main_method.add_instruction(JvmInstruction::Getstatic {
        class_name: "java/lang/System".to_string(),
        field_name: "out".to_string(),
        descriptor: "Ljava/io/PrintStream;".to_string(),
    });
    main_method.add_instruction(JvmInstruction::Ldc { symbol: "Hello, World!".to_string() });
    main_method.add_instruction(JvmInstruction::Invokevirtual {
        class_name: "java/io/PrintStream".to_string(),
        method_name: "println".to_string(),
        descriptor: "(Ljava/lang/String;)V".to_string(),
    });
    main_method.add_instruction(JvmInstruction::Return);

    program.add_method(main_method);

    program
}

/// 将程序写入字节数组
fn write_program_to_bytes(program: &JvmProgram) -> gaia_types::Result<Vec<u8>> {
    let buffer = Vec::new();
    let writer = ClassWriter::new(buffer);
    let result = writer.write(program);

    match result.result {
        Ok(bytes) => Ok(bytes),
        Err(error) => Err(error),
    }
}

/// 测试构建 Hello World 程序
#[test]
fn test_hello_world_program() {
    // 构建 Hello World 程序
    let program = build_hello_world_program();

    // 写入到字节数组
    let bytes = write_program_to_bytes(&program).expect("Failed to write class file");

    // 保存到文件
    let output_dir = test_path("class_writer/generated");
    std::fs::create_dir_all(&output_dir).expect("Failed to create output directory");
    let class_file_path = output_dir.join("HelloWorld.class");
    std::fs::write(&class_file_path, &bytes).expect("Failed to write class file");

    // 使用 Java 运行
    let output = std::process::Command::new("java")
        .arg("-cp")
        .arg(test_path("class_writer/generated"))
        .arg("HelloWorld")
        .output()
        .expect("Failed to execute java command");

    // 验证输出
    assert!(output.status.success(), "Java execution failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Hello, World!"), "Expected output not found");
}

#[test]
fn test_class_with_fields() {
    // 构建带字段的类
    let mut program = JvmProgram::new("TestClass".to_string());
    program.access_flags.is_public = true;

    // 添加字段
    let mut field = JvmField::new("count".to_string(), "I".to_string());
    field.access_flags.is_private = true;
    program.add_field(field);

    // 添加构造函数
    let mut constructor = JvmMethod::new("<init>".to_string(), "()V".to_string());
    constructor.access_flags.is_public = true;
    constructor.add_instruction(JvmInstruction::Aload { index: 0 }); // aload_0
    constructor.add_instruction(JvmInstruction::Invokespecial {
        class_name: "java/lang/Object".to_string(),
        method_name: "<init>".to_string(),
        descriptor: "()V".to_string(),
    }); // invokespecial Object.<init>
    constructor.add_instruction(JvmInstruction::Return); // return
    program.add_method(constructor);

    // 写入到字节数组
    let bytes = write_program_to_bytes(&program).expect("Failed to write class file");

    // 验证字节数组不为空
    assert!(!bytes.is_empty(), "Generated class file should not be empty");
}

#[test]
fn test_multiple_methods() {
    // 构建带多个方法的类
    let mut program = JvmProgram::new("MultiMethodClass".to_string());
    program.access_flags.is_public = true;

    // 添加 main 方法
    let mut main_method = JvmMethod::new("main".to_string(), "([Ljava/lang/String;)V".to_string());
    main_method.access_flags.is_public = true;
    main_method.access_flags.is_static = true;
    main_method.add_instruction(JvmInstruction::Return); // return
    program.add_method(main_method);

    // 添加辅助方法
    let mut helper_method = JvmMethod::new("helper".to_string(), "()I".to_string());
    helper_method.access_flags.is_private = true;
    helper_method.add_instruction(JvmInstruction::Iconst1); // iconst_1
    helper_method.add_instruction(JvmInstruction::Ireturn); // ireturn
    program.add_method(helper_method);

    // 写入到字节数组
    let bytes = write_program_to_bytes(&program).expect("Failed to write class file");

    // 验证字节数组不为空
    assert!(!bytes.is_empty(), "Generated class file should not be empty");
}
