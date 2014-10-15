use crate::types::PeInfo;

/// PE 视图结构
///
/// 提供 PE 文件的原始数据视图和摘要信息的组合。
/// 这个结构同时保留了文件的原始字节数据和解析后的关键信息。
#[derive(Debug, Clone)]
pub struct PeFileView {
    pub info: PeInfo,
}

/// 读取配置
#[derive(Debug, Clone)]
pub struct ReadConfig {
    pub validate_checksum: bool,
}

impl Default for ReadConfig {
    fn default() -> Self {
        Self { validate_checksum: true }
    }
}
