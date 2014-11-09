#![doc = include_str!("readme.md")]

use gaia_types::{writer::TextWriter, Result};
use std::fmt::Write;

/// WAT 文本写入器
///
/// 该写入器用于生成 WebAssembly Text (WAT) 格式代码，
/// 复用 gaia-types 的 TextWriter 提供缩进和换行管理。
#[derive(Debug)]
pub struct WatWriter<W> {
    writer: TextWriter<W>,
}

impl<W: Write> WatWriter<W> {
    /// 创建一个新的 WAT 写入器
    pub fn new(writer: W) -> Self {
        Self { writer: TextWriter::new(writer) }
    }

    // ========== 通用块结构 ==========

    /// 开始一个通用的括号块，例如：`(module`、`(func $main ...)`
    pub fn start_block(&mut self, head: &str) -> Result<()> {
        self.writer.write_line(&format!("({}", head))?;
        self.writer.indent();
        Ok(())
    }

    /// 结束最近的块，写入右括号 `)`
    pub fn end_block(&mut self) -> Result<()> {
        self.writer.dedent(")");
        self.writer.write_line(")")?;
        Ok(())
    }

    /// 写入一行带括号的表达式：`(<content>)`
    pub fn emit_paren(&mut self, content: &str) -> Result<()> {
        Ok(self.writer.write_line(&format!("({})", content))?)
    }

    /// 写入原始一行文本（不自动加括号）
    pub fn write_raw(&mut self, line: &str) -> Result<()> {
        Ok(self.writer.write_line(line)?)
    }

    // ========== 顶层结构 ==========

    /// 开始 `module`，可选模块名 `$name`
    pub fn start_module(&mut self, name: Option<&str>) -> Result<()> {
        match name {
            None => self.start_block("module"),
            Some(n) => self.start_block(&format!("module {}", Self::fmt_name(n))),
        }
    }

    /// 结束 `module`
    pub fn end_module(&mut self) -> Result<()> {
        self.end_block()
    }

    // ========== 导入/导出 ==========

    /// 导入函数：`(import "module" "field" (func $name (param ...) (result ...)))`
    pub fn import_func(
        &mut self,
        module: &str,
        field: &str,
        name: Option<&str>,
        params: &[&str],
        results: &[&str],
    ) -> Result<()> {
        let mut sig = String::new();
        if let Some(n) = name {
            sig.push_str(&format!(" {}", Self::fmt_name(n)));
        }
        if !params.is_empty() {
            sig.push_str(&format!(" (param {})", params.join(" ")));
        }
        if !results.is_empty() {
            sig.push_str(&format!(" (result {})", results.join(" ")));
        }
        self.emit_paren(&format!("import \"{}\" \"{}\" (func{})", module, field, sig))
    }

    /// 在函数头中添加导出标记（通常与 start_func 一起使用）
    fn fmt_export(export: Option<&str>) -> String {
        match export {
            Some(e) => format!(" (export \"{}\")", e),
            None => String::new(),
        }
    }

    /// 格式化名称，自动添加 `$` 前缀（若缺失）
    fn fmt_name(name: &str) -> String {
        if name.starts_with('$') {
            name.to_string()
        }
        else {
            format!("${}", name)
        }
    }

    // ========== 函数 ==========

    /// 开始函数定义：`(func $name (export "...") (param ...) (result ...)`
    pub fn start_func(&mut self, name: Option<&str>, export: Option<&str>, params: &[&str], results: &[&str]) -> Result<()> {
        let mut head = String::from("func");
        if let Some(n) = name {
            head.push(' ');
            head.push_str(&Self::fmt_name(n));
        }
        head.push_str(&Self::fmt_export(export));
        if !params.is_empty() {
            head.push_str(&format!(" (param {})", params.join(" ")));
        }
        if !results.is_empty() {
            head.push_str(&format!(" (result {})", results.join(" ")));
        }
        self.start_block(&head)
    }

    /// 结束函数定义：写入右括号
    pub fn end_func(&mut self) -> Result<()> {
        self.end_block()
    }

    // ========== 线性内存与数据 ==========

    /// 声明内存：`(memory $name (export "memory") pages)`（name/export 可选）
    pub fn emit_memory(&mut self, name: Option<&str>, export: Option<&str>, pages: u32) -> Result<()> {
        let mut parts = String::from("memory");
        if let Some(n) = name {
            parts.push(' ');
            parts.push_str(&Self::fmt_name(n));
        }
        if let Some(e) = export {
            parts.push_str(&format!(" (export \"{}\")", e));
        }
        parts.push_str(&format!(" {}", pages));
        self.emit_paren(&parts)
    }

    /// 数据段：`(data (i32.const offset) "text")`
    pub fn emit_data(&mut self, offset: u32, text: &str) -> Result<()> {
        self.emit_paren(&format!("data (i32.const {}) \"{}\"", offset, text))
    }

    // ========== 指令 ==========

    /// `i32.const <value>`
    pub fn emit_i32_const(&mut self, value: i32) -> Result<()> {
        self.write_raw(&format!("i32.const {}", value))
    }

    /// `i64.const <value>`
    pub fn emit_i64_const(&mut self, value: i64) -> Result<()> {
        self.write_raw(&format!("i64.const {}", value))
    }

    /// `call $func`
    pub fn emit_call(&mut self, func: &str) -> Result<()> {
        self.write_raw(&format!("call {}", Self::fmt_name(func)))
    }

    /// `drop`
    pub fn emit_drop(&mut self) -> Result<()> {
        self.write_raw("drop")
    }

    /// 顶层：`(start $func)`
    pub fn emit_start(&mut self, func: &str) -> Result<()> {
        self.emit_paren(&format!("start {}", Self::fmt_name(func)))
    }

    /// 获取内部写入器
    pub fn finish(self) -> W {
        self.writer.finish()
    }
}
