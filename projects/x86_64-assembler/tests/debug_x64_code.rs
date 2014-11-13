use gaia_types::helpers::Architecture;
use x86_64_assembler::{
    builder::ProgramBuilder,
    instruction::{Instruction, Operand, Register},
};

#[test]
fn main() {
    let mut program = ProgramBuilder::new(Architecture::X86_64);

    // 模拟 x64 版本的关键指令序列
    program.sub_reg_imm(Register::RSP, 40).unwrap();
    program.mov_reg_imm(Register::RAX, 0).unwrap();
    program.add_instruction(Instruction::Mov {
        dst: Operand::Mem { base: Some(Register::RSP), index: None, scale: 1, displacement: 32 },
        src: Operand::Reg(Register::RAX),
    });

    // GetStdHandle 调用
    program.mov_reg_imm(Register::RCX, -11).unwrap();
    program.call_indirect(0).unwrap();

    // WriteFile 参数设置
    program.mov_reg_reg(Register::RCX, Register::RAX).unwrap();

    // 关键的 lea 指令
    program.add_instruction(Instruction::Lea { dst: Register::RDX, displacement: 0, rip_relative: true });

    let len = 13i64; // "hello world\n" 长度
    program.add_instruction(Instruction::Mov { dst: Operand::Reg(Register::R8D), src: Operand::Imm { value: len, size: 32 } });

    program.add_instruction(Instruction::Lea { dst: Register::R9, displacement: 0, rip_relative: true });

    let code = program.compile_instructions().expect("Failed to compile x64 instructions");

    println!("Generated x64 code:");
    for (i, byte) in code.iter().enumerate() {
        print!("{:02x} ", byte);
        if (i + 1) % 16 == 0 {
            println!();
        }
    }
    println!();

    // 分析 lea 指令的位置
    let mut i = 0;
    while i < code.len() {
        if i + 6 < code.len() && code[i] == 0x48 && code[i + 1] == 0x8D && code[i + 2] == 0x15 {
            println!(
                "Found lea rdx, [rip+disp32] at offset {}: {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x}",
                i,
                code[i],
                code[i + 1],
                code[i + 2],
                code[i + 3],
                code[i + 4],
                code[i + 5],
                code[i + 6]
            );
            i += 7;
        }
        else if i + 6 < code.len() && code[i] == 0x4C && code[i + 1] == 0x8D && code[i + 2] == 0x0D {
            println!(
                "Found lea r9, [rip+disp32] at offset {}: {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x}",
                i,
                code[i],
                code[i + 1],
                code[i + 2],
                code[i + 3],
                code[i + 4],
                code[i + 5],
                code[i + 6]
            );
            i += 7;
        }
        else {
            i += 1;
        }
    }
}
