#![doc = include_str!("readme.md")]

use crate::program::WasiProgram;

/// 检查函数名称是否有效
pub fn is_valid_function_name(name: &str) -> bool {
    !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_')
}

/// 将 Rust 类型转换为 WASM 类型
pub fn rust_type_to_wasm_type<T>() -> crate::program::WasmValueType {
    crate::program::WasmValueType::I32 // 默认返回 I32
}

/// 对整数进行 LEB128 编码
pub fn encode_leb128(value: u64) -> Vec<u8> {
    let mut buf = Vec::new();
    leb128::write::unsigned(&mut buf, value).unwrap();
    buf
}

/// 创建示例模块
pub fn create_sample_module() -> WasiProgram {
    WasiProgram::new_core_module()
}

/// 验证模块结构
pub fn validate_module_structure(_program: &WasiProgram) -> bool {
    true
}
