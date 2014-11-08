use jvm_assembler::compile_jasm_to_class;
use std::{fs, path::Path};

pub mod easy_test;

#[test]
fn test_compile_hello_java() {
    let jasm_content = include_str!("formats/jasm/tests/HelloJava.jasm");

    // 编译 JASM 到 .class 文件
    let class_bytes = compile_jasm_to_class(jasm_content).expect("Failed to compile JASM");

    // 验证生成的字节码不为空
    assert!(!class_bytes.is_empty(), "Generated class bytes should not be empty");

    // 验证魔数
    assert_eq!(&class_bytes[0..4], &[0xCA, 0xFE, 0xBA, 0xBE], "Invalid magic number");

    // 将生成的字节码写入文件以便后续验证
    let output_path = "tests/output/HelloJava.class";
    if let Some(parent) = Path::new(output_path).parent() {
        fs::create_dir_all(parent).expect("Failed to create output directory");
    }
    fs::write(output_path, &class_bytes).expect("Failed to write class file");

    println!("Generated HelloJava.class with {} bytes", class_bytes.len());
}

#[test]
fn test_simple_class() {
    let jasm_content = r#"
public super class SimpleClass version 61:0
{
  public Method "<init>":"()V" 
    stack 1  locals 1
  {
         aload_0;
         invokespecial     Method java/lang/Object."<init>":"()V";
         return;
  }
}
"#;

    let class_bytes = compile_jasm_to_class(jasm_content).expect("Failed to compile simple class");

    // 验证基本结构
    assert!(!class_bytes.is_empty());
    assert_eq!(&class_bytes[0..4], &[0xCA, 0xFE, 0xBA, 0xBE]);

    // 验证版本号 (61:0)
    let major_version = u16::from_be_bytes([class_bytes[6], class_bytes[7]]);
    let minor_version = u16::from_be_bytes([class_bytes[4], class_bytes[5]]);
    assert_eq!(major_version, 61);
    assert_eq!(minor_version, 0);

    println!("Generated SimpleClass.class with {} bytes", class_bytes.len());
}
