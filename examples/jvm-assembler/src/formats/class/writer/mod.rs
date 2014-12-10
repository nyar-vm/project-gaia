#![doc = include_str!("readme.md")]
//! JVM Class 文件写入器
//!
//! 这个模块实现了将 JVM 程序转换为 Class 文件字节码的功能。

use crate::program::*;
use byteorder::BigEndian;
use gaia_types::{BinaryWriter, GaiaDiagnostics, Result};
use std::{collections::HashMap, io::Write};

/// Class 文件写入器
pub struct ClassWriter<W> {
    /// 二进制汇编器
    writer: BinaryWriter<W, BigEndian>,
    /// 常量池条目
    cp_entries: Vec<CpEntry>,
    /// 常量池查找表
    cp_map: HashMap<CpEntry, u16>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum CpEntry {
    Utf8(String),
    Class(u16),
    String(u16),
    Fieldref(u16, u16),
    Methodref(u16, u16),
    InterfaceMethodref(u16, u16),
    NameAndType(u16, u16),
    Integer(i32),
    Float(u32),
    Long(i64),
    Double(u64),
}

impl<W> ClassWriter<W> {
    /// 创建新的 Class 写入器
    pub fn new(writer: W) -> Self {
        Self { writer: BinaryWriter::new(writer), cp_entries: Vec::new(), cp_map: HashMap::new() }
    }

    /// 添加 Utf8 常量
    fn add_utf8(&mut self, s: String) -> u16 {
        self.add_cp_entry(CpEntry::Utf8(s))
    }

    /// 添加 Class 常量
    fn add_class(&mut self, name: String) -> u16 {
        let name_index = self.add_utf8(name);
        self.add_cp_entry(CpEntry::Class(name_index))
    }

    /// 添加 String 常量
    fn add_string(&mut self, value: String) -> u16 {
        let utf8_index = self.add_utf8(value);
        self.add_cp_entry(CpEntry::String(utf8_index))
    }

    /// 添加 NameAndType 常量
    fn add_name_and_type(&mut self, name: String, descriptor: String) -> u16 {
        let name_index = self.add_utf8(name);
        let descriptor_index = self.add_utf8(descriptor);
        self.add_cp_entry(CpEntry::NameAndType(name_index, descriptor_index))
    }

    /// 添加 Fieldref 常量
    fn add_field_ref(&mut self, class_name: String, name: String, descriptor: String) -> u16 {
        let class_index = self.add_class(class_name);
        let name_and_type_index = self.add_name_and_type(name, descriptor);
        self.add_cp_entry(CpEntry::Fieldref(class_index, name_and_type_index))
    }

    /// 添加 Methodref 常量
    fn add_method_ref(&mut self, class_name: String, name: String, descriptor: String) -> u16 {
        let class_index = self.add_class(class_name);
        let name_and_type_index = self.add_name_and_type(name, descriptor);
        self.add_cp_entry(CpEntry::Methodref(class_index, name_and_type_index))
    }

    /// 添加 InterfaceMethodref 常量
    fn add_interface_method_ref(&mut self, class_name: String, name: String, descriptor: String) -> u16 {
        let class_index = self.add_class(class_name);
        let name_and_type_index = self.add_name_and_type(name, descriptor);
        self.add_cp_entry(CpEntry::InterfaceMethodref(class_index, name_and_type_index))
    }

    /// 添加 Integer 常量
    fn add_int(&mut self, val: i32) -> u16 {
        self.add_cp_entry(CpEntry::Integer(val))
    }

    /// 添加 Float 常量
    fn add_float(&mut self, val: f32) -> u16 {
        self.add_cp_entry(CpEntry::Float(val.to_bits()))
    }

    /// 添加 Long 常量
    fn add_long(&mut self, val: i64) -> u16 {
        self.add_cp_entry(CpEntry::Long(val))
    }

    /// 添加 Double 常量
    fn add_double(&mut self, val: f64) -> u16 {
        self.add_cp_entry(CpEntry::Double(val.to_bits()))
    }

    fn add_cp_entry(&mut self, entry: CpEntry) -> u16 {
        if let Some(&index) = self.cp_map.get(&entry) {
            return index;
        }

        let index = (self.cp_entries.len() + 1) as u16;
        self.cp_entries.push(entry.clone());
        self.cp_map.insert(entry, index);

        // Long 和 Double 占用两个常量池槽位
        match self.cp_entries.last().unwrap() {
            CpEntry::Long(_) | CpEntry::Double(_) => {
                // 占位符，不实际写入，但增加长度
                self.cp_entries.push(CpEntry::Utf8("Padding".to_string()));
            }
            _ => {}
        }

        index
    }

    /// 完成写入并返回底层写入器
    pub fn finish(self) -> W {
        self.writer.finish()
    }
}

/// 计算 JVM 方法描述符中的参数数量
fn calculate_parameter_count(descriptor: &str) -> usize {
    if !descriptor.starts_with('(') {
        return 0;
    }

    let mut count = 0;
    let mut chars = descriptor.chars().skip(1); // 跳过 '('

    while let Some(c) = chars.next() {
        if c == ')' {
            break;
        }

        match c {
            'L' => {
                // 引用类型，跳过直到 ';'
                while let Some(c) = chars.next() {
                    if c == ';' {
                        break;
                    }
                }
                count += 1;
            }
            '[' => {
                // 数组类型，跳过直到基本类型或引用类型
                while let Some(c) = chars.next() {
                    if c == '[' {
                        continue;
                    }
                    if c == 'L' {
                        while let Some(c) = chars.next() {
                            if c == ';' {
                                break;
                            }
                        }
                    }
                    break;
                }
                count += 1;
            }
            'J' | 'D' => {
                // long 和 double 占用两个槽位
                count += 2;
            }
            _ => {
                // 其他基本类型 (I, S, B, C, Z, F)
                count += 1;
            }
        }
    }

    count
}

impl<W: Write> ClassWriter<W> {
    /// 将 ClassView 写入为二进制 Class 格式
    pub fn write(mut self, program: &JvmProgram) -> GaiaDiagnostics<W> {
        match self.write_class_file(program) {
            Ok(_) => GaiaDiagnostics::success(self.finish()),
            Err(error) => GaiaDiagnostics::failure(error),
        }
    }

