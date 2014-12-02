#![feature(once_cell_try)]
#![doc = include_str!("../readme.md")]

pub mod formats;
pub mod helpers;
pub mod program;

// 重新导出常用模块
pub use formats::wat;
pub use crate::formats::wasm::writer::WasmWriter;
