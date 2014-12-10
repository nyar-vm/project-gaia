#![doc = include_str!("readme.md")]

use crate::GaiaError;
use std::fmt::Write;

/// Text writer used for writing formatted text data
///
/// This struct provides formatting functionality for text output, including indentation management,
/// automatic line breaks, and support for custom indentation text.
///
/// # Type Parameters
///
/// * `W` - The underlying writer type, which must implement the `Write` trait.
///
/// # Example
///
/// ```rust
/// # use gaia_types::writer::TextWriter;
/// # use std::fmt::Write;
///
/// let mut buffer = String::new();
/// let mut writer = TextWriter::new(&mut buffer);
///
/// writer.write_line("function main() {").unwrap();
/// writer.indent("").unwrap();
/// writer.write_line("return 42;").unwrap();
/// writer.dedent("").unwrap();
/// writer.write_line("}").unwrap();
///
/// assert_eq!(buffer, "function main() {\n    return 42;\n}\n");
/// ```
#[derive(Debug)]
pub struct TextWriter<W> {
    /// Underlying writer
    writer: W,
    /// Current indentation level
    indent_level: u16,
    /// Indentation text (e.g., spaces or tabs)
    indent_text: &'static str,
}

impl<W: Write> TextWriter<W> {
    /// Creates a new text writer
    ///
    /// # Parameters
    ///
    /// * `writer` - Underlying writer
    /// * `indent_text` - Indentation text, such as "    " (4 spaces) or "\t" (tab)
    ///
    /// # Return Value
    ///
    /// Returns a new `TextWriter` instance
    ///
    /// # Example
    ///
    /// ```rust
    /// # use gaia_types::writer::TextWriter;
    /// # use std::fmt::Write;
    ///
    /// let mut buffer = String::new();
    /// let writer = TextWriter::new(&mut buffer); // Uses 4 spaces for indentation
    /// ```
    pub fn new(writer: W) -> Self {
        Self { writer, indent_level: 0, indent_text: "    " }
    }

    /// Returns the underlying writer
    pub fn into_inner(self) -> W {
        self.writer
    }

    /// Increases the indentation level
    ///
    /// Subsequent text written after calling this method will automatically have extra indentation added.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use gaia_types::writer::TextWriter;
    /// # use std::fmt::Write;
    ///
    /// let mut buffer = String::new();
    /// let mut writer = TextWriter::new(&mut buffer);
    ///
    /// writer.write_line("outer ").unwrap();
    /// writer.indent("{"); // Increase indentation
    /// writer.write_line("inner code").unwrap();
    /// writer.dedent("}"); // Decrease indentation
    /// ```
    pub fn indent(&mut self, text: &str) -> Result<u16, GaiaError> {
        if !text.is_empty() {
            self.writer.write_str(text)?
        }
        self.indent_level = self.indent_level.saturating_add(1);
        Ok(self.indent_level)
    }

    /// Decreases the indentation level
    ///
    /// Subsequent text written after calling this method will have its indentation level decreased.
    /// If the current indentation level is already 0, it will have no effect.
    ///
    /// # Example
    ///
    /// See the example for the `indent` method.
    pub fn dedent(&mut self, text: &str) -> Result<u16, GaiaError> {
        self.indent_level = self.indent_level.saturating_sub(1);
        if !text.is_empty() {
            self.writer.write_str(text)?
        }
        Ok(self.indent_level)
    }

    /// Writes a line of text
    ///
    /// This method automatically adds indentation corresponding to the current indentation level,
    /// and appends a newline character at the end of the text.
    ///
    /// # Parameters
    ///
    /// * `text` - The text content to be written
    ///
    /// # Return Value
    ///
    /// Returns `Result<()>`. If the write is successful, it returns `Ok(())`; otherwise, it returns an error.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use gaia_types::writer::TextWriter;
    /// # use std::fmt::Write;
    ///
    /// let mut buffer = String::new();
    /// let mut writer = TextWriter::new(&mut buffer);
    ///
    /// writer.write_line("This is a line of text").unwrap();
    /// ```
    pub fn write_line(&mut self, text: &str) -> Result<(), std::fmt::Error> {
        self.write_indent()?;
        writeln!(self.writer, "{}", text)
    }

    /// Writes text (without adding a newline)
    ///
    /// This method writes text content but does not automatically add a newline character.
    ///
    /// # Parameters
    ///
    /// * `text` - The text content to be written
    ///
    /// # Return Value
    ///
    /// Returns `Result<()>`. If the write is successful, it returns `Ok(())`; otherwise, it returns an error.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use gaia_types::writer::TextWriter;
    /// # use std::fmt::Write;
    ///
    /// let mut buffer = String::new();
    /// let mut writer = TextWriter::new(&mut buffer);
    ///
    /// writer.write("partial text").unwrap();
    /// writer.write_line("continued text").unwrap();
    /// ```
    pub fn write(&mut self, text: &str) -> Result<(), std::fmt::Error> {
        write!(self.writer, "{}", text)
    }

    /// Writes indentation for the current indentation level
    ///
    /// This method writes indentation text corresponding to the current indentation level.
    ///
    /// # Return Value
    ///
    /// Returns `Result<()>`. If the write is successful, it returns `Ok(())`; otherwise, it returns an error.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use gaia_types::writer::TextWriter;
    /// # use std::fmt::Write;
    ///
    /// let mut buffer = String::new();
    /// let mut writer = TextWriter::new(&mut buffer);
    ///
    /// writer.indent("").unwrap();
    /// writer.write_indent().unwrap(); // Writes 4 spaces
    /// writer.write_line("indented text").unwrap();
    /// ```
    pub fn write_indent(&mut self) -> Result<(), std::fmt::Error> {
        for _ in 0..self.indent_level {
            write!(self.writer, "{}", self.indent_text)?;
        }
        Ok(())
    }

    /// Gets the current indentation level
    ///
    /// # 返回值
    ///
    /// 返回当前的缩进级别（u16）
    ///
    /// # 示例
    ///
    /// ```rust
    /// # use gaia_types::writer::TextWriter;
    /// # use std::fmt::Write;
    ///
    /// let mut buffer = String::new();
    /// let mut writer = TextWriter::new(&mut buffer);
    ///
    /// assert_eq!(writer.indent_level(), 0);
    /// writer.indent("").unwrap();
    /// assert_eq!(writer.indent_level(), 1);
    /// ```
    pub fn indent_level(&self) -> u16 {
        self.indent_level
    }

    /// 获取内部写入器
    ///
    /// 这个方法会消耗 `TextWriter` 并返回内部的写入器
    ///
    /// # 返回值
    ///
    /// 返回内部的写入器 `W`
    ///
    /// # 示例
    ///
    /// ```rust
    /// # use gaia_types::writer::TextWriter;
    /// # use std::fmt::Write;
    ///
    /// let mut buffer = String::new();
    /// let writer = TextWriter::new(&mut buffer);
    /// let inner_writer = writer.finish();
    /// ```
    pub fn finish(self) -> W {
        self.writer
    }
}
