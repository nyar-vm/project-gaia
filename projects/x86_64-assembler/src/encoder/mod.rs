#![doc = include_str!("readme.md")]

use crate::instruction::{Instruction, Operand, Register};
use gaia_types::{helpers::Architecture, GaiaError, Result};

/// 指令编码器，用于将指令编码为字节码
#[derive(Debug, Clone)]
pub struct InstructionEncoder {
    architecture: Architecture,
}

impl InstructionEncoder {
    /// 创建新的指令编码器
    pub fn new(architecture: Architecture) -> Self {
        Self { architecture }
    }

    /// 编码指令为字节码
    pub fn encode(&self, instruction: &Instruction) -> Result<Vec<u8>> {
        match instruction {
            Instruction::Mov { dst, src } => self.encode_mov(dst, src),
            Instruction::Push { op } => self.encode_push(op),
            Instruction::Pop { dst } => self.encode_pop(dst),
            Instruction::Add { dst, src } => self.encode_add(dst, src),
            Instruction::Sub { dst, src } => self.encode_sub(dst, src),
            Instruction::Call { target } => self.encode_call(target),
            Instruction::Lea { dst, displacement, rip_relative } => self.encode_lea(dst, *displacement, *rip_relative),
            Instruction::Ret => Ok(vec![0xC3]),
            Instruction::Nop => Ok(vec![0x90]),
        }
    }

    fn encode_mov(&self, dest: &Operand, src: &Operand) -> Result<Vec<u8>> {
        match (dest, src) {
            (Operand::Reg(d), Operand::Reg(s)) => self.encode_mov_reg_reg(d, s),
            (Operand::Reg(d), Operand::Imm { value, size }) => self.encode_mov_reg_imm(d, *value, *size),
            (Operand::Mem { base, index, scale, displacement }, Operand::Reg(s)) => {
                self.encode_mov_mem_reg(base, index, *scale, *displacement, s)
            }
            (Operand::Reg(d), Operand::Mem { base, index, scale, displacement }) => {
                self.encode_mov_reg_mem(d, base, index, *scale, *displacement)
            }
            _ => Err(GaiaError::invalid_instruction(
                "Invalid operand combination for MOV".to_string(),
                self.architecture.clone(),
            )),
        }
    }

    fn encode_mov_reg_reg(&self, dest_reg: &Register, src_reg: &Register) -> Result<Vec<u8>> {
        let mut result = Vec::new();
        match self.architecture {
            Architecture::X86 => {
                result.push(0x89);
                result.push(self.encode_modrm(3, self.encode_register(src_reg)?, self.encode_register(dest_reg)?));
            }
            Architecture::X86_64 => {
                let rex = self.compose_rex(true, self.is_ext(src_reg), false, self.is_ext(dest_reg));
                if rex != 0 {
                    result.push(rex);
                }
                result.push(0x89);
                result.push(self.encode_modrm(3, self.encode_register(src_reg)?, self.encode_register(dest_reg)?));
            }
            _ => return Err(GaiaError::unsupported_architecture(self.architecture.clone())),
        }
        Ok(result)
    }

    fn encode_mov_reg_imm(&self, dest_reg: &Register, imm: i64, size: u8) -> Result<Vec<u8>> {
        let mut result = Vec::new();
        match self.architecture {
            Architecture::X86 => {
                let reg_code = self.encode_register(dest_reg)?;
                result.push(0xB8 + reg_code);
                result.extend_from_slice(&(imm as u32).to_le_bytes());
            }
            Architecture::X86_64 => {
                let reg_code = self.encode_register(dest_reg)?;
                let is64 = self.is_64(dest_reg) && size == 64;

                // 对于 32 位寄存器（如 ECX），不应该使用 REX 前缀
                let use_rex = is64 || self.is_ext(dest_reg);

                if use_rex {
                    let rex = self.compose_rex(is64, false, false, self.is_ext(dest_reg));
                    result.push(rex);
                }

                result.push(0xB8 + reg_code);
                if is64 {
                    result.extend_from_slice(&(imm as u64).to_le_bytes());
                }
                else {
                    result.extend_from_slice(&(imm as u32).to_le_bytes());
                }
            }
            _ => return Err(GaiaError::unsupported_architecture(self.architecture.clone())),
        }
        Ok(result)
    }

