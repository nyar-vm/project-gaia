/// WASI Import 适配器
///
/// 从 WASI/WebAssembly 格式导入到 Gaia 统一格式
use super::ImportAdapter;
use crate::instruction::*;
use gaia_types::{helpers::Architecture, *};

/// WASI Import 适配器
#[derive(Debug, Clone)]
pub struct WasiImportAdapter {
    /// 适配器配置
    config: WasiImportConfig,
}

/// WASI Import 配置
#[derive(Debug, Clone)]
pub struct WasiImportConfig {
    /// 是否解析调试信息
    pub parse_debug_info: bool,
    /// 是否解析自定义段
    pub parse_custom_sections: bool,
    /// 是否解析导入/导出表
    pub parse_import_export_tables: bool,
    /// 目标 WASM 版本
    pub target_wasm_version: u32,
}

impl Default for WasiImportConfig {
    fn default() -> Self {
        Self {
            parse_debug_info: false,
            parse_custom_sections: false,
            parse_import_export_tables: true,
            target_wasm_version: 1, // WASM 1.0
        }
    }
}

impl WasiImportAdapter {
    /// 创建新的 WASI Import 适配器
    pub fn new() -> Self {
        Self { config: WasiImportConfig::default() }
    }

    /// 使用指定配置创建 WASI Import 适配器
    pub fn with_config(config: WasiImportConfig) -> Self {
        Self { config }
    }

    /// 设置目标 WASM 版本
    pub fn set_target_wasm_version(&mut self, version: u32) {
        self.config.target_wasm_version = version;
    }
}

// 由于 wasm-assembler 项目的具体类型还需要进一步查看，这里先定义一个占位符类型
// 实际实现时需要使用 wasm-assembler 中的真实类型
#[derive(Debug, Clone)]
pub struct WasiInstruction {
    pub opcode: u8,
    pub operands: Vec<u8>,
    pub metadata: Option<String>,
}

