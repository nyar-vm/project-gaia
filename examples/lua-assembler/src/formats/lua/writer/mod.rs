use gaia_types::writer::TextWriter;
use oak_core::source::ToSource;
use oak_lua::ast::LuaRoot;
use oak_pretty_print::{to_doc::AsDocument, FormatConfig, Formatter};
use std::fmt::Write;

#[derive(Debug)]
pub struct LuaWriter<W> {
    writer: TextWriter<W>,
}

impl<W: Write> LuaWriter<W> {
    pub fn new(writer: TextWriter<W>) -> Self {
        Self { writer }
    }

    pub fn write_ast(&mut self, ast: &LuaRoot) -> Result<(), std::fmt::Error> {
        let source = ast.to_source_string();
        self.writer.write_line(&source)?;
        Ok(())
    }

    pub fn write_doc(&mut self, ast: &LuaRoot) -> Result<(), std::fmt::Error> {
        let doc = ast.as_document();
        let config = FormatConfig::default();
        let source = doc.render(config);
        self.writer.write_line(&source)?;
        Ok(())
    }

    pub fn finish(self) -> W {
        self.writer.finish()
    }
}
