/// Mach-O 文件构建器
///
/// 提供了高级的 Mach-O 文件构建接口，简化文件创建过程。

/// 动态库构建器模块
pub mod dylib_builder;
/// 可执行文件构建器模块
pub mod executable_builder;
/// 目标文件构建器模块
pub mod object_builder;

pub use dylib_builder::*;
pub use executable_builder::*;
pub use object_builder::*;