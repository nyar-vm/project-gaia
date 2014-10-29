/// IL Import 适配器
///
/// 从 .NET IL 格式导入到 Gaia 统一格式
use super::ImportAdapter;
use crate::instruction::*;
use gaia_types::*;

/// IL Import 适配器
#[derive(Debug, Clone)]
pub struct IlImportAdapter {
    /// 适配器配置
    config: IlImportConfig,
}

/// IL Import 配置
#[derive(Debug, Clone)]
pub struct IlImportConfig {
    /// 是否保留元数据
    pub preserve_metadata: bool,
    /// 是否解析类型信息
    pub parse_type_info: bool,
    /// 是否处理异常处理块
    pub handle_exceptions: bool,
}

impl Default for IlImportConfig {
    fn default() -> Self {
        Self { preserve_metadata: true, parse_type_info: true, handle_exceptions: true }
    }
}

impl IlImportAdapter {
    /// 创建新的 IL Import 适配器
    pub fn new() -> Self {
        Self { config: IlImportConfig::default() }
    }

    /// 使用指定配置创建 IL Import 适配器
    pub fn with_config(config: IlImportConfig) -> Self {
        Self { config }
    }
}

// 由于 clr-assembler 项目的具体类型还需要进一步查看，这里先定义一个占位符类型
// 实际实现时需要使用 clr-assembler 中的真实类型
#[derive(Debug, Clone)]
pub struct IlInstruction {
    pub opcode: String,
    pub operands: Vec<String>,
    pub metadata: Option<String>,
}

impl ImportAdapter<IlInstruction> for IlImportAdapter {
    fn import_instruction(&self, il_instruction: &IlInstruction) -> Result<GaiaInstruction> {
        // 根据 IL 指令的操作码转换为 Gaia 指令
        match il_instruction.opcode.as_str() {
            "ldconst" | "ldc.i4" => {
                // 加载 32 位整数常量
                if !il_instruction.operands.is_empty() {
                    if let Ok(value) = il_instruction.operands[0].parse::<i32>() {
                        Ok(GaiaInstruction::LoadConstant(GaiaConstant::Integer32(value)))
                    }
                    else {
                        Err(GaiaError::invalid_instruction(
                            &il_instruction.opcode,
                            gaia_types::helpers::Architecture::Other("IL".to_string()),
                        ))
                    }
                }
                else {
                    Err(GaiaError::invalid_instruction(
                        &il_instruction.opcode,
                        gaia_types::helpers::Architecture::Other("IL".to_string()),
                    ))
                }
            }
            "ldc.i8" => {
                // 加载 64 位整数常量
                if !il_instruction.operands.is_empty() {
                    if let Ok(value) = il_instruction.operands[0].parse::<i64>() {
                        Ok(GaiaInstruction::LoadConstant(GaiaConstant::Integer64(value)))
                    }
                    else {
                        Err(GaiaError::invalid_instruction(
                            &il_instruction.opcode,
                            gaia_types::helpers::Architecture::Other("IL".to_string()),
                        ))
                    }
                }
                else {
                    Err(GaiaError::invalid_instruction(
                        &il_instruction.opcode,
                        gaia_types::helpers::Architecture::Other("IL".to_string()),
                    ))
                }
            }
            "ldc.r4" => {
                // 加载 32 位浮点常量
                if !il_instruction.operands.is_empty() {
                    if let Ok(value) = il_instruction.operands[0].parse::<f32>() {
                        Ok(GaiaInstruction::LoadConstant(GaiaConstant::Float32(value)))
                    }
                    else {
                        Err(GaiaError::invalid_instruction(
                            &il_instruction.opcode,
                            gaia_types::helpers::Architecture::Other("IL".to_string()),
                        ))
                    }
                }
                else {
                    Err(GaiaError::invalid_instruction(
                        &il_instruction.opcode,
                        gaia_types::helpers::Architecture::Other("IL".to_string()),
                    ))
                }
            }
            "ldc.r8" => {
                // 加载 64 位浮点常量
                if !il_instruction.operands.is_empty() {
                    if let Ok(value) = il_instruction.operands[0].parse::<f64>() {
                        Ok(GaiaInstruction::LoadConstant(GaiaConstant::Float64(value)))
                    }
                    else {
                        Err(GaiaError::invalid_instruction(
                            &il_instruction.opcode,
                            gaia_types::helpers::Architecture::Other("IL".to_string()),
                        ))
                    }
                }
                else {
                    Err(GaiaError::invalid_instruction(
                        &il_instruction.opcode,
                        gaia_types::helpers::Architecture::Other("IL".to_string()),
                    ))
                }
            }
            "ldstr" => {
                // 加载字符串常量
                if !il_instruction.operands.is_empty() {
                    Ok(GaiaInstruction::LoadConstant(GaiaConstant::String(il_instruction.operands[0].clone())))
                }
                else {
                    Err(GaiaError::invalid_instruction(
                        &il_instruction.opcode,
                        gaia_types::helpers::Architecture::Other("IL".to_string()),
                    ))
                }
            }
            "ldloc" | "ldloc.0" | "ldloc.1" | "ldloc.2" | "ldloc.3" => {
                // 加载局部变量
                let index = if il_instruction.opcode == "ldloc" {
                    if !il_instruction.operands.is_empty() {
                        il_instruction.operands[0].parse::<u32>().unwrap_or(0)
                    }
                    else {
                        0
                    }
                }
                else {
                    // 从操作码中提取索引
                    il_instruction.opcode.chars().last().unwrap_or('0').to_digit(10).unwrap_or(0)
                };
                Ok(GaiaInstruction::LoadLocal(index))
            }
            "stloc" | "stloc.0" | "stloc.1" | "stloc.2" | "stloc.3" => {
                // 存储局部变量
                let index = if il_instruction.opcode == "stloc" {
                    if !il_instruction.operands.is_empty() {
                        il_instruction.operands[0].parse::<u32>().unwrap_or(0)
                    }
                    else {
                        0
                    }
                }
                else {
                    // 从操作码中提取索引
                    il_instruction.opcode.chars().last().unwrap_or('0').to_digit(10).unwrap_or(0)
                };
                Ok(GaiaInstruction::StoreLocal(index))
            }
            "call" => {
                // 函数调用
                if !il_instruction.operands.is_empty() {
                    Ok(GaiaInstruction::Call(il_instruction.operands[0].clone()))
                }
                else {
                    Err(GaiaError::invalid_instruction(
                        &il_instruction.opcode,
                        gaia_types::helpers::Architecture::Other("IL".to_string()),
                    ))
                }
            }
            "ret" => Ok(GaiaInstruction::Return),
            "add" => Ok(GaiaInstruction::Add),
            "sub" => Ok(GaiaInstruction::Subtract),
            "mul" => Ok(GaiaInstruction::Multiply),
            "div" => Ok(GaiaInstruction::Divide),
            "dup" => Ok(GaiaInstruction::Duplicate),
            "pop" => Ok(GaiaInstruction::Pop),
            _ => {
                // 未知指令
                Err(GaiaError::invalid_instruction(
                    &il_instruction.opcode,
                    gaia_types::helpers::Architecture::Other("IL".to_string()),
                ))
            }
        }
    }