impl ImportAdapter<WasiInstruction> for WasiImportAdapter {
    fn import_instruction(&self, wasi_instruction: &WasiInstruction) -> Result<GaiaInstruction> {
        match wasi_instruction.opcode {
            // 控制指令
            0x00 => Ok(GaiaInstruction::Comment("unreachable".to_string())), // unreachable -> comment
            0x01 => Ok(GaiaInstruction::Comment("nop".to_string())),         // nop

            // 常量指令
            0x41 => {
                // i32.const
                if wasi_instruction.operands.len() >= 4 {
                    let value = i32::from_le_bytes([
                        wasi_instruction.operands[0],
                        wasi_instruction.operands[1],
                        wasi_instruction.operands[2],
                        wasi_instruction.operands[3],
                    ]);
                    Ok(GaiaInstruction::LoadConstant(GaiaConstant::Integer32(value)))
                }
                else {
                    Err(GaiaError::invalid_instruction(
                        "i32.const requires 4 operand bytes",
                        Architecture::Other("WASI".to_string()),
                    ))
                }
            }
            0x42 => {
                // i64.const
                if wasi_instruction.operands.len() >= 8 {
                    let value = i64::from_le_bytes([
                        wasi_instruction.operands[0],
                        wasi_instruction.operands[1],
                        wasi_instruction.operands[2],
                        wasi_instruction.operands[3],
                        wasi_instruction.operands[4],
                        wasi_instruction.operands[5],
                        wasi_instruction.operands[6],
                        wasi_instruction.operands[7],
                    ]);
                    Ok(GaiaInstruction::LoadConstant(GaiaConstant::Integer64(value)))
                }
                else {
                    Err(GaiaError::invalid_instruction(
                        "i64.const requires 8 operand bytes",
                        Architecture::Other("WASI".to_string()),
                    ))
                }
            }
            0x43 => {
                // f32.const
                if wasi_instruction.operands.len() >= 4 {
                    let bytes = [
                        wasi_instruction.operands[0],
                        wasi_instruction.operands[1],
                        wasi_instruction.operands[2],
                        wasi_instruction.operands[3],
                    ];
                    let value = f32::from_le_bytes(bytes);
                    Ok(GaiaInstruction::LoadConstant(GaiaConstant::Float32(value)))
                }
                else {
                    Err(GaiaError::invalid_instruction(
                        "f32.const requires 4 operand bytes",
                        Architecture::Other("WASI".to_string()),
                    ))
                }
            }
            0x44 => {
                // f64.const
                if wasi_instruction.operands.len() >= 8 {
                    let bytes = [
                        wasi_instruction.operands[0],
                        wasi_instruction.operands[1],
                        wasi_instruction.operands[2],
                        wasi_instruction.operands[3],
                        wasi_instruction.operands[4],
                        wasi_instruction.operands[5],
                        wasi_instruction.operands[6],
                        wasi_instruction.operands[7],
                    ];
                    let value = f64::from_le_bytes(bytes);
                    Ok(GaiaInstruction::LoadConstant(GaiaConstant::Float64(value)))
                }
                else {
                    Err(GaiaError::invalid_instruction(
                        "f64.const requires 8 operand bytes",
                        Architecture::Other("WASI".to_string()),
                    ))
                }
            }

            // 局部变量指令
            0x20 => {
                // local.get
                if !wasi_instruction.operands.is_empty() {
                    let index = wasi_instruction.operands[0] as u32;
                    Ok(GaiaInstruction::LoadLocal(index))
                }
                else {
                    Err(GaiaError::invalid_instruction("local.get requires operand", Architecture::Other("WASI".to_string())))
                }
            }
            0x21 => {
                // local.set
                if !wasi_instruction.operands.is_empty() {
                    let index = wasi_instruction.operands[0] as u32;
                    Ok(GaiaInstruction::StoreLocal(index))
                }
                else {
                    Err(GaiaError::invalid_instruction("local.set requires operand", Architecture::Other("WASI".to_string())))
                }
            }
            0x22 => {
                // local.tee
                if !wasi_instruction.operands.is_empty() {
                    let index = wasi_instruction.operands[0] as u32;
                    // local.tee 相当于 dup + local.set，这里简化为 StoreLocal
                    Ok(GaiaInstruction::StoreLocal(index))
                }
                else {
                    Err(GaiaError::invalid_instruction("local.tee requires operand", Architecture::Other("WASI".to_string())))
                }
            }

            // 全局变量指令
            0x23 => {
                // global.get
                if !wasi_instruction.operands.is_empty() {
                    let index = wasi_instruction.operands[0] as u32;
                    // 全局变量访问，映射为 LoadArgument
                    Ok(GaiaInstruction::LoadArgument(index))
                }
                else {
                    Err(GaiaError::invalid_instruction("global.get requires operand", Architecture::Other("WASI".to_string())))
                }
            }
            0x24 => {
                // global.set
                if !wasi_instruction.operands.is_empty() {
                    let index = wasi_instruction.operands[0] as u32;
                    // 全局变量设置，映射为 StoreArgument
                    Ok(GaiaInstruction::StoreArgument(index))
                }
                else {
                    Err(GaiaError::invalid_instruction("global.set requires operand", Architecture::Other("WASI".to_string())))
                }
            }

            // 算术指令 (i32)
            0x6A => Ok(GaiaInstruction::Add),      // i32.add
            0x6B => Ok(GaiaInstruction::Subtract), // i32.sub
            0x6C => Ok(GaiaInstruction::Multiply), // i32.mul
            0x6D => Ok(GaiaInstruction::Divide),   // i32.div_s
            0x6E => Ok(GaiaInstruction::Divide),   // i32.div_u

            // 算术指令 (i64)
            0x7C => Ok(GaiaInstruction::Add),      // i64.add
            0x7D => Ok(GaiaInstruction::Subtract), // i64.sub
            0x7E => Ok(GaiaInstruction::Multiply), // i64.mul
            0x7F => Ok(GaiaInstruction::Divide),   // i64.div_s
            0x80 => Ok(GaiaInstruction::Divide),   // i64.div_u

            // 算术指令 (f32)
            0x92 => Ok(GaiaInstruction::Add),      // f32.add
            0x93 => Ok(GaiaInstruction::Subtract), // f32.sub
            0x94 => Ok(GaiaInstruction::Multiply), // f32.mul
            0x95 => Ok(GaiaInstruction::Divide),   // f32.div

            // 算术指令 (f64)
            0xA0 => Ok(GaiaInstruction::Add),      // f64.add
            0xA1 => Ok(GaiaInstruction::Subtract), // f64.sub
            0xA2 => Ok(GaiaInstruction::Multiply), // f64.mul
            0xA3 => Ok(GaiaInstruction::Divide),   // f64.div

            // 栈操作指令
            0x1A => Ok(GaiaInstruction::Pop),       // drop
            0x1B => Ok(GaiaInstruction::Duplicate), // select (简化为 dup)

            // 函数调用指令
            0x10 => {
                // call
                if !wasi_instruction.operands.is_empty() {
                    let function_index = wasi_instruction.operands[0];
                    Ok(GaiaInstruction::Call(format!("function_{}", function_index)))
                }
                else {
                    Err(GaiaError::invalid_instruction("call requires function index", Architecture::Other("WASI".to_string())))
                }
            }
            0x11 => {
                // call_indirect
                Ok(GaiaInstruction::Call("indirect_call".to_string()))
            }

            // 返回指令
            0x0F => Ok(GaiaInstruction::Return), // return

            // 其他指令暂时用 Comment 代替
            _ => Ok(GaiaInstruction::Comment("Unsupported WASI instruction".to_string())),
        }
    }

