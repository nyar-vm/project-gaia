use elf_assembler::{types::ElfFile, writer::ElfWriter};
use gaia_types::helpers::Architecture;
use std::io::Cursor;
use x86_64_assembler::{
    builder::ProgramBuilder,
    instruction::{Instruction, Operand, Register},
};

#[test]
fn test_hello_world_elf() {
    // 模拟生成打印 "Hello" 并退出的 ELF
    let mut builder = ProgramBuilder::new(Architecture::X86_64);

    // 这里由于 x86_64-assembler 可能还不支持所有指令（如 syscall）
    // 我们主要测试 ELF 的封装逻辑
    
    // mov rax, 1 (write syscall)
    builder.add_instruction(Instruction::Mov { 
        dst: Operand::reg(Register::RAX), 
        src: Operand::imm(1, 64) 
    });
    
    // 暂时用 Ret 结尾
    builder.add_instruction(Instruction::Ret);

    let machine_code = builder.compile_instructions().expect("Failed to compile x86_64");

    let elf = ElfFile::create_executable(machine_code);

    let mut buffer = Cursor::new(Vec::new());
    let mut writer = ElfWriter::new(&mut buffer);
    writer.write_elf_file(&elf).expect("Failed to write ELF");

    let binary = buffer.into_inner();

    // 验证 ELF 头部
    assert_eq!(&binary[0..4], b"\x7fELF");
    assert!(binary.len() > 64); // 至少包含头部
}
