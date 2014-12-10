use gaia_assembler::{
    assembler::GaiaAssembler,
    instruction::{CmpCondition, CoreInstruction, DomainInstruction, GaiaInstruction},
    program::{GaiaBlock, GaiaConstant, GaiaFunction, GaiaModule, GaiaTerminator},
    types::{GaiaSignature, GaiaType},
};
use gaia_types::{
    helpers::{AbiCompatible, ApiCompatible, Architecture, CompilationTarget},
    neural::{MatMulGroup, NeuralNode},
};

#[test]
fn test_gaia_to_pe_neural_flow() {
    // 1. 创建包含神经网络指令的 Gaia 程序
    let program = GaiaModule {
        name: "neural_test".to_string(),
        functions: vec![GaiaFunction {
            name: "main".to_string(),
            signature: GaiaSignature { params: vec![], return_type: GaiaType::I32 },
            blocks: vec![GaiaBlock {
                label: "entry".to_string(),
                instructions: vec![
                    GaiaInstruction::Core(CoreInstruction::Alloca(GaiaType::I32, 1)),
                    GaiaInstruction::Core(CoreInstruction::PushConstant(GaiaConstant::I32(10))),
                    GaiaInstruction::Core(CoreInstruction::StoreLocal(0, GaiaType::I32)),
                    GaiaInstruction::Core(CoreInstruction::Label("loop".to_string())),
                    GaiaInstruction::Core(CoreInstruction::LoadLocal(0, GaiaType::I32)),
                    GaiaInstruction::Core(CoreInstruction::PushConstant(GaiaConstant::I32(0))),
                    GaiaInstruction::Core(CoreInstruction::Cmp(CmpCondition::Gt, GaiaType::I32)),
                    GaiaInstruction::Core(CoreInstruction::BrFalse("end".to_string())),
                    GaiaInstruction::Domain(DomainInstruction::Neural(NeuralNode::MatMul(MatMulGroup {
                        m: 64,
                        n: 64,
                        k: 64,
                        transpose_a: false,
                        transpose_b: false,
                    }))),
                    GaiaInstruction::Core(CoreInstruction::LoadLocal(0, GaiaType::I32)),
                    GaiaInstruction::Core(CoreInstruction::PushConstant(GaiaConstant::I32(1))),
                    GaiaInstruction::Core(CoreInstruction::Sub(GaiaType::I32)),
                    GaiaInstruction::Core(CoreInstruction::StoreLocal(0, GaiaType::I32)),
                    GaiaInstruction::Core(CoreInstruction::Br("loop".to_string())),
                    GaiaInstruction::Core(CoreInstruction::Label("end".to_string())),
                    GaiaInstruction::Core(CoreInstruction::PushConstant(GaiaConstant::I32(0))),
                ],
                terminator: GaiaTerminator::Return,
            }],
            is_external: false,
        }],
        structs: vec![],
        classes: vec![],
        constants: vec![],
        globals: vec![],
        imports: vec![gaia_assembler::program::GaiaImport {
            library: "gaia_runtime.dll".to_string(),
            symbol: "gaia_matmul".to_string(),
        }],
    };

    // 2. 编译到 PE (x86_64)
    let assembler = GaiaAssembler::new();
    let target =
        CompilationTarget { build: Architecture::X86_64, host: AbiCompatible::PE, target: ApiCompatible::MicrosoftVisualC };

    let generated = assembler.compile(&program, &target).expect("Failed to compile neural program");
    let binary = generated.files.get("main.exe").expect("No main.exe generated");

    // 3. 验证
    verify_pe_binary(binary);

    // 检查是否包含 DLL 名称
    let binary_str = String::from_utf8_lossy(binary);
    assert!(binary_str.contains("kernel32.dll"), "Should contain kernel32.dll import");
    assert!(binary_str.contains("gaia_runtime.dll"), "Should contain gaia_runtime.dll import");
    assert!(binary_str.contains("gaia_matmul"), "Should contain gaia_matmul import");

    println!("Successfully generated a {} byte neural PE binary", binary.len());
}