    fn import_program(&self, wasi_instructions: &[WasiInstruction]) -> Result<GaiaProgram> {
        let mut gaia_instructions = Vec::new();

        for wasi_instruction in wasi_instructions {
            let gaia_instruction = self.import_instruction(wasi_instruction)?;
            gaia_instructions.push(gaia_instruction);
        }

        // 创建一个简单的程序结构
        let main_function = GaiaFunction {
            name: "_start".to_string(), // WASI 程序的入口点
            parameters: vec![],
            return_type: Some(GaiaType::Integer32),
            locals: vec![],
            instructions: gaia_instructions,
        };

        Ok(GaiaProgram { name: "ImportedWasiProgram".to_string(), functions: vec![main_function], constants: vec![] })
    }

    fn adapter_name(&self) -> &'static str {
        "WASI Import Adapter"
    }
}

impl Default for WasiImportAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasi_import_adapter_creation() {
        let adapter = WasiImportAdapter::new();
        assert_eq!(adapter.adapter_name(), "WASI Import Adapter");
    }

    #[test]
    fn test_wasi_i32_const_import() {
        let adapter = WasiImportAdapter::new();
        let wasi_instruction = WasiInstruction {
            opcode: 0x41,                // i32.const
            operands: vec![42, 0, 0, 0], // 42 in little-endian
            metadata: None,
        };

        let result = adapter.import_instruction(&wasi_instruction);
        assert!(result.is_ok());

        match result.unwrap() {
            GaiaInstruction::LoadConstant(GaiaConstant::Integer32(value)) => {
                assert_eq!(value, 42);
            }
            _ => panic!("Expected LoadConstant(Integer32(42))"),
        }
    }

    #[test]
    fn test_wasi_local_get_import() {
        let adapter = WasiImportAdapter::new();
        let wasi_instruction = WasiInstruction {
            opcode: 0x20, // local.get
            operands: vec![0],
            metadata: None,
        };

        let result = adapter.import_instruction(&wasi_instruction);
        assert!(result.is_ok());

        match result.unwrap() {
            GaiaInstruction::LoadLocal(index) => {
                assert_eq!(index, 0);
            }
            _ => panic!("Expected LoadLocal(0)"),
        }
    }

    #[test]
    fn test_wasi_i32_add_import() {
        let adapter = WasiImportAdapter::new();
        let wasi_instruction = WasiInstruction {
            opcode: 0x6A, // i32.add
            operands: vec![],
            metadata: None,
        };

        let result = adapter.import_instruction(&wasi_instruction);
        assert!(result.is_ok());

        match result.unwrap() {
            GaiaInstruction::Add => {}
            _ => panic!("Expected Add"),
        }
    }

    #[test]
    fn test_wasi_call_import() {
        let adapter = WasiImportAdapter::new();
        let wasi_instruction = WasiInstruction {
            opcode: 0x10,      // call
            operands: vec![5], // function index 5
            metadata: None,
        };

        let result = adapter.import_instruction(&wasi_instruction);
        assert!(result.is_ok());

        match result.unwrap() {
            GaiaInstruction::Call(function_name) => {
                assert_eq!(function_name, "function_5");
            }
            _ => panic!("Expected Call(function_5)"),
        }
    }

    #[test]
    fn test_simple_program_import() {
        let adapter = WasiImportAdapter::new();
        let wasi_instructions = vec![
            WasiInstruction {
                opcode: 0x41, // i32.const
                operands: vec![42, 0, 0, 0],
                metadata: None,
            },
            WasiInstruction {
                opcode: 0x21, // local.set
                operands: vec![0],
                metadata: None,
            },
            WasiInstruction {
                opcode: 0x0F, // return
                operands: vec![],
                metadata: None,
            },
        ];

        let result = adapter.import_program(&wasi_instructions);
        assert!(result.is_ok());

        let gaia_program = result.unwrap();
        assert_eq!(gaia_program.name, "ImportedWasiProgram");
        assert_eq!(gaia_program.functions.len(), 1);
        assert_eq!(gaia_program.functions[0].name, "_start");
        assert_eq!(gaia_program.functions[0].instructions.len(), 3);
    }
}
