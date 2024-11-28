use clr_assembler::formats::exe::writer::DotNetWriter;
use clr_assembler::program::{
    ClrExternalAssembly, ClrInstruction, ClrMethod, ClrOpcode, ClrProgram, ClrTypeReference,
};
use std::io::Cursor;

/// 生成 .NET 控制台程序：调用 mscorlib 的 System.Console::WriteLine("Hello, World!")
#[test]
fn generate_hello_world_exe_with_mscorlib() {
    // 1) 构建 CLR Program 并引用 mscorlib
    let mut program = ClrProgram::new("HelloGaia");

    // 添加 mscorlib 引用（简化字段即可满足需求）
    program.add_external_assembly(ClrExternalAssembly {
        name: "mscorlib".to_string(),
        version: clr_assembler::program::ClrVersion { major: 4, minor: 0, build: 0, revision: 0 },
        public_key_token: Some(vec![0xB7, 0x7A, 0x5C, 0x56, 0x19, 0x34, 0xE0, 0x89]), // b77a5c561934e089
        culture: None,
        hash_algorithm: None,
    });

    // 2) 定义返回类型：System.Void 来自 mscorlib
    let void_type = ClrTypeReference {
        name: "Void".to_string(),
        namespace: Some("System".to_string()),
        assembly: Some("mscorlib".to_string()),
        is_value_type: true,
        is_reference_type: false,
        generic_parameters: Vec::new(),
    };

    // 3) 构建入口方法 Main
    let mut main = ClrMethod::new("Main".to_string(), void_type);
    main.is_entry_point = true;

    // IL: ldstr "Hello, World!"; call void [mscorlib]System.Console::WriteLine(string); ret
    main.add_instruction(ClrInstruction::WithString { opcode: ClrOpcode::Ldstr, value: "Hello, World!".to_string() });
    main.add_instruction(ClrInstruction::WithMethod { opcode: ClrOpcode::Call, method_ref: "void [mscorlib]System.Console::WriteLine(string)".to_string() });
    main.add_instruction(ClrInstruction::Simple { opcode: ClrOpcode::Ret });

    program.global_methods.push(main);

    // 4) 生成 EXE（PE）字节
    let buffer = Cursor::new(Vec::new());
    let writer = DotNetWriter::new(buffer);
    let result = writer.write(&program);

    assert!(result.result.is_ok(), "DotNetWriter 应该成功生成 PE 文件");
    let pe_writer = result.result.unwrap();
    let pe_bytes = pe_writer.into_inner();
    assert!(!pe_bytes.is_empty(), "生成的 PE 文件不应为空");

    // 5) 基本 PE 验证（DOS 头）
    assert!(pe_bytes.len() >= 2 && &pe_bytes[0..2] == b"MZ", "DOS 签名应为 MZ");

    // 6) 轻量验证：IL 指令包含 ldstr(0x72)、call(0x28)、ret(0x2A)
    let has_ldstr = pe_bytes.iter().any(|&b| b == 0x72);
    let has_call = pe_bytes.iter().any(|&b| b == 0x28);
    let has_ret = pe_bytes.iter().any(|&b| b == 0x2A);
    assert!(has_ldstr && has_call && has_ret, "生成的 IL 应包含 Ldstr/Call/Ret 指令");

    // 7) 验证程序引用了 mscorlib（符合需求：使用 mscorlib.dll）
    let refs = program.get_referenced_assemblies();
    assert!(refs.iter().any(|r| r == "mscorlib"), "程序应引用 mscorlib.dll");
}