use gaia_types::writer::TextWriter;
use oak_core::source::ToSource;
use oak_llvm_ir::ast::LLirRoot;
use oak_pretty_print::{to_doc::AsDocument, FormatConfig};
use std::fmt::Write;

#[derive(Debug)]
pub struct LLvmWriter<W> {
    writer: TextWriter<W>,
}

impl<W: Write> LLvmWriter<W> {
    pub fn new(writer: TextWriter<W>) -> Self {
        Self { writer }
    }

    pub fn write_ast(&mut self, ast: &LLirRoot) -> Result<(), std::fmt::Error> {
        let source = ast.to_source_string();
        self.writer.write(&source)?;
        Ok(())
    }

    pub fn write_doc(&mut self, ast: &LLirRoot) -> Result<(), std::fmt::Error> {
        let doc = ast.as_document();
        let config = FormatConfig::default();
        let source = doc.render(config);
        self.writer.write(&source)?;
        Ok(())
    }

    pub fn finish(self) -> W {
        self.writer.finish()
    }
}
