use gaia_assembler::{
    assembler::GaiaAssembler,
    instruction::{DomainInstruction, GaiaInstruction},
    program::{GaiaBlock, GaiaFunction, GaiaModule, GaiaTerminator},
    types::{GaiaSignature, GaiaType},
};
use gaia_types::helpers::{AbiCompatible, ApiCompatible, Architecture, CompilationTarget};

#[test]
fn test_x86_backend_generation() {
    let program = GaiaModule {
        name: "test_x86".to_string(),
        functions: vec![GaiaFunction {
            name: "main".to_string(),
            signature: GaiaSignature { params: vec![], return_type: GaiaType::Void },
            blocks: vec![GaiaBlock {
                label: "entry".to_string(),
                instructions: vec![GaiaInstruction::Domain(DomainInstruction::Neural(gaia_types::neural::NeuralNode::MatMul(
                    gaia_types::neural::MatMulGroup { m: 1024, n: 1024, k: 1024, transpose_a: false, transpose_b: false },
                )))],
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

    let target =
        CompilationTarget { build: Architecture::X86_64, host: AbiCompatible::PE, target: ApiCompatible::MicrosoftVisualC };

    let assembler = GaiaAssembler::new();
    let result = assembler.compile(&program, &target).expect("Failed to compile");

    assert!(result.files.contains_key("main.exe"));
    let pe_bytes = &result.files["main.exe"];
    assert!(pe_bytes.len() > 0);
    assert_eq!(&pe_bytes[0..2], b"MZ");

    println!("Successfully generated x86 PE: {} bytes", pe_bytes.len());
}
