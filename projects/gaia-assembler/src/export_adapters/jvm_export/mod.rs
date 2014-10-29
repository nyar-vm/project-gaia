/// JVM Export 适配器
///
/// 从 Gaia 统一格式导出到 JVM 字节码格式
use super::ExportAdapter;
use crate::instruction::*;
use gaia_types::*;

/// JVM Export 适配器
#[derive(Debug, Clone)]
pub struct JvmExportAdapter {
    /// 适配器配置
    config: JvmExportConfig,
}

/// JVM Export 配置
#[derive(Debug, Clone)]
pub struct JvmExportConfig {
    /// 是否生成调试信息
    pub generate_debug_info: bool,
    /// 是否优化字节码
    pub optimize_bytecode: bool,
    /// 目标 JVM 版本
    pub target_jvm_version: u16,
    /// 是否生成栈映射表
    pub generate_stack_map_table: bool,
}

impl Default for JvmExportConfig {
    fn default() -> Self {
        Self {
            generate_debug_info: false,
            optimize_bytecode: false,
            target_jvm_version: 17, // Java 17
            generate_stack_map_table: true,
        }
    }
}

impl JvmExportAdapter {
    /// 创建新的 JVM Export 适配器
    pub fn new() -> Self {
        Self { config: JvmExportConfig::default() }
    }

    /// 使用指定配置创建 JVM Export 适配器
    pub fn with_config(config: JvmExportConfig) -> Self {
        Self { config }
    }

    /// 设置目标 JVM 版本
    pub fn set_target_jvm_version(&mut self, version: u16) {
        self.config.target_jvm_version = version;
    }
}

// 由于 jvm-assembler 项目的具体类型还需要进一步查看，这里先定义一个占位符类型
// 实际实现时需要使用 jvm-assembler 中的真实类型
#[derive(Debug, Clone)]
pub struct JvmInstruction {
    pub opcode: u8,
    pub operands: Vec<u8>,
    pub metadata: Option<String>,
}

