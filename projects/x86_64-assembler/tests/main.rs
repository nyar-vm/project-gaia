use gaia_types::helpers::Architecture;
use x86_64_assembler::{
    instruction::{Instruction, Operand, Register},
    X86_64Assembler,
};

#[test]
fn test_assembler_creation() {
    // 测试 x86 架构
    let result = X86_64Assembler::new(Architecture::X86);
    assert!(result.is_ok());

    // 测试 x86_64 架构
    let result = X86_64Assembler::new(Architecture::X86_64);
    assert!(result.is_ok());

    // 测试不支持的架构
    let result = X86_64Assembler::new(Architecture::ARM64);
    assert!(result.is_err());
}

#[test]
fn test_simple_mov_encoding() {
    let assembler = X86_64Assembler::new(Architecture::X86_64).unwrap();

    // 测试 MOV eax, ebx (寄存器到寄存器)
    let instruction = Instruction::Mov { dst: Operand::Reg(Register::EAX), src: Operand::Reg(Register::EBX) };

    let result = assembler.encode(&instruction);
    assert!(result.is_ok());
    let bytes = result.unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn test_push_pop_encoding() {
    let assembler = X86_64Assembler::new(Architecture::X86_64).unwrap();

    // 测试 PUSH rax
    let push_instruction = Instruction::Push { op: Operand::Reg(Register::RAX) };

    let result = assembler.encode(&push_instruction);
    assert!(result.is_ok());

    // 测试 POP rbx
    let pop_instruction = Instruction::Pop { dst: Operand::Reg(Register::RBX) };

    let result = assembler.encode(&pop_instruction);
    assert!(result.is_ok());
}

#[test]
fn test_ret_encoding() {
    let assembler = X86_64Assembler::new(Architecture::X86_64).unwrap();

    let instruction = Instruction::Ret;
    let result = assembler.encode(&instruction);
    assert!(result.is_ok());

    let bytes = result.unwrap();
    assert_eq!(bytes, vec![0xC3]); // RET 的机器码
}

#[test]
fn test_nop_encoding() {
    let assembler = X86_64Assembler::new(Architecture::X86_64).unwrap();

    let instruction = Instruction::Nop;
    let result = assembler.encode(&instruction);
    assert!(result.is_ok());

    let bytes = result.unwrap();
    assert_eq!(bytes, vec![0x90]); // NOP 的机器码
}

#[test]
fn test_architecture_switching() {
    let mut assembler = X86_64Assembler::new(Architecture::X86).unwrap();
    assert_eq!(assembler.architecture(), Architecture::X86);

    // 切换到 x86_64
    let result = assembler.set_architecture(Architecture::X86_64);
    assert!(result.is_ok());
    assert_eq!(assembler.architecture(), Architecture::X86_64);

    // 尝试切换到不支持的架构
    let result = assembler.set_architecture(Architecture::ARM64);
    assert!(result.is_err());
}

#[test]
fn test_add_sub_encoding() {
    let assembler = X86_64Assembler::new(Architecture::X86_64).unwrap();

    // 测试 ADD eax, ebx
    let add_instruction = Instruction::Add { dst: Operand::Reg(Register::EAX), src: Operand::Reg(Register::EBX) };

    let result = assembler.encode(&add_instruction);
    assert!(result.is_ok());

    // 测试 SUB eax, ebx
    let sub_instruction = Instruction::Sub { dst: Operand::Reg(Register::EAX), src: Operand::Reg(Register::EBX) };

    let result = assembler.encode(&sub_instruction);
    assert!(result.is_ok());
}

#[test]
fn test_call_encoding() {
    let assembler = X86_64Assembler::new(Architecture::X86_64).unwrap();

    // 测试 CALL label
    let call_instruction = Instruction::Call { target: Operand::Label("my_function".to_string()) };

    let result = assembler.encode(&call_instruction);
    assert!(result.is_ok());
}

#[test]
fn test_lea_encoding() {
    let assembler = X86_64Assembler::new(Architecture::X86_64).unwrap();

    // 测试 LEA rax, [rip + 0x100]
    let lea_instruction = Instruction::Lea { dst: Register::RAX, displacement: 0x100, rip_relative: true };

    let result = assembler.encode(&lea_instruction);
    assert!(result.is_ok());
}

#[test]
fn test_memory_operand_encoding() {
    let assembler = X86_64Assembler::new(Architecture::X86_64).unwrap();

    // 测试 MOV eax, [ebx]
    let instruction = Instruction::Mov {
        dst: Operand::Reg(Register::EAX),
        src: Operand::Mem { base: Some(Register::RBX), index: None, scale: 1, displacement: 0 },
    };

    let result = assembler.encode(&instruction);
    assert!(result.is_ok());
}

#[test]
fn test_decode_functionality() {
    let assembler = X86_64Assembler::new(Architecture::X86_64).unwrap();

    // 测试解码功能
    let bytes = vec![0xC3]; // RET 指令
    let result = assembler.decode(&bytes);
    assert!(result.is_ok());

    let instructions = result.unwrap();
    assert!(!instructions.is_empty());
}

#[test]
fn test_error_handling() {
    let assembler = X86_64Assembler::new(Architecture::X86_64).unwrap();

    // 测试无效指令组合 - POP 不支持立即数操作数
    let invalid_instruction = Instruction::Pop { dst: Operand::Imm { value: 42, size: 32 } };

    let result = assembler.encode(&invalid_instruction);
    assert!(result.is_err());
}
