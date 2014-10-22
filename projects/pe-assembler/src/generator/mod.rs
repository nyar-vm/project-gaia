//! PE 文件生成器模块
//!
//! 此模块提供简单的 PE 文件生成功能，用于创建基本的可执行文件。

use gaia_types::{helpers::Architecture, GaiaError};
/// PE 文件生成器 trait
pub trait PeGenerator {
    /// 生成 PE 文件
    fn generate(&self, arch: Architecture) -> Result<Vec<u8>, GaiaError>;
}