impl ExportAdapter<JvmInstruction> for JvmExportAdapter {
    fn export_instruction(&self, gaia_instruction: &GaiaInstruction) -> Result<JvmInstruction> {
        match gaia_instruction {
            GaiaInstruction::LoadConstant(constant) => {
                match constant {
                    GaiaConstant::Integer8(value) => {
                        let value = *value as i32; // 扩展为32位
                        match value {
                            -1 => Ok(JvmInstruction {
                                opcode: 0x02, // iconst_m1
                                operands: vec![],
                                metadata: None,
                            }),
                            0 => Ok(JvmInstruction {
                                opcode: 0x03, // iconst_0
                                operands: vec![],
                                metadata: None,
                            }),
                            1 => Ok(JvmInstruction {
                                opcode: 0x04, // iconst_1
                                operands: vec![],
                                metadata: None,
                            }),
                            2 => Ok(JvmInstruction {
                                opcode: 0x05, // iconst_2
                                operands: vec![],
                                metadata: None,
                            }),
                            3 => Ok(JvmInstruction {
                                opcode: 0x06, // iconst_3
                                operands: vec![],
                                metadata: None,
                            }),
                            4 => Ok(JvmInstruction {
                                opcode: 0x07, // iconst_4
                                operands: vec![],
                                metadata: None,
                            }),
                            5 => Ok(JvmInstruction {
                                opcode: 0x08, // iconst_5
                                operands: vec![],
                                metadata: None,
                            }),
                            _ => Ok(JvmInstruction {
                                opcode: 0x10, // bipush
                                operands: vec![value as u8],
                                metadata: None,
                            }),
                        }
                    }
                    GaiaConstant::Integer16(value) => {
                        let value = *value as i32; // 扩展为32位
                        match value {
                            -1 => Ok(JvmInstruction {
                                opcode: 0x02, // iconst_m1
                                operands: vec![],
                                metadata: None,
                            }),
                            0 => Ok(JvmInstruction {
                                opcode: 0x03, // iconst_0
                                operands: vec![],
                                metadata: None,
                            }),
                            1 => Ok(JvmInstruction {
                                opcode: 0x04, // iconst_1
                                operands: vec![],
                                metadata: None,
                            }),
                            2 => Ok(JvmInstruction {
                                opcode: 0x05, // iconst_2
                                operands: vec![],
                                metadata: None,
                            }),
                            3 => Ok(JvmInstruction {
                                opcode: 0x06, // iconst_3
                                operands: vec![],
                                metadata: None,
                            }),
                            4 => Ok(JvmInstruction {
                                opcode: 0x07, // iconst_4
                                operands: vec![],
                                metadata: None,
                            }),
                            5 => Ok(JvmInstruction {
                                opcode: 0x08, // iconst_5
                                operands: vec![],
                                metadata: None,
                            }),
                            -128..=127 => Ok(JvmInstruction {
                                opcode: 0x10, // bipush
                                operands: vec![value as u8],
                                metadata: None,
                            }),
                            _ => Ok(JvmInstruction {
                                opcode: 0x11, // sipush
                                operands: vec![(value >> 8) as u8, (value & 0xFF) as u8],
                                metadata: None,
                            }),
                        }
                    }
                    GaiaConstant::Integer32(value) => {
                        match *value {
                            -1 => Ok(JvmInstruction {
                                opcode: 0x02, // iconst_m1
                                operands: vec![],
                                metadata: None,
                            }),
                            0 => Ok(JvmInstruction {
                                opcode: 0x03, // iconst_0
                                operands: vec![],
                                metadata: None,
                            }),
                            1 => Ok(JvmInstruction {
                                opcode: 0x04, // iconst_1
                                operands: vec![],
                                metadata: None,
                            }),
                            2 => Ok(JvmInstruction {
                                opcode: 0x05, // iconst_2
                                operands: vec![],
                                metadata: None,
                            }),
                            3 => Ok(JvmInstruction {
                                opcode: 0x06, // iconst_3
                                operands: vec![],
                                metadata: None,
                            }),
                            4 => Ok(JvmInstruction {
                                opcode: 0x07, // iconst_4
                                operands: vec![],
                                metadata: None,
                            }),
                            5 => Ok(JvmInstruction {
                                opcode: 0x08, // iconst_5
                                operands: vec![],
                                metadata: None,
                            }),
                            -128..=127 => Ok(JvmInstruction {
                                opcode: 0x10, // bipush
                                operands: vec![*value as u8],
                                metadata: None,
                            }),
                            -32768..=32767 => Ok(JvmInstruction {
                                opcode: 0x11, // sipush
                                operands: vec![(*value >> 8) as u8, (*value & 0xFF) as u8],
                                metadata: None,
                            }),
                            _ => Ok(JvmInstruction {
                                opcode: 0x12,      // ldc (需要常量池索引)
                                operands: vec![1], // 占位符常量池索引
                                metadata: Some(format!("constant: {}", value)),
                            }),
                        }
                    }
                    GaiaConstant::Integer64(value) => {
                        match *value {
                            0 => Ok(JvmInstruction {
                                opcode: 0x09, // lconst_0
                                operands: vec![],
                                metadata: None,
                            }),
                            1 => Ok(JvmInstruction {
                                opcode: 0x0A, // lconst_1
                                operands: vec![],
                                metadata: None,
                            }),
                            _ => Ok(JvmInstruction {
                                opcode: 0x14,         // ldc2_w (需要常量池索引)
                                operands: vec![0, 1], // 占位符常量池索引
                                metadata: Some(format!("long constant: {}", value)),
                            }),
                        }
                    }
                    GaiaConstant::Float32(value) => {
                        if *value == 0.0 {
                            Ok(JvmInstruction {
                                opcode: 0x0B, // fconst_0
                                operands: vec![],
                                metadata: None,
                            })
                        }
                        else if *value == 1.0 {
                            Ok(JvmInstruction {
                                opcode: 0x0C, // fconst_1
                                operands: vec![],
                                metadata: None,
                            })
                        }
                        else if *value == 2.0 {
                            Ok(JvmInstruction {
                                opcode: 0x0D, // fconst_2
                                operands: vec![],
                                metadata: None,
                            })
                        }
                        else {
                            Ok(JvmInstruction {
                                opcode: 0x12,      // ldc (需要常量池索引)
                                operands: vec![1], // 占位符常量池索引
                                metadata: Some(format!("float constant: {}", value)),
                            })
                        }
                    }
                    GaiaConstant::Float64(value) => {
                        if *value == 0.0 {
                            Ok(JvmInstruction {
                                opcode: 0x0E, // dconst_0
                                operands: vec![],
                                metadata: None,
                            })
                        }
                        else if *value == 1.0 {
                            Ok(JvmInstruction {
                                opcode: 0x0F, // dconst_1
                                operands: vec![],
                                metadata: None,
                            })
                        }
                        else {
                            Ok(JvmInstruction {
                                opcode: 0x14,         // ldc2_w (需要常量池索引)
                                operands: vec![0, 1], // 占位符常量池索引
                                metadata: Some(format!("double constant: {}", value)),
                            })
                        }
                    }
                    GaiaConstant::String(_value) => {
                        Ok(JvmInstruction {
                            opcode: 0x12,      // ldc (需要常量池索引)
                            operands: vec![1], // 占位符常量池索引
                            metadata: Some("string constant".to_string()),
                        })
                    }
                    GaiaConstant::Boolean(value) => {
                        Ok(JvmInstruction {
                            opcode: if *value { 0x04 } else { 0x03 }, // iconst_1 或 iconst_0
                            operands: vec![],
                            metadata: None,
                        })
                    }
                    GaiaConstant::Null => {
                        Ok(JvmInstruction {
                            opcode: 0x01, // aconst_null
                            operands: vec![],
                            metadata: None,
                        })
                    }
                }
            }
            GaiaInstruction::LoadLocal(index) => {
                match *index {
                    0 => Ok(JvmInstruction {
                        opcode: 0x1A, // aload_0 (假设是引用类型)
                        operands: vec![],
                        metadata: None,
                    }),
                    1 => Ok(JvmInstruction {
                        opcode: 0x1B, // aload_1
                        operands: vec![],
                        metadata: None,
                    }),
                    2 => Ok(JvmInstruction {
                        opcode: 0x1C, // aload_2
                        operands: vec![],
                        metadata: None,
                    }),
                    3 => Ok(JvmInstruction {
                        opcode: 0x1D, // aload_3
                        operands: vec![],
                        metadata: None,
                    }),
                    _ => Ok(JvmInstruction {
                        opcode: 0x19, // aload
                        operands: vec![*index as u8],
                        metadata: None,
                    }),
                }
            }
            GaiaInstruction::StoreLocal(index) => {
                match *index {
                    0 => Ok(JvmInstruction {
                        opcode: 0x4C, // astore_0 (假设是引用类型)
                        operands: vec![],
                        metadata: None,
                    }),
                    1 => Ok(JvmInstruction {
                        opcode: 0x4D, // astore_1
                        operands: vec![],
                        metadata: None,
                    }),
                    2 => Ok(JvmInstruction {
                        opcode: 0x4E, // astore_2
                        operands: vec![],
                        metadata: None,
                    }),
                    3 => Ok(JvmInstruction {
                        opcode: 0x4F, // astore_3
                        operands: vec![],
                        metadata: None,
                    }),
                    _ => Ok(JvmInstruction {
                        opcode: 0x3A, // astore
                        operands: vec![*index as u8],
                        metadata: None,
                    }),
                }
            }
            GaiaInstruction::LoadArgument(index) => {
                // JVM 中参数和局部变量使用相同的指令
                match *index {
                    0 => Ok(JvmInstruction {
                        opcode: 0x2A, // aload_0 (this)
                        operands: vec![],
                        metadata: None,
                    }),
                    1 => Ok(JvmInstruction {
                        opcode: 0x2B, // aload_1
                        operands: vec![],
                        metadata: None,
                    }),
                    2 => Ok(JvmInstruction {
                        opcode: 0x2C, // aload_2
                        operands: vec![],
                        metadata: None,
                    }),
                    3 => Ok(JvmInstruction {
                        opcode: 0x2D, // aload_3
                        operands: vec![],
                        metadata: None,
                    }),
                    _ => Ok(JvmInstruction {
                        opcode: 0x19, // aload
                        operands: vec![*index as u8],
                        metadata: None,
                    }),
                }
            }
            GaiaInstruction::Call(_function_name) => {
                // 这里需要根据函数类型选择不同的调用指令
                Ok(JvmInstruction {
                    opcode: 0xB8,         // invokestatic (假设是静态方法)
                    operands: vec![0, 1], // 占位符常量池索引
                    metadata: Some("method call".to_string()),
                })
            }
            GaiaInstruction::Return => {
                Ok(JvmInstruction {
                    opcode: 0xB1, // return (void)
                    operands: vec![],
                    metadata: None,
                })
            }
            GaiaInstruction::Add => {
                Ok(JvmInstruction {
                    opcode: 0x60, // iadd (假设是整数加法)
                    operands: vec![],
                    metadata: None,
                })
            }
            GaiaInstruction::Subtract => {
                Ok(JvmInstruction {
                    opcode: 0x64, // isub (假设是整数减法)
                    operands: vec![],
                    metadata: None,
                })
            }
            GaiaInstruction::Multiply => {
                Ok(JvmInstruction {
                    opcode: 0x68, // imul (假设是整数乘法)
                    operands: vec![],
                    metadata: None,
                })
            }
            GaiaInstruction::Divide => {
                Ok(JvmInstruction {
                    opcode: 0x6C, // idiv (假设是整数除法)
                    operands: vec![],
                    metadata: None,
                })
            }
            GaiaInstruction::Duplicate => {
                Ok(JvmInstruction {
                    opcode: 0x59, // dup
                    operands: vec![],
                    metadata: None,
                })
            }
            GaiaInstruction::Pop => {
                Ok(JvmInstruction {
                    opcode: 0x57, // pop
                    operands: vec![],
                    metadata: None,
                })
            }
            _ => {
                // 其他指令暂时用 nop 代替
                Ok(JvmInstruction {
                    opcode: 0x00, // nop
                    operands: vec![],
                    metadata: None,
                })
            }
        }
    }

