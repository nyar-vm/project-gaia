/// JVM Import 适配器
///
/// 从 JVM 字节码格式导入到 Gaia 统一格式
use super::ImportAdapter;
use crate::instruction::*;
use gaia_types::{helpers::Architecture, *};

/// JVM Import 适配器
#[derive(Debug, Clone)]
pub struct JvmImportAdapter {
    /// 适配器配置
    config: JvmImportConfig,
}

/// JVM Import 配置
#[derive(Debug, Clone)]
pub struct JvmImportConfig {
    /// 是否解析调试信息
    pub parse_debug_info: bool,
    /// 是否解析注解
    pub parse_annotations: bool,
    /// 是否解析泛型信息
    pub parse_generics: bool,
    /// 目标 JVM 版本
    pub target_jvm_version: u16,
}

impl Default for JvmImportConfig {
    fn default() -> Self {
        Self {
            parse_debug_info: false,
            parse_annotations: false,
            parse_generics: false,
            target_jvm_version: 17, // Java 17
        }
    }
}

impl JvmImportAdapter {
    /// 创建新的 JVM Import 适配器
    pub fn new() -> Self {
        Self { config: JvmImportConfig::default() }
    }

    /// 使用指定配置创建 JVM Import 适配器
    pub fn with_config(config: JvmImportConfig) -> Self {
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

impl ImportAdapter<JvmInstruction> for JvmImportAdapter {
    fn import_instruction(&self, jvm_instruction: &JvmInstruction) -> Result<GaiaInstruction> {
        match jvm_instruction.opcode {
            // 常量加载指令
            0x01 => Ok(GaiaInstruction::LoadConstant(GaiaConstant::Null)), // aconst_null
            0x02 => Ok(GaiaInstruction::LoadConstant(GaiaConstant::Integer32(-1))), // iconst_m1
            0x03 => Ok(GaiaInstruction::LoadConstant(GaiaConstant::Integer32(0))), // iconst_0
            0x04 => Ok(GaiaInstruction::LoadConstant(GaiaConstant::Integer32(1))), // iconst_1
            0x05 => Ok(GaiaInstruction::LoadConstant(GaiaConstant::Integer32(2))), // iconst_2
            0x06 => Ok(GaiaInstruction::LoadConstant(GaiaConstant::Integer32(3))), // iconst_3
            0x07 => Ok(GaiaInstruction::LoadConstant(GaiaConstant::Integer32(4))), // iconst_4
            0x08 => Ok(GaiaInstruction::LoadConstant(GaiaConstant::Integer32(5))), // iconst_5
            0x09 => Ok(GaiaInstruction::LoadConstant(GaiaConstant::Integer64(0))), // lconst_0
            0x0A => Ok(GaiaInstruction::LoadConstant(GaiaConstant::Integer64(1))), // lconst_1
            0x0B => Ok(GaiaInstruction::LoadConstant(GaiaConstant::Float32(0.0))), // fconst_0
            0x0C => Ok(GaiaInstruction::LoadConstant(GaiaConstant::Float32(1.0))), // fconst_1
            0x0D => Ok(GaiaInstruction::LoadConstant(GaiaConstant::Float32(2.0))), // fconst_2
            0x0E => Ok(GaiaInstruction::LoadConstant(GaiaConstant::Float64(0.0))), // dconst_0
            0x0F => Ok(GaiaInstruction::LoadConstant(GaiaConstant::Float64(1.0))), // dconst_1

            // 字节和短整型常量
            0x10 => {
                // bipush
                if !jvm_instruction.operands.is_empty() {
                    let value = jvm_instruction.operands[0] as i8 as i32;
                    Ok(GaiaInstruction::LoadConstant(GaiaConstant::Integer32(value)))
                }
                else {
                    Err(GaiaError::invalid_instruction("bipush requires operand", Architecture::Other("JVM".to_string())))
                }
            }
            0x11 => {
                // sipush
                if jvm_instruction.operands.len() >= 2 {
                    let value = ((jvm_instruction.operands[0] as i16) << 8) | (jvm_instruction.operands[1] as i16);
                    Ok(GaiaInstruction::LoadConstant(GaiaConstant::Integer32(value as i32)))
                }
                else {
                    Err(GaiaError::invalid_instruction("sipush requires 2 operands", Architecture::Other("JVM".to_string())))
                }
            }

            // 局部变量加载指令
            0x15 => Ok(GaiaInstruction::LoadLocal(0)), // iload_0
            0x16 => Ok(GaiaInstruction::LoadLocal(1)), // iload_1
            0x17 => Ok(GaiaInstruction::LoadLocal(2)), // iload_2
            0x18 => Ok(GaiaInstruction::LoadLocal(3)), // iload_3
            0x1A => Ok(GaiaInstruction::LoadLocal(0)), // lload_0
            0x1B => Ok(GaiaInstruction::LoadLocal(1)), // lload_1
            0x1C => Ok(GaiaInstruction::LoadLocal(2)), // lload_2
            0x1D => Ok(GaiaInstruction::LoadLocal(3)), // lload_3
            0x1E => Ok(GaiaInstruction::LoadLocal(0)), // fload_0
            0x1F => Ok(GaiaInstruction::LoadLocal(1)), // fload_1
            0x20 => Ok(GaiaInstruction::LoadLocal(2)), // fload_2
            0x21 => Ok(GaiaInstruction::LoadLocal(3)), // fload_3
            0x22 => Ok(GaiaInstruction::LoadLocal(0)), // dload_0
            0x23 => Ok(GaiaInstruction::LoadLocal(1)), // dload_1
            0x24 => Ok(GaiaInstruction::LoadLocal(2)), // dload_2
            0x25 => Ok(GaiaInstruction::LoadLocal(3)), // dload_3
            0x2A => Ok(GaiaInstruction::LoadLocal(0)), // aload_0
            0x2B => Ok(GaiaInstruction::LoadLocal(1)), // aload_1
            0x2C => Ok(GaiaInstruction::LoadLocal(2)), // aload_2
            0x2D => Ok(GaiaInstruction::LoadLocal(3)), // aload_3

            // 局部变量存储指令
            0x3B => Ok(GaiaInstruction::StoreLocal(0)), // istore_0
            0x3C => Ok(GaiaInstruction::StoreLocal(1)), // istore_1
            0x3D => Ok(GaiaInstruction::StoreLocal(2)), // istore_2
            0x3E => Ok(GaiaInstruction::StoreLocal(3)), // istore_3
            0x40 => Ok(GaiaInstruction::StoreLocal(0)), // lstore_0
            0x41 => Ok(GaiaInstruction::StoreLocal(1)), // lstore_1
            0x42 => Ok(GaiaInstruction::StoreLocal(2)), // lstore_2
            0x43 => Ok(GaiaInstruction::StoreLocal(3)), // lstore_3
            0x44 => Ok(GaiaInstruction::StoreLocal(0)), // fstore_0
            0x45 => Ok(GaiaInstruction::StoreLocal(1)), // fstore_1
            0x46 => Ok(GaiaInstruction::StoreLocal(2)), // fstore_2
            0x47 => Ok(GaiaInstruction::StoreLocal(3)), // fstore_3
            0x48 => Ok(GaiaInstruction::StoreLocal(0)), // dstore_0
            0x49 => Ok(GaiaInstruction::StoreLocal(1)), // dstore_1
            0x4A => Ok(GaiaInstruction::StoreLocal(2)), // dstore_2
            0x4B => Ok(GaiaInstruction::StoreLocal(3)), // dstore_3
            0x4C => Ok(GaiaInstruction::StoreLocal(0)), // astore_0
            0x4D => Ok(GaiaInstruction::StoreLocal(1)), // astore_1
            0x4E => Ok(GaiaInstruction::StoreLocal(2)), // astore_2
            0x4F => Ok(GaiaInstruction::StoreLocal(3)), // astore_3

            // 算术指令
            0x60 => Ok(GaiaInstruction::Add),      // iadd
            0x61 => Ok(GaiaInstruction::Add),      // ladd
            0x62 => Ok(GaiaInstruction::Add),      // fadd
            0x63 => Ok(GaiaInstruction::Add),      // dadd
            0x64 => Ok(GaiaInstruction::Subtract), // isub
            0x65 => Ok(GaiaInstruction::Subtract), // lsub
            0x66 => Ok(GaiaInstruction::Subtract), // fsub
            0x67 => Ok(GaiaInstruction::Subtract), // dsub
            0x68 => Ok(GaiaInstruction::Multiply), // imul
            0x69 => Ok(GaiaInstruction::Multiply), // lmul
            0x6A => Ok(GaiaInstruction::Multiply), // fmul
            0x6B => Ok(GaiaInstruction::Multiply), // dmul
            0x6C => Ok(GaiaInstruction::Divide),   // idiv
            0x6D => Ok(GaiaInstruction::Divide),   // ldiv
            0x6E => Ok(GaiaInstruction::Divide),   // fdiv
            0x6F => Ok(GaiaInstruction::Divide),   // ddiv

            // 栈操作指令
            0x59 => Ok(GaiaInstruction::Duplicate), // dup
            0x57 => Ok(GaiaInstruction::Pop),       // pop
            0x58 => Ok(GaiaInstruction::Pop),       // pop2 (简化为 pop)

            // 方法调用指令
            0xB6 => {
                // invokevirtual
                // 这里需要从常量池中解析方法名，暂时使用占位符
                Ok(GaiaInstruction::Call("virtual_method".to_string()))
            }
            0xB7 => {
                // invokespecial
                Ok(GaiaInstruction::Call("special_method".to_string()))
            }
            0xB8 => {
                // invokestatic
                Ok(GaiaInstruction::Call("static_method".to_string()))
            }
            0xB9 => {
                // invokeinterface
                Ok(GaiaInstruction::Call("interface_method".to_string()))
            }

            // 返回指令
            0xB1 => Ok(GaiaInstruction::Return), // return
            0xAC => Ok(GaiaInstruction::Return), // ireturn
            0xAD => Ok(GaiaInstruction::Return), // lreturn
            0xAE => Ok(GaiaInstruction::Return), // freturn
            0xAF => Ok(GaiaInstruction::Return), // dreturn
            0xB0 => Ok(GaiaInstruction::Return), // areturn

            // 其他指令暂时用 Comment 代替
            _ => Ok(GaiaInstruction::Comment("Unsupported JVM instruction".to_string())),
        }
    }

    fn import_program(&self, jvm_instructions: &[JvmInstruction]) -> Result<GaiaProgram> {
        let mut gaia_instructions = Vec::new();

        for jvm_instruction in jvm_instructions {
            let gaia_instruction = self.import_instruction(jvm_instruction)?;
            gaia_instructions.push(gaia_instruction);
        }

        // 创建一个简单的程序结构
        let main_function = GaiaFunction {
            name: "main".to_string(),
            parameters: vec![],
            return_type: Some(GaiaType::Integer32),
            locals: vec![],
            instructions: gaia_instructions,
        };

        Ok(GaiaProgram { name: "ImportedJvmProgram".to_string(), functions: vec![main_function], constants: vec![] })
    }

    fn adapter_name(&self) -> &'static str {
        "JVM Import Adapter"
    }
}

impl Default for JvmImportAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jvm_import_adapter_creation() {
        let adapter = JvmImportAdapter::new();
        assert_eq!(adapter.adapter_name(), "JVM Import Adapter");
    }

