use gaia_types::helpers::Architecture;
use x86_64_assembler::{
    instruction::{Instruction, Operand, Register},
    X86_64Assembler,
};

#[test]
fn test_mov_8bit_encoding() {
    let assembler = X86_64Assembler::new(Architecture::X86_64).unwrap();

    // MOV al, 0x12 -> B0 12
    let instr = Instruction::Mov { dst: Operand::Reg(Register::AL), src: Operand::Imm { value: 0x12, size: 8 } };
    let bytes = assembler.encode(&instr).unwrap();
    assert_eq!(bytes, vec![0xB0, 0x12]);

    // MOV r8b, 0x34 -> 41 B0 34 (REX.B=1 for R8B)
    let instr = Instruction::Mov { dst: Operand::Reg(Register::R8B), src: Operand::Imm { value: 0x34, size: 8 } };
    let bytes = assembler.encode(&instr).unwrap();
    assert_eq!(bytes, vec![0x41, 0xB0, 0x34]);

    // MOV [rax], al -> 88 00
    let instr = Instruction::Mov {
        dst: Operand::Mem { base: Some(Register::RAX), index: None, scale: 1, displacement: 0 },
        src: Operand::Reg(Register::AL),
    };
    let bytes = assembler.encode(&instr).unwrap();
    assert_eq!(bytes, vec![0x88, 0x00]);

    // MOV al, [rax] -> 8A 00
    let instr = Instruction::Mov {
        dst: Operand::Reg(Register::AL),
        src: Operand::Mem { base: Some(Register::RAX), index: None, scale: 1, displacement: 0 },
    };
    let bytes = assembler.encode(&instr).unwrap();
    assert_eq!(bytes, vec![0x8A, 0x00]);

    // MOV [r8], al -> 41 88 00
    let instr = Instruction::Mov {
        dst: Operand::Mem { base: Some(Register::R8), index: None, scale: 1, displacement: 0 },
        src: Operand::Reg(Register::AL),
    };
    let bytes = assembler.encode(&instr).unwrap();
    assert_eq!(bytes, vec![0x41, 0x88, 0x00]);

    // MOV r8b, [rax] -> 44 8A 00
    let instr = Instruction::Mov {
        dst: Operand::Reg(Register::R8B),
        src: Operand::Mem { base: Some(Register::RAX), index: None, scale: 1, displacement: 0 },
    };
    let bytes = assembler.encode(&instr).unwrap();
    assert_eq!(bytes, vec![0x44, 0x8A, 0x00]);
}

#[test]
fn test_mov_16bit_encoding() {
    let assembler = X86_64Assembler::new(Architecture::X86_64).unwrap();

    // MOV ax, 0x1234 -> 66 B8 34 12
    let instr = Instruction::Mov { dst: Operand::Reg(Register::AX), src: Operand::Imm { value: 0x1234, size: 16 } };
    let bytes = assembler.encode(&instr).unwrap();
    assert_eq!(bytes, vec![0x66, 0xB8, 0x34, 0x12]);

    // MOV [rax], ax -> 66 89 00
    let instr = Instruction::Mov {
        dst: Operand::Mem { base: Some(Register::RAX), index: None, scale: 1, displacement: 0 },
        src: Operand::Reg(Register::AX),
    };
    let bytes = assembler.encode(&instr).unwrap();
    assert_eq!(bytes, vec![0x66, 0x89, 0x00]);

    // MOV ax, [rax] -> 66 8B 00
    let instr = Instruction::Mov {
        dst: Operand::Reg(Register::AX),
        src: Operand::Mem { base: Some(Register::RAX), index: None, scale: 1, displacement: 0 },
    };
    let bytes = assembler.encode(&instr).unwrap();
    assert_eq!(bytes, vec![0x66, 0x8B, 0x00]);
}

#[test]
fn test_mov_mem_imm_encoding() {
    let assembler = X86_64Assembler::new(Architecture::X86_64).unwrap();

    // MOV BYTE PTR [rax], 0x12 -> C6 00 12
    let instr = Instruction::Mov {
        dst: Operand::Mem { base: Some(Register::RAX), index: None, scale: 1, displacement: 0 },
        src: Operand::Imm { value: 0x12, size: 8 },
    };
    let bytes = assembler.encode(&instr).unwrap();
    assert_eq!(bytes, vec![0xC6, 0x00, 0x12]);

    // MOV WORD PTR [rax], 0x1234 -> 66 C7 00 34 12
    let instr = Instruction::Mov {
        dst: Operand::Mem { base: Some(Register::RAX), index: None, scale: 1, displacement: 0 },
        src: Operand::Imm { value: 0x1234, size: 16 },
    };
    let bytes = assembler.encode(&instr).unwrap();
    assert_eq!(bytes, vec![0x66, 0xC7, 0x00, 0x34, 0x12]);

    // MOV DWORD PTR [rax], 0x12345678 -> C7 00 78 56 34 12
    let instr = Instruction::Mov {
        dst: Operand::Mem { base: Some(Register::RAX), index: None, scale: 1, displacement: 0 },
        src: Operand::Imm { value: 0x12345678, size: 32 },
    };
    let bytes = assembler.encode(&instr).unwrap();
    assert_eq!(bytes, vec![0xC7, 0x00, 0x78, 0x56, 0x34, 0x12]);
}
