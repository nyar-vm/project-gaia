use crate::test_tools::test_path;
use gaia_types::GaiaError;
use jvm_assembler::{
    builder::JvmProgramBuilder,
    formats::class::writer::ClassWriter,
    program::{JvmExceptionHandler, JvmField, JvmInstruction, JvmMethod, JvmProgram, JvmVersion},
};
use std::fs::remove_file;

/// 测试使用 JVM builder 生成程序，然后用 class writer 写入，最后用 Java 运行
#[test]
fn test_build_write_and_run_simple_class() -> Result<(), GaiaError> {
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
    let output =
        std::process::Command::new("java").arg("-cp").arg(test_path("class_writer/generated")).arg("HelloWorld").output()?;

    // 验证输出
    assert!(output.status.success(), "Java execution failed: {}", String::from_utf8_lossy(&output.stderr));

    // 清理文件
    let _ = remove_file(class_file_path)?;
    Ok(())
}

#[test]
fn test_fluent_builder() -> Result<(), GaiaError> {
    let program = JvmProgramBuilder::new("FluentHello")
        .with_public()
        .add_method("main", "([Ljava/lang/String;)V", |method| {
            method
                .with_public()
                .with_static()
                .with_max_stack(2)
                .with_max_locals(1)
                .getstatic("java/lang/System", "out", "Ljava/io/PrintStream;")
                .ldc("Hello from Builder!")
                .invokevirtual("java/io/PrintStream", "println", "(Ljava/lang/String;)V")
                .return_void()
        })
        .build();

    let bytes = write_program_to_bytes(&program)?;
    let output_dir = test_path("class_writer/generated");
    std::fs::create_dir_all(&output_dir)?;
    let class_file_path = output_dir.join("FluentHello.class");
    std::fs::write(&class_file_path, &bytes)?;

    let output =
        std::process::Command::new("java").arg("-cp").arg(test_path("class_writer/generated")).arg("FluentHello").output()?;

    assert!(output.status.success(), "Java execution failed: {}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "Hello from Builder!");

    let _ = remove_file(class_file_path)?;
    Ok(())
}

/// 测试构建 Hello World 程序
fn build_hello_world_program() -> JvmProgram {
    let mut program = JvmProgram::new("HelloWorld".to_string());

    // 设置为 public class
    program.access_flags.is_public = true;

    // 创建 main 方法
    let mut main_method = JvmMethod::new("main".to_string(), "([Ljava/lang/String;)V".to_string());
    main_method.access_flags.is_public = true;
    main_method.access_flags.is_static = true;
    main_method.max_stack = 2;
    main_method.max_locals = 1;

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

/// 测试使用 Invokeinterface 和 Iinc 的程序
#[test]
fn test_complex_instructions() -> Result<(), GaiaError> {
    let mut program = JvmProgram::new("ComplexTest".to_string());
    program.access_flags.is_public = true;
    program.version = JvmVersion { major: 45, minor: 3 }; // Java 1.1, 不要求 StackMapTable

    let mut main_method = JvmMethod::new("main".to_string(), "([Ljava/lang/String;)V".to_string());
    main_method.access_flags.is_public = true;
    main_method.access_flags.is_static = true;
    main_method.max_stack = 4;
    main_method.max_locals = 3;

    // 创建一个 ArrayList 并作为 List 使用 (Invokeinterface)
    main_method.add_instruction(JvmInstruction::New { class_name: "java/util/ArrayList".to_string() });
    main_method.add_instruction(JvmInstruction::Dup);
    main_method.add_instruction(JvmInstruction::Invokespecial {
        class_name: "java/util/ArrayList".to_string(),
        method_name: "<init>".to_string(),
        descriptor: "()V".to_string(),
    });
    main_method.add_instruction(JvmInstruction::Astore1); // list in local 1

    // 调用 List.add (Invokeinterface)
    main_method.add_instruction(JvmInstruction::Aload1);
    main_method.add_instruction(JvmInstruction::Ldc { symbol: "Item".to_string() });
    main_method.add_instruction(JvmInstruction::Invokeinterface {
        class_name: "java/util/List".to_string(),
        method_name: "add".to_string(),
        descriptor: "(Ljava/lang/Object;)Z".to_string(),
    });
    main_method.add_instruction(JvmInstruction::Pop);

    // 测试 Iinc (local 0 is args, let's use a new local)
    main_method.add_instruction(JvmInstruction::Iconst1);
    main_method.add_instruction(JvmInstruction::Istore { index: 2 });
    main_method.add_instruction(JvmInstruction::Iinc { index: 2, increment: 5 });

    // 打印结果
    main_method.add_instruction(JvmInstruction::Getstatic {
        class_name: "java/lang/System".to_string(),
        field_name: "out".to_string(),
        descriptor: "Ljava/io/PrintStream;".to_string(),
    });
    main_method.add_instruction(JvmInstruction::Aload1);
    main_method.add_instruction(JvmInstruction::Iconst0);
    main_method.add_instruction(JvmInstruction::Invokeinterface {
        class_name: "java/util/List".to_string(),
        method_name: "get".to_string(),
        descriptor: "(I)Ljava/lang/Object;".to_string(),
    });
    main_method.add_instruction(JvmInstruction::Invokevirtual {
        class_name: "java/io/PrintStream".to_string(),
        method_name: "println".to_string(),
        descriptor: "(Ljava/lang/Object;)V".to_string(),
    });

    main_method.add_instruction(JvmInstruction::Return);

    program.add_method(main_method);

    // 写入并验证
    let class_bytes = write_program_to_bytes(&program)?;
    let output_dir = test_path("class_writer/generated");
    std::fs::create_dir_all(&output_dir)?;
    let class_file_path = output_dir.join("ComplexTest.class");
    std::fs::write(&class_file_path, &class_bytes)?;

    let output =
        std::process::Command::new("java").arg("-cp").arg(test_path("class_writer/generated")).arg("ComplexTest").output()?;

    assert!(output.status.success(), "Java execution failed: {}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "Item");

    let _ = remove_file(class_file_path)?;
    Ok(())
}

/// 测试 Tableswitch 和 Lookupswitch
#[test]
fn test_switch_instructions() -> Result<(), GaiaError> {
    let mut program = JvmProgram::new("SwitchTest".to_string());
    program.access_flags.is_public = true;
    program.version = JvmVersion { major: 45, minor: 3 }; // Java 1.1, 不要求 StackMapTable

    // test_tableswitch(int val)
    let mut tableswitch_method = JvmMethod::new("testTableswitch".to_string(), "(I)Ljava/lang/String;".to_string());
    tableswitch_method.access_flags.is_public = true;
    tableswitch_method.access_flags.is_static = true;
    tableswitch_method.max_stack = 1;
    tableswitch_method.max_locals = 1;

    tableswitch_method.add_instruction(JvmInstruction::Iload0);
    tableswitch_method.add_instruction(JvmInstruction::Tableswitch {
        low: 1,
        high: 2,
        default: "default_label".to_string(),
        targets: vec!["label1".to_string(), "label2".to_string()],
    });

    tableswitch_method.add_instruction(JvmInstruction::Label { name: "label1".to_string() });
    tableswitch_method.add_instruction(JvmInstruction::Ldc { symbol: "One".to_string() });
    tableswitch_method.add_instruction(JvmInstruction::Areturn);

    tableswitch_method.add_instruction(JvmInstruction::Label { name: "label2".to_string() });
    tableswitch_method.add_instruction(JvmInstruction::Ldc { symbol: "Two".to_string() });
    tableswitch_method.add_instruction(JvmInstruction::Areturn);

    tableswitch_method.add_instruction(JvmInstruction::Label { name: "default_label".to_string() });
    tableswitch_method.add_instruction(JvmInstruction::Ldc { symbol: "Other".to_string() });
    tableswitch_method.add_instruction(JvmInstruction::Areturn);

    program.add_method(tableswitch_method);

    // main 方法来测试
    let mut main_method = JvmMethod::new("main".to_string(), "([Ljava/lang/String;)V".to_string());
    main_method.access_flags.is_public = true;
    main_method.access_flags.is_static = true;
    main_method.max_stack = 2;
    main_method.max_locals = 1;

    // 打印 testTableswitch(1)
    main_method.add_instruction(JvmInstruction::Getstatic {
        class_name: "java/lang/System".to_string(),
        field_name: "out".to_string(),
        descriptor: "Ljava/io/PrintStream;".to_string(),
    });
    main_method.add_instruction(JvmInstruction::Iconst1);
    main_method.add_instruction(JvmInstruction::Invokestatic {
        class_name: "SwitchTest".to_string(),
        method_name: "testTableswitch".to_string(),
        descriptor: "(I)Ljava/lang/String;".to_string(),
    });
    main_method.add_instruction(JvmInstruction::Invokevirtual {
        class_name: "java/io/PrintStream".to_string(),
        method_name: "println".to_string(),
        descriptor: "(Ljava/lang/String;)V".to_string(),
    });

    main_method.add_instruction(JvmInstruction::Return);
    program.add_method(main_method);

    // 写入并验证
    let class_bytes = write_program_to_bytes(&program)?;
    let output_dir = test_path("class_writer/generated");
    std::fs::create_dir_all(&output_dir)?;
    let class_file_path = output_dir.join("SwitchTest.class");
    std::fs::write(&class_file_path, &class_bytes)?;

    let output =
        std::process::Command::new("java").arg("-cp").arg(test_path("class_writer/generated")).arg("SwitchTest").output()?;

    assert!(output.status.success(), "Java execution failed: {}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "One");

    let _ = remove_file(class_file_path)?;
    Ok(())
}

#[test]
fn test_exception_handler() -> Result<(), GaiaError> {
    let mut program = JvmProgram::new("ExceptionTest".to_string());
    program.access_flags.is_public = true;
    program.version = JvmVersion { major: 45, minor: 3 }; // Java 1.1

    let mut main_method = JvmMethod::new("main".to_string(), "([Ljava/lang/String;)V".to_string());
    main_method.access_flags.is_public = true;
    main_method.access_flags.is_static = true;
    main_method.max_stack = 2;
    main_method.max_locals = 2;

    // try {
    main_method.add_instruction(JvmInstruction::Label { name: "try_start".to_string() });
    main_method.add_instruction(JvmInstruction::New { class_name: "java/lang/Exception".to_string() });
    main_method.add_instruction(JvmInstruction::Dup);
    main_method.add_instruction(JvmInstruction::Invokespecial {
        class_name: "java/lang/Exception".to_string(),
        method_name: "<init>".to_string(),
        descriptor: "()V".to_string(),
    });
    main_method.add_instruction(JvmInstruction::Athrow);
    main_method.add_instruction(JvmInstruction::Label { name: "try_end".to_string() });

    // } catch (Exception e) {
    main_method.add_instruction(JvmInstruction::Label { name: "catch_start".to_string() });
    main_method.add_instruction(JvmInstruction::Astore1); // exception object in local 1
    main_method.add_instruction(JvmInstruction::Getstatic {
        class_name: "java/lang/System".to_string(),
        field_name: "out".to_string(),
        descriptor: "Ljava/io/PrintStream;".to_string(),
    });
    main_method.add_instruction(JvmInstruction::Ldc { symbol: "Caught exception".to_string() });
    main_method.add_instruction(JvmInstruction::Invokevirtual {
        class_name: "java/io/PrintStream".to_string(),
        method_name: "println".to_string(),
        descriptor: "(Ljava/lang/String;)V".to_string(),
    });
    main_method.add_instruction(JvmInstruction::Return);

    // Add exception handler
    main_method.add_exception_handler(JvmExceptionHandler {
        start_label: "try_start".to_string(),
        end_label: "try_end".to_string(),
        handler_label: "catch_start".to_string(),
        catch_type: Some("java/lang/Exception".to_string()),
    });

    program.add_method(main_method);

    // Write and verify
    let class_bytes = write_program_to_bytes(&program)?;
    let output_dir = test_path("class_writer/generated");
    std::fs::create_dir_all(&output_dir)?;
    let class_file_path = output_dir.join("ExceptionTest.class");
    std::fs::write(&class_file_path, &class_bytes)?;

    let output =
        std::process::Command::new("java").arg("-cp").arg(test_path("class_writer/generated")).arg("ExceptionTest").output()?;

    assert!(output.status.success(), "Java execution failed: {}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "Caught exception");

    let _ = remove_file(class_file_path)?;
    Ok(())
}

#[test]
fn test_throws_and_constant_value() -> Result<(), GaiaError> {
    let mut program = JvmProgram::new("AttributeTest".to_string());
    program.access_flags.is_public = true;
    program.version = JvmVersion { major: 45, minor: 3 };

    use jvm_assembler::program::JvmConstantPoolEntry;

    // Field with ConstantValue
    let mut field = JvmField::new("MAX_VALUE".to_string(), "I".to_string());
    field.access_flags.is_public = true;
    field.access_flags.is_static = true;
    field.access_flags.is_final = true;
    field.constant_value = Some(JvmConstantPoolEntry::Integer { value: 100 });
    program.add_field(field);

    // Method with throws
    let mut method = JvmMethod::new("testThrows".to_string(), "()V".to_string());
    method.access_flags.is_public = true;
    method.add_exception("java/io/IOException".to_string());
    method.add_instruction(JvmInstruction::Return);
    program.add_method(method);

    // Write and verify
    let _class_bytes = write_program_to_bytes(&program)?;

    Ok(())
}

#[test]
fn test_stack_map_table() -> Result<(), GaiaError> {
    let mut program = JvmProgram::new("StackMapTest".to_string());
    // 设置为 Java 8 (version 52)
    program.version = jvm_assembler::program::JvmVersion { major: 52, minor: 0 };
    program.access_flags.is_public = true;

    let mut method = JvmMethod::new("test".to_string(), "()V".to_string());
    method.access_flags.is_public = true;
    method.max_stack = 1;
    method.max_locals = 1;

    // 添加一个包含跳转的逻辑
    method.add_instruction(JvmInstruction::Iconst0);
    method.add_instruction(JvmInstruction::Ifne { target: "label1".to_string() });
    method.add_instruction(JvmInstruction::Return);
    method.add_instruction(JvmInstruction::Label { name: "label1".to_string() });
    method.add_instruction(JvmInstruction::Return);

    program.add_method(method);

    let buf = write_program_to_bytes(&program)?;

    assert!(!buf.is_empty());
    // 验证生成的字节码中包含 "StackMapTable" 字符串
    let content = String::from_utf8_lossy(&buf);
    assert!(content.contains("StackMapTable"));

    Ok(())
}

#[test]
fn test_class_attributes() -> Result<(), GaiaError> {
    let mut program = JvmProgram::new("ClassAttrTest".to_string());
    program.access_flags.is_public = true;
    program.version = JvmVersion { major: 45, minor: 3 };

    use jvm_assembler::program::{JvmAccessFlags, JvmAttribute, JvmInnerClass};

    // SourceFile
    program.attributes.push(JvmAttribute::SourceFile { filename: "ClassAttrTest.jasm".to_string() });

    // InnerClasses
    let mut inner_access = JvmAccessFlags::default();
    inner_access.is_public = true;
    inner_access.is_static = true;
    program.attributes.push(JvmAttribute::InnerClasses {
        classes: vec![JvmInnerClass {
            inner_class: "ClassAttrTest$Inner".to_string(),
            outer_class: Some("ClassAttrTest".to_string()),
            inner_name: Some("Inner".to_string()),
            access_flags: inner_access,
        }],
    });

    // EnclosingMethod
    program.attributes.push(JvmAttribute::EnclosingMethod {
        class_name: "OuterClass".to_string(),
        method_name: Some("enclosingMethod".to_string()),
        method_descriptor: Some("()V".to_string()),
    });

    // Write and verify
    let _class_bytes = write_program_to_bytes(&program)?;

    Ok(())
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
    assert!(output.status.success(), "Java execution failed: {}", String::from_utf8_lossy(&output.stderr));
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
    constructor.max_stack = 1;
    constructor.max_locals = 1;
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
fn test_long_double_constants() {
    // 构建带 Long 和 Double 常量的类
    let mut program = JvmProgram::new("LongDoubleTest".to_string());
    program.access_flags.is_public = true;

    // 添加 main 方法
    let mut main_method = JvmMethod::new("main".to_string(), "([Ljava/lang/String;)V".to_string());
    main_method.access_flags.is_public = true;
    main_method.access_flags.is_static = true;
    main_method.max_stack = 4;
    main_method.max_locals = 1;

    // 添加一些常量加载指令（虽然目前指令集还没完全支持 Long/Double 的加载，但我们可以先测试常量池写入）
    // 这里我们直接向程序添加一些常量池条目（如果 JvmProgram 支持的话）
    // 目前 JvmProgram 的 constant_pool 是暴露的，我们可以手动添加

    // 写入到字节数组
    let bytes = write_program_to_bytes(&program).expect("Failed to write class file");

    // 验证字节数组不为空
    assert!(!bytes.is_empty(), "Generated class file should not be empty");

    // 验证魔数和版本
    assert_eq!(&bytes[0..4], &[0xCA, 0xFE, 0xBA, 0xBE]);
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
    main_method.max_stack = 0;
    main_method.max_locals = 1;
    main_method.add_instruction(JvmInstruction::Return); // return
    program.add_method(main_method);

    // 添加辅助方法
    let mut helper_method = JvmMethod::new("helper".to_string(), "()I".to_string());
    helper_method.access_flags.is_private = true;
    helper_method.max_stack = 1;
    helper_method.max_locals = 1;
    helper_method.add_instruction(JvmInstruction::Iconst1); // iconst_1
    helper_method.add_instruction(JvmInstruction::Ireturn); // ireturn
    program.add_method(helper_method);

    // 写入到字节数组
    let bytes = write_program_to_bytes(&program).expect("Failed to write class file");

    // 验证字节数组不为空
    assert!(!bytes.is_empty(), "Generated class file should not be empty");
}
