/// WASI Export 适配器
///
/// 从 Gaia 统一格式导出到 WASI/WebAssembly 格式
use super::ExportAdapter;
use crate::instruction::*;
use gaia_types::*;

/// WASI Export 适配器
#[derive(Debug, Clone)]
pub struct WasiExportAdapter {
    /// 适配器配置
    config: WasiExportConfig,
}

/// WASI Export 配置
#[derive(Debug, Clone)]
pub struct WasiExportConfig {
    /// 是否生成调试信息
    pub generate_debug_info: bool,
    /// 是否优化 WASM 代码
    pub optimize_wasm: bool,
    /// 目标 WASM 版本
    pub target_wasm_version: u32,
    /// 是否生成 WASI 导入
    pub generate_wasi_imports: bool,
}

impl Default for WasiExportConfig {
    fn default() -> Self {
        Self {
            generate_debug_info: false,
            optimize_wasm: false,
            target_wasm_version: 1, // WASM 1.0
            generate_wasi_imports: true,
        }
    }
}

impl WasiExportAdapter {
    /// 创建新的 WASI Export 适配器
    pub fn new() -> Self {
        Self { config: WasiExportConfig::default() }
    }

    /// 使用指定配置创建 WASI Export 适配器
    pub fn with_config(config: WasiExportConfig) -> Self {
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

impl ExportAdapter<WasiInstruction> for WasiExportAdapter {
    fn export_instruction(&self, gaia_instruction: &GaiaInstruction) -> Result<WasiInstruction> {
        match gaia_instruction {
            GaiaInstruction::LoadConstant(constant) => {
                match constant {
                    GaiaConstant::Integer8(value) => {
                        Ok(WasiInstruction {
                            opcode: 0x41, // i32.const (8位整数扩展为32位)
                            operands: (*value as i32).to_le_bytes().to_vec(),
                            metadata: None,
                        })
                    }
                    GaiaConstant::Integer16(value) => {
                        Ok(WasiInstruction {
                            opcode: 0x41, // i32.const (16位整数扩展为32位)
                            operands: (*value as i32).to_le_bytes().to_vec(),
                            metadata: None,
                        })
                    }
                    GaiaConstant::Integer32(value) => {
                        Ok(WasiInstruction {
                            opcode: 0x41, // i32.const
                            operands: value.to_le_bytes().to_vec(),
                            metadata: None,
                        })
                    }
                    GaiaConstant::Integer64(value) => {
                        Ok(WasiInstruction {
                            opcode: 0x42, // i64.const
                            operands: value.to_le_bytes().to_vec(),
                            metadata: None,
                        })
                    }
                    GaiaConstant::Float32(value) => {
                        Ok(WasiInstruction {
                            opcode: 0x43, // f32.const
                            operands: value.to_le_bytes().to_vec(),
                            metadata: None,
                        })
                    }
                    GaiaConstant::Float64(value) => {
                        Ok(WasiInstruction {
                            opcode: 0x44, // f64.const
                            operands: value.to_le_bytes().to_vec(),
                            metadata: None,
                        })
                    }
                    GaiaConstant::String(_value) => {
                        // 字符串常量需要存储在数据段中，这里简化处理
                        Ok(WasiInstruction {
                            opcode: 0x41,               // i32.const (字符串地址)
                            operands: vec![0, 0, 0, 0], // 占位符地址
                            metadata: Some("string constant address".to_string()),
                        })
                    }
                    GaiaConstant::Boolean(value) => {
                        Ok(WasiInstruction {
                            opcode: 0x41, // i32.const
                            operands: if *value { 1i32 } else { 0i32 }.to_le_bytes().to_vec(),
                            metadata: None,
                        })
                    }
                    GaiaConstant::Null => {
                        Ok(WasiInstruction {
                            opcode: 0x41, // i32.const
                            operands: 0i32.to_le_bytes().to_vec(),
                            metadata: None,
                        })
                    }
                }
            }
            GaiaInstruction::LoadLocal(index) => {
                Ok(WasiInstruction {
                    opcode: 0x20, // local.get
                    operands: self.encode_leb128_u32(*index),
                    metadata: None,
                })
            }
            GaiaInstruction::StoreLocal(index) => {
                Ok(WasiInstruction {
                    opcode: 0x21, // local.set
                    operands: self.encode_leb128_u32(*index),
                    metadata: None,
                })
            }
            GaiaInstruction::LoadArgument(index) => {
                // 在 WASM 中，参数也是局部变量
                Ok(WasiInstruction {
                    opcode: 0x20, // local.get
                    operands: self.encode_leb128_u32(*index),
                    metadata: None,
                })
            }
            GaiaInstruction::StoreArgument(index) => {
                // 在 WASM 中，参数也是局部变量
                Ok(WasiInstruction {
                    opcode: 0x21, // local.set
                    operands: self.encode_leb128_u32(*index),
                    metadata: None,
                })
            }
            GaiaInstruction::Call(function_name) => {
                // 这里需要从函数名映射到函数索引，暂时使用占位符
                Ok(WasiInstruction {
                    opcode: 0x10,      // call
                    operands: vec![0], // 占位符函数索引
                    metadata: Some(format!("call {}", function_name)),
                })
            }
            GaiaInstruction::Return => {
                Ok(WasiInstruction {
                    opcode: 0x0F, // return
                    operands: vec![],
                    metadata: None,
                })
            }
            GaiaInstruction::Add => {
                // 默认使用 i32.add，实际应该根据类型选择
                Ok(WasiInstruction {
                    opcode: 0x6A, // i32.add
                    operands: vec![],
                    metadata: None,
                })
            }
            GaiaInstruction::Subtract => {
                Ok(WasiInstruction {
                    opcode: 0x6B, // i32.sub
                    operands: vec![],
                    metadata: None,
                })
            }
            GaiaInstruction::Multiply => {
                Ok(WasiInstruction {
                    opcode: 0x6C, // i32.mul
                    operands: vec![],
                    metadata: None,
                })
            }
            GaiaInstruction::Divide => {
                Ok(WasiInstruction {
                    opcode: 0x6D, // i32.div_s
                    operands: vec![],
                    metadata: None,
                })
            }
            GaiaInstruction::Duplicate => {
                // WASM 没有直接的 dup 指令，需要使用 local.get + local.tee 模拟
                Ok(WasiInstruction {
                    opcode: 0x22,      // local.tee (简化处理)
                    operands: vec![0], // 临时局部变量索引
                    metadata: Some("duplicate simulation".to_string()),
                })
            }
            GaiaInstruction::Pop => {
                Ok(WasiInstruction {
                    opcode: 0x1A, // drop
                    operands: vec![],
                    metadata: None,
                })
            }
            GaiaInstruction::Comment(_) => {
                Ok(WasiInstruction {
                    opcode: 0x01, // nop
                    operands: vec![],
                    metadata: None,
                })
            }
            _ => {
                // 其他指令暂时用 nop 代替
                Ok(WasiInstruction {
                    opcode: 0x01, // nop
                    operands: vec![],
                    metadata: None,
                })
            }
        }
    }

    fn export_program(&self, gaia_program: &GaiaProgram) -> Result<Vec<WasiInstruction>> {
        let mut wasi_instructions = Vec::new();

        // 处理所有函数
        for function in &gaia_program.functions {
            // 转换函数指令
            for gaia_instruction in &function.instructions {
                let wasi_instruction = self.export_instruction(gaia_instruction)?;
                wasi_instructions.push(wasi_instruction);
            }
        }

        Ok(wasi_instructions)
    }

    fn adapter_name(&self) -> &'static str {
        "WASI Export Adapter"
    }

    fn generate_binary(&self, wasi_instructions: &[WasiInstruction]) -> Result<Vec<u8>> {
        // 这里应该生成完整的 WASM 模块格式
        // 目前先返回一个简单的 WASM 模块结构
        let mut wasm_module = Vec::new();

        // WASM 魔数和版本
        wasm_module.extend_from_slice(&[0x00, 0x61, 0x73, 0x6D]); // "\0asm"
        wasm_module.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]); // version 1

