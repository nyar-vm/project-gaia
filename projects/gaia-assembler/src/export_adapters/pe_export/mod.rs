/// PE Export 适配器
///
/// 从 Gaia 统一格式导出到 PE 格式
use super::ExportAdapter;
use crate::instruction::*;
use gaia_types::*;

/// PE Export 适配器
#[derive(Debug, Clone)]
pub struct PeExportAdapter {
    /// 适配器配置
    config: PeExportConfig,
}

/// PE Export 配置
#[derive(Debug, Clone)]
pub struct PeExportConfig {
    /// 目标架构
    pub target_arch: gaia_types::helpers::Architecture,
    /// 是否生成调试信息
    pub generate_debug_info: bool,
    /// 是否优化代码
    pub optimize: bool,
    /// 入口点名称
    pub entry_point: String,
}

impl Default for PeExportConfig {
    fn default() -> Self {
        Self {
            target_arch: gaia_types::helpers::Architecture::X86_64,
            generate_debug_info: false,
            optimize: false,
            entry_point: "main".to_string(),
        }
    }
}

impl PeExportAdapter {
    /// 创建新的 PE Export 适配器
    pub fn new() -> Self {
        Self { config: PeExportConfig::default() }
    }

    /// 使用指定配置创建 PE Export 适配器
    pub fn with_config(config: PeExportConfig) -> Self {
        Self { config }
    }

    /// 设置目标架构
    pub fn set_target_arch(&mut self, arch: gaia_types::helpers::Architecture) {
        self.config.target_arch = arch;
    }

    /// 设置入口点
    pub fn set_entry_point(&mut self, entry_point: String) {
        self.config.entry_point = entry_point;
    }
}

// 由于 pe-assembler 项目的具体类型还需要进一步查看，这里先定义一个占位符类型
// 实际实现时需要使用 pe-assembler 中的真实类型
#[derive(Debug, Clone)]
pub struct PeInstruction {
    pub opcode: String,
    pub operands: Vec<String>,
}

impl ExportAdapter<PeInstruction> for PeExportAdapter {
    fn export_instruction(&self, gaia_instruction: &GaiaInstruction) -> Result<PeInstruction> {
        match gaia_instruction {
            GaiaInstruction::LoadConstant(constant) => {
                match constant {
                    GaiaConstant::Integer8(value) => {
                        Ok(PeInstruction { opcode: "mov".to_string(), operands: vec!["al".to_string(), value.to_string()] })
                    }
                    GaiaConstant::Integer16(value) => {
                        Ok(PeInstruction { opcode: "mov".to_string(), operands: vec!["ax".to_string(), value.to_string()] })
                    }
                    GaiaConstant::Integer32(value) => {
                        Ok(PeInstruction { opcode: "mov".to_string(), operands: vec!["eax".to_string(), value.to_string()] })
                    }
                    GaiaConstant::Integer64(value) => {
                        Ok(PeInstruction { opcode: "mov".to_string(), operands: vec!["rax".to_string(), value.to_string()] })
                    }
                    GaiaConstant::String(value) => {
                        // 字符串需要特殊处理，这里简化为加载地址
                        Ok(PeInstruction {
                            opcode: "lea".to_string(),
                            operands: vec!["rax".to_string(), format!("[{}]", value)],
                        })
                    }
                    GaiaConstant::Boolean(value) => Ok(PeInstruction {
                        opcode: "mov".to_string(),
                        operands: vec!["eax".to_string(), if *value { "1" } else { "0" }.to_string()],
                    }),
                    _ => Ok(PeInstruction { opcode: "nop".to_string(), operands: vec![] }),
                }
            }
            GaiaInstruction::LoadLocal(index) => Ok(PeInstruction {
                opcode: "mov".to_string(),
                operands: vec!["eax".to_string(), format!("[rbp-{}]", (index + 1) * 4)],
            }),
            GaiaInstruction::StoreLocal(index) => Ok(PeInstruction {
                opcode: "mov".to_string(),
                operands: vec![format!("[rbp-{}]", (index + 1) * 4), "eax".to_string()],
            }),
            GaiaInstruction::Call(function_name) => {
                Ok(PeInstruction { opcode: "call".to_string(), operands: vec![function_name.clone()] })
            }
            GaiaInstruction::Return => Ok(PeInstruction { opcode: "ret".to_string(), operands: vec![] }),
            GaiaInstruction::Add => {
                Ok(PeInstruction { opcode: "add".to_string(), operands: vec!["eax".to_string(), "ebx".to_string()] })
            }
            GaiaInstruction::Subtract => {
                Ok(PeInstruction { opcode: "sub".to_string(), operands: vec!["eax".to_string(), "ebx".to_string()] })
            }
            _ => {
                // 其他指令暂时用 nop 代替
                Ok(PeInstruction { opcode: "nop".to_string(), operands: vec![] })
            }
        }
    }

