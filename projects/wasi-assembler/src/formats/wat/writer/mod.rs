#![doc = include_str!("readme.md")]

use crate::formats::wat::ast::*;
use gaia_types::{writer::TextWriter, Result};
use std::{
    fmt::Write,
    ops::{Deref, DerefMut},
};

/// WAT 文本写入器
///
/// 该写入器用于生成 WebAssembly Text (WAT) 格式代码，
/// 复用 gaia-types 的 TextWriter 提供缩进和换行管理。
#[derive(Debug)]
pub struct WatWriter<W> {
    writer: TextWriter<W>,
}

impl<W> Deref for WatWriter<W> {
    type Target = TextWriter<W>;

    fn deref(&self) -> &Self::Target {
        &self.writer
    }
}

impl<W> DerefMut for WatWriter<W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.writer
    }
}

impl<W: Write> WatWriter<W> {
    /// 创建一个新的 WAT 写入器
    pub fn new(writer: TextWriter<W>) -> Self {
        Self { writer }
    }
}

impl<W: Write> WatWriter<W> {
    /// 写入 WAT AST 到文本
    pub fn write_ast(&mut self, ast: &WatRoot) -> Result<()> {
        for item in &ast.items {
            // match item {
            //     WatItem::CoreModule(module) => self.write_core_module(module)?,
            //     WatItem::Component(component) => self.write_component(component)?,
            //     WatItem::Module(module) => self.write_core_module(module)?,
            //     WatItem::CustomSection(custom_section) => self.write_custom_section(custom_section)?,
            // }
        }
        Ok(())
    }

    fn write_core_module(&mut self, module: &WatCoreModule) -> Result<()> {
        self.start_block(&format!("module {}", module.name.clone().unwrap_or_default()))?;
        for item in &module.items {
            // match item {
            //     WatCoreModuleItem::Func(func) => self.write_core_func(func)?,
            //     WatCoreModuleItem::Export(export) => self.write_core_export(export)?,
            //     WatCoreModuleItem::Import(import) => self.write_core_import(import)?,
            //     WatCoreModuleItem::Memory(memory) => self.write_memory(memory)?,
            //     WatCoreModuleItem::Table(table) => self.write_table(table)?,
            //     WatCoreModuleItem::Global(global) => self.write_global(global)?,
            //     WatCoreModuleItem::Start(start) => self.emit_start(start)?,
            //     WatCoreModuleItem::Data(data) => self.emit_data(data)?,
            //     WatCoreModuleItem::Elem(elem) => self.write_elem(elem)?,
            // }
        }
        for custom_section in &module.custom_sections {
            self.write_custom_section(custom_section)?;
        }
        self.end_block()?;
        Ok(())
    }

    fn write_component(&mut self, component: &WatComponent) -> Result<()> {
        self.start_block(&format!("component {}", component.name.clone().unwrap_or_default()))?;
        for item in &component.items {
            // match item {
            //     WatComponentItem::Type(type_def) => self.write_type_definition(type_def)?,
            //     WatComponentItem::CoreFunc(func) => self.write_core_func(func)?,
            //     WatComponentItem::Export(export) => self.write_export(export)?,
            //     WatComponentItem::Import(import) => self.write_import(import)?,
            //     WatComponentItem::Memory(memory) => self.write_memory(memory)?,
            //     WatComponentItem::Table(table) => self.write_table(table)?,
            //     WatComponentItem::Global(global) => self.write_global(global)?,
            //     WatComponentItem::CustomSection(custom_section) => self.write_custom_section(custom_section)?,
            // }
        }
        self.end_block()?;
        Ok(())
    }

    fn write_type_definition(&mut self, type_def: &WatTypeDefinition) -> Result<()> {
        self.start_block(&format!("type {}", type_def.name.clone().unwrap_or_default()))?;
        match &type_def.type_content {
            WatType::Func(func_type) => self.write_func_type(func_type)?,
            _ => unimplemented!("Type definition conversion not yet implemented for {:?}", type_def.type_content),
        }
        self.end_block()?;
        Ok(())
    }

    fn write_func_type(&mut self, func_type: &WatFuncType) -> Result<()> {
        self.start_block("func")?;
        for param in &func_type.params {
            self.writer.write_line(&format!("(param {})", param.param_type))?;
        }
        for result in &func_type.results {
            self.writer.write_line(&format!("(result {})", result.result_type))?;
        }
        self.end_block()?;
        Ok(())
    }

    fn write_core_func(&mut self, func: &WatCoreFunc) -> Result<()> {
        self.start_block(&format!("func {}", func.name.clone().unwrap_or_default()))?;
        self.write_func_type_signature(&func.func_type)?;
        if let Some(body) = &func.body {
            for instruction in body {
                self.write_instruction(instruction)?;
            }
        }
        self.end_block()?;
        Ok(())
    }

