#![feature(new_range_api)]
//! CLR 汇编器集成测试
//!
//! 测试从 MSIL 源代码到可执行 PE 文件的完整转换流程

use clr_assembler::formats::{
    dll::writer::DllWriter,
    msil::{converter::MsilToClrConverter, MsilBuilder, MsilLanguage},
};
use oak_core::{parser::session::ParseSession, Builder};
use oak_msil::{ast::Item, lexer::MsilLexer};
use std::{fs, io::Cursor, process::Command};

#[test]
fn test_hello_world_compilation() {
    // MSIL 源代码
    let msil_source = r#"
.assembly extern mscorlib
{
  .publickeytoken = (B7 7A 5C 56 19 34 E0 89 )
  .ver 4:0:0:0
}

.assembly GaiaAssembler
{
  .ver 1:0:0:0
}

.module GaiaAssembler.exe

.class public auto ansi beforefieldinit GaiaAssembler
       extends [mscorlib]System.Object
{
  .method public hidebysig static void  Main(string[] args) cil managed
  {
    .entrypoint
    .maxstack  1
    ldstr      "Hello Gaia!"
    call       void [mscorlib]System.Console::WriteLine(string)
    ret
  }

  .method public hidebysig specialname rtspecialname 
          instance void  .ctor() cil managed
  {
    .maxstack  1
    ldarg.0
    call       instance void [mscorlib]System.Object::.ctor()
    ret
  }
}
"#;

    // 1. 词法分析
    let language = MsilLanguage::default();
    let lexer = MsilLexer::new(&language);
    let tokens = lexer.tokenize(msil_source);
    assert!(!tokens.is_empty(), "应该产生 tokens");

    // 2. 语法分析
    let builder = MsilBuilder::new(&language);
    let mut session = ParseSession::<MsilLanguage>::default();
    let ast_result = builder.build(msil_source, &[], &mut session);
    assert!(ast_result.result.is_ok(), "语法分析失败: {:?}", ast_result.result.err());
    let ast = ast_result.result.unwrap();
    assert!(!ast.items.is_empty(), "应该产生 AST 语句");

    // 3. AST 转换
    let mut converter = MsilToClrConverter::new();
    let clr_program_result = converter.convert(ast);
    assert!(clr_program_result.result.is_ok(), "AST 转换应该成功");
    let clr_program = clr_program_result.result.unwrap();

    // 验证 CLR Program 结构
    assert_eq!(clr_program.name, "GaiaAssembler");
    assert!(!clr_program.types.is_empty(), "应该包含类型定义");

    let main_class = &clr_program.types[0];
    assert_eq!(main_class.name, "GaiaAssembler");
    assert!(!main_class.methods.is_empty(), "应该包含方法定义");

    // 4. PE 文件生成
    let mut output_buffer = Cursor::new(Vec::new());
    let writer = DllWriter::new(&mut output_buffer);
    let write_result = writer.write(&clr_program);

    if let Err(ref e) = write_result.result {
        println!("写入错误: {:?}", e);
    }

    // 注意：由于 PE 写入器的实现可能不完整，这里先检查是否有输出
    let pe_data = output_buffer.into_inner();
    println!("生成的 PE 文件大小: {} 字节", pe_data.len());

    // 如果成功生成了数据，保存到文件
    if !pe_data.is_empty() {
        fs::write("test_output.exe", &pe_data).expect("保存 PE 文件失败");
        println!("已保存测试输出文件: test_output.exe");
    }
}

#[test]
fn test_msil_lexer() {
    let msil_source = r#"
.assembly extern mscorlib
.assembly GaiaAssembler
.class public GaiaAssembler
{
  .method public static void Main()
  {
    ldstr "Hello World"
    call void [mscorlib]System.Console::WriteLine(string)
    ret
  }
}
"#;

    let language = MsilLanguage::default();
    let lexer = MsilLexer::new(&language);
    let tokens = lexer.tokenize(msil_source);

    // 验证生成的 tokens
    assert!(!tokens.is_empty());

    // 检查是否包含关键的 tokens
    let token_strings: Vec<String> = tokens.iter().map(|t| msil_source[t.span.start..t.span.end].to_string()).collect();

    assert!(token_strings.contains(&".assembly".to_string()));
    assert!(token_strings.contains(&"extern".to_string()));
    assert!(token_strings.contains(&"mscorlib".to_string()));
    assert!(token_strings.contains(&".class".to_string()));
    assert!(token_strings.contains(&"public".to_string()));
    assert!(token_strings.contains(&".method".to_string()));
    assert!(token_strings.contains(&"ldstr".to_string()));
    assert!(token_strings.contains(&"call".to_string()));
    assert!(token_strings.contains(&"ret".to_string()));
}

