use gaia_types::helpers::Url;

/// 写入配置
#[derive(Debug, Clone)]
pub struct WriterConfig {
    pub url: Option<Url>,
}
