#![feature(once_cell_try)]
#![warn(missing_docs)]
#![doc = include_str!("readme.md")]

pub mod analyzer;
pub mod builder;
pub mod formats;
pub mod helpers;
pub mod optimizer;
pub mod program;

// // 重新导出主要类型和函数
// pub use crate::formats::jasm::ast::to_program::{convert_jasm_to_jvm, JasmToJvmConverter};
// 重新导出主要类型和函数
pub use crate::program::{opcodes, JvmConstantPoolEntry, JvmField, JvmInstruction, JvmMethod};

// 导出 gaia-types 中的常用类型
pub use gaia_types::{GaiaError, Result};
