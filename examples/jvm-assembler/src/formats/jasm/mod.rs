use gaia_types::Result;
use oak_core::{source::ToSource, Builder, ParseSession};
use oak_jasm::{JasmBuilder, JasmLanguage, JasmRoot};
use oak_pretty_print::{AsDocument, FormatConfig, Formatter};

pub fn parse(source: &str) -> Result<JasmRoot> {
    let language = JasmLanguage::default();
    let builder = JasmBuilder::new(&language);
    let mut session_cache = ParseSession::<JasmLanguage>::default();
    let session = builder.build(source, &[], &mut session_cache);
    if session.has_errors() {
        // TODO: convert oak diagnostics to gaia diagnostics
        return Err(gaia_types::GaiaError::custom_error("JASM parse error"));
    }
    Ok(session.result.unwrap())
}

pub fn to_source(root: &JasmRoot) -> String {
    root.to_source_string()
}

pub fn to_doc(root: &JasmRoot) -> String {
    let doc = root.as_document();
    let config = FormatConfig::default();
    doc.render(config)
}
