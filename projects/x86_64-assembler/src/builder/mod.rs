#![doc = include_str!("readme.md")]

use crate::instruction::{Instruction, Operand, Register};
use gaia_types::{helpers::Architecture, Result};

/// 通用程序构建器，用于生成汇编指令序列
#[derive(Debug, Clone)]
pub struct ProgramBuilder {
    architecture: Architecture,
    instructions: Vec<Instruction>,
    data_sections: Vec<DataSection>,
}

/// 数据段定义
#[derive(Debug, Clone)]
pub struct DataSection {
    /// 数据段名称
    pub name: String,
    /// 数据内容
    pub data: Vec<u8>,
    /// 对齐要求（可选）
    pub alignment: Option<u32>,
}

impl ProgramBuilder {
    /// 创建新的程序构建器
    pub fn new(architecture: Architecture) -> Self {
        Self { architecture, instructions: Vec::new(), data_sections: Vec::new() }
    }

    /// 获取当前架构
    pub fn architecture(&self) -> &Architecture {
        &self.architecture
    }

    /// 添加指令
    pub fn add_instruction(&mut self, instruction: Instruction) -> &mut Self {
        self.instructions.push(instruction);
        self
    }

    /// 批量添加指令
    pub fn add_instructions(&mut self, instructions: Vec<Instruction>) -> &mut Self {
        self.instructions.extend(instructions);
        self
    }

    /// 添加数据段
    pub fn add_data_section(&mut self, name: String, data: Vec<u8>) -> &mut Self {
        self.data_sections.push(DataSection { name, data, alignment: None });
        self
    }

    /// 添加带对齐的数据段
    pub fn add_data_section_with_alignment(&mut self, name: String, data: Vec<u8>, alignment: u32) -> &mut Self {
        self.data_sections.push(DataSection { name, data, alignment: Some(alignment) });
        self
    }

    /// 获取所有指令
    pub fn instructions(&self) -> &[Instruction] {
        &self.instructions
    }

    /// 获取所有数据段
    pub fn data_sections(&self) -> &[DataSection] {
        &self.data_sections
    }

    /// 清空所有指令
    pub fn clear_instructions(&mut self) -> &mut Self {
        self.instructions.clear();
        self
    }

    /// 清空所有数据段
    pub fn clear_data_sections(&mut self) -> &mut Self {
        self.data_sections.clear();
        self
    }

    /// 编译指令为字节码
    pub fn compile_instructions(&self) -> Result<Vec<u8>> {
        let assembler = crate::X86_64Assembler::new(self.architecture.clone())?;
        let mut code = Vec::new();

        for instruction in &self.instructions {
            let bytes = assembler.encode(instruction)?;
            code.extend(bytes);
        }

        Ok(code)
    }

    // === 便捷方法：创建常用指令 ===

    /// 创建 MOV 寄存器到寄存器指令
    pub fn mov_reg_reg(&mut self, dest: Register, src: Register) -> Result<&mut Self> {
        self.add_instruction(Instruction::Mov { dst: Operand::reg(dest), src: Operand::reg(src) });
        Ok(self)
    }

    /// 创建 MOV 立即数到寄存器指令
    pub fn mov_reg_imm(&mut self, dest: Register, value: i64) -> Result<&mut Self> {
        // 根据架构和寄存器类型确定立即数大小
        let size = match self.architecture {
            Architecture::X86 => 32,
            Architecture::X86_64 => {
                // 对于 x64，检查寄存器类型
                match dest {
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
                    | Register::R15D => 32,
                    _ => 64,
                }
            }
            _ => 32,
        };
        self.add_instruction(Instruction::Mov { dst: Operand::reg(dest), src: Operand::imm(value, size) });
        Ok(self)
    }

    /// 创建 MOV 标签地址到寄存器指令
    pub fn mov_reg_label(&mut self, dest: Register, label: String) -> Result<&mut Self> {
        self.add_instruction(Instruction::Mov { dst: Operand::reg(dest), src: Operand::label(label) });
        Ok(self)
    }

    /// 创建 PUSH 立即数指令
    pub fn push_imm(&mut self, value: i64) -> Result<&mut Self> {
        self.add_instruction(Instruction::Push { op: Operand::imm(value, 32) });
        Ok(self)
    }

    /// 创建 PUSH 寄存器指令
    pub fn push_reg(&mut self, reg: Register) -> Result<&mut Self> {
        self.add_instruction(Instruction::Push { op: Operand::reg(reg) });
        Ok(self)
    }

    /// 创建 PUSH 标签地址指令
    pub fn push_label(&mut self, label: String) -> Result<&mut Self> {
        self.add_instruction(Instruction::Push { op: Operand::label(label) });
        Ok(self)
    }

    /// 创建 POP 寄存器指令
    pub fn pop_reg(&mut self, reg: Register) -> Result<&mut Self> {
        self.add_instruction(Instruction::Pop { dst: Operand::reg(reg) });
        Ok(self)
    }

    /// 创建 CALL 指令
    pub fn call(&mut self, target: String) -> Result<&mut Self> {
        self.add_instruction(Instruction::Call { target: Operand::label(target) });
        Ok(self)
    }

    /// 创建间接 CALL 指令 - CALL [address]
    pub fn call_indirect(&mut self, address: i32) -> Result<&mut Self> {
        // CALL [address]
        self.add_instruction(Instruction::Call { target: Operand::mem(None, None, 1, address) });
        Ok(self)
    }

    /// 创建 RET 指令
    pub fn ret(&mut self) -> Result<&mut Self> {
        self.add_instruction(Instruction::Ret);
        Ok(self)
    }

    /// 创建 NOP 指令
    pub fn nop(&mut self) -> Result<&mut Self> {
        self.add_instruction(Instruction::Nop);
        Ok(self)
    }

    /// 创建 ADD 指令
    pub fn add_reg_imm(&mut self, dest: Register, value: i64) -> Result<&mut Self> {
        self.add_instruction(Instruction::Add { dst: Operand::reg(dest), src: Operand::imm(value, 32) });
        Ok(self)
    }

    /// 创建 SUB 指令
    pub fn sub_reg_imm(&mut self, dest: Register, value: i64) -> Result<&mut Self> {
        self.add_instruction(Instruction::Sub { dst: Operand::reg(dest), src: Operand::imm(value, 32) });
        Ok(self)
    }
}

impl Default for ProgramBuilder {
    fn default() -> Self {
        Self::new(Architecture::X86_64)
    }
}