    fn export_program(&self, gaia_program: &GaiaProgram) -> Result<Vec<JvmInstruction>> {
        let mut jvm_instructions = Vec::new();

        // 处理所有函数
        for function in &gaia_program.functions {
            // 转换函数指令
            for gaia_instruction in &function.instructions {
                let jvm_instruction = self.export_instruction(gaia_instruction)?;
                jvm_instructions.push(jvm_instruction);
            }
        }

        Ok(jvm_instructions)
    }

    fn adapter_name(&self) -> &'static str {
        "JVM Export Adapter"
    }

    fn generate_binary(&self, jvm_instructions: &[JvmInstruction]) -> Result<Vec<u8>> {
        // 这里应该生成完整的 .class 文件格式
        // 目前先返回一个简单的字节码序列
        let mut bytecode = Vec::new();

        // 简化的 .class 文件头部 (魔数)
        bytecode.extend_from_slice(&[0xCA, 0xFE, 0xBA, 0xBE]); // 魔数
        bytecode.extend_from_slice(&[0x00, 0x00]); // 次版本号
        bytecode.extend_from_slice(&[0x00, 0x3D]); // 主版本号 (Java 17)

        // 常量池计数 (占位符)
        bytecode.extend_from_slice(&[0x00, 0x01]); // 常量池大小

        // 访问标志 (占位符)
        bytecode.extend_from_slice(&[0x00, 0x21]); // ACC_PUBLIC | ACC_SUPER

        // 类索引 (占位符)
        bytecode.extend_from_slice(&[0x00, 0x00]);

        // 父类索引 (占位符)
        bytecode.extend_from_slice(&[0x00, 0x00]);

        // 接口计数
        bytecode.extend_from_slice(&[0x00, 0x00]);

        // 字段计数
        bytecode.extend_from_slice(&[0x00, 0x00]);

        // 方法计数
        bytecode.extend_from_slice(&[0x00, 0x01]);

        // 方法信息 (占位符)
        bytecode.extend_from_slice(&[0x00, 0x09]); // 访问标志 ACC_PUBLIC | ACC_STATIC
        bytecode.extend_from_slice(&[0x00, 0x00]); // 名称索引
        bytecode.extend_from_slice(&[0x00, 0x00]); // 描述符索引
        bytecode.extend_from_slice(&[0x00, 0x01]); // 属性计数

        // Code 属性
        bytecode.extend_from_slice(&[0x00, 0x00]); // 属性名称索引

        // Code 属性长度 (占位符)
        let code_length_pos = bytecode.len();
        bytecode.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);

        // 最大栈深度和局部变量数 (占位符)
        bytecode.extend_from_slice(&[0x00, 0x02]); // max_stack
        bytecode.extend_from_slice(&[0x00, 0x01]); // max_locals

        // 代码长度
        let code_start_pos = bytecode.len() + 4;
        let code_length = jvm_instructions.len() as u32;
        bytecode.extend_from_slice(&[((code_length & 0xFF000000) >> 24) as u8]);
        bytecode.extend_from_slice(&[((code_length & 0x00FF0000) >> 16) as u8]);
        bytecode.extend_from_slice(&[((code_length & 0x0000FF00) >> 8) as u8]);
        bytecode.extend_from_slice(&[(code_length & 0x000000FF) as u8]);

        // 实际字节码
        for instruction in jvm_instructions {
            bytecode.push(instruction.opcode);
            bytecode.extend_from_slice(&instruction.operands);
        }

        // 异常表长度
        bytecode.extend_from_slice(&[0x00, 0x00]);

        // 代码属性的属性计数
        bytecode.extend_from_slice(&[0x00, 0x00]);

        // 更新 Code 属性长度
        let code_length = bytecode.len() - code_start_pos;
        bytecode[code_length_pos] = ((code_length & 0xFF000000) >> 24) as u8;
        bytecode[code_length_pos + 1] = ((code_length & 0x00FF0000) >> 16) as u8;
        bytecode[code_length_pos + 2] = ((code_length & 0x0000FF00) >> 8) as u8;
        bytecode[code_length_pos + 3] = (code_length & 0x000000FF) as u8;

        // 类属性计数
        bytecode.extend_from_slice(&[0x00, 0x00]);

        Ok(bytecode)
    }
}

