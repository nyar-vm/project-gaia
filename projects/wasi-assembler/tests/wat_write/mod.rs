#[test]
fn it_works() {
    assert_eq!(2 + 2, 4);
}

#[test]
fn test_wasi_program_to_wat_and_run() {
    use wasi_assembler::program::{WasiProgram, WasiProgramType, WasiFunction, WasiFunctionType, WasiInstruction, WasmValueType, WasiExport, WasmExportType};
    use wasi_assembler::formats::wat::writer::WatWriter;
    use gaia_types::writer::TextWriter;
    use crate::test_tools::test_run_wat_component;

    // 1. 使用 WasiProgramBuilder 创建一个简单的 WASI 程序
    let func_type = WasiFunctionType {
        params: vec![],
        results: vec![],
    };

    let func_main = WasiFunction {
        type_index: 0,
        locals: vec![],
        body: vec![
            WasiInstruction::I32Const { value: 42 },
            WasiInstruction::Drop,
        ],
    };

    let export_main = WasiExport {
        name: "_start".to_string(),
        export_type: WasmExportType::Function { function_index: 0 },
    };

    let program = WasiProgram::builder(WasiProgramType::CoreModule)
        .with_function_type(func_type)
        .with_function(func_main)
        .with_export(export_main)
        .build()
        .unwrap();

    // 2. 将 WASI 程序转换为 WAT 字符串
    let mut wat_string_writer = String::new();
    let text_writer = TextWriter::new(&mut wat_string_writer);
    let mut wat_writer = WatWriter::new(text_writer);

    let wat_root = program.to_wat().result.unwrap();
    wat_writer.write_ast(&wat_root).unwrap();
    let wat_content = wat_writer.finish();

    println!("Generated WAT:
{}", wat_content);

    // 3. 使用 test_run_wat_component 运行生成的 WAT 字符串
    let result = test_run_wat_component(&wat_content);

    // 4. 验证执行结果
    assert!(result.is_ok(), "WASI program execution failed: {:?}", result.err());
}
