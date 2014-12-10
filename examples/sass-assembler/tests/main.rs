use sass_assembler::{
    instructions::{SassInstruction, SassReg},
    program::SassProgram,
    SassWriter,
};

#[test]
fn test_sass_generation() {
    let instructions =
        vec![SassInstruction::FAdd { dst: SassReg::R(0), src0: SassReg::R(1), src1: SassReg::R(2) }, SassInstruction::Exit];

    let mut program = SassProgram::new("test_kernel".to_string());
    program.kernels.push(sass_assembler::program::SassKernel { name: "test_kernel".to_string(), instructions });
    let writer = SassWriter::new();
    let binary = writer.write(&program).expect("Failed to generate SASS binary");

    assert!(binary.starts_with(b"\x7fELF"));
    let content = String::from_utf8_lossy(&binary[4..]);
    assert!(content.contains(".section .text.test_kernel"));
    assert!(content.contains("FADD R0, R1, R2"));
    assert!(content.contains("EXIT"));
}
