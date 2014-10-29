/// IL Export 适配器
///
/// 从 Gaia 统一格式导出到 .NET IL 格式
use super::ExportAdapter;
use crate::instruction::*;
use gaia_types::*;

/// IL Export 适配器
#[derive(Debug, Clone)]
pub struct IlExportAdapter {
    /// 适配器配置
    config: IlExportConfig,
}

/// IL Export 配置
#[derive(Debug, Clone)]
pub struct IlExportConfig {
    /// 是否生成元数据
    pub generate_metadata: bool,
    /// 是否优化 IL 代码
    pub optimize_il: bool,
    /// 目标 .NET 版本
    pub target_framework: String,
    /// 是否生成调试信息
    pub generate_debug_info: bool,
}

impl Default for IlExportConfig {
    fn default() -> Self {
        Self { generate_metadata: true, optimize_il: false, target_framework: "net8.0".to_string(), generate_debug_info: false }
    }
}

impl IlExportAdapter {
    /// 创建新的 IL Export 适配器
    pub fn new() -> Self {
        Self { config: IlExportConfig::default() }
    }

    /// 使用指定配置创建 IL Export 适配器
    pub fn with_config(config: IlExportConfig) -> Self {
        Self { config }
    }

    /// 设置目标框架
    pub fn set_target_framework(&mut self, framework: String) {
        self.config.target_framework = framework;
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

impl ExportAdapter<IlInstruction> for IlExportAdapter {
    fn export_instruction(&self, gaia_instruction: &GaiaInstruction) -> Result<IlInstruction> {
        match gaia_instruction {
            GaiaInstruction::LoadConstant(constant) => match constant {
                GaiaConstant::Integer8(value) => {
                    Ok(IlInstruction { opcode: "ldc.i4".to_string(), operands: vec![value.to_string()], metadata: None })
                }
                GaiaConstant::Integer16(value) => {
                    Ok(IlInstruction { opcode: "ldc.i4".to_string(), operands: vec![value.to_string()], metadata: None })
                }
                GaiaConstant::Integer32(value) => {
                    Ok(IlInstruction { opcode: "ldc.i4".to_string(), operands: vec![value.to_string()], metadata: None })
                }
                GaiaConstant::Integer64(value) => {
                    Ok(IlInstruction { opcode: "ldc.i8".to_string(), operands: vec![value.to_string()], metadata: None })
                }
                GaiaConstant::Float32(value) => {
                    Ok(IlInstruction { opcode: "ldc.r4".to_string(), operands: vec![value.to_string()], metadata: None })
                }
                GaiaConstant::Float64(value) => {
                    Ok(IlInstruction { opcode: "ldc.r8".to_string(), operands: vec![value.to_string()], metadata: None })
                }
                GaiaConstant::String(value) => {
                    Ok(IlInstruction { opcode: "ldstr".to_string(), operands: vec![format!("\"{}\"", value)], metadata: None })
                }
                GaiaConstant::Boolean(value) => Ok(IlInstruction {
                    opcode: "ldc.i4".to_string(),
                    operands: vec![if *value { "1" } else { "0" }.to_string()],
                    metadata: None,
                }),
                GaiaConstant::Null => Ok(IlInstruction { opcode: "ldnull".to_string(), operands: vec![], metadata: None }),
            },
            GaiaInstruction::LoadLocal(index) => match *index {
                0 => Ok(IlInstruction { opcode: "ldloc.0".to_string(), operands: vec![], metadata: None }),
                1 => Ok(IlInstruction { opcode: "ldloc.1".to_string(), operands: vec![], metadata: None }),
                2 => Ok(IlInstruction { opcode: "ldloc.2".to_string(), operands: vec![], metadata: None }),
                3 => Ok(IlInstruction { opcode: "ldloc.3".to_string(), operands: vec![], metadata: None }),
                _ => Ok(IlInstruction { opcode: "ldloc".to_string(), operands: vec![index.to_string()], metadata: None }),
            },
            GaiaInstruction::StoreLocal(index) => match *index {
                0 => Ok(IlInstruction { opcode: "stloc.0".to_string(), operands: vec![], metadata: None }),
                1 => Ok(IlInstruction { opcode: "stloc.1".to_string(), operands: vec![], metadata: None }),
                2 => Ok(IlInstruction { opcode: "stloc.2".to_string(), operands: vec![], metadata: None }),
                3 => Ok(IlInstruction { opcode: "stloc.3".to_string(), operands: vec![], metadata: None }),
                _ => Ok(IlInstruction { opcode: "stloc".to_string(), operands: vec![index.to_string()], metadata: None }),
            },
            GaiaInstruction::LoadArgument(index) => match *index {
                0 => Ok(IlInstruction { opcode: "ldarg.0".to_string(), operands: vec![], metadata: None }),
                1 => Ok(IlInstruction { opcode: "ldarg.1".to_string(), operands: vec![], metadata: None }),
                2 => Ok(IlInstruction { opcode: "ldarg.2".to_string(), operands: vec![], metadata: None }),
                3 => Ok(IlInstruction { opcode: "ldarg.3".to_string(), operands: vec![], metadata: None }),
                _ => Ok(IlInstruction { opcode: "ldarg".to_string(), operands: vec![index.to_string()], metadata: None }),
            },
            GaiaInstruction::StoreArgument(index) => {
                Ok(IlInstruction { opcode: "starg".to_string(), operands: vec![index.to_string()], metadata: None })
            }
            GaiaInstruction::Call(function_name) => {
                Ok(IlInstruction { opcode: "call".to_string(), operands: vec![function_name.clone()], metadata: None })
            }
            GaiaInstruction::Return => Ok(IlInstruction { opcode: "ret".to_string(), operands: vec![], metadata: None }),
            GaiaInstruction::Add => Ok(IlInstruction { opcode: "add".to_string(), operands: vec![], metadata: None }),
            GaiaInstruction::Subtract => Ok(IlInstruction { opcode: "sub".to_string(), operands: vec![], metadata: None }),
            GaiaInstruction::Multiply => Ok(IlInstruction { opcode: "mul".to_string(), operands: vec![], metadata: None }),
            GaiaInstruction::Divide => Ok(IlInstruction { opcode: "div".to_string(), operands: vec![], metadata: None }),
            GaiaInstruction::Duplicate => Ok(IlInstruction { opcode: "dup".to_string(), operands: vec![], metadata: None }),
            GaiaInstruction::Pop => Ok(IlInstruction { opcode: "pop".to_string(), operands: vec![], metadata: None }),
            _ => {
                // 其他指令暂时用 nop 代替
                Ok(IlInstruction { opcode: "nop".to_string(), operands: vec![], metadata: None })
            }
        }
    }

    fn export_program(&self, gaia_program: &GaiaProgram) -> Result<Vec<IlInstruction>> {
        let mut il_instructions = Vec::new();

        // 添加程序头部
        il_instructions.push(IlInstruction {
            opcode: ".assembly".to_string(),
            operands: vec![gaia_program.name.clone()],
            metadata: Some("assembly declaration".to_string()),
        });

        il_instructions.push(IlInstruction {
            opcode: ".ver".to_string(),
            operands: vec!["1:0:0:0".to_string()],
            metadata: None,
        });

        // 处理所有函数
        for function in &gaia_program.functions {
            // 添加方法声明
            il_instructions.push(IlInstruction {
                opcode: ".method".to_string(),
                operands: vec!["public".to_string(), "static".to_string(), "void".to_string(), function.name.clone()],
                metadata: Some("method declaration".to_string()),
            });

            // 添加方法体开始
            il_instructions.push(IlInstruction { opcode: "{".to_string(), operands: vec![], metadata: None });

            // 如果有局部变量，添加 .locals 声明
            if !function.locals.is_empty() {
                let locals_str = function
                    .locals
                    .iter()
                    .enumerate()
                    .map(|(i, t)| format!("[{}] {}", i, self.gaia_type_to_il_type(t)))
                    .collect::<Vec<_>>()
                    .join(", ");

                il_instructions.push(IlInstruction {
                    opcode: ".locals".to_string(),
                    operands: vec![format!("init ({})", locals_str)],
                    metadata: None,
                });
            }

            // 转换函数指令
            for gaia_instruction in &function.instructions {
                let il_instruction = self.export_instruction(gaia_instruction)?;
                il_instructions.push(il_instruction);
            }

            // 添加方法体结束
            il_instructions.push(IlInstruction { opcode: "}".to_string(), operands: vec![], metadata: None });
        }

        Ok(il_instructions)
    }

    fn adapter_name(&self) -> &'static str {
        "IL Export Adapter"
    }

    fn generate_binary(&self, il_instructions: &[IlInstruction]) -> Result<Vec<u8>> {
        // 这里应该调用 clr-assembler 来生成实际的 .NET 程序集
        // 目前先返回一个简单的 IL 文本表示
        let mut il_text = String::new();

        for instruction in il_instructions {
            if instruction.opcode.starts_with('.') || instruction.opcode == "{" || instruction.opcode == "}" {
                // 指令或声明
                il_text.push_str(&instruction.opcode);
                if !instruction.operands.is_empty() {
                    il_text.push(' ');
                    il_text.push_str(&instruction.operands.join(" "));
                }
                il_text.push('\n');
            }
            else {
                // 普通指令，添加缩进
                il_text.push_str("    ");
                il_text.push_str(&instruction.opcode);
                if !instruction.operands.is_empty() {
                    il_text.push(' ');
                    il_text.push_str(&instruction.operands.join(" "));
                }
                il_text.push('\n');
            }
        }

        Ok(il_text.into_bytes())
    }
}

impl IlExportAdapter {
    /// 将 Gaia 类型转换为 IL 类型字符串
    fn gaia_type_to_il_type(&self, gaia_type: &GaiaType) -> String {
        match gaia_type {
            GaiaType::Integer8 => "int8".to_string(),
            GaiaType::Integer16 => "int16".to_string(),
            GaiaType::Integer32 => "int32".to_string(),
            GaiaType::Integer64 => "int64".to_string(),
            GaiaType::Float32 => "float32".to_string(),
            GaiaType::Float64 => "float64".to_string(),
            GaiaType::String => "string".to_string(),
            GaiaType::Boolean => "bool".to_string(),
            GaiaType::Object => "object".to_string(),
            GaiaType::Pointer => "native int".to_string(),
            GaiaType::Array(inner) => format!("{}[]", self.gaia_type_to_il_type(inner)),
            GaiaType::Custom(name) => name.clone(),
        }
    }
}

impl Default for IlExportAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_il_export_adapter_creation() {
        let adapter = IlExportAdapter::new();
        assert_eq!(adapter.adapter_name(), "IL Export Adapter");
    }

