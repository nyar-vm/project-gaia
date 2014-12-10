use oak_core::{
    parser::{ParseSession, Parser},
    source::SourceText,
};
use oak_wat::WatLanguage;
use std::fs;
use wasi_assembler::formats::wat::parser::WatParser;

#[test]
fn test_parse_hello_world() {
    let wat_content = fs::read_to_string("tests/wat_read/hello_world.wat").expect("Failed to read hello_world.wat");
    let source = SourceText::new(wat_content);

    let language = WatLanguage::default();
    let parser = WatParser::new(&language);
    let mut cache = ParseSession::default();
    let output = parser.parse(&source, &[], &mut cache);

    assert!(output.result.is_ok());
    println!("Successfully parsed hello_world.wat");
}

#[test]
fn test_parse_core_module() {
    let wat_content = fs::read_to_string("tests/wat_read/core_module.wat").expect("Failed to read core_module.wat");
    let source = SourceText::new(wat_content);

    let language = WatLanguage::default();
    let parser = WatParser::new(&language);
    let mut cache = ParseSession::default();
    let output = parser.parse(&source, &[], &mut cache);

    assert!(output.result.is_ok());
    println!("Successfully parsed core_module.wat");
}

#[test]
fn test_parse_component_model() {
    let wat_content = fs::read_to_string("tests/wat_read/component_model.wat").expect("Failed to read component_model.wat");
    let source = SourceText::new(wat_content);

    let language = WatLanguage::default();
    let parser = WatParser::new(&language);
    let mut cache = ParseSession::default();
    let output = parser.parse(&source, &[], &mut cache);

    assert!(output.result.is_ok());
    println!("Successfully parsed component_model.wat");
}

#[test]
fn test_wat_to_program_conversion() {
    let wat_content = fs::read_to_string("tests/wat_read/hello_world.wat").expect("Failed to read hello_world.wat");
    let source = SourceText::new(wat_content);

    let language = WatLanguage::default();
    let parser = WatParser::new(&language);
    let mut cache = ParseSession::default();
    let _output = parser.parse(&source, &[], &mut cache);

    // TODO: Convert GreenNode to AST and then to WasiProgram
    println!("WAT to WasiProgram conversion test skipped (waiting for AST mapping)");
}

#[test]
fn test_program_to_wat_conversion() {
    use wasi_assembler::program::{
        WasiExport, WasiFunction, WasiFunctionType, WasiProgram, WasiProgramType, WasmExportType, WasmValueType,
    };

    // 创建一个简单的 WasiProgram
    let mut program = WasiProgram::new(WasiProgramType::CoreModule);
    program.name = Some("test_program".to_string());

    // 添加函数类型
    let func_type =
        WasiFunctionType { params: vec![WasmValueType::I32, WasmValueType::I32], results: vec![WasmValueType::I32] };
    program.function_types.push(func_type);

    // 添加函数
    let func = WasiFunction {
        type_index: 0,
        body: vec![], // 空函数体
        locals: vec![],
    };
    program.functions.push(func);

    // 添加导出
    let export = WasiExport { name: "add".to_string(), export_type: WasmExportType::Function { function_index: 0 } };
    program.exports.push(export);

    // 测试 WasiProgram 到 WAT 的转换
    let wat_result = program.to_wat();
    match wat_result {
        Ok(wat_ast) => {
            println!("Successfully converted WasiProgram to WAT");
            println!("WAT items count: {}", wat_ast.items.len());
        }
        Err(e) => {
            println!("WAT conversion failed: {:?}", e);
            panic!("WAT conversion should not fail for simple program");
        }
    }
}
