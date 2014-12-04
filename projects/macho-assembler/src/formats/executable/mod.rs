/// Mach-O 可执行文件格式实现
///
/// 提供了读写 Mach-O 可执行文件的功能。

/// 可执行文件读取器模块
pub mod reader;
/// 可执行文件写入器模块
pub mod writer;

pub use reader::*;
pub use writer::*;