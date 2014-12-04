use gaia_types::{helpers::Architecture, GaiaError};
use pe_assembler::{helpers::PeBuilder, types::SubsystemType};
use std::process::Command;
use x86_64_assembler::{
    builder::ProgramBuilder,
    instruction::{Instruction, Operand, Register},
};

fn main() -> Result<(), GaiaError> {
    let message = "Hello, world!\n";
    let exe = print(message)?;
    std::fs::write("hello_world.exe", exe)?;
    Command::new("hello_world.exe").status()?;
    Ok(())
}

fn print(message: &str) -> Result<Vec<u8>, GaiaError> {
    let mut gasm = ProgramBuilder::new(Architecture::X86_64);
    gasm.sub_reg_imm(Register::RSP, 40)?;
    gasm.mov_reg_imm(Register::RCX, -11)?;
    gasm.call_indirect(0)?;
    gasm.mov_reg_reg(Register::RCX, Register::RAX)?;
    gasm.add_instruction(Instruction::Lea { dst: Register::RDX, displacement: 0, rip_relative: true });
    let len = message.len() as i64;
    gasm.mov_reg_imm(Register::R8D, len)?;
    gasm.add_instruction(Instruction::Lea { dst: Register::R9, displacement: 0, rip_relative: true });
    gasm.mov_reg_imm(Register::RAX, 0)?;
    gasm.add_instruction(Instruction::Mov {
        dst: Operand::Mem { base: Some(Register::RSP), index: None, scale: 1, displacement: 32 },
        src: Operand::Reg(Register::RAX),
    });
    gasm.call_indirect(1)?;
    gasm.mov_reg_imm(Register::RCX, 0)?;
    gasm.call_indirect(2)?;
    gasm.add_reg_imm(Register::RSP, 40)?;
    gasm.ret()?;

    let code = gasm.compile_instructions()?;
    let mut data = Vec::new();
    data.extend_from_slice(message.as_bytes());
    data.push(0);
    let pad = (4 - (data.len() % 4)) % 4;
    for _ in 0..pad {
        data.push(0);
    }
    data.extend_from_slice(&[0u8; 4]);

    PeBuilder::new()
        .architecture(Architecture::X86_64)
        .subsystem(SubsystemType::Console)
        .entry_point(0x1000)
        .image_base(0x140000000)
        .import_functions("kernel32.dll", &["GetStdHandle", "WriteFile", "ExitProcess"])
        .code(code)
        .data(data)
        .generate()
}
