/// Mach-O 目标文件格式实现
///
/// 提供了读写 Mach-O 目标文件的功能。

/// 目标文件读取器模块
pub mod reader;
/// 目标文件写入器模块
pub mod writer;

pub use reader::*;
pub use writer::*;