use gaia_types::BinaryWriter;
use std::{fs, path::Path};
use wasi_assembler::{
    formats::wasm::writer::WasmWriter,
    wat::{lexer::WatLexer, parser::WatParser},
};

#[test]
fn hello_world() {
    // 读取 hello_world.wat 文件
    let wat_path = Path::new("tests/wasm_write/hello_world.wat");
    let wat_content = fs::read_to_string(wat_path).expect("Failed to read hello_world.wat file");

    // 步骤 1: 词法分析
    let lexer = WatLexer::new();
    let token_stream_result = lexer.tokenize(&wat_content);
    assert!(token_stream_result.result.is_ok(), "Failed to tokenize WAT content");
    let token_stream = token_stream_result.result.unwrap();

    // 步骤 2: 语法分析
    let parser = WatParser::new(None);
    let ast = parser.parse(token_stream).expect("Failed to parse WAT tokens");

    // 步骤 3: 编译为 Program
    let mut compiler = WatCompiler::new();
    let program_result = compiler.compile(ast);
    assert!(program_result.result.is_ok(), "Failed to compile AST to Program");
    let program = program_result.result.unwrap();

    // 步骤 4: 生成 WASM 字节码
    let binary_assembler: gaia_types::BinaryWriter<Vec<u8>, byteorder::LittleEndian> =
        gaia_types::BinaryWriter::new(Vec::new());
    let mut wasm_writer = WasmWriter::new(binary_assembler);
    let wasm_bytes_result = wasm_writer.write(program);

    // 检查结果
    assert!(wasm_bytes_result.result.is_ok(), "Failed to generate WASM bytes");

    let wasm_bytes = wasm_bytes_result.result.unwrap();

    // 验证 WASM 魔数
    assert_eq!(&wasm_bytes[0..4], &[0x00, 0x61, 0x73, 0x6D], "Invalid WASM magic number");

    // 验证版本号（组件模型或核心模块）
    let version = &wasm_bytes[4..8];
    assert!(
        version == &[0x01, 0x00, 0x00, 0x00] || version == &[0x0A, 0x00, 0x01, 0x00],
        "Invalid WASM version: {:?}",
        version
    );

    // 写入输出文件用于调试
    let output_path = Path::new("tests/wasm_write/hello_world.wasm");
    fs::write(output_path, &wasm_bytes).expect("Failed to write output WASM file");

    println!("Successfully compiled hello_world.wat to hello_world.wasm");
    println!("Generated {} bytes of WASM bytecode", wasm_bytes.len());
}
