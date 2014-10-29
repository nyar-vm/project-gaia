//! 导入适配器测试

use gaia_assembler::import_adapters::*;

#[test]
fn test_pe_import_adapter() {
    let adapter = PeImportAdapter::new();
    assert_eq!(adapter.adapter_name(), "PE Import Adapter");

    // 测试基本指令导入
    let pe_instruction =
        pe_assembler::PeInstruction::Mov { dest: pe_assembler::Register::Eax, src: pe_assembler::Operand::Immediate(42) };

    let result = adapter.import_instruction(&pe_instruction);
    assert!(result.is_ok());

    let gaia_instruction = result.unwrap();
    match gaia_instruction {
        GaiaInstruction::LoadConstant(GaiaConstant::Int32(value)) => {
            assert_eq!(value, 42);
        }
        _ => panic!("Expected LoadConstant instruction"),
    }
}

#[test]
fn test_il_import_adapter() {
    let adapter = IlImportAdapter::new();
    assert_eq!(adapter.adapter_name(), "IL Import Adapter");

    // 测试 IL 指令导入
    let il_instruction = il_assembler::IlInstruction::Ldloc(0);
    let result = adapter.import_instruction(&il_instruction);
    assert!(result.is_ok());

    let gaia_instruction = result.unwrap();
    match gaia_instruction {
        GaiaInstruction::LoadLocal(index) => {
            assert_eq!(index, 0);
        }
        _ => panic!("Expected LoadLocal instruction"),
    }
}

#[test]
fn test_jvm_import_adapter() {
    let adapter = JvmImportAdapter::new();
    assert_eq!(adapter.adapter_name(), "JVM Import Adapter");

    // 测试 JVM 指令导入
    let jvm_instruction = jvm_assembler::JvmInstruction::Bipush(42);
    let result = adapter.import_instruction(&jvm_instruction);
    assert!(result.is_ok());

    let gaia_instruction = result.unwrap();
    match gaia_instruction {
        GaiaInstruction::LoadConstant(GaiaConstant::Int32(value)) => {
            assert_eq!(value, 42);
        }
        _ => panic!("Expected LoadConstant instruction"),
    }
}

#[test]
fn test_wasi_import_adapter() {
    let adapter = WasiImportAdapter::new();
    assert_eq!(adapter.adapter_name(), "WASI Import Adapter");

    // 测试 WASI 指令导入
    let wasi_instruction = wasi_assembler::WasiInstruction {
        opcode: 0x41,                // i32.const
        operands: vec![42, 0, 0, 0], // little-endian 42
    };

    let result = adapter.import_instruction(&wasi_instruction);
    assert!(result.is_ok());

    let gaia_instruction = result.unwrap();
    match gaia_instruction {
        GaiaInstruction::LoadConstant(GaiaConstant::Int32(value)) => {
            assert_eq!(value, 42);
        }
        _ => panic!("Expected LoadConstant instruction"),
    }
}

#[test]
fn test_unsupported_instruction_handling() {
    let adapter = WasiImportAdapter::new();

    // 测试不支持的指令
    let unsupported_instruction = wasi_assembler::WasiInstruction {
        opcode: 0xFF, // 不存在的操作码
        operands: vec![],
    };

    let result = adapter.import_instruction(&unsupported_instruction);
    assert!(result.is_ok());

    let gaia_instruction = result.unwrap();
    match gaia_instruction {
        GaiaInstruction::Comment(msg) => {
            assert!(msg.contains("Unsupported"));
        }
        _ => panic!("Expected Comment instruction for unsupported opcode"),
    }
}

#[test]
fn test_program_import() {
    let adapter = PeImportAdapter::new();

    // 创建一个简单的 PE 程序
    let pe_program = pe_assembler::PeProgram {
        functions: vec![pe_assembler::PeFunction {
            name: "main".to_string(),
            instructions: vec![
                pe_assembler::PeInstruction::Mov {
                    dest: pe_assembler::Register::Eax,
                    src: pe_assembler::Operand::Immediate(42),
                },
                pe_assembler::PeInstruction::Ret,
            ],
            parameters: vec![],
            return_type: Some(pe_assembler::PeType::Int32),
        }],
    };

    let result = adapter.import_program(&pe_program);
    assert!(result.is_ok());

    let gaia_program = result.unwrap();
    assert_eq!(gaia_program.functions.len(), 1);
    assert_eq!(gaia_program.functions[0].name, "main");
    assert_eq!(gaia_program.functions[0].instructions.len(), 2);
}
