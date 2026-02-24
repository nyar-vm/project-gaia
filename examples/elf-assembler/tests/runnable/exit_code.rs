use elf_assembler::{types::ElfFile, writer::ElfWriter};
use gaia_types::helpers::Architecture;
use std::io::Cursor;
use x86_64_assembler::{
    builder::ProgramBuilder,
    instruction::{Instruction, Operand, Register},
};

#[test]
fn test_exit_code_elf() {
    // 模拟生成退出代码为 42 的 ELF
    let mut builder = ProgramBuilder::new(Architecture::X86_64);

    // mov rax, 60 (exit syscall)
    builder.add_instruction(Instruction::Mov { dst: Operand::reg(Register::RAX), src: Operand::imm(60, 64) });
    // mov rdi, 42 (exit code)
    builder.add_instruction(Instruction::Mov { dst: Operand::reg(Register::RDI), src: Operand::imm(42, 64) });
    // 暂时用 Ret，因为 x86_64-assembler 尚未实现 Syscall
    builder.add_instruction(Instruction::Ret);

    let machine_code = builder.compile_instructions().expect("Failed to compile x86_64");

    let elf = ElfFile::create_executable(machine_code);

    let mut buffer = Cursor::new(Vec::new());
    let mut writer = ElfWriter::new(&mut buffer);
    writer.write_elf_file(&elf).expect("Failed to write ELF");

    let binary = buffer.into_inner();

    // 验证 ELF 魔数
    assert_eq!(&binary[0..4], b"\x7fELF");
    // 验证是 64 位 ELF
    assert_eq!(binary[4], 2);
}