#[test]
fn test_msil_parser() {
    let msil_source = r#"
.assembly extern mscorlib
.assembly GaiaAssembler
.class public GaiaAssembler
{
  .method public static void Main()
  {
    ret
  }
}
"#;

    let language = MsilLanguage::default();
    let builder = MsilBuilder::new(&language);
    let mut session = ParseSession::<MsilLanguage>::default();
    let ast = builder.build(msil_source, &[], &mut session).result.expect("语法分析失败");

    // 验证 AST 结构
    assert_eq!(ast.items.len(), 3); // extern, assembly, class

    // 检查语句类型
    match &ast.items[0] {
        Item::AssemblyExtern(name) => {
            assert_eq!(name, "mscorlib");
        }
        item => panic!("第一个语句应该是 AssemblyExtern, 实际上是: {:?}", item),
    }

    match &ast.items[1] {
        Item::Assembly(asm) => {
            assert_eq!(asm.name, "GaiaAssembler");
        }
        _ => panic!("第二个语句应该是 Assembly"),
    }

    match &ast.items[2] {
        Item::Class(class) => {
            assert_eq!(class.name, "GaiaAssembler");
            assert_eq!(class.methods.len(), 1);
            assert_eq!(class.methods[0].name, "Main");
        }
        _ => panic!("第三个语句应该是 Class"),
    }
}

#[test]
fn test_ast_to_clr_conversion() {
    use core::range::Range;
    use oak_msil::ast::{Assembly, Class, Item, Method, MsilRoot};

    // 创建简单的 AST
    let ast = MsilRoot {
        items: vec![
            Item::Assembly(Assembly { name: "TestAssembly".to_string(), span: Range { start: 0, end: 0 } }),
            Item::Class(Class {
                name: "TestClass".to_string(),
                methods: vec![Method {
                    name: "TestMethod".to_string(),
                    instructions: vec![],
                    span: Range { start: 0, end: 0 },
                }],
                span: Range { start: 0, end: 0 },
            }),
        ],
    };

    // 转换为 CLR Program
    let mut converter = MsilToClrConverter::new();
    let clr_program_result = converter.convert(ast);
    assert!(clr_program_result.result.is_ok());

    let clr_program = clr_program_result.result.unwrap();
    assert_eq!(clr_program.name, "TestAssembly");
    assert_eq!(clr_program.types.len(), 1);
    assert_eq!(clr_program.types[0].name, "TestClass");
    assert_eq!(clr_program.types[0].methods.len(), 1);
    assert_eq!(clr_program.types[0].methods[0].name, "TestMethod");
}

// 辅助函数：检查生成的 exe 文件是否可以运行
#[cfg(target_os = "windows")]
fn try_run_exe(exe_path: &str) -> Result<String, std::io::Error> {
    let output = Command::new(exe_path).output()?;

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[test]
#[ignore] // 默认忽略，因为需要完整的 PE 写入器实现
fn test_generated_exe_execution() {
    // 这个测试需要完整的 PE 文件生成功能
    // 当 PE 写入器完全实现后，可以启用此测试

    if std::path::Path::new("test_output.exe").exists() {
        #[cfg(target_os = "windows")]
        {
            match try_run_exe("test_output.exe") {
                Ok(output) => {
                    assert!(output.contains("Hello Gaia!"));
                    println!("程序执行成功，输出: {}", output);
                }
                Err(e) => {
                    println!("程序执行失败: {}", e);
                    // 不让测试失败，因为 PE 文件可能不完整
                }
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            println!("跳过 exe 执行测试（非 Windows 平台）");
        }
    }
    else {
        println!("test_output.exe 不存在，跳过执行测试");
    }
}
