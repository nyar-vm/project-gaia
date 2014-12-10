use gaia_types::Result;
use oak_core::{source::ToSource, Builder, ParseSession};
pub use oak_msil::{ast::*, builder::MsilBuilder, language::MsilLanguage, parser::MsilParser};

pub mod converter;

/// MSIL 读取配置
#[derive(Debug, Clone, Copy, Default)]
pub struct MsilReadConfig {
    /// 是否包含调试信息
    pub include_debug_info: bool,
}

pub fn parse(source: &str) -> Result<MsilRoot> {
    let language = MsilLanguage::default();
    let builder = MsilBuilder::new(&language);
    let mut session_cache = ParseSession::<MsilLanguage>::default();
    let session = builder.build(source, &[], &mut session_cache);
    if session.has_errors() {
        return Err(gaia_types::GaiaError::custom_error("MSIL parse error"));
    }
    Ok(session.result.unwrap())
}

pub fn to_source(root: &MsilRoot) -> String {
    root.to_source_string()
}

pub fn to_doc(root: &MsilRoot) -> String {
    use oak_pretty_print::{AsDocument, FormatConfig};
    let doc = root.as_document();
    let config = FormatConfig::default();
    doc.render(config)
}