    fn collect_attribute_constants(&mut self, attribute: &JvmAttribute) {
        match attribute {
            JvmAttribute::SourceFile { filename } => {
                self.add_utf8("SourceFile".to_string());
                self.add_utf8(filename.clone());
            }
            JvmAttribute::ConstantValue { value } => {
                self.add_utf8("ConstantValue".to_string());
                match value {
                    JvmConstantPoolEntry::Integer { .. } => {}
                    JvmConstantPoolEntry::Float { .. } => {}
                    JvmConstantPoolEntry::Long { .. } => {}
                    JvmConstantPoolEntry::Double { .. } => {}
                    JvmConstantPoolEntry::String { value } => {
                        self.add_string(value.clone());
                    }
                    _ => {}
                }
            }
            JvmAttribute::Exceptions { exceptions } => {
                self.add_utf8("Exceptions".to_string());
                for exc in exceptions {
                    self.add_class(exc.clone());
                }
            }
            JvmAttribute::Signature { signature } => {
                self.add_utf8("Signature".to_string());
                self.add_utf8(signature.clone());
            }
            JvmAttribute::StackMapTable { frames } => {
                self.add_utf8("StackMapTable".to_string());
                for frame in frames {
                    match frame {
                        JvmStackMapFrame::SameLocals1StackItem { stack, .. }
                        | JvmStackMapFrame::SameLocals1StackItemExtended { stack, .. } => {
                            self.collect_verification_type_constants(stack);
                        }
                        JvmStackMapFrame::Append { locals, .. } => {
                            for vt in locals {
                                self.collect_verification_type_constants(vt);
                            }
                        }
                        JvmStackMapFrame::Full { locals, stack, .. } => {
                            for vt in locals {
                                self.collect_verification_type_constants(vt);
                            }
                            for vt in stack {
                                self.collect_verification_type_constants(vt);
                            }
                        }
                        _ => {}
                    }
                }
            }
            JvmAttribute::InnerClasses { classes } => {
                self.add_utf8("InnerClasses".to_string());
                for inner in classes {
                    self.add_class(inner.inner_class.clone());
                    if let Some(outer) = &inner.outer_class {
                        self.add_class(outer.clone());
                    }
                    if let Some(name) = &inner.inner_name {
                        self.add_utf8(name.clone());
                    }
                }
            }
            JvmAttribute::EnclosingMethod { class_name, method_name, method_descriptor } => {
                self.add_utf8("EnclosingMethod".to_string());
                self.add_class(class_name.clone());
                if let (Some(name), Some(desc)) = (method_name, method_descriptor) {
                    self.add_name_and_type(name.clone(), desc.clone());
                }
            }
            JvmAttribute::Code { attributes, exception_table, .. } => {
                self.add_utf8("Code".to_string());
                for attr in attributes {
                    self.collect_attribute_constants(attr);
                }
                for handler in exception_table {
                    if handler.catch_type_index > 0 {
                        // catch_type_index 已经在常量池中了，这里不需要额外操作
                        // 因为 RawExceptionHandler 是已经转换后的二进制表示
                    }
                }
            }
            JvmAttribute::LineNumberTable { .. } => {
                self.add_utf8("LineNumberTable".to_string());
            }
            JvmAttribute::LocalVariableTable { entries } => {
                self.add_utf8("LocalVariableTable".to_string());
                for entry in entries {
                    self.add_utf8(entry.name.clone());
                    self.add_utf8(entry.descriptor.clone());
                }
            }
            JvmAttribute::Unknown { name, .. } => {
                self.add_utf8(name.clone());
            }
        }
    }

    fn collect_verification_type_constants(&mut self, vt: &JvmVerificationType) {
        if let JvmVerificationType::Object { class_name } = vt {
            self.add_class(class_name.clone());
        }
    }

