use gaia_types::{helpers::Architecture, GaiaError};
use x86_64_assembler::{builder::ProgramBuilder, instruction::Register};

#[test]
fn main() -> Result<(), GaiaError> {
    let mut program = ProgramBuilder::new(Architecture::X86);
    let message = "Hello, World!";
    let msg_len = message.len() as i64;

    // 模拟 hello_world.rs 中的代码序列
    program
        .push_imm(msg_len) // 记录长度，供修补器识别后续的 msg 占位
        .unwrap()
        .push_label("msg".to_string()) // msg 的占位，将被修补为 .data 起始地址
        .unwrap()
        .pop_reg(Register::EBX) // EBX = &msg
        .unwrap()
        .pop_reg(Register::ECX) // 弹出用于修补的 msg_len，占位不再作为参数
        .unwrap();

    let code = program.compile_instructions().expect("Failed to compile x86 instructions");

    println!("Generated x86 code ({} bytes):", code.len());
    for (i, byte) in code.iter().enumerate() {
        if i % 16 == 0 {
            print!("\n{:04x}: ", i);
        }
        print!("{:02x} ", byte);
    }
    println!();

    // 分析指令
    println!("\nInstruction analysis:");
    let mut i = 0;
    while i < code.len() {
        match code[i] {
            0x68 => {
                if i + 4 < code.len() {
                    let imm = u32::from_le_bytes([code[i + 1], code[i + 2], code[i + 3], code[i + 4]]);
                    println!("  {:04x}: push 0x{:08x}", i, imm);
                    i += 5;
                }
                else {
                    println!("  {:04x}: push (incomplete)", i);
                    i += 1;
                }
            }
            0x5b => {
                println!("  {:04x}: pop ebx", i);
                i += 1;
            }
            0x59 => {
                println!("  {:04x}: pop ecx", i);
                i += 1;
            }
            _ => {
                println!("  {:04x}: 0x{:02x}", i, code[i]);
                i += 1;
            }
        }
    }

    Ok(())
}