    fn export_program(&self, gaia_program: &GaiaProgram) -> Result<Vec<PeInstruction>> {
        let mut pe_instructions = Vec::new();

        // 添加程序入口
        pe_instructions.push(PeInstruction { opcode: "section".to_string(), operands: vec![".text".to_string()] });

        // 处理所有函数
        for function in &gaia_program.functions {
            // 添加函数标签
            pe_instructions.push(PeInstruction { opcode: "label".to_string(), operands: vec![function.name.clone()] });

            // 函数序言
            pe_instructions.push(PeInstruction { opcode: "push".to_string(), operands: vec!["rbp".to_string()] });
            pe_instructions
                .push(PeInstruction { opcode: "mov".to_string(), operands: vec!["rbp".to_string(), "rsp".to_string()] });

            // 转换函数指令
            for gaia_instruction in &function.instructions {
                let pe_instruction = self.export_instruction(gaia_instruction)?;
                pe_instructions.push(pe_instruction);
            }

            // 函数尾声（如果没有显式的 return）
            if !function.instructions.iter().any(|inst| matches!(inst, GaiaInstruction::Return)) {
                pe_instructions.push(PeInstruction { opcode: "pop".to_string(), operands: vec!["rbp".to_string()] });
                pe_instructions.push(PeInstruction { opcode: "ret".to_string(), operands: vec![] });
            }
        }

        Ok(pe_instructions)
    }

    fn adapter_name(&self) -> &'static str {
        "PE Export Adapter"
    }

    fn generate_binary(&self, pe_instructions: &[PeInstruction]) -> Result<Vec<u8>> {
        // 这里应该调用 pe-assembler 来生成实际的 PE 二进制文件
        // 目前先返回一个简单的占位符
        let mut binary = Vec::new();

        // PE 文件头占位符
        binary.extend_from_slice(b"MZ"); // DOS 头签名
        binary.resize(64, 0); // DOS 头

        // PE 签名
        binary.extend_from_slice(b"PE\0\0");

        // 简化的机器码生成
        for instruction in pe_instructions {
            match instruction.opcode.as_str() {
                "mov" => {
                    // 简化的 mov 指令编码
                    binary.push(0xB8); // mov eax, imm32
                    if instruction.operands.len() >= 2 {
                        if let Ok(value) = instruction.operands[1].parse::<i32>() {
                            binary.extend_from_slice(&value.to_le_bytes());
                        }
                    }
                }
                "call" => {
                    // 简化的 call 指令编码
                    binary.push(0xE8); // call rel32
                    binary.extend_from_slice(&[0, 0, 0, 0]); // 占位符地址
                }
                "ret" => {
                    binary.push(0xC3); // ret
                }
                _ => {
                    // 其他指令用 nop 代替
                    binary.push(0x90); // nop
                }
            }
        }

        Ok(binary)
    }
}

impl Default for PeExportAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pe_export_adapter_creation() {
        let adapter = PeExportAdapter::new();
        assert_eq!(adapter.adapter_name(), "PE Export Adapter");
    }

    #[test]
    fn test_gaia_load_constant_export() {
        let adapter = PeExportAdapter::new();
        let gaia_instruction = GaiaInstruction::LoadConstant(GaiaConstant::Integer32(42));

        let result = adapter.export_instruction(&gaia_instruction);
        assert!(result.is_ok());

        let pe_instruction = result.unwrap();
        assert_eq!(pe_instruction.opcode, "mov");
        assert_eq!(pe_instruction.operands, vec!["eax", "42"]);
    }

    #[test]
    fn test_gaia_call_export() {
        let adapter = PeExportAdapter::new();
        let gaia_instruction = GaiaInstruction::Call("printf".to_string());

        let result = adapter.export_instruction(&gaia_instruction);
        assert!(result.is_ok());

        let pe_instruction = result.unwrap();
        assert_eq!(pe_instruction.opcode, "call");
        assert_eq!(pe_instruction.operands, vec!["printf"]);
    }

    #[test]
    fn test_simple_program_export() {
        let adapter = PeExportAdapter::new();
        let gaia_program = GaiaProgram {
            name: "test".to_string(),
            functions: vec![GaiaFunction {
                name: "main".to_string(),
                parameters: vec![],
                return_type: None,
                locals: vec![],
                instructions: vec![GaiaInstruction::LoadConstant(GaiaConstant::Integer32(42)), GaiaInstruction::Return],
            }],
            constants: vec![],
        };

        let result = adapter.export_program(&gaia_program);
        assert!(result.is_ok());

        let pe_instructions = result.unwrap();
        assert!(!pe_instructions.is_empty());

        // 检查是否包含函数标签
        assert!(pe_instructions.iter().any(|inst| inst.opcode == "label" && inst.operands.contains(&"main".to_string())));
    }
}
