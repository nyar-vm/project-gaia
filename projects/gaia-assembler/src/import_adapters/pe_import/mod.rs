/// PE Import 适配器
///
/// 从 PE 格式导入到 Gaia 统一格式
use super::ImportAdapter;
use crate::instruction::*;
use gaia_types::*;

/// PE Import 适配器
#[derive(Debug, Clone)]
pub struct PeImportAdapter {
    /// 适配器配置
    config: PeImportConfig,
}

/// PE Import 配置
#[derive(Debug, Clone)]
pub struct PeImportConfig {
    /// 是否保留调试信息
    pub preserve_debug_info: bool,
    /// 是否解析导入表
    pub parse_imports: bool,
    /// 是否解析导出表
    pub parse_exports: bool,
}

impl Default for PeImportConfig {
    fn default() -> Self {
        Self { preserve_debug_info: true, parse_imports: true, parse_exports: true }
    }
}

impl PeImportAdapter {
    /// 创建新的 PE Import 适配器
    pub fn new() -> Self {
        Self { config: PeImportConfig::default() }
    }

    /// 使用指定配置创建 PE Import 适配器
    pub fn with_config(config: PeImportConfig) -> Self {
        Self { config }
    }
}

// 由于 pe-assembler 项目的具体类型还需要进一步查看，这里先定义一个占位符类型
// 实际实现时需要使用 pe-assembler 中的真实类型
#[derive(Debug, Clone)]
pub struct PeInstruction {
    pub opcode: String,
    pub operands: Vec<String>,
}

impl ImportAdapter<PeInstruction> for PeImportAdapter {
    fn import_instruction(&self, pe_instruction: &PeInstruction) -> Result<GaiaInstruction> {
        // 根据 PE 指令的操作码转换为 Gaia 指令
        match pe_instruction.opcode.as_str() {
            "mov" => {
                // 处理 mov 指令
                if pe_instruction.operands.len() >= 2 {
                    // 简化处理：将 mov 转换为加载常量或存储指令
                    if pe_instruction.operands[1].parse::<i32>().is_ok() {
                        let value = pe_instruction.operands[1].parse::<i32>().unwrap();
                        Ok(GaiaInstruction::LoadConstant(GaiaConstant::Integer32(value)))
                    }
                    else {
                        Ok(GaiaInstruction::LoadLocal(0)) // 简化处理
                    }
                }
                else {
                    Err(GaiaError::invalid_instruction(&pe_instruction.opcode, gaia_types::helpers::Architecture::X86_64))
                }
            }
            "call" => {
                // 处理函数调用
                if !pe_instruction.operands.is_empty() {
                    Ok(GaiaInstruction::Call(pe_instruction.operands[0].clone()))
                }
                else {
                    Err(GaiaError::invalid_instruction(&pe_instruction.opcode, gaia_types::helpers::Architecture::X86_64))
                }
            }
            "ret" => Ok(GaiaInstruction::Return),
            _ => {
                // 未知指令
                Err(GaiaError::invalid_instruction(&pe_instruction.opcode, gaia_types::helpers::Architecture::X86_64))
            }
        }
    }

    fn import_program(&self, pe_instructions: &[PeInstruction]) -> Result<GaiaProgram> {
        let mut gaia_instructions = Vec::new();

        for pe_instruction in pe_instructions {
            let gaia_instruction = self.import_instruction(&pe_instruction.clone())?;
            gaia_instructions.push(gaia_instruction);
        }

        // 创建一个简单的 Gaia 程序
        let main_function = GaiaFunction {
            name: "main".to_string(),
            parameters: vec![],
            return_type: None,
            locals: vec![],
            instructions: gaia_instructions,
        };

        Ok(GaiaProgram { name: "imported_pe_program".to_string(), functions: vec![main_function], constants: vec![] })
    }

    fn adapter_name(&self) -> &'static str {
        "PE Import Adapter"
    }
}

impl Default for PeImportAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pe_import_adapter_creation() {
        let adapter = PeImportAdapter::new();
        assert_eq!(adapter.adapter_name(), "PE Import Adapter");
    }

    #[test]
    fn test_pe_mov_instruction_import() {
        let adapter = PeImportAdapter::new();
        let pe_instruction = PeInstruction { opcode: "mov".to_string(), operands: vec!["eax".to_string(), "42".to_string()] };

        let result = adapter.import_instruction(&pe_instruction);
        assert!(result.is_ok());

        if let Ok(GaiaInstruction::LoadConstant(GaiaConstant::Integer32(value))) = result {
            assert_eq!(value, 42);
        }
        else {
            panic!("Expected LoadConstant instruction");
        }
    }

    #[test]
    fn test_pe_call_instruction_import() {
        let adapter = PeImportAdapter::new();
        let pe_instruction = PeInstruction { opcode: "call".to_string(), operands: vec!["printf".to_string()] };

        let result = adapter.import_instruction(&pe_instruction);
        assert!(result.is_ok());

        if let Ok(GaiaInstruction::Call(function_name)) = result {
            assert_eq!(function_name, "printf");
        }
        else {
            panic!("Expected Call instruction");
        }
    }
}
