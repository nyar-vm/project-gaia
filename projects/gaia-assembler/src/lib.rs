pub mod assembler;
pub mod backends;
pub mod export_adapters;
pub mod import_adapters;
/// Gaia Universal Assembler
///
/// 通用汇编器，支持多平台指令集转换
/// 使用对象传递而非字符串拼接，复用现有项目的类型定义
pub mod instruction;

// 重新导出核心类型
pub use assembler::*;
pub use backends::*;
pub use export_adapters::*;
pub use import_adapters::*;
pub use instruction::*;
pub use parser::*;

// 重新导出 gaia-types 中的类型
pub use gaia_types::*;
