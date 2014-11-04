pub mod assembler;
pub mod backends;
/// 统一的适配器接口定义, 替代 import 和 export adapter
pub mod unified_adapters;
/// Gaia Universal Assembler
///
/// 通用汇编器，支持多平台指令集转换
/// 使用对象传递而非字符串拼接，复用现有项目的类型定义
pub mod instruction;
/// 配置管理模块
pub mod config;

// 重新导出核心类型
pub use assembler::*;
pub use backends::*;
pub use instruction::*;
pub use unified_adapters::*;
pub use config::*;
pub use parser::*;

// 重新导出 gaia-types 中的类型
pub use gaia_types::*;