    fn encode_mov_mem_reg(
        &self,
        base: &Option<Register>,
        index: &Option<Register>,
        scale: u8,
        displacement: i32,
        src_reg: &Register,
    ) -> Result<Vec<u8>> {
        let mut result = Vec::new();
        match self.architecture {
            Architecture::X86 => {
                result.push(0x89);
            }
            Architecture::X86_64 => {
                let rex =
                    self.compose_rex(true, self.is_ext(src_reg), false, base.as_ref().map(|b| self.is_ext(b)).unwrap_or(false));
                if rex != 0 {
                    result.push(rex);
                }
                result.push(0x89);
            }
            _ => return Err(GaiaError::unsupported_architecture(self.architecture.clone())),
        }
        let (modrm, sib_bytes, disp_bytes) =
            self.encode_memory_operand(base, index, &scale, &displacement, self.encode_register(src_reg)?)?;
        result.push(modrm);
        result.extend(sib_bytes);
        result.extend(disp_bytes);
        Ok(result)
    }

    fn encode_mov_reg_mem(
        &self,
        dest_reg: &Register,
        base: &Option<Register>,
        index: &Option<Register>,
        scale: u8,
        displacement: i32,
    ) -> Result<Vec<u8>> {
        let mut result = Vec::new();
        match self.architecture {
            Architecture::X86 => {
                result.push(0x8B);
            }
            Architecture::X86_64 => {
                let rex = self.compose_rex(
                    true,
                    self.is_ext(dest_reg),
                    false,
                    base.as_ref().map(|b| self.is_ext(b)).unwrap_or(false),
                );
                if rex != 0 {
                    result.push(rex);
                }
                result.push(0x8B);
            }
            _ => return Err(GaiaError::unsupported_architecture(self.architecture.clone())),
        }
        let (modrm, sib_bytes, disp_bytes) =
            self.encode_memory_operand(base, index, &scale, &displacement, self.encode_register(dest_reg)?)?;
        result.push(modrm);
        result.extend(sib_bytes);
        result.extend(disp_bytes);
        Ok(result)
    }

    fn encode_push(&self, operand: &Operand) -> Result<Vec<u8>> {
        let mut result = Vec::new();
        match operand {
            Operand::Reg(reg) => {
                if self.architecture == Architecture::X86_64 && self.is_ext(reg) {
                    result.push(self.compose_rex(false, false, false, true));
                }
                let reg_code = self.encode_register(reg)?;
                result.push(0x50 + reg_code);
            }
            Operand::Imm { value, .. } => {
                if *value >= i8::MIN as i64 && *value <= i8::MAX as i64 {
                    result.push(0x6A);
                    result.push(*value as u8);
                }
                else {
                    result.push(0x68);
                    result.extend_from_slice(&(*value as u32).to_le_bytes());
                }
            }
            Operand::Label(_) => {
                result.push(0x68);
                result.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
            }
            _ => return Err(GaiaError::invalid_instruction("Invalid operand for PUSH".to_string(), self.architecture.clone())),
        }
        Ok(result)
    }

    fn encode_pop(&self, operand: &Operand) -> Result<Vec<u8>> {
        let mut result = Vec::new();
        match operand {
            Operand::Reg(reg) => {
                if self.architecture == Architecture::X86_64 && self.is_ext(reg) {
                    result.push(self.compose_rex(false, false, false, true));
                }
                let reg_code = self.encode_register(reg)?;
                result.push(0x58 + reg_code);
            }
            _ => return Err(GaiaError::invalid_instruction("Invalid operand for POP".to_string(), self.architecture.clone())),
        }
        Ok(result)
    }

