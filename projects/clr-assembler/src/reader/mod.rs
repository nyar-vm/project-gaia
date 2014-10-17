use gaia_types::helpers::Url;

mod dot_net_reader;
pub use dot_net_reader::DotNetReader;

/// 读取配置
#[derive(Clone, Debug)]
pub struct ReadConfig {
    pub url: Option<Url>,
}

impl Default for ReadConfig {
    fn default() -> Self {
        Self { url: None }
    }
}
