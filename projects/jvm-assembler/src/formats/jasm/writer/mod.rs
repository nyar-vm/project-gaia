#![doc = include_str!("readme.md")]
use gaia_types::{writer::TextWriter, Result};
use std::fmt::Write;

/// JASM 文本写入器
///
/// 提供将 JVM 指令与类/方法结构写入到 JASM 文本格式的能力，
/// 复用 gaia-types 的 TextWriter 来处理缩进与换行。
#[derive(Debug)]
pub struct JasmWriter<W> {
    /// 底层的文本写入器
    writer: TextWriter<W>,
}

impl<W: Write> JasmWriter<W> {
    /// 创建一个新的 JASM 写入器
    ///
    /// # 参数
    /// * `writer` - 底层实现了 `fmt::Write` 的写入器
    pub fn new(writer: W) -> Self {
        Self { writer: TextWriter::new(writer) }
    }

    // ===== 类定义写入 =====

    /// 写入类头并进入类体（增加缩进）
    ///
    /// 示例输出：
    /// `public super class HelloJava version 65:0`
    /// `{`
    pub fn start_class(&mut self, modifiers: &[&str], name: &str, version: Option<&str>) -> Result<()> {
        let mut line = String::new();
        if !modifiers.is_empty() {
            line.push_str(&modifiers.join(" "));
            line.push(' ');
        }
        line.push_str("class ");
        line.push_str(name);
        if let Some(ver) = version {
            line.push_str(" version ");
            line.push_str(ver);
        }
        self.writer.write_line(&line)?;
        self.writer.indent("{")?;
        Ok(())
    }

    /// 写入 SourceFile 行（可选）
    ///
    /// 示例输出：`SourceFile "HelloJava.java";`
    pub fn write_source_file(&mut self, source_file: &str) -> Result<()> {
        self.writer.write_line(&format!("SourceFile \"{}\";", source_file))?;
        Ok(())
    }

    /// 结束类体（减少缩进）
    pub fn end_class(&mut self) -> Result<()> {
        self.writer.dedent("}")?;
        Ok(())
    }

    // ===== 方法定义写入 =====

    /// 写入方法头并进入方法体（在类体基础上增加一层缩进）
    ///
    /// 示例输出：
    /// `public static Method main:"([Ljava/lang/String;)V"`
    /// `stack 2  locals 1`
    /// `{`
    pub fn start_method(
        &mut self,
        modifiers: &[&str],
        name_and_descriptor: &str,
        stack: Option<u32>,
        locals: Option<u32>,
    ) -> Result<()> {
        let mut line = String::new();
        if !modifiers.is_empty() {
            line.push_str(&modifiers.join(" "));
            line.push(' ');
        }
        line.push_str("Method ");
        line.push_str(name_and_descriptor);
        self.writer.write_line(&line)?;

        if stack.is_some() || locals.is_some() {
            let mut decl = String::new();
            if let Some(s) = stack {
                decl.push_str("stack ");
                decl.push_str(&s.to_string());
                if locals.is_some() {
                    decl.push(' ');
                }
            }
            if let Some(l) = locals {
                decl.push_str("locals ");
                decl.push_str(&l.to_string());
            }
            self.writer.write_line(&decl)?;
        }
        self.writer.indent("{")?;
        Ok(())
    }

    /// 结束方法体（恢复到类体缩进）
    pub fn end_method(&mut self) -> Result<()> {
        self.writer.dedent("}")?;
        Ok(())
    }

    // ===== 指令写入 =====

    /// 写入简单指令（带分号）
    ///
    /// 例如：`return;`、`nop;`
    pub fn emit_simple(&mut self, instruction: &str) -> Result<()> {
        self.writer.write_line(&format!("{};", instruction))?;
        Ok(())
    }

    /// 写入 `ldc String "...";` 常量加载
    pub fn emit_ldc_string(&mut self, literal: &str) -> Result<()> {
        self.writer.write_line(&format!("ldc String \"{}\";", literal))?;
        Ok(())
    }

    /// 写入方法调用指令
    ///
    /// 例如：`invokespecial Method java/lang/Object."<init>":"()V";`
    pub fn emit_method_call(&mut self, instruction: &str, method_ref: &str) -> Result<()> {
        self.writer.write_line(&format!("{} Method {};", instruction, method_ref))?;
        Ok(())
    }

    /// 写入字段访问指令
    ///
    /// 例如：`getstatic Field java/lang/System.out:"Ljava/io/PrintStream;";`
    pub fn emit_field_access(&mut self, instruction: &str, field_ref: &str) -> Result<()> {
        self.writer.write_line(&format!("{} Field {};", instruction, field_ref))?;
        Ok(())
    }

    // ===== 常用便捷方法 =====

    pub fn emit_return(&mut self) -> Result<()> {
        self.emit_simple("return")
    }
    pub fn emit_aload_0(&mut self) -> Result<()> {
        self.emit_simple("aload_0")
    }
    pub fn emit_dup(&mut self) -> Result<()> {
        self.emit_simple("dup")
    }
    pub fn emit_pop(&mut self) -> Result<()> {
        self.emit_simple("pop")
    }
    pub fn emit_nop(&mut self) -> Result<()> {
        self.emit_simple("nop")
    }

    /// 获取内部写入器
    pub fn finish(self) -> W {
        self.writer.finish()
    }
}