    fn encode_add(&self, dest: &Operand, src: &Operand) -> Result<Vec<u8>> {
        let mut result = Vec::new();
        match (dest, src) {
            (Operand::Reg(d), Operand::Reg(s)) => {
                match self.architecture {
                    Architecture::X86 => {
                        result.push(0x01);
                    }
                    Architecture::X86_64 => {
                        let rex = self.compose_rex(true, self.is_ext(s), false, self.is_ext(d));
                        if rex != 0 {
                            result.push(rex);
                        }
                        result.push(0x01);
                    }
                    _ => return Err(GaiaError::unsupported_architecture(self.architecture.clone())),
                }
                result.push(self.encode_modrm(3, self.encode_register(s)?, self.encode_register(d)?));
            }
            (Operand::Reg(d), Operand::Imm { value, .. }) => {
                match self.architecture {
                    Architecture::X86 => {
                        result.push(0x81);
                        result.push(self.encode_modrm(3, 0, self.encode_register(d)?));
                    }
                    Architecture::X86_64 => {
                        let rex = self.compose_rex(true, false, false, self.is_ext(d));
                        if rex != 0 {
                            result.push(rex);
                        }
                        result.push(0x81);
                        result.push(self.encode_modrm(3, 0, self.encode_register(d)?));
                    }
                    _ => return Err(GaiaError::unsupported_architecture(self.architecture.clone())),
                }
                result.extend_from_slice(&(*value as u32).to_le_bytes());
            }
            _ => {
                return Err(GaiaError::invalid_instruction(
                    "Invalid operand combination for ADD".to_string(),
                    self.architecture.clone(),
                ))
            }
        }
        Ok(result)
    }

    fn encode_sub(&self, dest: &Operand, src: &Operand) -> Result<Vec<u8>> {
        let mut result = Vec::new();
        match (dest, src) {
            (Operand::Reg(d), Operand::Reg(s)) => {
                match self.architecture {
                    Architecture::X86 => {
                        result.push(0x29);
                    }
                    Architecture::X86_64 => {
                        let rex = self.compose_rex(true, self.is_ext(s), false, self.is_ext(d));
                        if rex != 0 {
                            result.push(rex);
                        }
                        result.push(0x29);
                    }
                    _ => return Err(GaiaError::unsupported_architecture(self.architecture.clone())),
                }
                result.push(self.encode_modrm(3, self.encode_register(s)?, self.encode_register(d)?));
            }
            (Operand::Reg(d), Operand::Imm { value, .. }) => {
                match self.architecture {
                    Architecture::X86 => {
                        result.push(0x81);
                        result.push(self.encode_modrm(3, 5, self.encode_register(d)?));
                    }
                    Architecture::X86_64 => {
                        let rex = self.compose_rex(true, false, false, self.is_ext(d));
                        if rex != 0 {
                            result.push(rex);
                        }
                        result.push(0x81);
                        result.push(self.encode_modrm(3, 5, self.encode_register(d)?));
                    }
                    _ => return Err(GaiaError::unsupported_architecture(self.architecture.clone())),
                }
                result.extend_from_slice(&(*value as u32).to_le_bytes());
            }
            _ => {
                return Err(GaiaError::invalid_instruction(
                    "Invalid operand combination for SUB".to_string(),
                    self.architecture.clone(),
                ))
            }
        }
        Ok(result)
    }

    fn encode_call(&self, operand: &Operand) -> Result<Vec<u8>> {
        let mut result = Vec::new();
        match operand {
            Operand::Label(_) => {
                result.push(0xE8);
                result.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
            }
            Operand::Reg(reg) => {
                if self.architecture == Architecture::X86_64 && self.is_ext(reg) {
                    result.push(self.compose_rex(false, true, false, false));
                }
                result.push(0xFF);
                let reg_code = self.encode_register(reg)?;
                result.push(self.encode_modrm(3, 2, reg_code));
            }
            Operand::Mem { base, index, scale: _scale, displacement } => {
                if base.is_none() && index.is_none() {
                    result.push(0xFF);
                    result.push(0x15);
                    result.extend_from_slice(&displacement.to_le_bytes());
                }
                else {
                    return Err(GaiaError::not_implemented("Complex CALL memory operand not implemented".to_string()));
                }
            }
            _ => return Err(GaiaError::invalid_instruction("Invalid operand for CALL".to_string(), self.architecture.clone())),
        }
        Ok(result)
    }

