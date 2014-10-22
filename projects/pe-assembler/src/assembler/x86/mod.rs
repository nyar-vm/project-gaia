use crate::assembler::EncodedInstruction;
use gaia_types::GaiaError;

// 定义 X86 指令类型
#[derive(Debug, Clone)]
pub struct X86Instruction {
    pub opcode: String,
    pub operands: Vec<String>,
}

/// X86 代码生成器 - 提供常用指令的编码
pub struct X86CodeBuilder {
    code: Vec<u8>,
}

impl X86CodeBuilder {
    /// 创建新的代码生成器
    pub fn new() -> Self {
        Self { code: Vec::new() }
    }

    /// 获取当前代码
    pub fn code(&self) -> &[u8] {
        &self.code
    }

    /// 获取代码的可变引用
    pub fn code_mut(&mut self) -> &mut Vec<u8> {
        &mut self.code
    }

    /// 完成构建并返回代码
    pub fn build(self) -> Vec<u8> {
        self.code
    }

    /// 添加原始字节
    pub fn raw_bytes(mut self, bytes: &[u8]) -> Self {
        self.code.extend_from_slice(bytes);
        self
    }

    /// PUSH 立即数 (8位)
    pub fn push_imm8(mut self, value: i8) -> Self {
        self.code.push(0x6A);
        self.code.push(value as u8);
        self
    }

    /// PUSH 立即数 (32位)
    pub fn push_imm32(mut self, value: i32) -> Self {
        self.code.push(0x68);
        self.code.extend_from_slice(&value.to_le_bytes());
        self
    }

    /// CALL 间接调用 [地址]
    pub fn call_indirect(mut self, address: u32) -> Self {
        self.code.extend_from_slice(&[0xFF, 0x15]);
        self.code.extend_from_slice(&address.to_le_bytes());
        self
    }

    /// MOV EAX, ECX
    pub fn mov_eax_ecx(mut self) -> Self {
        self.code.extend_from_slice(&[0x89, 0xC8]);
        self
    }

    /// MOV ECX, EAX  
    pub fn mov_ecx_eax(mut self) -> Self {
        self.code.extend_from_slice(&[0x89, 0xC1]);
        self
    }

    /// PUSH ECX
    pub fn push_ecx(mut self) -> Self {
        self.code.push(0x51);
        self
    }

    /// PUSH EAX
    pub fn push_eax(mut self) -> Self {
        self.code.push(0x50);
        self
    }

    /// INT 3 (断点)
    pub fn int3(mut self) -> Self {
        self.code.push(0xCC);
        self
    }

    /// NOP
    pub fn nop(mut self) -> Self {
        self.code.push(0x90);
        self
    }

    /// RET
    pub fn ret(mut self) -> Self {
        self.code.push(0xC3);
        self
    }

    /// 生成Hello World程序的完整代码
    pub fn hello_world_program() -> Vec<u8> {
        X86CodeBuilder::new()
            // 调用 GetStdHandle(STD_OUTPUT_HANDLE)
            .push_imm8(-11) // push -11 (STD_OUTPUT_HANDLE)
            .call_indirect(0x402010) // call dword ptr [0x402010] (GetStdHandle)
            .mov_ecx_eax() // mov ecx, eax (保存句柄到ecx)
            // 调用 WriteConsoleA(handle, text, length, written, reserved)
            .push_imm8(0) // push 0 (reserved)
            .push_imm8(0) // push 0 (written - 可以为NULL)
            .push_imm8(13) // push 13 (length of "Hello World!\n")
            .push_imm32(0x402000) // push 0x402000 (address of "Hello World!\n")
            .push_ecx() // push ecx (handle)
            .call_indirect(0x402014) // call dword ptr [0x402014] (WriteConsoleA)
            // 调用 ExitProcess(0)
            .push_imm8(0) // push 0 (exit code)
            .call_indirect(0x402018) // call dword ptr [0x402018] (ExitProcess)
            // 不应该到达这里
            .int3() // int 3
            .build()
    }

    /// 生成简单的退出程序代码
    pub fn exit_program(exit_code: i32) -> Vec<u8> {
        X86CodeBuilder::new()
            .push_imm32(exit_code) // push exit_code
            .call_indirect(0x402000) // call dword ptr [ExitProcess]
            .int3() // int 3 (不应该到达)
            .build()
    }

    /// 生成消息框程序代码
    pub fn message_box_program() -> Vec<u8> {
        X86CodeBuilder::new()
            .push_imm32(0) // push 0 (MB_OK)
            .push_imm32(0x402020) // push title_address
            .push_imm32(0x402000) // push message_address
            .push_imm32(0) // push 0 (hWnd)
            .call_indirect(0x402010) // call dword ptr [MessageBoxA]
            .push_imm32(0) // push 0 (exit code)
            .call_indirect(0x402014) // call dword ptr [ExitProcess]
            .build()
    }
}