    fn import_program(&self, il_instructions: &[IlInstruction]) -> Result<GaiaProgram> {
        let mut gaia_instructions = Vec::new();

        for il_instruction in il_instructions {
            let gaia_instruction = self.import_instruction(&il_instruction.clone())?;
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

        Ok(GaiaProgram { name: "imported_il_program".to_string(), functions: vec![main_function], constants: vec![] })
    }

    fn adapter_name(&self) -> &'static str {
        "IL Import Adapter"
    }
}

impl Default for IlImportAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_il_import_adapter_creation() {
        let adapter = IlImportAdapter::new();
        assert_eq!(adapter.adapter_name(), "IL Import Adapter");
    }

    #[test]
    fn test_il_ldc_i4_instruction_import() {
        let adapter = IlImportAdapter::new();
        let il_instruction = IlInstruction { opcode: "ldc.i4".to_string(), operands: vec!["42".to_string()], metadata: None };

        let result = adapter.import_instruction(il_instruction);
        assert!(result.is_ok());

        if let Ok(GaiaInstruction::LoadConstant(GaiaConstant::Integer32(value))) = result {
            assert_eq!(value, 42);
        }
        else {
            panic!("Expected LoadConstant instruction");
        }
    }

    #[test]
    fn test_il_ldstr_instruction_import() {
        let adapter = IlImportAdapter::new();
        let il_instruction =
            IlInstruction { opcode: "ldstr".to_string(), operands: vec!["Hello, World!".to_string()], metadata: None };

        let result = adapter.import_instruction(il_instruction);
        assert!(result.is_ok());

        if let Ok(GaiaInstruction::LoadConstant(GaiaConstant::String(value))) = result {
            assert_eq!(value, "Hello, World!");
        }
        else {
            panic!("Expected LoadConstant instruction");
        }
    }

    #[test]
    fn test_il_call_instruction_import() {
        let adapter = IlImportAdapter::new();
        let il_instruction = IlInstruction {
            opcode: "call".to_string(),
            operands: vec!["System.Console::WriteLine".to_string()],
            metadata: None,
        };

        let result = adapter.import_instruction(il_instruction);
        assert!(result.is_ok());

        if let Ok(GaiaInstruction::Call(function_name)) = result {
            assert_eq!(function_name, "System.Console::WriteLine");
        }
        else {
            panic!("Expected Call instruction");
        }
    }
}