        // Type Section (函数签名)
        wasm_module.push(0x01); // section id
        wasm_module.push(0x07); // section size
        wasm_module.push(0x01); // 1 type
        wasm_module.push(0x60); // func type
        wasm_module.push(0x00); // 0 parameters
        wasm_module.push(0x01); // 1 result
        wasm_module.push(0x7F); // i32

        // Function Section (函数索引)
        wasm_module.push(0x03); // section id
        wasm_module.push(0x02); // section size
        wasm_module.push(0x01); // 1 function
        wasm_module.push(0x00); // type index 0

        // Export Section (导出函数)
        wasm_module.push(0x07); // section id
        wasm_module.push(0x07); // section size
        wasm_module.push(0x01); // 1 export
        wasm_module.push(0x04); // name length
        wasm_module.extend_from_slice(b"main"); // export name
        wasm_module.push(0x00); // export kind (function)
        wasm_module.push(0x00); // function index

        // Code Section (函数体)
        wasm_module.push(0x0A); // section id

        // 计算代码段大小
        let mut code_body = Vec::new();
        code_body.push(0x01); // 1 function body

        // 函数体
        let mut function_body = Vec::new();
        function_body.push(0x00); // 0 locals

        // 添加指令
        for instruction in wasi_instructions {
            function_body.push(instruction.opcode);
            function_body.extend_from_slice(&instruction.operands);
        }

        // 添加 end 指令
        function_body.push(0x0B); // end

        // 编码函数体大小
        let function_body_size = self.encode_leb128_u32(function_body.len() as u32);
        code_body.extend_from_slice(&function_body_size);
        code_body.extend_from_slice(&function_body);

