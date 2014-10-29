//! 导出适配器测试

use gaia_assembler::export_adapters::*;

#[test]
fn test_pe_export_adapter() {
    let adapter = PeExportAdapter::new();
    assert_eq!(adapter.adapter_name(), "PE Export Adapter");

    // 测试基本指令导出
    let gaia_instruction = GaiaInstruction::LoadConstant(GaiaConstant::Int32(42));
    let result = adapter.export_instruction(&gaia_instruction);
    assert!(result.is_ok());

    let pe_instruction = result.unwrap();
    // 验证导出的 PE 指令格式
    assert!(format!("{:?}", pe_instruction).contains("42"));
}

#[test]
fn test_il_export_adapter() {
    let adapter = IlExportAdapter::new();
    assert_eq!(adapter.adapter_name(), "IL Export Adapter");

    // 测试 IL 指令导出
    let gaia_instruction = GaiaInstruction::LoadLocal(0);
    let result = adapter.export_instruction(&gaia_instruction);
    assert!(result.is_ok());

    let il_instruction = result.unwrap();
    // 验证导出的 IL 指令格式
    assert!(format!("{:?}", il_instruction).contains("Ldloc"));
}

#[test]
fn test_jvm_export_adapter() {
    let adapter = JvmExportAdapter::new();
    assert_eq!(adapter.adapter_name(), "JVM Export Adapter");

    // 测试 JVM 指令导出
    let gaia_instruction = GaiaInstruction::LoadConstant(GaiaConstant::Int32(42));
    let result = adapter.export_instruction(&gaia_instruction);
    assert!(result.is_ok());

    let jvm_instruction = result.unwrap();
    // 验证导出的 JVM 指令格式
    assert!(format!("{:?}", jvm_instruction).contains("42"));
}

#[test]
fn test_wasi_export_adapter() {
    let adapter = WasiExportAdapter::new();
    assert_eq!(adapter.adapter_name(), "WASI Export Adapter");

    // 测试 WASI 指令导出
    let gaia_instruction = GaiaInstruction::LoadConstant(GaiaConstant::Int32(42));
    let result = adapter.export_instruction(&gaia_instruction);
    assert!(result.is_ok());

    let wasi_instruction = result.unwrap();
    // 验证导出的 WASI 指令格式
    assert_eq!(wasi_instruction.opcode, 0x41); // i32.const
    assert_eq!(wasi_instruction.operands, vec![42, 0, 0, 0]);
}

#[test]
fn test_program_export() {
    let adapter = PeExportAdapter::new();

    // 创建一个简单的 Gaia 程序
    let gaia_program = GaiaProgram {
        functions: vec![GaiaFunction {
            name: "main".to_string(),
            instructions: vec![GaiaInstruction::LoadConstant(GaiaConstant::Int32(42)), GaiaInstruction::Return],
            parameters: vec![],
            return_type: Some(GaiaType::Int32),
        }],
    };

    let result = adapter.export_program(&gaia_program);
    assert!(result.is_ok());

    let pe_program = result.unwrap();
    assert_eq!(pe_program.functions.len(), 1);
    assert_eq!(pe_program.functions[0].name, "main");
    assert_eq!(pe_program.functions[0].instructions.len(), 2);
}

#[test]
fn test_unsupported_instruction_export() {
    let adapter = PeExportAdapter::new();

    // 测试不支持的指令导出
    let gaia_instruction = GaiaInstruction::Comment("Test comment".to_string());
    let result = adapter.export_instruction(&gaia_instruction);

    // 注释指令可能不被所有目标架构支持，应该返回错误或跳过
    match result {
        Ok(_) => {
            // 如果支持，验证结果
        }
        Err(err) => {
            // 如果不支持，验证错误类型
            assert!(format!("{:?}", err).contains("not supported") || format!("{:?}", err).contains("NotImplemented"));
        }
    }
}

#[test]
fn test_round_trip_conversion() {
    // 测试往返转换：Gaia -> PE -> Gaia
    let original_instruction = GaiaInstruction::LoadConstant(GaiaConstant::Int32(42));

    // 导出到 PE
    let pe_adapter = PeExportAdapter::new();
    let pe_instruction = pe_adapter.export_instruction(&original_instruction).unwrap();

    // 再导入回 Gaia
    let import_adapter = PeImportAdapter::new();
    let imported_instruction = import_adapter.export_instruction(&pe_instruction).unwrap();

    // 验证往返转换的一致性
    match (&original_instruction, &imported_instruction) {
        (
            GaiaInstruction::LoadConstant(GaiaConstant::Int32(orig)),
            GaiaInstruction::LoadConstant(GaiaConstant::Int32(imported)),
        ) => {
            assert_eq!(orig, imported);
        }
        _ => panic!("Round-trip conversion failed"),
    }
}