fn verify_pe_binary(binary: &[u8]) {
    // MZ Header
    assert!(binary.len() >= 0x40, "Binary too small for PE");
    assert_eq!(&binary[0..2], b"MZ", "Missing MZ header");

    // PE Offset
    let pe_offset = u32::from_le_bytes(binary[0x3C..0x40].try_into().unwrap()) as usize;
    assert!(binary.len() >= pe_offset + 4, "Binary too small for PE signature");

    // PE Signature
    assert_eq!(&binary[pe_offset..pe_offset + 4], b"PE\0\0", "Missing PE signature");

    // Machine Type (x86_64 = 0x8664)
    let machine = u16::from_le_bytes(binary[pe_offset + 4..pe_offset + 6].try_into().unwrap());
    assert_eq!(machine, 0x8664, "Incorrect machine type (expected x86_64)");
}

#[test]
fn test_gaia_to_pe_arithmetic_flow() {
    // 1. 创建包含复杂算术和位运算的 Gaia 程序
    let program = GaiaModule {
        name: "arith_test".to_string(),
        functions: vec![GaiaFunction {
            name: "main".to_string(),
            signature: GaiaSignature { params: vec![], return_type: GaiaType::I32 },
            blocks: vec![GaiaBlock {
                label: "entry".to_string(),
                instructions: vec![
                    // Reserve space for locals
                    GaiaInstruction::Core(CoreInstruction::Alloca(GaiaType::I32, 3)),
                    // x = 10 + 20
                    GaiaInstruction::Core(CoreInstruction::PushConstant(GaiaConstant::I32(10))),
                    GaiaInstruction::Core(CoreInstruction::PushConstant(GaiaConstant::I32(20))),
                    GaiaInstruction::Core(CoreInstruction::Add(GaiaType::I32)),
                    GaiaInstruction::Core(CoreInstruction::StoreLocal(0, GaiaType::I32)),
                    // y = x << 2
                    GaiaInstruction::Core(CoreInstruction::LoadLocal(0, GaiaType::I32)),
                    GaiaInstruction::Core(CoreInstruction::PushConstant(GaiaConstant::I32(2))),
                    GaiaInstruction::Core(CoreInstruction::Shl(GaiaType::I32)),
                    GaiaInstruction::Core(CoreInstruction::StoreLocal(1, GaiaType::I32)),
                    // z = (y | 1) & 0xFF
                    GaiaInstruction::Core(CoreInstruction::LoadLocal(1, GaiaType::I32)),
                    GaiaInstruction::Core(CoreInstruction::PushConstant(GaiaConstant::I32(1))),
                    GaiaInstruction::Core(CoreInstruction::Or(GaiaType::I32)),
                    GaiaInstruction::Core(CoreInstruction::PushConstant(GaiaConstant::I32(0xFF))),
                    GaiaInstruction::Core(CoreInstruction::And(GaiaType::I32)),
                    GaiaInstruction::Core(CoreInstruction::StoreLocal(2, GaiaType::I32)),
                    // return z
                    GaiaInstruction::Core(CoreInstruction::LoadLocal(2, GaiaType::I32)),
                ],
                terminator: GaiaTerminator::Return,
            }],
            is_external: false,
        }],
        structs: vec![],
        classes: vec![],
        constants: vec![],
        globals: vec![],
        imports: vec![],
    };

    // 2. 编译到 PE (x86_64)
    let assembler = GaiaAssembler::new();
    let target =
        CompilationTarget { build: Architecture::X86_64, host: AbiCompatible::PE, target: ApiCompatible::MicrosoftVisualC };

    let generated = assembler.compile(&program, &target).expect("Failed to compile arithmetic program");
    let binary = generated.files.get("main.exe").expect("No main.exe generated");

    // 3. 验证
    verify_pe_binary(binary);

    // 检查是否包含关键导入
    let binary_str = String::from_utf8_lossy(binary);
    assert!(binary_str.contains("kernel32.dll"), "Should contain kernel32.dll import");

    println!("Successfully generated a {} byte arithmetic PE binary", binary.len());
}
