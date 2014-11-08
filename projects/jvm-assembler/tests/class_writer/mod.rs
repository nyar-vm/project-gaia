use jvm_assembler::{
    self, convert_jasm_to_jvm,
    formats::{
        class::writer::ClassWriter,
        jasm::{lexer::JasmLexer, parser::JasmParser},
    },
};
use std::{fs, path::Path};

#[test]
fn hello_java_class_generation() {
    // 读取 HelloJava.jasm 文件
    let jasm_path = Path::new("tests/jasm_reader/HelloJava/HelloJava.jasm");
    let jasm_content = fs::read_to_string(jasm_path).expect("Failed to read HelloJava.jasm file");

    // 步骤 1: 词法分析
    let lexer = JasmLexer::new();
    let token_stream_result = lexer.tokenize(&jasm_content);
    assert!(token_stream_result.result.is_ok(), "Failed to tokenize JASM content");
    let token_stream = token_stream_result.result.unwrap();

    // 步骤 2: 语法分析
    let parser = JasmParser::new(None);
    let ast = parser.parse(token_stream).expect("Failed to parse JASM tokens");

    // 步骤 3: 编译为 JVM Program
    let program_result = convert_jasm_to_jvm(ast);
    assert!(program_result.result.is_ok(), "Failed to compile AST to JVM Program");
    let program = program_result.result.unwrap();

    // 步骤 4: 生成 Class 字节码
    let binary_assembler: gaia_types::BinaryAssembler<Vec<u8>, byteorder::BigEndian> =
        gaia_types::BinaryAssembler::new(Vec::new());
    let mut class_writer = ClassWriter::new(binary_assembler);
    let class_bytes_result = class_writer.write(program);

    // 检查结果
    assert!(class_bytes_result.result.is_ok(), "Failed to generate Class bytes");

    let class_bytes = class_bytes_result.result.unwrap();

    // 验证 Class 魔数
    assert_eq!(&class_bytes[0..4], &[0xCA, 0xFE, 0xBA, 0xBE], "Invalid Class magic number");

    // 写入输出文件用于调试和验证
    let output_path = Path::new("tests/class_writer/HelloJava.class");
    fs::create_dir_all(output_path.parent().unwrap()).expect("Failed to create output directory");
    fs::write(output_path, &class_bytes).expect("Failed to write output Class file");

    println!("Successfully compiled HelloJava.jasm to HelloJava.class");
    println!("Generated {} bytes of Class bytecode", class_bytes.len());

    // 验证生成的 Class 文件可以被 Java 运行时识别
    // 注意：这里只是基本的格式验证，实际运行需要完整的常量池索引
    assert!(class_bytes.len() > 10, "Class file too small");
}

#[test]
fn verify_class_file_structure() {
    // 这个测试验证生成的 Class 文件的基本结构
    let class_path = Path::new("tests/class_writer/HelloJava.class");

    if class_path.exists() {
        let class_bytes = fs::read(class_path).expect("Failed to read generated Class file");

        // 验证魔数
        assert_eq!(&class_bytes[0..4], &[0xCA, 0xFE, 0xBA, 0xBE]);

        // 验证版本号（应该是合理的 Java 版本）
        let minor_version = u16::from_be_bytes([class_bytes[4], class_bytes[5]]);
        let major_version = u16::from_be_bytes([class_bytes[6], class_bytes[7]]);

        println!("Class file version: {}.{}", major_version, minor_version);
        assert!(major_version >= 45, "Java version too old"); // Java 1.1+
        assert!(major_version <= 70, "Java version too new"); // 合理的上限

        // 验证常量池大小
        let constant_pool_count = u16::from_be_bytes([class_bytes[8], class_bytes[9]]);
        println!("Constant pool count: {}", constant_pool_count);
        assert!(constant_pool_count > 0, "Constant pool should not be empty");

        println!("Class file structure validation passed");
    }
    else {
        panic!("HelloJava.class not found. Run hello_java_class_generation test first.");
    }
}
