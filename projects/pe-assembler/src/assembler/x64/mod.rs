use crate::assembler::EncodedInstruction;
use gaia_types::GaiaError;

// 定义 X64 指令类型
#[derive(Debug, Clone)]
pub struct X64Instruction {
    pub opcode: String,
    pub operands: Vec<String>,
}

pub fn encode_x64(_instruction: X64Instruction) -> Result<EncodedInstruction, GaiaError> {
    todo!("实现 X64 指令编码")
}

pub fn decode_x64(_data: Vec<u8>) -> Result<X64Instruction, GaiaError> {
    todo!("实现 X64 指令解码")
}