    #[test]
    fn test_jvm_iconst_0_import() {
        let adapter = JvmImportAdapter::new();
        let jvm_instruction = JvmInstruction {
            opcode: 0x03, // iconst_0
            operands: vec![],
            metadata: None,
        };

        let result = adapter.import_instruction(&jvm_instruction);
        assert!(result.is_ok());

        match result.unwrap() {
            GaiaInstruction::LoadConstant(GaiaConstant::Integer32(value)) => {
                assert_eq!(value, 0);
            }
            _ => panic!("Expected LoadConstant(Integer32(0))"),
        }
    }

    #[test]
    fn test_jvm_bipush_import() {
        let adapter = JvmImportAdapter::new();
        let jvm_instruction = JvmInstruction {
            opcode: 0x10, // bipush
            operands: vec![42],
            metadata: None,
        };

        let result = adapter.import_instruction(&jvm_instruction);
        assert!(result.is_ok());

        match result.unwrap() {
            GaiaInstruction::LoadConstant(GaiaConstant::Integer32(value)) => {
                assert_eq!(value, 42);
            }
            _ => panic!("Expected LoadConstant(Integer32(42))"),
        }
    }

    #[test]
    fn test_jvm_iload_0_import() {
        let adapter = JvmImportAdapter::new();
        let jvm_instruction = JvmInstruction {
            opcode: 0x15, // iload_0
            operands: vec![],
            metadata: None,
        };

        let result = adapter.import_instruction(&jvm_instruction);
        assert!(result.is_ok());

        match result.unwrap() {
            GaiaInstruction::LoadLocal(index) => {
                assert_eq!(index, 0);
            }
            _ => panic!("Expected LoadLocal(0)"),
        }
    }