    fn write_func_type_signature(&mut self, func_type: &WatCoreFuncType) -> Result<()> {
        if !func_type.params.is_empty() {
            let params_str = func_type.params.iter().map(|p| p.to_string()).collect::<Vec<String>>().join(" ");
            self.write_line(&format!("(param {})", params_str))?;
        }
        if !func_type.results.is_empty() {
            let results_str = func_type.results.iter().map(|r| r.to_string()).collect::<Vec<String>>().join(" ");
            self.write_line(&format!("(result {})", results_str))?;
        }
        Ok(())
    }

    fn write_instruction(&mut self, instruction: &WatInstruction) -> Result<()> {
        let operands_str = instruction.operands.iter().map(|op| op.to_string()).collect::<Vec<String>>().join(" ");
        self.write_line(&format!("{} {}", instruction.opcode, operands_str))?;
        Ok(())
    }

    fn write_core_export(&mut self, export: &WatCoreExport) -> Result<()> {
        self.start_block(&format!("export \"{}\"", export.name))?;
        match &export.export_item {
            WatCoreExportItem::Func(func_name) => self.write_line(&format!("(func {})", func_name))?,
            _ => unimplemented!("Core export item conversion not yet implemented for {:?}", export.export_item),
        }
        self.end_block()?;
        Ok(())
    }

    fn write_export(&mut self, export: &WatExport) -> Result<()> {
        self.start_block(&format!("export \"{}\"", export.name))?;
        match &export.export_item {
            WatExportItem::Func(func_name) => self.write_line(&format!("(func {})", func_name))?,
            _ => unimplemented!("Export item conversion not yet implemented for {:?}", export.export_item),
        }
        self.end_block()?;
        Ok(())
    }

    fn write_core_import(&mut self, import: &WatCoreImport) -> Result<()> {
        self.start_block(&format!("import \"{}\" \"{}\"", import.module, import.name))?;
        match &import.import_type {
            WatCoreImportType::Func(func_type) => {
                self.start_block("func")?;
                self.write_func_type_signature(func_type)?;
                self.end_block()?;
            }
            _ => unimplemented!("Core import type conversion not yet implemented for {:?}", import.import_type),
        }
        self.end_block()?;
        Ok(())
    }

    fn write_import(&mut self, import: &WatImport) -> Result<()> {
        self.start_block(&format!("import \"{}\" \"{}\"", import.module, import.name))?;
        match &import.import_type {
            WatImportType::Func(func_type) => {
                self.start_block("func")?;
                self.write_func_type(func_type)?;
                self.end_block()?;
            }
            _ => unimplemented!("Import type conversion not yet implemented for {:?}", import.import_type),
        }
        self.end_block()?;
        Ok(())
    }

    fn write_custom_section(&mut self, custom_section: &WatCustomSection) -> Result<()> {
        self.start_block(&format!("custom \"{}\"", custom_section.name))?;
        self.write_line(&format!("\"{}\"", String::from_utf8_lossy(&custom_section.data)))?;
        self.end_block()?;
        Ok(())
    }

    fn write_data(&mut self, data: &WatCoreData) -> Result<()> {
        self.start_block("data")?;
        if let Some(name) = &data.name {
            self.write_line(&format!("${}", name))?;
        }
        if let Some(memory) = &data.memory {
            self.write_line(&format!("(memory ${})", memory))?;
        }
        for instruction in &data.offset {
            self.write_instruction(instruction)?;
        }
        let data_str = String::from_utf8_lossy(&data.data);
        self.write_line(&format!("\"{}\"", data_str))?;
        self.end_block()?;
        Ok(())
    }

    fn write_elements(&mut self, elements: &[String]) -> Result<()> {
        self.start_block("item")?;
        for element in elements {
            self.write_line(&format!("{}", element))?;
        }
        self.end_block()?;
        Ok(())
    }

    fn write_elem(&mut self, elem: &WatCoreElem) -> Result<()> {
        self.start_block("elem")?;
        for instruction in &elem.offset {
            self.write_instruction(instruction)?;
        }
        self.write_elements(&elem.elements)?;
        self.end_block()?;
        Ok(())
    }

    // ========== 通用块结构 ==========

    /// 开始一个通用的括号块，例如：`(module`、`(func $main ...)`
    pub fn start_block(&mut self, head: &str) -> Result<()> {
        self.writer.write_line(&format!("({}", head))?;
        self.writer.indent(")")?;
        Ok(())
    }

    /// 结束最近的块，写入右括号 `)`
    pub fn end_block(&mut self) -> Result<()> {
        self.writer.dedent(")")?;
        Ok(())
    }

    /// 写入一行带括号的表达式：`(<content>)`
    pub fn emit_paren(&mut self, content: &str) -> Result<()> {
        Ok(self.write_line(&format!("({})", content))?)
    }

    /// 写入原始一行文本（不自动加括号）
    pub fn write_raw(&mut self, line: &str) -> Result<()> {
        Ok(self.write_line(line)?)
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