impl Default for X86CodeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// 编码X86指令
pub fn encode_x86(instruction: X86Instruction) -> Result<EncodedInstruction, GaiaError> {
    let bytes = match instruction.opcode.as_str() {
        "nop" => vec![0x90],
        "int3" => vec![0xCC],
        "ret" => vec![0xC3],
        "push" => {
            if instruction.operands.is_empty() {
                return Err(GaiaError::syntax_error(
                    "PUSH instruction requires operand",
                    gaia_types::SourceLocation::default(),
                ));
            }

            let operand = &instruction.operands[0];
            if operand == "eax" {
                vec![0x50]
            }
            else if operand == "ecx" {
                vec![0x51]
            }
            else if operand == "edx" {
                vec![0x52]
            }
            else if operand == "ebx" {
                vec![0x53]
            }
            else if operand.starts_with('#') {
                // 立即数
                let value_str = &operand[1..];
                if let Ok(value) = value_str.parse::<i8>() {
                    vec![0x6A, value as u8]
                }
                else if let Ok(value) = value_str.parse::<i32>() {
                    let mut bytes = vec![0x68];
                    bytes.extend_from_slice(&value.to_le_bytes());
                    bytes
                }
                else {
                    return Err(GaiaError::syntax_error("Invalid immediate value", gaia_types::SourceLocation::default()));
                }
            }
            else {
                return Err(GaiaError::syntax_error("Unsupported PUSH operand", gaia_types::SourceLocation::default()));
            }
        }
        "mov" => {
            if instruction.operands.len() != 2 {
                return Err(GaiaError::syntax_error(
                    "MOV instruction requires two operands",
                    gaia_types::SourceLocation::default(),
                ));
            }

            let dst = &instruction.operands[0];
            let src = &instruction.operands[1];

            match (dst.as_str(), src.as_str()) {
                ("eax", "ecx") => vec![0x89, 0xC8],
                ("ecx", "eax") => vec![0x89, 0xC1],
                ("edx", "eax") => vec![0x89, 0xC2],
                ("ebx", "eax") => vec![0x89, 0xC3],
                _ => return Err(GaiaError::syntax_error("Unsupported MOV operands", gaia_types::SourceLocation::default())),
            }
        }
        "call" => {
            if instruction.operands.is_empty() {
                return Err(GaiaError::syntax_error(
                    "CALL instruction requires operand",
                    gaia_types::SourceLocation::default(),
                ));
            }

            let operand = &instruction.operands[0];
            if operand.starts_with('[') && operand.ends_with(']') {
                // 间接调用 [地址]
                let addr_str = &operand[1..operand.len() - 1];
                if let Ok(address) = u32::from_str_radix(addr_str.trim_start_matches("0x"), 16) {
                    let mut bytes = vec![0xFF, 0x15];
                    bytes.extend_from_slice(&address.to_le_bytes());
                    bytes
                }
                else {
                    return Err(GaiaError::syntax_error("Invalid call address", gaia_types::SourceLocation::default()));
                }
            }
            else {
                return Err(GaiaError::syntax_error("Unsupported CALL format", gaia_types::SourceLocation::default()));
            }
        }
        _ => {
            return Err(GaiaError::syntax_error(
                &format!("Unsupported instruction: {}", instruction.opcode),
                gaia_types::SourceLocation::default(),
            ))
        }
    };

    Ok(EncodedInstruction { length: bytes.len(), bytes })
}

/// 解码X86指令
pub fn decode_x86(data: Vec<u8>) -> Result<X86Instruction, GaiaError> {
    if data.is_empty() {
        return Err(GaiaError::syntax_error("Empty instruction data", gaia_types::SourceLocation::default()));
    }

    let opcode = match data[0] {
        0x90 => "nop".to_string(),
        0xCC => "int3".to_string(),
        0xC3 => "ret".to_string(),
        0x50 => "push".to_string(),
        0x51 => "push".to_string(),
        0x52 => "push".to_string(),
        0x53 => "push".to_string(),
        0x6A => "push".to_string(),
        0x68 => "push".to_string(),
        0x89 => "mov".to_string(),
        0xFF => "call".to_string(),
        _ => {
            return Err(GaiaError::syntax_error(
                &format!("Unknown opcode: 0x{:02X}", data[0]),
                gaia_types::SourceLocation::default(),
            ))
        }
    };

    let operands = match data[0] {
        0x50 => vec!["eax".to_string()],
        0x51 => vec!["ecx".to_string()],
        0x52 => vec!["edx".to_string()],
        0x53 => vec!["ebx".to_string()],
        0x6A => {
            if data.len() < 2 {
                return Err(GaiaError::syntax_error("Incomplete PUSH imm8 instruction", gaia_types::SourceLocation::default()));
            }
            vec![format!("#{}", data[1] as i8)]
        }
        0x68 => {
            if data.len() < 5 {
                return Err(GaiaError::syntax_error(
                    "Incomplete PUSH imm32 instruction",
                    gaia_types::SourceLocation::default(),
                ));
            }
            let value = i32::from_le_bytes([data[1], data[2], data[3], data[4]]);
            vec![format!("#{}", value)]
        }
        0x89 => {
            if data.len() < 2 {
                return Err(GaiaError::syntax_error("Incomplete MOV instruction", gaia_types::SourceLocation::default()));
            }
            match data[1] {
                0xC8 => vec!["eax".to_string(), "ecx".to_string()],
                0xC1 => vec!["ecx".to_string(), "eax".to_string()],
                0xC2 => vec!["edx".to_string(), "eax".to_string()],
                0xC3 => vec!["ebx".to_string(), "eax".to_string()],
                _ => {
                    return Err(GaiaError::syntax_error(
                        &format!("Unknown MOV variant: 0x{:02X}", data[1]),
                        gaia_types::SourceLocation::default(),
                    ))
                }
            }
        }
        0xFF => {
            if data.len() < 6 || data[1] != 0x15 {
                return Err(GaiaError::syntax_error(
                    "Incomplete or unsupported CALL instruction",
                    gaia_types::SourceLocation::default(),
                ));
            }
            let address = u32::from_le_bytes([data[2], data[3], data[4], data[5]]);
            vec![format!("[0x{:08X}]", address)]
        }
        _ => Vec::new(),
    };

    Ok(X86Instruction { opcode, operands })
}
