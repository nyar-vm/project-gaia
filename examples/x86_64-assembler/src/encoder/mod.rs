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
            Instruction::Cmp { dst, src } => self.encode_cmp(dst, src),
            Instruction::Jmp { target } => self.encode_jmp(target),
            Instruction::Jnz { target } => self.encode_jnz(target),
            Instruction::Jz { target } => self.encode_jz(target),
            Instruction::Dec { op } => self.encode_dec(op),
            Instruction::Inc { op } => self.encode_inc(op),
            Instruction::Movss { dst, src } => self.encode_movss(dst, src),
            Instruction::Addss { dst, src } => self.encode_addss(dst, src),
            Instruction::Subss { dst, src } => self.encode_subss(dst, src),
            Instruction::Mulss { dst, src } => self.encode_mulss(dst, src),
            Instruction::Divss { dst, src } => self.encode_divss(dst, src),
            Instruction::Maxss { dst, src } => self.encode_maxss(dst, src),
            Instruction::Xor { dst, src } => self.encode_xor(dst, src),
            Instruction::Xorps { dst, src } => self.encode_xorps(dst, src),
            Instruction::Imul { dst, src } => self.encode_imul(dst, src),
            Instruction::Mul { src } => self.encode_unary_op(src, 4, "MUL"),
            Instruction::Div { src } => self.encode_unary_op(src, 6, "DIV"),
            Instruction::Idiv { src } => self.encode_unary_op(src, 7, "IDIV"),
            Instruction::Cqo => Ok(vec![0x48, 0x99]),
            Instruction::Label(_) => Ok(vec![]),
        }
    }

    fn encode_unary_op(&self, src: &Operand, reg_digit: u8, name: &str) -> Result<Vec<u8>> {
        let mut result = Vec::new();
        match src {
            Operand::Reg(s) => match self.architecture {
                Architecture::X86_64 => {
                    let rex = self.compose_rex(true, false, false, self.is_ext(s));
                    if rex != 0 {
                        result.push(rex);
                    }
                    result.push(0xF7);
                    result.push(self.encode_modrm(3, reg_digit, self.encode_register(s)?));
                }
                _ => return Err(GaiaError::unsupported_architecture(self.architecture.clone())),
            },
            _ => {
                return Err(GaiaError::invalid_instruction(
                    format!("Invalid operand combination for {}", name),
                    self.architecture.clone(),
                ))
            }
        }
        Ok(result)
    }

    fn encode_imul(&self, dst: &Register, src: &Operand) -> Result<Vec<u8>> {
        let mut result = Vec::new();
        match src {
            Operand::Reg(s) => match self.architecture {
                Architecture::X86_64 => {
                    let rex = self.compose_rex(true, self.is_ext(dst), false, self.is_ext(s));
                    if rex != 0 {
                        result.push(rex);
                    }
                    result.push(0x0F);
                    result.push(0xAF);
                    result.push(self.encode_modrm(3, self.encode_register(dst)?, self.encode_register(s)?));
                }
                _ => return Err(GaiaError::unsupported_architecture(self.architecture.clone())),
            },
            _ => {
                return Err(GaiaError::invalid_instruction(
                    "Invalid operand combination for IMUL".to_string(),
                    self.architecture.clone(),
                ))
            }
        }
        Ok(result)
    }

    fn encode_cmp(&self, dst: &Operand, src: &Operand) -> Result<Vec<u8>> {
        let mut result = Vec::new();
        match (dst, src) {
            (Operand::Reg(d), Operand::Reg(s)) => {
                let rex = self.compose_rex(true, self.is_ext(s), false, self.is_ext(d));
                if rex != 0 {
                    result.push(rex);
                }
                result.push(0x39);
                result.push(self.encode_modrm(3, self.encode_register(s)?, self.encode_register(d)?));
            }
            (Operand::Reg(d), Operand::Imm { value, .. }) => {
                let rex = self.compose_rex(true, false, false, self.is_ext(d));
                if rex != 0 {
                    result.push(rex);
                }
                result.push(0x81);
                result.push(self.encode_modrm(3, 7, self.encode_register(d)?));
                result.extend_from_slice(&(*value as u32).to_le_bytes());
            }
            _ => return Err(GaiaError::not_implemented("CMP encoding".to_string())),
        }
        Ok(result)
    }

    fn encode_jmp(&self, target: &Operand) -> Result<Vec<u8>> {
        let mut result = Vec::new();
        match target {
            Operand::Label(_) => {
                result.push(0xEB); // Short jump placeholder
                result.push(0x00);
            }
            _ => return Err(GaiaError::not_implemented("JMP encoding".to_string())),
        }
        Ok(result)
    }

    fn encode_jnz(&self, target: &Operand) -> Result<Vec<u8>> {
        let mut result = Vec::new();
        match target {
            Operand::Label(_) => {
                result.push(0x75); // JNZ short
                result.push(0x00);
            }
            _ => return Err(GaiaError::not_implemented("JNZ encoding".to_string())),
        }
        Ok(result)
    }

    fn encode_jz(&self, target: &Operand) -> Result<Vec<u8>> {
        let mut result = Vec::new();
        match target {
            Operand::Label(_) => {
                result.push(0x74); // JZ short
                result.push(0x00);
            }
            _ => return Err(GaiaError::not_implemented("JZ encoding".to_string())),
        }
        Ok(result)
    }

    fn encode_dec(&self, op: &Operand) -> Result<Vec<u8>> {
        let mut result = Vec::new();
        match op {
            Operand::Reg(r) => {
                let rex = self.compose_rex(true, false, false, self.is_ext(r));
                if rex != 0 {
                    result.push(rex);
                }
                result.push(0xFF);
                result.push(self.encode_modrm(3, 1, self.encode_register(r)?));
            }
            _ => return Err(GaiaError::not_implemented("DEC encoding".to_string())),
        }
        Ok(result)
    }

    fn encode_inc(&self, op: &Operand) -> Result<Vec<u8>> {
        let mut result = Vec::new();
        match op {
            Operand::Reg(r) => {
                let rex = self.compose_rex(true, false, false, self.is_ext(r));
                if rex != 0 {
                    result.push(rex);
                }
                result.push(0xFF);
                result.push(self.encode_modrm(3, 0, self.encode_register(r)?));
            }
            _ => return Err(GaiaError::not_implemented("INC encoding".to_string())),
        }
        Ok(result)
    }

    fn encode_movss(&self, dst: &Operand, src: &Operand) -> Result<Vec<u8>> {
        let mut result = vec![0xF3, 0x0F];
        match (dst, src) {
            (Operand::Reg(d), Operand::Reg(s)) => {
                result.push(0x10);
                result.push(self.encode_modrm(3, self.encode_register(s)?, self.encode_register(d)?));
            }
            (Operand::Reg(d), Operand::Mem { base, index, scale, displacement }) => {
                result.push(0x10);
                let (modrm, sib, disp) =
                    self.encode_memory_operand(base, index, scale, displacement, self.encode_register(d)?)?;
                result.push(modrm);
                result.extend(sib);
                result.extend(disp);
            }
            (Operand::Mem { base, index, scale, displacement }, Operand::Reg(s)) => {
                result.push(0x11);
                let (modrm, sib, disp) =
                    self.encode_memory_operand(base, index, scale, displacement, self.encode_register(s)?)?;
                result.push(modrm);
                result.extend(sib);
                result.extend(disp);
            }
            _ => return Err(GaiaError::not_implemented("MOVSS encoding".to_string())),
        }
        Ok(result)
    }

    fn encode_sse_binop(&self, dst: &Operand, src: &Operand, opcode: u8) -> Result<Vec<u8>> {
        let mut result = vec![0xF3, 0x0F];
        match (dst, src) {
            (Operand::Reg(d), Operand::Reg(s)) => {
                result.push(opcode);
                result.push(self.encode_modrm(3, self.encode_register(d)?, self.encode_register(s)?));
            }
            (Operand::Reg(d), Operand::Mem { base, index, scale, displacement }) => {
                result.push(opcode);
                let (modrm, sib, disp) =
                    self.encode_memory_operand(base, index, scale, displacement, self.encode_register(d)?)?;
                result.push(modrm);
                result.extend(sib);
                result.extend(disp);
            }
            _ => return Err(GaiaError::not_implemented("SSE binop encoding".to_string())),
        }
        Ok(result)
    }

    fn encode_addss(&self, dst: &Operand, src: &Operand) -> Result<Vec<u8>> {
        self.encode_sse_binop(dst, src, 0x58)
    }
    fn encode_subss(&self, dst: &Operand, src: &Operand) -> Result<Vec<u8>> {
        self.encode_sse_binop(dst, src, 0x5C)
    }
    fn encode_mulss(&self, dst: &Operand, src: &Operand) -> Result<Vec<u8>> {
        self.encode_sse_binop(dst, src, 0x59)
    }
    fn encode_divss(&self, dst: &Operand, src: &Operand) -> Result<Vec<u8>> {
        self.encode_sse_binop(dst, src, 0x5E)
    }
    fn encode_maxss(&self, dst: &Operand, src: &Operand) -> Result<Vec<u8>> {
        self.encode_sse_binop(dst, src, 0x5F)
    }

    fn encode_xorps(&self, dst: &Operand, src: &Operand) -> Result<Vec<u8>> {
        let mut result = vec![0x0F, 0x57];
        match (dst, src) {
            (Operand::Reg(d), Operand::Reg(s)) => {
                result.push(self.encode_modrm(3, self.encode_register(d)?, self.encode_register(s)?));
            }
            _ => return Err(GaiaError::not_implemented("XORPS encoding".to_string())),
        }
        Ok(result)
    }

    fn encode_xor(&self, dst: &Operand, src: &Operand) -> Result<Vec<u8>> {
        let mut result = Vec::new();
        match (dst, src) {
            (Operand::Reg(d), Operand::Reg(s)) => {
                let rex = self.compose_rex(true, self.is_ext(s), false, self.is_ext(d));
                if rex != 0 {
                    result.push(rex);
                }
                result.push(0x31);
                result.push(self.encode_modrm(3, self.encode_register(s)?, self.encode_register(d)?));
            }
            _ => return Err(GaiaError::not_implemented("XOR encoding".to_string())),
        }
        Ok(result)
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
            (Operand::Mem { base, index, scale, displacement }, Operand::Imm { value, size }) => {
                self.encode_mov_mem_imm(base, index, *scale, *displacement, *value, *size)
            }
            _ => Err(GaiaError::invalid_instruction(
                "Invalid operand combination for MOV".to_string(),
                self.architecture.clone(),
            )),
        }
    }

    fn encode_mov_mem_imm(
        &self,
        base: &Option<Register>,
        index: &Option<Register>,
        scale: u8,
        displacement: i32,
        imm: i64,
        size: u8,
    ) -> Result<Vec<u8>> {
        let mut result = Vec::new();
        match self.architecture {
            Architecture::X86 => {
                if size == 16 {
                    result.push(0x66);
                }
                if size == 8 {
                    result.push(0xC6);
                }
                else {
                    result.push(0xC7);
                }
            }
            Architecture::X86_64 => {
                let is64 = size == 64;
                let is16 = size == 16;
                let is8 = size == 8;

                if is16 {
                    result.push(0x66);
                }

                let rex = self.compose_rex(is64, false, false, base.as_ref().map(|b| self.is_ext(b)).unwrap_or(false));
                if rex != 0 {
                    result.push(rex);
                }
                if is8 {
                    result.push(0xC6);
                }
                else {
                    result.push(0xC7);
                }
            }
            _ => return Err(GaiaError::unsupported_architecture(self.architecture.clone())),
        }
        let (modrm, sib_bytes, disp_bytes) = self.encode_memory_operand(base, index, &scale, &displacement, 0)?;
        result.push(modrm);
        result.extend(sib_bytes);
        result.extend(disp_bytes);

        if size == 8 {
            result.push(imm as u8);
        }
        else if size == 16 {
            result.extend_from_slice(&(imm as u16).to_le_bytes());
        }
        else {
            // 注意：x64 下 MOV [mem], imm64 不存在，C7 总是使用 32 位立即数（带符号扩展）
            result.extend_from_slice(&(imm as u32).to_le_bytes());
        }
        Ok(result)
    }

    fn encode_mov_reg_reg(&self, dest_reg: &Register, src_reg: &Register) -> Result<Vec<u8>> {
        let mut result = Vec::new();
        match self.architecture {
            Architecture::X86 => {
                if self.is_16(dest_reg) {
                    result.push(0x66);
                }
                if self.is_8(dest_reg) {
                    result.push(0x88);
                }
                else {
                    result.push(0x89);
                }
                result.push(self.encode_modrm(3, self.encode_register(src_reg)?, self.encode_register(dest_reg)?));
            }
            Architecture::X86_64 => {
                let is64 = self.is_64(dest_reg);
                let is16 = self.is_16(dest_reg);
                let is8 = self.is_8(dest_reg);

                if is16 {
                    result.push(0x66);
                }

                // 在 x64 下，MOV 寄存器间传送：
                // 1. 如果是 64 位操作 (is64)，需要 REX.W
                // 2. 如果涉及扩展寄存器 (R8-R15, R8D-R15D, R8W-R15W, R8B-R15B)，需要 REX.R/B
                // 3. 如果是 8 位操作且使用了 SIL, DIL, BPL, SPL (本项未定义)，需要 REX (即使内容为 0)
                //    这里虽然没定义 SIL 等，但如果未来添加了，compose_rex 逻辑也是对的。
                let rex = self.compose_rex(is64, self.is_ext(src_reg), false, self.is_ext(dest_reg));
                if rex != 0 {
                    result.push(rex);
                }

                if is8 {
                    result.push(0x88);
                }
                else {
                    result.push(0x89);
                }
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
                if self.is_8(dest_reg) {
                    result.push(0xB0 + reg_code);
                    result.push(imm as u8);
                }
                else if self.is_16(dest_reg) {
                    result.push(0x66);
                    result.push(0xB8 + reg_code);
                    result.extend_from_slice(&(imm as u16).to_le_bytes());
                }
                else {
                    result.push(0xB8 + reg_code);
                    result.extend_from_slice(&(imm as u32).to_le_bytes());
                }
            }
            Architecture::X86_64 => {
                let reg_code = self.encode_register(dest_reg)?;
                let is64 = self.is_64(dest_reg);
                let is16 = self.is_16(dest_reg);
                let is8 = self.is_8(dest_reg);

                if is16 {
                    result.push(0x66);
                }

                // MOV reg, imm 在 x64 下的 REX 规则：
                // 1. 如果是 64 位立即数加载到 64 位寄存器 (is64)，需要 REX.W (0x48)
                // 2. 如果涉及扩展寄存器 (R8-R15 等)，需要 REX.B (0x41)
                // 3. 特殊情况：如果是 8 位低位寄存器 (AL, BL, CL, DL) 加载，不需要 REX。
                //    但如果是 8 位扩展寄存器 (R8B-R15B)，需要 REX.B。
                let rex = self.compose_rex(is64, false, false, self.is_ext(dest_reg));
                if rex != 0 {
                    result.push(rex);
                }

                if is8 {
                    result.push(0xB0 + reg_code);
                    result.push(imm as u8);
                }
                else {
                    result.push(0xB8 + reg_code);
                    if is64 {
                        result.extend_from_slice(&(imm as u64).to_le_bytes());
                    }
                    else if is16 {
                        result.extend_from_slice(&(imm as u16).to_le_bytes());
                    }
                    else {
                        result.extend_from_slice(&(imm as u32).to_le_bytes());
                    }
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
                if self.is_16(src_reg) {
                    result.push(0x66);
                }
                if self.is_8(src_reg) {
                    result.push(0x88);
                }
                else {
                    result.push(0x89);
                }
            }
            Architecture::X86_64 => {
                let is64 = self.is_64(src_reg);
                let is16 = self.is_16(src_reg);
                let is8 = self.is_8(src_reg);

                if is16 {
                    result.push(0x66);
                }

                // MOV [mem], reg 在 x64 下的 REX 规则：
                // 1. 如果是 64 位操作 (is64)，需要 REX.W
                // 2. 如果涉及扩展寄存器 (src_reg 是 R8-R15 等，或者 base 是 R8-R15 等)，需要 REX.R/B
                let rex =
                    self.compose_rex(is64, self.is_ext(src_reg), false, base.as_ref().map(|b| self.is_ext(b)).unwrap_or(false));
                if rex != 0 {
                    result.push(rex);
                }
                if is8 {
                    result.push(0x88);
                }
                else {
                    result.push(0x89);
                }
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
                if self.is_16(dest_reg) {
                    result.push(0x66);
                }
                if self.is_8(dest_reg) {
                    result.push(0x8A);
                }
                else {
                    result.push(0x8B);
                }
            }
            Architecture::X86_64 => {
                let is64 = self.is_64(dest_reg);
                let is16 = self.is_16(dest_reg);
                let is8 = self.is_8(dest_reg);

                if is16 {
                    result.push(0x66);
                }

                // MOV reg, [mem] 在 x64 下的 REX 规则：
                // 1. 如果是 64 位操作 (is64)，需要 REX.W
                // 2. 如果涉及扩展寄存器 (dest_reg 是 R8-R15 等，或者 base 是 R8-R15 等)，需要 REX.R/B
                let rex = self.compose_rex(
                    is64,
                    self.is_ext(dest_reg),
                    false,
                    base.as_ref().map(|b| self.is_ext(b)).unwrap_or(false),
                );
                if rex != 0 {
                    result.push(rex);
                }

                if is8 {
                    result.push(0x8A);
                }
                else {
                    result.push(0x8B);
                }
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
            Register::AL | Register::AX | Register::EAX | Register::RAX => Ok(0),
            Register::CL | Register::CX | Register::ECX | Register::RCX => Ok(1),
            Register::DL | Register::DX | Register::EDX | Register::RDX => Ok(2),
            Register::BL | Register::BX | Register::EBX | Register::RBX => Ok(3),
            Register::AH | Register::SP | Register::ESP | Register::RSP => Ok(4),
            Register::CH | Register::BP | Register::EBP | Register::RBP => Ok(5),
            Register::DH | Register::SI | Register::ESI | Register::RSI => Ok(6),
            Register::BH | Register::DI | Register::EDI | Register::RDI => Ok(7),
            Register::R8 | Register::R8D | Register::R8W | Register::R8B => Ok(0),
            Register::R9 | Register::R9D | Register::R9W | Register::R9B => Ok(1),
            Register::R10 | Register::R10D | Register::R10W | Register::R10B => Ok(2),
            Register::R11 | Register::R11D | Register::R11W | Register::R11B => Ok(3),
            Register::R12 | Register::R12D | Register::R12W | Register::R12B => Ok(4),
            Register::R13 | Register::R13D | Register::R13W | Register::R13B => Ok(5),
            Register::R14 | Register::R14D | Register::R14W | Register::R14B => Ok(6),
            Register::R15 | Register::R15D | Register::R15W | Register::R15B => Ok(7),
            Register::XMM0 => Ok(0),
            Register::XMM1 => Ok(1),
            Register::XMM2 => Ok(2),
            Register::XMM3 => Ok(3),
            Register::XMM4 => Ok(4),
            Register::XMM5 => Ok(5),
            Register::XMM6 => Ok(6),
            Register::XMM7 => Ok(7),
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
        if !w && !r && !x && !b {
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

    fn is_64(&self, reg: &Register) -> bool {
        matches!(
            reg,
            Register::RAX
                | Register::RBX
                | Register::RCX
                | Register::RDX
                | Register::RSI
                | Register::RDI
                | Register::RSP
                | Register::RBP
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

    fn is_32(&self, reg: &Register) -> bool {
        matches!(
            reg,
            Register::EAX
                | Register::EBX
                | Register::ECX
                | Register::EDX
                | Register::ESI
                | Register::EDI
                | Register::ESP
                | Register::EBP
                | Register::R8D
                | Register::R9D
                | Register::R10D
                | Register::R11D
                | Register::R12D
                | Register::R13D
                | Register::R14D
                | Register::R15D
        )
    }

    fn is_16(&self, reg: &Register) -> bool {
        matches!(
            reg,
            Register::AX
                | Register::BX
                | Register::CX
                | Register::DX
                | Register::SI
                | Register::DI
                | Register::SP
                | Register::BP
                | Register::R8W
                | Register::R9W
                | Register::R10W
                | Register::R11W
                | Register::R12W
                | Register::R13W
                | Register::R14W
                | Register::R15W
        )
    }

    fn is_8(&self, reg: &Register) -> bool {
        matches!(
            reg,
            Register::AL
                | Register::BL
                | Register::CL
                | Register::DL
                | Register::AH
                | Register::BH
                | Register::CH
                | Register::DH
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

    fn is_8_low(&self, reg: &Register) -> bool {
        matches!(
            reg,
            Register::AL
                | Register::BL
                | Register::CL
                | Register::DL
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
}