impl Default for JvmExportAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jvm_export_adapter_creation() {
        let adapter = JvmExportAdapter::new();
        assert_eq!(adapter.adapter_name(), "JVM Export Adapter");
    }

    #[test]
    fn test_gaia_load_constant_export() {
        let adapter = JvmExportAdapter::new();
        let gaia_instruction = GaiaInstruction::LoadConstant(GaiaConstant::Integer32(0));

        let result = adapter.export_instruction(&gaia_instruction);
        assert!(result.is_ok());

        let jvm_instruction = result.unwrap();
        assert_eq!(jvm_instruction.opcode, 0x03); // iconst_0
        assert!(jvm_instruction.operands.is_empty());
    }

    #[test]
    fn test_gaia_bipush_export() {
        let adapter = JvmExportAdapter::new();
        let gaia_instruction = GaiaInstruction::LoadConstant(GaiaConstant::Integer32(42));

        let result = adapter.export_instruction(&gaia_instruction);
        assert!(result.is_ok());

        let jvm_instruction = result.unwrap();
        assert_eq!(jvm_instruction.opcode, 0x10); // bipush
        assert_eq!(jvm_instruction.operands, vec![42]);
    }

    #[test]
    fn test_gaia_load_local_export() {
        let adapter = JvmExportAdapter::new();
        let gaia_instruction = GaiaInstruction::LoadLocal(0);

        let result = adapter.export_instruction(&gaia_instruction);
        assert!(result.is_ok());

        let jvm_instruction = result.unwrap();
        assert_eq!(jvm_instruction.opcode, 0x1A); // aload_0
        assert!(jvm_instruction.operands.is_empty());
    }

    #[test]
    fn test_gaia_add_export() {
        let adapter = JvmExportAdapter::new();
        let gaia_instruction = GaiaInstruction::Add;

        let result = adapter.export_instruction(&gaia_instruction);
        assert!(result.is_ok());

        let jvm_instruction = result.unwrap();
        assert_eq!(jvm_instruction.opcode, 0x60); // iadd
        assert!(jvm_instruction.operands.is_empty());
    }

    #[test]
    fn test_simple_program_export() {
        let adapter = JvmExportAdapter::new();
        let gaia_program = GaiaProgram {
            name: "TestProgram".to_string(),
            functions: vec![GaiaFunction {
                name: "main".to_string(),
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

        let jvm_instructions = result.unwrap();
        assert_eq!(jvm_instructions.len(), 3);

        // 检查第一条指令是 bipush 42
        assert_eq!(jvm_instructions[0].opcode, 0x10); // bipush
        assert_eq!(jvm_instructions[0].operands, vec![42]);

        // 检查第二条指令是 astore_0
        assert_eq!(jvm_instructions[1].opcode, 0x4C); // astore_0

        // 检查第三条指令是 return
        assert_eq!(jvm_instructions[2].opcode, 0xB1); // return
    }
}
