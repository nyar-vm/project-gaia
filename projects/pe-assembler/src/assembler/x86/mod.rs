use crate::assembler::EncodedInstruction;
use gaia_types::GaiaError;

// 定义 X86 指令类型
#[derive(Debug, Clone)]
pub struct X86Instruction {
    pub opcode: String,
    pub operands: Vec<String>,
}

pub fn encode_x86(instruction: X86Instruction) -> Result<EncodedInstruction, GaiaError> {
    todo!("实现 X86 指令编码")
}

pub fn decode_x86(data: Vec<u8>) -> Result<X86Instruction, GaiaError> {
    todo!("实现 X86 指令解码")
}