    #[test]
    fn test_gaia_load_constant_export() {
        let adapter = IlExportAdapter::new();
        let gaia_instruction = GaiaInstruction::LoadConstant(GaiaConstant::Integer32(42));

        let result = adapter.export_instruction(&gaia_instruction);
        assert!(result.is_ok());

        let il_instruction = result.unwrap();
        assert_eq!(il_instruction.opcode, "ldc.i4");
        assert_eq!(il_instruction.operands, vec!["42"]);
    }

    #[test]
    fn test_gaia_string_constant_export() {
        let adapter = IlExportAdapter::new();
        let gaia_instruction = GaiaInstruction::LoadConstant(GaiaConstant::String("Hello".to_string()));

        let result = adapter.export_instruction(&gaia_instruction);
        assert!(result.is_ok());

        let il_instruction = result.unwrap();
        assert_eq!(il_instruction.opcode, "ldstr");
        assert_eq!(il_instruction.operands, vec!["\"Hello\""]);
    }

    #[test]
    fn test_gaia_call_export() {
        let adapter = IlExportAdapter::new();
        let gaia_instruction = GaiaInstruction::Call("System.Console::WriteLine".to_string());

        let result = adapter.export_instruction(&gaia_instruction);
        assert!(result.is_ok());

        let il_instruction = result.unwrap();
        assert_eq!(il_instruction.opcode, "call");
        assert_eq!(il_instruction.operands, vec!["System.Console::WriteLine"]);
    }

    #[test]
    fn test_simple_program_export() {
        let adapter = IlExportAdapter::new();
        let gaia_program = GaiaProgram {
            name: "TestProgram".to_string(),
            functions: vec![GaiaFunction {
                name: "Main".to_string(),
                parameters: vec![],
                return_type: None,
                locals: vec![GaiaType::Integer32],
                instructions: vec![
                    GaiaInstruction::LoadConstant(GaiaConstant::Integer32(42)),
                    GaiaInstruction::StoreLocal(0),
                    GaiaInstruction::Return,
                ],
            }],
            constants: vec![],
        };

        let result = adapter.export_program(&gaia_program);
        assert!(result.is_ok());

        let il_instructions = result.unwrap();
        assert!(!il_instructions.is_empty());

        // 检查是否包含程序集声明
        assert!(il_instructions
            .iter()
            .any(|inst| inst.opcode == ".assembly" && inst.operands.contains(&"TestProgram".to_string())));

        // 检查是否包含方法声明
        assert!(il_instructions.iter().any(|inst| inst.opcode == ".method" && inst.operands.contains(&"Main".to_string())));
    }
}