    fn encode_lea(&self, dest_reg: &Register, displacement: i32, rip_relative: bool) -> Result<Vec<u8>> {
        let mut result = Vec::new();
        match self.architecture {
            Architecture::X86_64 => {
                let reg_code = self.encode_register(dest_reg)?;

                if rip_relative {
                    // 为了匹配修补器的期望模式：
                    // lea rdx, [rip+disp32] -> 0x48 0x8D 0x15 (rdx = 2, ModR/M = 0x15)
                    // lea r9, [rip+disp32] -> 0x4C 0x8D 0x0D (r9 = 1, ModR/M = 0x0D)
                    if self.is_ext(dest_reg) {
                        // 扩展寄存器 (R8-R15)
                        result.push(0x4C); // REX.R = 1
                        result.push(0x8D);
                        // ModR/M: mod=00, reg=(reg_code&7), rm=101 (RIP-relative)
                        // 对于 R9: reg_code=1, ModR/M = 00_001_101 = 0x0D
                        let modrm = (0 << 6) | ((reg_code & 7) << 3) | 5;
                        result.push(modrm);
                    }
                    else {
                        // 标准寄存器 (RAX-RDI)
                        result.push(0x48); // REX.W = 1
                        result.push(0x8D);
                        // ModR/M: mod=00, reg=reg_code, rm=101 (RIP-relative)
                        // 对于 RDX: reg_code=2, ModR/M = 00_010_101 = 0x15
                        let modrm = (0 << 6) | (reg_code << 3) | 5;
                        result.push(modrm);
                    }
                }
                else {
                    // 非 RIP-relative 的 LEA
                    let rex_byte = if self.is_ext(dest_reg) { 0x4C } else { 0x48 };
                    result.push(rex_byte);
                    result.push(0x8D);
                    let modrm = self.encode_modrm(0, reg_code, 5);
                    result.push(modrm);
                }
                result.extend_from_slice(&(displacement as u32).to_le_bytes());
            }
            Architecture::X86 => {
                let reg_code = self.encode_register(dest_reg)?;
                result.push(0x8D);
                let modrm = self.encode_modrm(0, reg_code, 5);
                result.push(modrm);
                result.extend_from_slice(&(displacement as u32).to_le_bytes());
            }
            _ => return Err(GaiaError::unsupported_architecture(self.architecture.clone())),
        }
        Ok(result)
    }

    fn encode_register(&self, reg: &Register) -> Result<u8> {
        match reg {
            Register::EAX | Register::RAX => Ok(0),
            Register::ECX | Register::RCX => Ok(1),
            Register::EDX | Register::RDX => Ok(2),
            Register::EBX | Register::RBX => Ok(3),
            Register::ESP | Register::RSP => Ok(4),
            Register::EBP | Register::RBP => Ok(5),
            Register::ESI | Register::RSI => Ok(6),
            Register::EDI | Register::RDI => Ok(7),
            Register::R8 | Register::R8D | Register::R8W | Register::R8B => Ok(0),
            Register::R9 | Register::R9D | Register::R9W | Register::R9B => Ok(1),
            Register::R10 | Register::R10D | Register::R10W | Register::R10B => Ok(2),
            Register::R11 | Register::R11D | Register::R11W | Register::R11B => Ok(3),
            Register::R12 | Register::R12D | Register::R12W | Register::R12B => Ok(4),
            Register::R13 | Register::R13D | Register::R13W | Register::R13B => Ok(5),
            Register::R14 | Register::R14D | Register::R14W | Register::R14B => Ok(6),
            Register::R15 | Register::R15D | Register::R15W | Register::R15B => Ok(7),
            _ => Err(GaiaError::not_implemented(format!("Register encoding for {:?} not implemented", reg))),
        }
    }