        // 编码代码段大小
        let code_section_size = self.encode_leb128_u32(code_body.len() as u32);
        wasm_module.extend_from_slice(&code_section_size);
        wasm_module.extend_from_slice(&code_body);

        Ok(wasm_module)
    }
}

impl WasiExportAdapter {
    /// 编码 LEB128 无符号整数
    fn encode_leb128_u32(&self, mut value: u32) -> Vec<u8> {
        let mut result = Vec::new();
        loop {
            let mut byte = (value & 0x7F) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80;
            }
            result.push(byte);
            if value == 0 {
                break;
            }
        }
        result
    }
}

impl Default for WasiExportAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasi_export_adapter_creation() {
        let adapter = WasiExportAdapter::new();
        assert_eq!(adapter.adapter_name(), "WASI Export Adapter");
    }

    #[test]
    fn test_gaia_load_constant_export() {
        let adapter = WasiExportAdapter::new();
        let gaia_instruction = GaiaInstruction::LoadConstant(GaiaConstant::Integer32(42));

        let result = adapter.export_instruction(&gaia_instruction);
        assert!(result.is_ok());

        let wasi_instruction = result.unwrap();
        assert_eq!(wasi_instruction.opcode, 0x41); // i32.const
        assert_eq!(wasi_instruction.operands, vec![42, 0, 0, 0]); // 42 in little-endian
    }

    #[test]
    fn test_gaia_load_local_export() {
        let adapter = WasiExportAdapter::new();
        let gaia_instruction = GaiaInstruction::LoadLocal(0);

        let result = adapter.export_instruction(&gaia_instruction);
        assert!(result.is_ok());

        let wasi_instruction = result.unwrap();
        assert_eq!(wasi_instruction.opcode, 0x20); // local.get
        assert_eq!(wasi_instruction.operands, vec![0]); // index 0
    }

    #[test]
    fn test_gaia_add_export() {
        let adapter = WasiExportAdapter::new();
        let gaia_instruction = GaiaInstruction::Add;

        let result = adapter.export_instruction(&gaia_instruction);
        assert!(result.is_ok());

        let wasi_instruction = result.unwrap();
        assert_eq!(wasi_instruction.opcode, 0x6A); // i32.add
        assert!(wasi_instruction.operands.is_empty());
    }

    #[test]
    fn test_gaia_call_export() {
        let adapter = WasiExportAdapter::new();
        let gaia_instruction = GaiaInstruction::Call("test_function".to_string());

        let result = adapter.export_instruction(&gaia_instruction);
        assert!(result.is_ok());

        let wasi_instruction = result.unwrap();
        assert_eq!(wasi_instruction.opcode, 0x10); // call
        assert_eq!(wasi_instruction.operands, vec![0]); // placeholder function index
        assert!(wasi_instruction.metadata.is_some());
    }

    #[test]
    fn test_simple_program_export() {
        let adapter = WasiExportAdapter::new();
        let gaia_program = GaiaProgram {
            name: "TestProgram".to_string(),
            functions: vec![GaiaFunction {
                name: "_start".to_string(),
                parameters: vec![],
                return_type: Some(GaiaType::Integer32),
                locals: vec![GaiaType::Integer32],
                instructions: vec![
                    GaiaInstruction::LoadConstant(GaiaConstant::Integer32(42)),
                    GaiaInstruction::StoreLocal(0),
                    GaiaInstruction::LoadLocal(0),
                    GaiaInstruction::Return,
                ],
            }],
            constants: vec![],
        };

        let result = adapter.export_program(&gaia_program);
        assert!(result.is_ok());

        let wasi_instructions = result.unwrap();
        assert_eq!(wasi_instructions.len(), 4);

        // 检查第一条指令是 i32.const 42
        assert_eq!(wasi_instructions[0].opcode, 0x41); // i32.const
        assert_eq!(wasi_instructions[0].operands, vec![42, 0, 0, 0]);

        // 检查第二条指令是 local.set 0
        assert_eq!(wasi_instructions[1].opcode, 0x21); // local.set
        assert_eq!(wasi_instructions[1].operands, vec![0]);

        // 检查第三条指令是 local.get 0
        assert_eq!(wasi_instructions[2].opcode, 0x20); // local.get
        assert_eq!(wasi_instructions[2].operands, vec![0]);

        // 检查第四条指令是 return
        assert_eq!(wasi_instructions[3].opcode, 0x0F); // return
    }

    #[test]
    fn test_leb128_encoding() {
        let adapter = WasiExportAdapter::new();

        // 测试小数值
        assert_eq!(adapter.encode_leb128_u32(0), vec![0]);
        assert_eq!(adapter.encode_leb128_u32(127), vec![127]);

        // 测试需要多字节的数值
        assert_eq!(adapter.encode_leb128_u32(128), vec![0x80, 0x01]);
        assert_eq!(adapter.encode_leb128_u32(300), vec![0xAC, 0x02]);
    }
}
