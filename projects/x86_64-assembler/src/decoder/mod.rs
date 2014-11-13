#![doc = include_str!("readme.md")]
use crate::instruction::{Instruction, Operand, Register};
use gaia_types::{helpers::Architecture, GaiaError, Result};
/// 指令解码器，用于将字节码解码为指令
#[derive(Debug, Clone)]
pub struct InstructionDecoder {
    architecture: Architecture,
}

impl InstructionDecoder {
    /// 创建新的指令解码器
    pub fn new(architecture: Architecture) -> Self {
        Self { architecture }
    }

    /// 解码字节码为指令序列
    pub fn decode(&self, bytes: &[u8]) -> Result<Vec<Instruction>> {
        let mut instructions = Vec::new();
        let mut offset = 0;

        while offset < bytes.len() {
            match self.decode_instruction(&bytes[offset..]) {
                Ok((instruction, size)) => {
                    instructions.push(instruction);
                    offset += size;
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        Ok(instructions)
    }

    fn decode_instruction(&self, bytes: &[u8]) -> Result<(Instruction, usize)> {
        if bytes.is_empty() {
            return Err(GaiaError::invalid_data("Empty instruction bytes"));
        }

        let mut offset = 0;
        let mut has_rex = false;
        let mut rex_prefix = 0u8;
        let mut prefix_size = 0;

        // Check for REX prefix (64-bit mode)
        if self.architecture == Architecture::X86_64 && (bytes[0] & 0xF0) == 0x40 {
            has_rex = true;
            rex_prefix = bytes[0];
            offset += 1;
            prefix_size += 1;
        }

        let opcode = bytes[offset];
        offset += 1;

        let (instruction, size) = match opcode {
            0x89 => self.decode_mov_reg_reg(bytes, offset, has_rex, rex_prefix),
            0x8B => self.decode_mov_reg_mem(bytes, offset, has_rex, rex_prefix),
            0x50..=0x57 => self.decode_push_reg(bytes, offset - 1, has_rex, rex_prefix),
            0x58..=0x5F => self.decode_pop_reg(bytes, offset - 1, has_rex, rex_prefix),
            0x01 => self.decode_add_reg_reg(bytes, offset, has_rex, rex_prefix),
            0x29 => self.decode_sub_reg_reg(bytes, offset, has_rex, rex_prefix),
            0x6A => self.decode_push_imm8(bytes, offset),
            0x68 => self.decode_push_imm32(bytes, offset),
            0xB8..=0xBF => self.decode_mov_reg_imm(bytes, offset - 1, has_rex, rex_prefix),
            0xC3 => self.decode_ret(bytes, offset),
            0x90 => self.decode_nop(bytes, offset),
            _ => Err(GaiaError::invalid_instruction(format!("Unknown opcode: 0x{:02X}", opcode), self.architecture.clone())),
        }?;

        Ok((instruction, prefix_size + size))
    }

    fn decode_mov_reg_reg(&self, bytes: &[u8], offset: usize, has_rex: bool, rex_prefix: u8) -> Result<(Instruction, usize)> {
        if offset >= bytes.len() {
            return Err(GaiaError::invalid_data("Incomplete MOV reg,reg instruction"));
        }

        let modrm = bytes[offset];
        let mod_bits = (modrm >> 6) & 0x03;
        let reg = (modrm >> 3) & 0x07;
        let rm = modrm & 0x07;

        if mod_bits != 3 {
            return Err(GaiaError::invalid_instruction(
                "Expected register-register MOV".to_string(),
                self.architecture.clone(),
            ));
        }

        let dest_reg = self.decode_register(rm, has_rex, rex_prefix, false)?;
        let src_reg = self.decode_register(reg, has_rex, rex_prefix, true)?;

        let instruction = Instruction::Mov { dst: Operand::Reg(dest_reg), src: Operand::Reg(src_reg) };
        Ok((instruction, offset + 1))
    }

    fn decode_mov_reg_mem(&self, bytes: &[u8], offset: usize, has_rex: bool, rex_prefix: u8) -> Result<(Instruction, usize)> {
        if offset >= bytes.len() {
            return Err(GaiaError::invalid_data("Incomplete MOV reg,mem instruction"));
        }

        let modrm = bytes[offset];
        let mod_bits = (modrm >> 6) & 0x03;
        let reg = (modrm >> 3) & 0x07;
        let rm = modrm & 0x07;

        let dest_reg = self.decode_register(reg, has_rex, rex_prefix, false)?;

        // Simplified memory operand decoding
        if mod_bits == 0 && rm == 0x05 {
            // [disp32]
            if offset + 4 >= bytes.len() {
                return Err(GaiaError::invalid_data("Incomplete displacement"));
            }
            let displacement = i32::from_le_bytes([bytes[offset + 1], bytes[offset + 2], bytes[offset + 3], bytes[offset + 4]]);
            let instruction = Instruction::Mov {
                dst: Operand::Reg(dest_reg),
                src: Operand::Mem { base: None, index: None, scale: 1, displacement },
            };
            Ok((instruction, offset + 5))
        }
        else if mod_bits == 1 {
            // [reg + disp8]
            if offset + 1 >= bytes.len() {
                return Err(GaiaError::invalid_data("Incomplete displacement"));
            }
            let displacement = bytes[offset + 1] as i8 as i32;
            let base_reg = self.decode_register(rm, has_rex, rex_prefix, false)?;
            let instruction = Instruction::Mov {
                dst: Operand::Reg(dest_reg),
                src: Operand::Mem { base: Some(base_reg), index: None, scale: 1, displacement },
            };
            Ok((instruction, offset + 2))
        }
        else if mod_bits == 2 {
            // [reg + disp32]
            if offset + 4 >= bytes.len() {
                return Err(GaiaError::invalid_data("Incomplete displacement"));
            }
            let displacement = i32::from_le_bytes([bytes[offset + 1], bytes[offset + 2], bytes[offset + 3], bytes[offset + 4]]);
            let base_reg = self.decode_register(rm, has_rex, rex_prefix, false)?;
            let instruction = Instruction::Mov {
                dst: Operand::Reg(dest_reg),
                src: Operand::Mem { base: Some(base_reg), index: None, scale: 1, displacement },
            };
            Ok((instruction, offset + 5))
        }
        else if mod_bits == 3 {
            // Register-register (should not happen for 0x8B)
            Err(GaiaError::invalid_instruction(
                "Invalid addressing mode for MOV reg,mem".to_string(),
                self.architecture.clone(),
            ))
        }
        else {
            Err(GaiaError::invalid_instruction("Complex addressing mode not supported".to_string(), self.architecture.clone()))
        }
    }

    fn decode_push_reg(&self, bytes: &[u8], offset: usize, has_rex: bool, rex_prefix: u8) -> Result<(Instruction, usize)> {
        let opcode = bytes[offset];
        let reg_code = opcode & 0x07;

        let reg = self.decode_register(reg_code, has_rex, rex_prefix, false)?;
        let instruction = Instruction::Push { op: Operand::Reg(reg) };
        Ok((instruction, offset + 1))
    }

    fn decode_pop_reg(&self, bytes: &[u8], offset: usize, has_rex: bool, rex_prefix: u8) -> Result<(Instruction, usize)> {
        let opcode = bytes[offset];
        let reg_code = opcode & 0x07;

        let reg = self.decode_register(reg_code, has_rex, rex_prefix, false)?;
        let instruction = Instruction::Pop { dst: Operand::Reg(reg) };
        Ok((instruction, offset + 1))
    }

    fn decode_add_reg_reg(&self, bytes: &[u8], offset: usize, has_rex: bool, rex_prefix: u8) -> Result<(Instruction, usize)> {
        if offset >= bytes.len() {
            return Err(GaiaError::invalid_data("Incomplete ADD reg,reg instruction"));
        }

        let modrm = bytes[offset];
        let mod_bits = (modrm >> 6) & 0x03;
        let reg = (modrm >> 3) & 0x07;
        let rm = modrm & 0x07;

        if mod_bits != 3 {
            return Err(GaiaError::invalid_instruction(
                "Expected register-register ADD".to_string(),
                self.architecture.clone(),
            ));
        }

        let dest_reg = self.decode_register(rm, has_rex, rex_prefix, false)?;
        let src_reg = self.decode_register(reg, has_rex, rex_prefix, true)?;

        let instruction = Instruction::Add { dst: Operand::Reg(dest_reg), src: Operand::Reg(src_reg) };
        Ok((instruction, offset + 1))
    }

    fn decode_sub_reg_reg(&self, bytes: &[u8], offset: usize, has_rex: bool, rex_prefix: u8) -> Result<(Instruction, usize)> {
        if offset >= bytes.len() {
            return Err(GaiaError::invalid_data("Incomplete SUB reg,reg instruction"));
        }

        let modrm = bytes[offset];
        let mod_bits = (modrm >> 6) & 0x03;
        let reg = (modrm >> 3) & 0x07;
        let rm = modrm & 0x07;

        if mod_bits != 3 {
            return Err(GaiaError::invalid_instruction(
                "Expected register-register SUB".to_string(),
                self.architecture.clone(),
            ));
        }

        let dest_reg = self.decode_register(rm, has_rex, rex_prefix, false)?;
        let src_reg = self.decode_register(reg, has_rex, rex_prefix, true)?;

        let instruction = Instruction::Sub { dst: Operand::Reg(dest_reg), src: Operand::Reg(src_reg) };
        Ok((instruction, offset + 1))
    }

    fn decode_push_imm8(&self, bytes: &[u8], offset: usize) -> Result<(Instruction, usize)> {
        if offset >= bytes.len() {
            return Err(GaiaError::invalid_data("Incomplete PUSH imm8 instruction"));
        }

        let imm = bytes[offset] as i8 as i64;
        let instruction = Instruction::Push { op: Operand::Imm { value: imm, size: 8 } };
        Ok((instruction, offset + 1))
    }

    fn decode_push_imm32(&self, bytes: &[u8], offset: usize) -> Result<(Instruction, usize)> {
        if offset + 3 >= bytes.len() {
            return Err(GaiaError::invalid_data("Incomplete PUSH imm32 instruction"));
        }

        let imm = i32::from_le_bytes([bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]]) as i64;
        let instruction = Instruction::Push { op: Operand::Imm { value: imm, size: 32 } };
        Ok((instruction, offset + 4))
    }

    fn decode_mov_reg_imm(&self, bytes: &[u8], offset: usize, has_rex: bool, rex_prefix: u8) -> Result<(Instruction, usize)> {
        let opcode = bytes[offset];
        let reg_code = opcode & 0x07;

        let dest_reg = self.decode_register(reg_code, has_rex, rex_prefix, false)?;

        // Determine immediate size based on architecture
        let imm_size = if self.architecture == Architecture::X86_64 && has_rex { 64 } else { 32 };
        let imm_bytes = if imm_size == 64 { 8 } else { 4 };

        if offset + 1 + imm_bytes - 1 >= bytes.len() {
            return Err(GaiaError::invalid_data("Incomplete MOV reg,imm instruction"));
        }

        let imm = if imm_size == 64 {
            i64::from_le_bytes([
                bytes[offset + 1],
                bytes[offset + 2],
                bytes[offset + 3],
                bytes[offset + 4],
                bytes[offset + 5],
                bytes[offset + 6],
                bytes[offset + 7],
                bytes[offset + 8],
            ])
        }
        else {
            i32::from_le_bytes([bytes[offset + 1], bytes[offset + 2], bytes[offset + 3], bytes[offset + 4]]) as i64
        };

        let instruction =
            Instruction::Mov { dst: Operand::Reg(dest_reg), src: Operand::Imm { value: imm, size: imm_size as u8 } };
        Ok((instruction, 1 + imm_bytes))
    }

    fn decode_ret(&self, _bytes: &[u8], offset: usize) -> Result<(Instruction, usize)> {
        Ok((Instruction::Ret, offset))
    }

    fn decode_nop(&self, _bytes: &[u8], offset: usize) -> Result<(Instruction, usize)> {
        Ok((Instruction::Nop, offset))
    }

    fn decode_register(&self, reg_code: u8, has_rex: bool, rex_prefix: u8, is_rex_r: bool) -> Result<Register> {
        let extended = if has_rex && is_rex_r {
            (rex_prefix & 0x04) != 0
        }
        else if has_rex {
            (rex_prefix & 0x01) != 0
        }
        else {
            false
        };

        let reg_index = if extended { reg_code + 8 } else { reg_code };

        match self.architecture {
            Architecture::X86 => match reg_index {
                0 => Ok(Register::EAX),
                1 => Ok(Register::ECX),
                2 => Ok(Register::EDX),
                3 => Ok(Register::EBX),
                4 => Ok(Register::ESP),
                5 => Ok(Register::EBP),
                6 => Ok(Register::ESI),
                7 => Ok(Register::EDI),
                _ => Err(GaiaError::invalid_instruction(
                    format!("Invalid register code for X86: {}", reg_index),
                    self.architecture.clone(),
                )),
            },
            Architecture::X86_64 => match reg_index {
                0 => Ok(Register::RAX),
                1 => Ok(Register::RCX),
                2 => Ok(Register::RDX),
                3 => Ok(Register::RBX),
                4 => Ok(Register::RSP),
                5 => Ok(Register::RBP),
                6 => Ok(Register::RSI),
                7 => Ok(Register::RDI),
                8 => Ok(Register::R8),
                9 => Ok(Register::R9),
                10 => Ok(Register::R10),
                11 => Ok(Register::R11),
                12 => Ok(Register::R12),
                13 => Ok(Register::R13),
                14 => Ok(Register::R14),
                15 => Ok(Register::R15),
                _ => Err(GaiaError::invalid_instruction(
                    format!("Invalid register code for X86_64: {}", reg_index),
                    self.architecture.clone(),
                )),
            },
            _ => Err(GaiaError::unsupported_architecture(self.architecture.clone())),
        }
    }
}