    /// 写入 Class 文件
    fn write_class_file(&mut self, program: &JvmProgram) -> Result<()> {
        // 1. 预收集所有常量
        let this_class_idx = self.add_class(program.name.clone());
        let super_class_idx = if let Some(super_name) = &program.super_class {
            self.add_class(super_name.clone())
        }
        else {
            self.add_class("java/lang/Object".to_string())
        };

        // 收集类属性常量
        for attr in &program.attributes {
            self.collect_attribute_constants(attr);
        }

        // 收集字段常量
        for field in &program.fields {
            self.add_utf8(field.name.clone());
            self.add_utf8(field.descriptor.clone());
            for attr in &field.attributes {
                self.collect_attribute_constants(attr);
            }
            if field.constant_value.is_some() {
                self.add_utf8("ConstantValue".to_string());
            }
        }

        // 收集方法常量
        let code_utf8_idx = self.add_utf8("Code".to_string());
        for method in &program.methods {
            self.add_utf8(method.name.clone());
            self.add_utf8(method.descriptor.clone());

            for handler in &method.exception_handlers {
                if let Some(catch_type) = &handler.catch_type {
                    self.add_class(catch_type.clone());
                }
            }

            for attr in &method.attributes {
                self.collect_attribute_constants(attr);
            }
            if !method.exceptions.is_empty() {
                self.add_utf8("Exceptions".to_string());
            }

            // 预生成字节码以获取标签位置，用于 StackMapTable 生成
            let (_, label_positions) = self.generate_method_bytecode(method);

            if program.version.major >= 50 {
                self.add_utf8("StackMapTable".to_string());
                // 如果需要自动生成 StackMapTable，预先收集其常量
                let has_stack_map = method.attributes.iter().any(|a| matches!(a, JvmAttribute::StackMapTable { .. }));
                if !has_stack_map {
                    let analyzer = crate::analyzer::StackMapAnalyzer::new(program.name.clone(), method, &label_positions);
                    let frames = analyzer.analyze();
                    for frame in &frames {
                        match frame {
                            JvmStackMapFrame::SameLocals1StackItem { stack, .. }
                            | JvmStackMapFrame::SameLocals1StackItemExtended { stack, .. } => {
                                self.collect_verification_type_constants(stack);
                            }
                            JvmStackMapFrame::Append { locals, .. } => {
                                for vt in locals {
                                    self.collect_verification_type_constants(vt);
                                }
                            }
                            JvmStackMapFrame::Full { locals, stack, .. } => {
                                for vt in locals {
                                    self.collect_verification_type_constants(vt);
                                }
                                for vt in stack {
                                    self.collect_verification_type_constants(vt);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }

            // 收集指令中的常量
            for inst in &method.instructions {
                match inst {
                    JvmInstruction::Ldc { symbol } | JvmInstruction::LdcW { symbol } | JvmInstruction::Ldc2W { symbol } => {
                        self.add_string(symbol.clone());
                    }
                    JvmInstruction::Getstatic { class_name, field_name, descriptor }
                    | JvmInstruction::Putstatic { class_name, field_name, descriptor }
                    | JvmInstruction::Getfield { class_name, field_name, descriptor }
                    | JvmInstruction::Putfield { class_name, field_name, descriptor } => {
                        self.add_field_ref(class_name.clone(), field_name.clone(), descriptor.clone());
                    }
                    JvmInstruction::Invokevirtual { class_name, method_name, descriptor }
                    | JvmInstruction::Invokespecial { class_name, method_name, descriptor }
                    | JvmInstruction::Invokestatic { class_name, method_name, descriptor }
                    | JvmInstruction::Invokedynamic { class_name, method_name, descriptor } => {
                        self.add_method_ref(class_name.clone(), method_name.clone(), descriptor.clone());
                    }
                    JvmInstruction::Invokeinterface { class_name, method_name, descriptor } => {
                        self.add_interface_method_ref(class_name.clone(), method_name.clone(), descriptor.clone());
                    }
                    JvmInstruction::New { class_name }
                    | JvmInstruction::Anewarray { class_name }
                    | JvmInstruction::Checkcast { class_name }
                    | JvmInstruction::Instanceof { class_name }
                    | JvmInstruction::Multianewarray { class_name, .. } => {
                        self.add_class(class_name.clone());
                    }
                    _ => {}
                }
            }
        }

        // 2. 开始写入
        // 写入魔数
        self.writer.write_u32(0xCAFEBABE)?;

        // 写入版本信息
        self.writer.write_u16(program.version.minor)?;
        self.writer.write_u16(program.version.major)?;

        // 写入常量池
        self.write_constant_pool()?;

        // 写入访问标志
        self.writer.write_u16(program.access_flags.to_flags())?;

        // 写入类索引（this_class）
        self.writer.write_u16(this_class_idx)?;

        // 写入超类索引（super_class）
        self.writer.write_u16(super_class_idx)?;

        // 写入接口数量（暂时为0）
        self.writer.write_u16(0)?;

        // 写入字段
        self.write_fields(program)?;

        // 写入方法
        self.write_methods(program, code_utf8_idx)?;

        // 写入类属性
        self.writer.write_u16(program.attributes.len() as u16)?;
        for attr in &program.attributes {
            self.write_attribute(attr)?;
        }

        Ok(())
    }

    /// 写入常量池
    fn write_constant_pool(&mut self) -> Result<()> {
        // 写入常量池计数（+1 因为索引从1开始）
        self.writer.write_u16((self.cp_entries.len() + 1) as u16)?;

        // 写入常量池条目
        let mut i = 0;
        while i < self.cp_entries.len() {
            let entry = &self.cp_entries[i];
            match entry {
                CpEntry::Utf8(s) => {
                    self.writer.write_u8(1)?; // CONSTANT_Utf8 tag
                    self.writer.write_u16(s.len() as u16)?;
                    self.writer.write_all(s.as_bytes())?;
                }
                CpEntry::Class(name_idx) => {
                    self.writer.write_u8(7)?; // CONSTANT_Class tag
                    self.writer.write_u16(*name_idx)?;
                }
                CpEntry::String(utf8_idx) => {
                    self.writer.write_u8(8)?; // CONSTANT_String tag
                    self.writer.write_u16(*utf8_idx)?;
                }
                CpEntry::NameAndType(name_idx, desc_idx) => {
                    self.writer.write_u8(12)?; // CONSTANT_NameAndType tag
                    self.writer.write_u16(*name_idx)?;
                    self.writer.write_u16(*desc_idx)?;
                }
                CpEntry::Fieldref(class_idx, nt_idx) => {
                    self.writer.write_u8(9)?; // CONSTANT_Fieldref tag
                    self.writer.write_u16(*class_idx)?;
                    self.writer.write_u16(*nt_idx)?;
                }
                CpEntry::Methodref(class_idx, nt_idx) => {
                    self.writer.write_u8(10)?; // CONSTANT_Methodref tag
                    self.writer.write_u16(*class_idx)?;
                    self.writer.write_u16(*nt_idx)?;
                }
                CpEntry::InterfaceMethodref(class_idx, nt_idx) => {
                    self.writer.write_u8(11)?; // CONSTANT_InterfaceMethodref tag
                    self.writer.write_u16(*class_idx)?;
                    self.writer.write_u16(*nt_idx)?;
                }
                CpEntry::Integer(val) => {
                    self.writer.write_u8(3)?; // CONSTANT_Integer tag
                    self.writer.write_i32(*val)?;
                }
                CpEntry::Float(bits) => {
                    self.writer.write_u8(4)?; // CONSTANT_Float tag
                    self.writer.write_u32(*bits)?;
                }
                CpEntry::Long(val) => {
                    self.writer.write_u8(5)?; // CONSTANT_Long tag
                    self.writer.write_i64(*val)?;
                    i += 1; // Long 占用两个槽位
                }
                CpEntry::Double(bits) => {
                    self.writer.write_u8(6)?; // CONSTANT_Double tag
                    self.writer.write_u64(*bits)?;
                    i += 1; // Double 占用两个槽位
                }
            }
            i += 1;
        }

        Ok(())
    }

    /// 写入字段
    fn write_fields(&mut self, program: &JvmProgram) -> Result<()> {
        self.writer.write_u16(program.fields.len() as u16)?;

        for field in &program.fields {
            self.writer.write_u16(field.access_flags.to_flags())?;
            let name_idx = self.add_utf8(field.name.clone());
            let desc_idx = self.add_utf8(field.descriptor.clone());
            self.writer.write_u16(name_idx)?;
            self.writer.write_u16(desc_idx)?;

            // 计算属性数量
            let mut attr_count = field.attributes.len() as u16;
            if field.constant_value.is_some() {
                attr_count += 1;
            }
            self.writer.write_u16(attr_count)?;

            // 写入 ConstantValue 属性
            if let Some(val) = &field.constant_value {
                self.write_attribute(&JvmAttribute::ConstantValue { value: val.clone() })?;
            }

            // 写入其他属性
            for attr in &field.attributes {
                self.write_attribute(attr)?;
            }
        }

        Ok(())
    }

    /// 写入方法
    fn write_methods(&mut self, program: &JvmProgram, code_utf8_idx: u16) -> Result<()> {
        self.writer.write_u16(program.methods.len() as u16)?;

        let exceptions_utf8_idx = self.add_utf8("Exceptions".to_string());

        for method in &program.methods {
            self.writer.write_u16(method.access_flags.to_flags())?;
            let name_idx = self.add_utf8(method.name.clone());
            let desc_idx = self.add_utf8(method.descriptor.clone());
            self.writer.write_u16(name_idx)?;
            self.writer.write_u16(desc_idx)?;

            // 计算属性数量
            let mut attribute_count = 1; // 总是包含 Code 属性
            if !method.exceptions.is_empty() {
                attribute_count += 1;
            }
            attribute_count += method.attributes.len() as u16;

            // 写入属性数量
            self.writer.write_u16(attribute_count)?;

            // 1. 写入 Code 属性
            self.write_code_attribute(program, method, code_utf8_idx)?;

            // 2. 写入 Exceptions 属性
            if !method.exceptions.is_empty() {
                self.writer.write_u16(exceptions_utf8_idx)?;
                let attr_len = 2 + method.exceptions.len() * 2;
                self.writer.write_u32(attr_len as u32)?;
                self.writer.write_u16(method.exceptions.len() as u16)?;
                for exc in &method.exceptions {
                    let exc_idx = self.add_class(exc.clone());
                    self.writer.write_u16(exc_idx)?;
                }
            }

            // 3. 写入其他属性
            for attr in &method.attributes {
                self.write_attribute(attr)?;
            }
        }

        Ok(())
    }

    /// 写入 Code 属性
    fn write_code_attribute(&mut self, program: &JvmProgram, method: &JvmMethod, code_utf8_idx: u16) -> Result<()> {
        // Code 属性名称索引
        self.writer.write_u16(code_utf8_idx)?;

        let (bytecode, label_positions) = self.generate_method_bytecode(method);

        // 解析异常表
        let mut raw_exception_table = Vec::new();
        for handler in &method.exception_handlers {
            let start_pc = *label_positions.get(&handler.start_label).unwrap_or(&0) as u16;
            let end_pc = *label_positions.get(&handler.end_label).unwrap_or(&0) as u16;
            let handler_pc = *label_positions.get(&handler.handler_label).unwrap_or(&0) as u16;
            let catch_type_index =
                if let Some(catch_type) = &handler.catch_type { self.add_class(catch_type.clone()) } else { 0 };
            raw_exception_table.push(RawExceptionHandler { start_pc, end_pc, handler_pc, catch_type_index });
        }

        // 收集并准备子属性
        let mut code_attributes = Vec::new();

        // 自动生成 StackMapTable (如果版本 >= Java 6 且尚未手动提供)
        if program.version.major >= 50 {
            let has_stack_map = method.attributes.iter().any(|a| matches!(a, JvmAttribute::StackMapTable { .. }));
            if !has_stack_map {
                let analyzer = crate::analyzer::StackMapAnalyzer::new(program.name.clone(), method, &label_positions);
                let frames = analyzer.analyze();
                if !frames.is_empty() {
                    code_attributes.push(JvmAttribute::StackMapTable { frames });
                }
            }
        }

        // 准备子属性数据
        let mut sub_attr_buf = Vec::new();
        // 这里需要临时替换 writer 以写入到缓冲区
        let mut temp_writer = ClassWriter {
            writer: BinaryWriter::new(&mut sub_attr_buf),
            cp_entries: self.cp_entries.clone(),
            cp_map: self.cp_map.clone(),
        };

        // 写入自动生成的属性
        for attr in &code_attributes {
            temp_writer.write_attribute(attr)?;
        }
        // 写入原有的属性 (假设它们是 Code 属性的子属性，目前简化处理)
        // 注意：在实际 JVM 中，LineNumberTable 等是 Code 的子属性
        // 我们目前的结构中，JvmMethod.attributes 包含了所有属性
        // 需要区分哪些是方法属性，哪些是 Code 子属性
        // 这里暂时只处理自动生成的

        // 同步常量池
        self.cp_entries = temp_writer.cp_entries;
        self.cp_map = temp_writer.cp_map;

        // 计算总长度
        let exception_table_len = raw_exception_table.len() * 8;
        let attribute_length = 2 + 2 + 4 + bytecode.len() + 2 + exception_table_len + 2 + sub_attr_buf.len();
        self.writer.write_u32(attribute_length as u32)?;

        // max_stack 和 max_locals
        self.writer.write_u16(method.max_stack)?;
        self.writer.write_u16(method.max_locals)?;

        // 字节码长度和字节码
        self.writer.write_u32(bytecode.len() as u32)?;
        self.writer.write_all(&bytecode)?;

        // 异常表
        self.writer.write_u16(raw_exception_table.len() as u16)?;
        for handler in raw_exception_table {
            self.writer.write_u16(handler.start_pc)?;
            self.writer.write_u16(handler.end_pc)?;
            self.writer.write_u16(handler.handler_pc)?;
            self.writer.write_u16(handler.catch_type_index)?;
        }

        // 子属性数量
        self.writer.write_u16(code_attributes.len() as u16)?;
        self.writer.write_all(&sub_attr_buf)?;

        Ok(())
    }

    /// 生成方法的字节码及标签位置
    fn generate_method_bytecode(&mut self, method: &JvmMethod) -> (Vec<u8>, HashMap<String, i32>) {
        let mut bytecode = Vec::new();
        let mut label_positions = HashMap::new();
        let mut jump_patches = Vec::new(); // (instr_pos, patch_pos, target_label_name, is_wide)

        // 第一遍：计算指令位置并记录标签位置
        for instruction in &method.instructions {
            match instruction {
                JvmInstruction::Label { name } => {
                    label_positions.insert(name.clone(), bytecode.len() as i32);
                }
                _ => {
                    let pos = bytecode.len();
                    self.emit_instruction(instruction, &mut bytecode, &mut jump_patches, pos);
                }
            }
        }

        // 第二遍：解析跳转偏移量
        for (instr_pos, patch_pos, target_name, is_wide) in jump_patches {
            if let Some(&target_pos) = label_positions.get(&target_name) {
                let offset = target_pos - instr_pos as i32;
                if is_wide {
                    // 4 字节偏移量
                    bytecode[patch_pos] = ((offset >> 24) & 0xFF) as u8;
                    bytecode[patch_pos + 1] = ((offset >> 16) & 0xFF) as u8;
                    bytecode[patch_pos + 2] = ((offset >> 8) & 0xFF) as u8;
                    bytecode[patch_pos + 3] = (offset & 0xFF) as u8;
                }
                else {
                    // 2 字节偏移量
                    bytecode[patch_pos] = ((offset >> 8) & 0xFF) as u8;
                    bytecode[patch_pos + 1] = (offset & 0xFF) as u8;
                }
            }
        }

        // 如果方法没有指令，添加一个 return 指令
        if bytecode.is_empty() {
            bytecode.push(0xB1); // return
        }

        (bytecode, label_positions)
    }

    /// 写入属性
    fn write_attribute(&mut self, attribute: &JvmAttribute) -> Result<()> {
        match attribute {
            JvmAttribute::SourceFile { filename } => {
                let name_idx = self.add_utf8("SourceFile".to_string());
                let file_idx = self.add_utf8(filename.clone());
                self.writer.write_u16(name_idx)?;
                self.writer.write_u32(2)?;
                self.writer.write_u16(file_idx)?;
            }
            JvmAttribute::ConstantValue { value } => {
                let name_idx = self.add_utf8("ConstantValue".to_string());
                let val_idx = match value {
                    JvmConstantPoolEntry::Integer { value } => self.add_int(*value),
                    JvmConstantPoolEntry::Float { value } => self.add_float(*value),
                    JvmConstantPoolEntry::Long { value } => self.add_long(*value),
                    JvmConstantPoolEntry::Double { value } => self.add_double(*value),
                    JvmConstantPoolEntry::String { value } => self.add_string(value.clone()),
                    _ => 0,
                };
                self.writer.write_u16(name_idx)?;
                self.writer.write_u32(2)?;
                self.writer.write_u16(val_idx)?;
            }
            JvmAttribute::Exceptions { exceptions } => {
                let name_idx = self.add_utf8("Exceptions".to_string());
                let attr_len = 2 + exceptions.len() * 2;
                self.writer.write_u16(name_idx)?;
                self.writer.write_u32(attr_len as u32)?;
                self.writer.write_u16(exceptions.len() as u16)?;
                for exc in exceptions {
                    let exc_idx = self.add_class(exc.clone());
                    self.writer.write_u16(exc_idx)?;
                }
            }
            JvmAttribute::Signature { signature } => {
                let name_idx = self.add_utf8("Signature".to_string());
                let sig_idx = self.add_utf8(signature.clone());
                self.writer.write_u16(name_idx)?;
                self.writer.write_u32(2)?;
                self.writer.write_u16(sig_idx)?;
            }
            JvmAttribute::StackMapTable { frames } => {
                let name_idx = self.add_utf8("StackMapTable".to_string());
                let mut buf = Vec::new();
                buf.extend_from_slice(&(frames.len() as u16).to_be_bytes());
                for frame in frames {
                    self.write_stack_map_frame_to_buf(frame, &mut buf)?;
                }
                self.writer.write_u16(name_idx)?;
                self.writer.write_u32(buf.len() as u32)?;
                self.writer.write_all(&buf)?;
            }
            JvmAttribute::InnerClasses { classes } => {
                let name_idx = self.add_utf8("InnerClasses".to_string());
                let attr_len = 2 + classes.len() * 8;
                self.writer.write_u16(name_idx)?;
                self.writer.write_u32(attr_len as u32)?;
                self.writer.write_u16(classes.len() as u16)?;
                for inner in classes {
                    let inner_idx = self.add_class(inner.inner_class.clone());
                    let outer_idx = if let Some(outer) = &inner.outer_class { self.add_class(outer.clone()) } else { 0 };
                    let name_idx = if let Some(name) = &inner.inner_name { self.add_utf8(name.clone()) } else { 0 };
                    self.writer.write_u16(inner_idx)?;
                    self.writer.write_u16(outer_idx)?;
                    self.writer.write_u16(name_idx)?;
                    self.writer.write_u16(inner.access_flags.to_flags())?;
                }
            }
            JvmAttribute::EnclosingMethod { class_name, method_name, method_descriptor } => {
                let name_idx = self.add_utf8("EnclosingMethod".to_string());
                let class_idx = self.add_class(class_name.clone());
                let nt_idx = if let (Some(name), Some(desc)) = (method_name, method_descriptor) {
                    self.add_name_and_type(name.clone(), desc.clone())
                }
                else {
                    0
                };
                self.writer.write_u16(name_idx)?;
                self.writer.write_u32(4)?;
                self.writer.write_u16(class_idx)?;
                self.writer.write_u16(nt_idx)?;
            }
            JvmAttribute::Unknown { name, data } => {
                let name_idx = self.add_utf8(name.clone());
                self.writer.write_u16(name_idx)?;
                self.writer.write_u32(data.len() as u32)?;
                self.writer.write_all(data)?;
            }
            _ => {
                // Code, LineNumberTable, LocalVariableTable 等需要特殊处理或暂不支持
            }
        }
        Ok(())
    }

    fn write_verification_type_to_buf(&mut self, vt: &JvmVerificationType, buf: &mut Vec<u8>) -> Result<()> {
        match vt {
            JvmVerificationType::Top => buf.push(0),
            JvmVerificationType::Integer => buf.push(1),
            JvmVerificationType::Float => buf.push(2),
            JvmVerificationType::Double => buf.push(3),
            JvmVerificationType::Long => buf.push(4),
            JvmVerificationType::Null => buf.push(5),
            JvmVerificationType::UninitializedThis => buf.push(6),
            JvmVerificationType::Object { class_name } => {
                buf.push(7);
                let class_idx = self.add_class(class_name.clone());
                buf.extend_from_slice(&class_idx.to_be_bytes());
            }
            JvmVerificationType::Uninitialized { offset } => {
                buf.push(8);
                buf.extend_from_slice(&offset.to_be_bytes());
            }
        }
        Ok(())
    }

    fn write_stack_map_frame_to_buf(&mut self, frame: &JvmStackMapFrame, buf: &mut Vec<u8>) -> Result<()> {
        match frame {
            JvmStackMapFrame::Same { offset_delta } => {
                if *offset_delta <= 63 {
                    buf.push(*offset_delta as u8);
                }
                else {
                    buf.push(251);
                    buf.extend_from_slice(&offset_delta.to_be_bytes());
                }
            }
            JvmStackMapFrame::SameLocals1StackItem { offset_delta, stack } => {
                if *offset_delta <= 63 {
                    buf.push((*offset_delta + 64) as u8);
                    self.write_verification_type_to_buf(stack, buf)?;
                }
                else {
                    buf.push(247);
                    buf.extend_from_slice(&offset_delta.to_be_bytes());
                    self.write_verification_type_to_buf(stack, buf)?;
                }
            }
            JvmStackMapFrame::SameLocals1StackItemExtended { offset_delta, stack } => {
                buf.push(247);
                buf.extend_from_slice(&offset_delta.to_be_bytes());
                self.write_verification_type_to_buf(stack, buf)?;
            }
            JvmStackMapFrame::Chop { offset_delta, k } => {
                buf.push(251 - k);
                buf.extend_from_slice(&offset_delta.to_be_bytes());
            }
            JvmStackMapFrame::SameExtended { offset_delta } => {
                buf.push(251);
                buf.extend_from_slice(&offset_delta.to_be_bytes());
            }
            JvmStackMapFrame::Append { offset_delta, locals } => {
                buf.push((251 + locals.len()) as u8);
                buf.extend_from_slice(&offset_delta.to_be_bytes());
                for vt in locals {
                    self.write_verification_type_to_buf(vt, buf)?;
                }
            }
            JvmStackMapFrame::Full { offset_delta, locals, stack } => {
                buf.push(255);
                buf.extend_from_slice(&offset_delta.to_be_bytes());
                buf.extend_from_slice(&(locals.len() as u16).to_be_bytes());
                for vt in locals {
                    self.write_verification_type_to_buf(vt, buf)?;
                }
                buf.extend_from_slice(&(stack.len() as u16).to_be_bytes());
                for vt in stack {
                    self.write_verification_type_to_buf(vt, buf)?;
                }
            }
        }
        Ok(())
    }

    /// 发射单条指令的字节码
    fn emit_instruction(
        &mut self,
        instruction: &JvmInstruction,
        bytecode: &mut Vec<u8>,
        jump_patches: &mut Vec<(usize, usize, String, bool)>,
        pos: usize,
    ) {
        match instruction {
            JvmInstruction::Nop => bytecode.push(0x00),
            JvmInstruction::AconstNull => bytecode.push(0x01),
            JvmInstruction::IconstM1 => bytecode.push(0x02),
            JvmInstruction::Iconst0 => bytecode.push(0x03),
            JvmInstruction::Iconst1 => bytecode.push(0x04),
            JvmInstruction::Iconst2 => bytecode.push(0x05),
            JvmInstruction::Iconst3 => bytecode.push(0x06),
            JvmInstruction::Iconst4 => bytecode.push(0x07),
            JvmInstruction::Iconst5 => bytecode.push(0x08),
            JvmInstruction::Lconst0 => bytecode.push(0x09),
            JvmInstruction::Lconst1 => bytecode.push(0x0A),
            JvmInstruction::Fconst0 => bytecode.push(0x0B),
            JvmInstruction::Fconst1 => bytecode.push(0x0C),
            JvmInstruction::Fconst2 => bytecode.push(0x0D),
            JvmInstruction::Dconst0 => bytecode.push(0x0E),
            JvmInstruction::Dconst1 => bytecode.push(0x0F),

            JvmInstruction::Bipush { value } => {
                bytecode.push(0x10);
                bytecode.push(*value as u8);
            }
            JvmInstruction::Sipush { value } => {
                bytecode.push(0x11);
                bytecode.push((*value >> 8) as u8);
                bytecode.push((*value & 0xFF) as u8);
            }

            JvmInstruction::Ldc { symbol } => {
                let index = self.add_string(symbol.clone());
                if index < 256 {
                    bytecode.push(0x12); // ldc
                    bytecode.push(index as u8);
                }
                else {
                    bytecode.push(0x13); // ldc_w
                    bytecode.push((index >> 8) as u8);
                    bytecode.push((index & 0xFF) as u8);
                }
            }
            JvmInstruction::LdcW { symbol } => {
                let index = self.add_string(symbol.clone());
                bytecode.push(0x13); // ldc_w
                bytecode.push((index >> 8) as u8);
                bytecode.push((index & 0xFF) as u8);
            }
            JvmInstruction::Ldc2W { symbol } => {
                let index = self.add_string(symbol.clone());
                bytecode.push(0x14); // ldc2_w
                bytecode.push((index >> 8) as u8);
                bytecode.push((index & 0xFF) as u8);
            }

            JvmInstruction::Iload { index } => {
                if *index <= 3 {
                    bytecode.push(0x1A + *index as u8);
                }
                else if *index <= 255 {
                    bytecode.push(0x15);
                    bytecode.push(*index as u8);
                }
                else {
                    bytecode.push(0xC4); // wide
                    bytecode.push(0x15);
                    bytecode.push((*index >> 8) as u8);
                    bytecode.push((*index & 0xFF) as u8);
                }
            }
            JvmInstruction::Iload0 => bytecode.push(0x1A),
            JvmInstruction::Iload1 => bytecode.push(0x1B),
            JvmInstruction::Iload2 => bytecode.push(0x1C),
            JvmInstruction::Iload3 => bytecode.push(0x1D),

            JvmInstruction::Lload { index } => {
                if *index <= 3 {
                    bytecode.push(0x1E + *index as u8);
                }
                else if *index <= 255 {
                    bytecode.push(0x16);
                    bytecode.push(*index as u8);
                }
                else {
                    bytecode.push(0xC4); // wide
                    bytecode.push(0x16);
                    bytecode.push((*index >> 8) as u8);
                    bytecode.push((*index & 0xFF) as u8);
                }
            }
            JvmInstruction::Lload0 => bytecode.push(0x1E),
            JvmInstruction::Lload1 => bytecode.push(0x1F),
            JvmInstruction::Lload2 => bytecode.push(0x20),
            JvmInstruction::Lload3 => bytecode.push(0x21),

            JvmInstruction::Fload { index } => {
                if *index <= 3 {
                    bytecode.push(0x22 + *index as u8);
                }
                else if *index <= 255 {
                    bytecode.push(0x17);
                    bytecode.push(*index as u8);
                }
                else {
                    bytecode.push(0xC4); // wide
                    bytecode.push(0x17);
                    bytecode.push((*index >> 8) as u8);
                    bytecode.push((*index & 0xFF) as u8);
                }
            }
            JvmInstruction::Fload0 => bytecode.push(0x22),
            JvmInstruction::Fload1 => bytecode.push(0x23),
            JvmInstruction::Fload2 => bytecode.push(0x24),
            JvmInstruction::Fload3 => bytecode.push(0x25),

            JvmInstruction::Dload { index } => {
                if *index <= 3 {
                    bytecode.push(0x26 + *index as u8);
                }
                else if *index <= 255 {
                    bytecode.push(0x18);
                    bytecode.push(*index as u8);
                }
                else {
                    bytecode.push(0xC4); // wide
                    bytecode.push(0x18);
                    bytecode.push((*index >> 8) as u8);
                    bytecode.push((*index & 0xFF) as u8);
                }
            }
            JvmInstruction::Dload0 => bytecode.push(0x26),
            JvmInstruction::Dload1 => bytecode.push(0x27),
            JvmInstruction::Dload2 => bytecode.push(0x28),
            JvmInstruction::Dload3 => bytecode.push(0x29),

            JvmInstruction::Aload { index } => {
                if *index <= 3 {
                    bytecode.push(0x2A + *index as u8);
                }
                else if *index <= 255 {
                    bytecode.push(0x19);
                    bytecode.push(*index as u8);
                }
                else {
                    bytecode.push(0xC4); // wide
                    bytecode.push(0x19);
                    bytecode.push((*index >> 8) as u8);
                    bytecode.push((*index & 0xFF) as u8);
                }
            }
            JvmInstruction::Aload0 => bytecode.push(0x2A),
            JvmInstruction::Aload1 => bytecode.push(0x2B),
            JvmInstruction::Aload2 => bytecode.push(0x2C),
            JvmInstruction::Aload3 => bytecode.push(0x2D),

            JvmInstruction::Istore { index } => {
                if *index <= 3 {
                    bytecode.push(0x3B + *index as u8);
                }
                else if *index <= 255 {
                    bytecode.push(0x36);
                    bytecode.push(*index as u8);
                }
                else {
                    bytecode.push(0xC4); // wide
                    bytecode.push(0x36);
                    bytecode.push((*index >> 8) as u8);
                    bytecode.push((*index & 0xFF) as u8);
                }
            }
            JvmInstruction::Istore0 => bytecode.push(0x3B),
            JvmInstruction::Istore1 => bytecode.push(0x3C),
            JvmInstruction::Istore2 => bytecode.push(0x3D),
            JvmInstruction::Istore3 => bytecode.push(0x3E),

            JvmInstruction::Lstore { index } => {
                if *index <= 3 {
                    bytecode.push(0x3F + *index as u8);
                }
                else if *index <= 255 {
                    bytecode.push(0x37);
                    bytecode.push(*index as u8);
                }
                else {
                    bytecode.push(0xC4); // wide
                    bytecode.push(0x37);
                    bytecode.push((*index >> 8) as u8);
                    bytecode.push((*index & 0xFF) as u8);
                }
            }
            JvmInstruction::Lstore0 => bytecode.push(0x3F),
            JvmInstruction::Lstore1 => bytecode.push(0x40),
            JvmInstruction::Lstore2 => bytecode.push(0x41),
            JvmInstruction::Lstore3 => bytecode.push(0x42),

            JvmInstruction::Fstore { index } => {
                if *index <= 3 {
                    bytecode.push(0x43 + *index as u8);
                }
                else if *index <= 255 {
                    bytecode.push(0x38);
                    bytecode.push(*index as u8);
                }
                else {
                    bytecode.push(0xC4); // wide
                    bytecode.push(0x38);
                    bytecode.push((*index >> 8) as u8);
                    bytecode.push((*index & 0xFF) as u8);
                }
            }
            JvmInstruction::Fstore0 => bytecode.push(0x43),
            JvmInstruction::Fstore1 => bytecode.push(0x44),
            JvmInstruction::Fstore2 => bytecode.push(0x45),
            JvmInstruction::Fstore3 => bytecode.push(0x46),

            JvmInstruction::Dstore { index } => {
                if *index <= 3 {
                    bytecode.push(0x47 + *index as u8);
                }
                else if *index <= 255 {
                    bytecode.push(0x39);
                    bytecode.push(*index as u8);
                }
                else {
                    bytecode.push(0xC4); // wide
                    bytecode.push(0x39);
                    bytecode.push((*index >> 8) as u8);
                    bytecode.push((*index & 0xFF) as u8);
                }
            }
            JvmInstruction::Dstore0 => bytecode.push(0x47),
            JvmInstruction::Dstore1 => bytecode.push(0x48),
            JvmInstruction::Dstore2 => bytecode.push(0x49),
            JvmInstruction::Dstore3 => bytecode.push(0x4A),

            JvmInstruction::Astore { index } => {
                if *index <= 3 {
                    bytecode.push(0x4B + *index as u8);
                }
                else if *index <= 255 {
                    bytecode.push(0x3A);
                    bytecode.push(*index as u8);
                }
                else {
                    bytecode.push(0xC4); // wide
                    bytecode.push(0x3A);
                    bytecode.push((*index >> 8) as u8);
                    bytecode.push((*index & 0xFF) as u8);
                }
            }
            JvmInstruction::Astore0 => bytecode.push(0x4B),
            JvmInstruction::Astore1 => bytecode.push(0x4C),
            JvmInstruction::Astore2 => bytecode.push(0x4D),
            JvmInstruction::Astore3 => bytecode.push(0x4E),

            JvmInstruction::Pop => bytecode.push(0x57),
            JvmInstruction::Pop2 => bytecode.push(0x58),
            JvmInstruction::Dup => bytecode.push(0x59),
            JvmInstruction::DupX1 => bytecode.push(0x5A),
            JvmInstruction::DupX2 => bytecode.push(0x5B),
            JvmInstruction::Dup2 => bytecode.push(0x5C),
            JvmInstruction::Dup2X1 => bytecode.push(0x5D),
            JvmInstruction::Dup2X2 => bytecode.push(0x5E),
            JvmInstruction::Swap => bytecode.push(0x5F),

            JvmInstruction::Iadd => bytecode.push(0x60),
            JvmInstruction::Ladd => bytecode.push(0x61),
            JvmInstruction::Fadd => bytecode.push(0x62),
            JvmInstruction::Dadd => bytecode.push(0x63),
            JvmInstruction::Isub => bytecode.push(0x64),
            JvmInstruction::Lsub => bytecode.push(0x65),
            JvmInstruction::Fsub => bytecode.push(0x66),
            JvmInstruction::Dsub => bytecode.push(0x67),
            JvmInstruction::Imul => bytecode.push(0x68),
            JvmInstruction::Lmul => bytecode.push(0x69),
            JvmInstruction::Fmul => bytecode.push(0x6A),
            JvmInstruction::Dmul => bytecode.push(0x6B),
            JvmInstruction::Idiv => bytecode.push(0x6C),
            JvmInstruction::Ldiv => bytecode.push(0x6D),
            JvmInstruction::Fdiv => bytecode.push(0x6E),
            JvmInstruction::Ddiv => bytecode.push(0x6F),
            JvmInstruction::Irem => bytecode.push(0x70),
            JvmInstruction::Lrem => bytecode.push(0x71),
            JvmInstruction::Frem => bytecode.push(0x72),
            JvmInstruction::Drem => bytecode.push(0x73),
            JvmInstruction::Ineg => bytecode.push(0x74),
            JvmInstruction::Lneg => bytecode.push(0x75),
            JvmInstruction::Fneg => bytecode.push(0x76),
            JvmInstruction::Dneg => bytecode.push(0x77),

            JvmInstruction::Ishl => bytecode.push(0x78),
            JvmInstruction::Lshl => bytecode.push(0x79),
            JvmInstruction::Ishr => bytecode.push(0x7A),
            JvmInstruction::Lshr => bytecode.push(0x7B),
            JvmInstruction::Iushr => bytecode.push(0x7C),
            JvmInstruction::Lushr => bytecode.push(0x7D),
            JvmInstruction::Iand => bytecode.push(0x7E),
            JvmInstruction::Land => bytecode.push(0x7F),
            JvmInstruction::Ior => bytecode.push(0x80),
            JvmInstruction::Lor => bytecode.push(0x81),
            JvmInstruction::Ixor => bytecode.push(0x82),
            JvmInstruction::Lxor => bytecode.push(0x83),

            JvmInstruction::Lcmp => bytecode.push(0x94),
            JvmInstruction::Fcmpl => bytecode.push(0x95),
            JvmInstruction::Fcmpg => bytecode.push(0x96),
            JvmInstruction::Dcmpl => bytecode.push(0x97),
            JvmInstruction::Dcmpg => bytecode.push(0x98),

            JvmInstruction::Ifeq { target } => {
                jump_patches.push((pos, pos + 1, target.clone(), false));
                bytecode.push(0x99);
                bytecode.push(0);
                bytecode.push(0);
            }
            JvmInstruction::Ifne { target } => {
                jump_patches.push((pos, pos + 1, target.clone(), false));
                bytecode.push(0x9A);
                bytecode.push(0);
                bytecode.push(0);
            }
            JvmInstruction::Iflt { target } => {
                jump_patches.push((pos, pos + 1, target.clone(), false));
                bytecode.push(0x9B);
                bytecode.push(0);
                bytecode.push(0);
            }
            JvmInstruction::Ifge { target } => {
                jump_patches.push((pos, pos + 1, target.clone(), false));
                bytecode.push(0x9C);
                bytecode.push(0);
                bytecode.push(0);
            }
            JvmInstruction::Ifgt { target } => {
                jump_patches.push((pos, pos + 1, target.clone(), false));
                bytecode.push(0x9D);
                bytecode.push(0);
                bytecode.push(0);
            }
            JvmInstruction::Ifle { target } => {
                jump_patches.push((pos, pos + 1, target.clone(), false));
                bytecode.push(0x9E);
                bytecode.push(0);
                bytecode.push(0);
            }
            JvmInstruction::IfIcmpeq { target } => {
                jump_patches.push((pos, pos + 1, target.clone(), false));
                bytecode.push(0x9F);
                bytecode.push(0);
                bytecode.push(0);
            }
            JvmInstruction::IfIcmpne { target } => {
                jump_patches.push((pos, pos + 1, target.clone(), false));
                bytecode.push(0xA0);
                bytecode.push(0);
                bytecode.push(0);
            }
            JvmInstruction::IfIcmplt { target } => {
                jump_patches.push((pos, pos + 1, target.clone(), false));
                bytecode.push(0xA1);
                bytecode.push(0);
                bytecode.push(0);
            }
            JvmInstruction::IfIcmpge { target } => {
                jump_patches.push((pos, pos + 1, target.clone(), false));
                bytecode.push(0xA2);
                bytecode.push(0);
                bytecode.push(0);
            }
            JvmInstruction::IfIcmpgt { target } => {
                jump_patches.push((pos, pos + 1, target.clone(), false));
                bytecode.push(0xA3);
                bytecode.push(0);
                bytecode.push(0);
            }
            JvmInstruction::IfIcmple { target } => {
                jump_patches.push((pos, pos + 1, target.clone(), false));
                bytecode.push(0xA4);
                bytecode.push(0);
                bytecode.push(0);
            }
            JvmInstruction::IfAcmpeq { target } => {
                jump_patches.push((pos, pos + 1, target.clone(), false));
                bytecode.push(0xA5);
                bytecode.push(0);
                bytecode.push(0);
            }
            JvmInstruction::IfAcmpne { target } => {
                jump_patches.push((pos, pos + 1, target.clone(), false));
                bytecode.push(0xA6);
                bytecode.push(0);
                bytecode.push(0);
            }
            JvmInstruction::Ifnull { target } => {
                jump_patches.push((pos, pos + 1, target.clone(), false));
                bytecode.push(0xC6);
                bytecode.push(0);
                bytecode.push(0);
            }
            JvmInstruction::Ifnonnull { target } => {
                jump_patches.push((pos, pos + 1, target.clone(), false));
                bytecode.push(0xC7);
                bytecode.push(0);
                bytecode.push(0);
            }
            JvmInstruction::Goto { target } => {
                jump_patches.push((pos, pos + 1, target.clone(), false));
                bytecode.push(0xA7);
                bytecode.push(0);
                bytecode.push(0);
            }
            JvmInstruction::GotoW { target } => {
                jump_patches.push((pos, pos + 1, target.clone(), true));
                bytecode.push(0xC8);
                bytecode.push(0);
                bytecode.push(0);
                bytecode.push(0);
                bytecode.push(0);
            }

            JvmInstruction::Return => bytecode.push(0xB1),
            JvmInstruction::Ireturn => bytecode.push(0xAC),
            JvmInstruction::Lreturn => bytecode.push(0xAD),
            JvmInstruction::Freturn => bytecode.push(0xAE),
            JvmInstruction::Dreturn => bytecode.push(0xAF),
            JvmInstruction::Areturn => bytecode.push(0xB0),

            JvmInstruction::Arraylength => bytecode.push(0xBE),
            JvmInstruction::Athrow => bytecode.push(0xBF),
            JvmInstruction::Monitorenter => bytecode.push(0xC2),
            JvmInstruction::Monitorexit => bytecode.push(0xC3),

            JvmInstruction::Iaload => bytecode.push(0x2E),
            JvmInstruction::Laload => bytecode.push(0x2F),
            JvmInstruction::Faload => bytecode.push(0x30),
            JvmInstruction::Daload => bytecode.push(0x31),
            JvmInstruction::Aaload => bytecode.push(0x32),
            JvmInstruction::Baload => bytecode.push(0x33),
            JvmInstruction::Saload => bytecode.push(0x34),

            JvmInstruction::Iastore => bytecode.push(0x4F),
            JvmInstruction::Lastore => bytecode.push(0x50),
            JvmInstruction::Fastore => bytecode.push(0x51),
            JvmInstruction::Dastore => bytecode.push(0x52),
            JvmInstruction::Aastore => bytecode.push(0x53),
            JvmInstruction::Bastore => bytecode.push(0x54),
            JvmInstruction::Sastore => bytecode.push(0x55),

            JvmInstruction::Getstatic { class_name, field_name, descriptor } => {
                let index = self.add_field_ref(class_name.clone(), field_name.clone(), descriptor.clone());
                bytecode.push(0xB2); // getstatic
                bytecode.push((index >> 8) as u8);
                bytecode.push((index & 0xFF) as u8);
            }
            JvmInstruction::Putstatic { class_name, field_name, descriptor } => {
                let index = self.add_field_ref(class_name.clone(), field_name.clone(), descriptor.clone());
                bytecode.push(0xB3); // putstatic
                bytecode.push((index >> 8) as u8);
                bytecode.push((index & 0xFF) as u8);
            }
            JvmInstruction::Getfield { class_name, field_name, descriptor } => {
                let index = self.add_field_ref(class_name.clone(), field_name.clone(), descriptor.clone());
                bytecode.push(0xB4); // getfield
                bytecode.push((index >> 8) as u8);
                bytecode.push((index & 0xFF) as u8);
            }
            JvmInstruction::Putfield { class_name, field_name, descriptor } => {
                let index = self.add_field_ref(class_name.clone(), field_name.clone(), descriptor.clone());
                bytecode.push(0xB5); // putfield
                bytecode.push((index >> 8) as u8);
                bytecode.push((index & 0xFF) as u8);
            }
            JvmInstruction::Invokevirtual { class_name, method_name, descriptor } => {
                let index = self.add_method_ref(class_name.clone(), method_name.clone(), descriptor.clone());
                bytecode.push(0xB6); // invokevirtual
                bytecode.push((index >> 8) as u8);
                bytecode.push((index & 0xFF) as u8);
            }
            JvmInstruction::Invokespecial { class_name, method_name, descriptor } => {
                let index = self.add_method_ref(class_name.clone(), method_name.clone(), descriptor.clone());
                bytecode.push(0xB7); // invokespecial
                bytecode.push((index >> 8) as u8);
                bytecode.push((index & 0xFF) as u8);
            }
            JvmInstruction::Invokestatic { class_name, method_name, descriptor } => {
                let index = self.add_method_ref(class_name.clone(), method_name.clone(), descriptor.clone());
                bytecode.push(0xB8); // invokestatic
                bytecode.push((index >> 8) as u8);
                bytecode.push((index & 0xFF) as u8);
            }
            JvmInstruction::Invokeinterface { class_name, method_name, descriptor } => {
                let index = self.add_interface_method_ref(class_name.clone(), method_name.clone(), descriptor.clone());
                bytecode.push(0xB9); // invokeinterface
                bytecode.push((index >> 8) as u8);
                bytecode.push((index & 0xFF) as u8);

                // 计算参数数量（包括 this）
                let count = calculate_parameter_count(descriptor) + 1;
                bytecode.push(count as u8);
                bytecode.push(0);
            }
            JvmInstruction::Invokedynamic { class_name, method_name, descriptor } => {
                let index = self.add_method_ref(class_name.clone(), method_name.clone(), descriptor.clone());
                bytecode.push(0xBA); // invokedynamic
                bytecode.push((index >> 8) as u8);
                bytecode.push((index & 0xFF) as u8);
                bytecode.push(0);
                bytecode.push(0);
            }
            JvmInstruction::New { class_name } => {
                let index = self.add_class(class_name.clone());
                bytecode.push(0xBB); // new
                bytecode.push((index >> 8) as u8);
                bytecode.push((index & 0xFF) as u8);
            }
            JvmInstruction::Newarray { atype } => {
                bytecode.push(0xBC); // newarray
                bytecode.push(*atype);
            }
            JvmInstruction::Anewarray { class_name } => {
                let index = self.add_class(class_name.clone());
                bytecode.push(0xBD); // anewarray
                bytecode.push((index >> 8) as u8);
                bytecode.push((index & 0xFF) as u8);
            }
            JvmInstruction::Multianewarray { class_name, dimensions } => {
                let index = self.add_class(class_name.clone());
                bytecode.push(0xC5); // multianewarray
                bytecode.push((index >> 8) as u8);
                bytecode.push((index & 0xFF) as u8);
                bytecode.push(*dimensions);
            }
            JvmInstruction::Checkcast { class_name } => {
                let index = self.add_class(class_name.clone());
                bytecode.push(0xC0); // checkcast
                bytecode.push((index >> 8) as u8);
                bytecode.push((index & 0xFF) as u8);
            }
            JvmInstruction::Instanceof { class_name } => {
                let index = self.add_class(class_name.clone());
                bytecode.push(0xC1); // instanceof
                bytecode.push((index >> 8) as u8);
                bytecode.push((index & 0xFF) as u8);
            }
            JvmInstruction::Jsr { target } => {
                jump_patches.push((pos, pos + 1, target.clone(), false));
                bytecode.push(0xA8); // jsr
                bytecode.push(0);
                bytecode.push(0);
            }
            JvmInstruction::Ret { index } => {
                if *index <= 255 {
                    bytecode.push(0xA9); // ret
                    bytecode.push(*index as u8);
                }
                else {
                    bytecode.push(0xC4); // wide
                    bytecode.push(0xA9);
                    bytecode.push((*index >> 8) as u8);
                    bytecode.push((*index & 0xFF) as u8);
                }
            }
            JvmInstruction::Iinc { index, increment } => {
                if *index <= 255 && *increment >= -128 && *increment <= 127 {
                    bytecode.push(0x84); // iinc
                    bytecode.push(*index as u8);
                    bytecode.push(*increment as u8);
                }
                else {
                    bytecode.push(0xC4); // wide
                    bytecode.push(0x84);
                    bytecode.push((*index >> 8) as u8);
                    bytecode.push((*index & 0xFF) as u8);
                    bytecode.push((*increment >> 8) as u8);
                    bytecode.push((*increment & 0xFF) as u8);
                }
            }
            JvmInstruction::JsrW { target } => {
                jump_patches.push((pos, pos + 1, target.clone(), true));
                bytecode.push(0xC9); // jsr_w
                bytecode.push(0);
                bytecode.push(0);
                bytecode.push(0);
                bytecode.push(0);
            }

            JvmInstruction::Lookupswitch { default, pairs } => {
                bytecode.push(0xAB); // lookupswitch

                // 填充以对齐到 4 字节边界
                let padding = (4 - (bytecode.len() % 4)) % 4;
                for _ in 0..padding {
                    bytecode.push(0);
                }

                // default 偏移量
                let default_patch_pos = bytecode.len();
                jump_patches.push((pos, default_patch_pos, default.clone(), true));
                bytecode.push(0);
                bytecode.push(0);
                bytecode.push(0);
                bytecode.push(0);

                // npairs
                let npairs = pairs.len() as i32;
                bytecode.push((npairs >> 24) as u8);
                bytecode.push((npairs >> 16) as u8);
                bytecode.push((npairs >> 8) as u8);
                bytecode.push(npairs as u8);

                // match-offset pairs
                for (val, target) in pairs {
                    // match
                    bytecode.push((val >> 24) as u8);
                    bytecode.push((val >> 16) as u8);
                    bytecode.push((val >> 8) as u8);
                    bytecode.push(*val as u8);

                    // offset
                    let patch_pos = bytecode.len();
                    jump_patches.push((pos, patch_pos, target.clone(), true));
                    bytecode.push(0);
                    bytecode.push(0);
                    bytecode.push(0);
                    bytecode.push(0);
                }
            }
            JvmInstruction::Tableswitch { low, high, default, targets } => {
                bytecode.push(0xAA); // tableswitch

                // 填充以对齐到 4 字节边界
                let padding = (4 - (bytecode.len() % 4)) % 4;
                for _ in 0..padding {
                    bytecode.push(0);
                }

                // default 偏移量
                let default_patch_pos = bytecode.len();
                jump_patches.push((pos, default_patch_pos, default.clone(), true));
                bytecode.push(0);
                bytecode.push(0);
                bytecode.push(0);
                bytecode.push(0);

                // low
                bytecode.push((low >> 24) as u8);
                bytecode.push((low >> 16) as u8);
                bytecode.push((low >> 8) as u8);
                bytecode.push(*low as u8);

                // high
                bytecode.push((high >> 24) as u8);
                bytecode.push((high >> 16) as u8);
                bytecode.push((high >> 8) as u8);
                bytecode.push(*high as u8);

                // offsets
                for target in targets {
                    let patch_pos = bytecode.len();
                    jump_patches.push((pos, patch_pos, target.clone(), true));
                    bytecode.push(0);
                    bytecode.push(0);
                    bytecode.push(0);
                    bytecode.push(0);
                }
            }
            JvmInstruction::Label { .. } => {}           // 已经在第一遍处理过
            JvmInstruction::Wide => bytecode.push(0xC4), // standalone wide is unusual but possible
        }
    }
}