    #[test]
    fn test_jvm_iadd_import() {
        let adapter = JvmImportAdapter::new();
        let jvm_instruction = JvmInstruction {
            opcode: 0x60, // iadd
            operands: vec![],
            metadata: None,
        };

        let result = adapter.import_instruction(&jvm_instruction);
        assert!(result.is_ok());

        match result.unwrap() {
            GaiaInstruction::Add => {}
            _ => panic!("Expected Add"),
        }
    }

    #[test]
    fn test_simple_program_import() {
        let adapter = JvmImportAdapter::new();
        let jvm_instructions = vec![
            JvmInstruction {
                opcode: 0x03, // iconst_0
                operands: vec![],
                metadata: None,
            },
            JvmInstruction {
                opcode: 0x3B, // istore_0
                operands: vec![],
                metadata: None,
            },
            JvmInstruction {
                opcode: 0xB1, // return
                operands: vec![],
                metadata: None,
            },
        ];

        let result = adapter.import_program(&jvm_instructions);
        assert!(result.is_ok());

        let gaia_program = result.unwrap();
        assert_eq!(gaia_program.name, "ImportedJvmProgram");
        assert_eq!(gaia_program.functions.len(), 1);
        assert_eq!(gaia_program.functions[0].instructions.len(), 3);
    }
}