    fn encode_modrm(&self, mod_bits: u8, reg: u8, rm: u8) -> u8 {
        (mod_bits << 6) | (reg << 3) | rm
    }

    fn encode_memory_operand(
        &self,
        base: &Option<Register>,
        index: &Option<Register>,
        _scale: &u8,
        displacement: &i32,
        reg: u8,
    ) -> Result<(u8, Vec<u8>, Vec<u8>)> {
        let mut sib_bytes = Vec::new();
        let mut disp_bytes = Vec::new();
        match (base, index) {
            (Some(base_reg), None) => {
                let base_code = self.encode_register(base_reg)?;
                // 使用 RSP/R12 作为基址时需要 SIB（rm=4，index=4 表示无索引）
                let use_sib = base_code == 4;
                if *displacement == 0 {
                    let rm = if use_sib { 4 } else { base_code };
                    let modrm = self.encode_modrm(0, reg, rm);
                    if use_sib {
                        sib_bytes.push(((0 & 0x3) << 6) | ((4 & 0x7) << 3) | (base_code & 0x7));
                    }
                    Ok((modrm, sib_bytes, disp_bytes))
                }
                else if *displacement >= -128 && *displacement <= 127 {
                    let rm = if use_sib { 4 } else { base_code };
                    let modrm = self.encode_modrm(1, reg, rm);
                    if use_sib {
                        sib_bytes.push(((0 & 0x3) << 6) | ((4 & 0x7) << 3) | (base_code & 0x7));
                    }
                    disp_bytes.push(*displacement as u8);
                    Ok((modrm, sib_bytes, disp_bytes))
                }
                else {
                    let rm = if use_sib { 4 } else { base_code };
                    let modrm = self.encode_modrm(2, reg, rm);
                    if use_sib {
                        sib_bytes.push(((0 & 0x3) << 6) | ((4 & 0x7) << 3) | (base_code & 0x7));
                    }
                    disp_bytes.extend_from_slice(&displacement.to_le_bytes());
                    Ok((modrm, sib_bytes, disp_bytes))
                }
            }
            (None, None) => {
                let modrm = self.encode_modrm(0, reg, 5);
                disp_bytes.extend_from_slice(&displacement.to_le_bytes());
                Ok((modrm, sib_bytes, disp_bytes))
            }
            _ => Err(GaiaError::not_implemented("Complex memory operand encoding not yet implemented".to_string())),
        }
    }

    fn compose_rex(&self, w: bool, r: bool, x: bool, b: bool) -> u8 {
        if self.architecture != Architecture::X86_64 {
            return 0;
        }
        let mut rex = 0x40;
        if w {
            rex |= 0x08;
        }
        if r {
            rex |= 0x04;
        }
        if x {
            rex |= 0x02;
        }
        if b {
            rex |= 0x01;
        }
        rex
    }

    fn is_ext(&self, reg: &Register) -> bool {
        matches!(
            reg,
            Register::R8
                | Register::R9
                | Register::R10
                | Register::R11
                | Register::R12
                | Register::R13
                | Register::R14
                | Register::R15
                | Register::R8D
                | Register::R9D
                | Register::R10D
                | Register::R11D
                | Register::R12D
                | Register::R13D
                | Register::R14D
                | Register::R15D
                | Register::R8W
                | Register::R9W
                | Register::R10W
                | Register::R11W
                | Register::R12W
                | Register::R13W
                | Register::R14W
                | Register::R15W
                | Register::R8B
                | Register::R9B
                | Register::R10B
                | Register::R11B
                | Register::R12B
                | Register::R13B
                | Register::R14B
                | Register::R15B
        )
    }

    fn is_64(&self, reg: &Register) -> bool {
        matches!(
            reg,
            Register::RAX
                | Register::RCX
                | Register::RDX
                | Register::RBX
                | Register::RSP
                | Register::RBP
                | Register::RSI
                | Register::RDI
                | Register::R8
                | Register::R9
                | Register::R10
                | Register::R11
                | Register::R12
                | Register::R13
                | Register::R14
                | Register::R15
        )
    }
}
