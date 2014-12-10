use elf_assembler::{types::ElfFile, writer::ElfWriter};
use gaia_types::helpers::Architecture;
use std::io::Cursor;
use x86_64_assembler::{
    builder::ProgramBuilder,
    instruction::{Instruction, Operand, Register},
};

#[test]
fn test_gaia_to_elf_flow() {
    // 1. 模拟 gaia-compiler 生成指令
    let mut builder = ProgramBuilder::new(Architecture::X86_64);

    // 生成简单的退出代码为 42 的汇编 (Linux syscall)
    // 暂时用 Ret 替代 Syscall，因为 x86_64-assembler 尚未实现 Syscall 指令
    builder.add_instruction(Instruction::Mov { dst: Operand::reg(Register::RAX), src: Operand::imm(60, 64) });
    builder.add_instruction(Instruction::Mov { dst: Operand::reg(Register::RDI), src: Operand::imm(42, 64) });
    builder.add_instruction(Instruction::Ret);

    let machine_code = builder.compile_instructions().expect("Failed to compile x86_64");

    // 2. 使用 project-gaia 的 elf-assembler 包装成 ELF
    let elf = ElfFile::create_executable(machine_code);

    // 3. 写入二进制
    let mut buffer = Cursor::new(Vec::new());
    let mut writer = ElfWriter::new(&mut buffer);
    writer.write_elf_file(&elf).expect("Failed to write ELF");

    let binary = buffer.into_inner();

    // 验证魔数
    assert_eq!(&binary[0..4], b"\x7fELF");
    println!("Successfully generated a {} byte ELF binary", binary.len());
}
