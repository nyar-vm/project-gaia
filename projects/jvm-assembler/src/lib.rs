#![doc = include_str!("../readme.md")]

pub mod converter;
pub mod helpers;
pub mod reader;
pub mod writer;

// 重新导出主要类型和函数
pub use crate::{
    converter::{convert_jasm_to_jvm, JasmToJvmConverter},
    helpers::{opcodes, ConstantPoolEntry, JvmClass, JvmField, JvmInstruction, JvmMethod},
    reader::{read_class_from_bytes, read_instructions_from_bytes, JvmReader},
    writer::{write_class_to_bytes, write_instructions_to_bytes, JvmWriter},
};

// 导出 gaia-types 中的常用类型
pub use gaia_types::{GaiaError, Result};

/// 从 JASM 文本生成 .class 文件字节码
pub fn compile_jasm_to_class(jasm_text: &str) -> Result<Vec<u8>> {
    // 1. 解析 JASM 文本为 AST
    let jvm_class = convert_jasm_to_jvm(jasm_text)?;

    // 2. 将 JVM 类转换为字节码
    write_class_to_bytes(&jvm_class)
}
