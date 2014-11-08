use url::Url;

pub mod ast;
pub mod converter;
pub mod lexer;
pub mod parser;
pub mod writer;

#[derive(Debug)]
pub struct MsilReadConfig {
    pub url: Option<Url>,
}

/// MSIL 写入器配置
#[derive(Debug, Clone)]
pub struct MsilWriterConfig {
    /// 是否生成调试信息
    pub generate_debug_info: bool,
    /// 目标 URL
    pub url: Option<Url>,
}

impl Default for MsilReadConfig {
    fn default() -> Self {
        Self { url: None }
    }
}
