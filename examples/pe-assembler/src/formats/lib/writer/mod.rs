/// COFF 写入配置
#[derive(Copy, Debug, Clone)]
pub struct WriteConfig {
    /// 是否包含调试信息
    pub include_debug_info: bool,
    /// 是否优化输出
    pub optimize: bool,
}

impl Default for WriteConfig {
    fn default() -> Self {
        Self { include_debug_info: false, optimize: true }
    }
}

impl WriteConfig {
    /// 创建新的写入配置
    pub fn new() -> Self {
        Self::default()
    }
}
