//! JVM Class 文件写入器
//!
//! 这个模块实现了将 JVM 程序转换为 Class 文件字节码的功能。
use crate::formats::class::view::ClassView;

use crate::program::*;
use byteorder::{BigEndian, WriteBytesExt};
use gaia_types::{GaiaDiagnostics, GaiaError, Result};
use std::{collections::HashMap, io::Write};

/// Class 文件写入器
pub struct ClassWriter<W> {
    /// 二进制汇编器
    writer: W,
}

impl<W> ClassWriter<W> {
    /// 创建新的 Class 写入器
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    /// 完成写入并返回底层写入器
    pub fn finish(self) -> W {
        self.writer
    }
}

impl<W: Write> ClassWriter<W> {
    /// 将 ClassView 写入为二进制 Class 格式
    pub fn write(mut self, class_view: ClassView) -> GaiaDiagnostics<W> {
        match self.write_class_file(class_view) {
            Ok(_) => GaiaDiagnostics::success(self.finish()),
            Err(error) => GaiaDiagnostics::failure(error),
        }
    }

    /// 写入 Class 文件
    fn write_class_file(&mut self, class_view: ClassView) -> Result<()> {
        // 构建常量池索引映射
        let mut constant_pool_indices = HashMap::new();
        let mut constant_pool_entries = Vec::new();

        // 添加必要的常量池条目
        self.build_constant_pool(&class_view, &mut constant_pool_entries, &mut constant_pool_indices)?;

        // 写入魔数
        self.writer.write_u32::<BigEndian>(0xCAFEBABE)?;

        // 写入版本号
        self.writer.write_u16::<BigEndian>(class_view.version.minor)?;
        self.writer.write_u16::<BigEndian>(class_view.version.major)?;

        // 写入常量池
        self.write_constant_pool(&class_view.constant_pool)?;

        // 写入访问标志
        self.writer.write_u16::<BigEndian>(class_view.access_flags.to_flags())?;

        // 写入类名索引
        let this_class_index = self.get_class_index(&class_view.this_class, &constant_pool_indices)?;
        self.writer.write_u16::<BigEndian>(this_class_index)?;

        // 写入超类名索引
        let super_class_index = if let Some(super_class) = &class_view.super_class {
            self.get_class_index(super_class, &constant_pool_indices)?
        }
        else {
            0 // 只有 Object 类没有超类
        };
        self.writer.write_u16::<BigEndian>(super_class_index)?;

        // 写入接口数量和接口索引
        self.writer.write_u16::<BigEndian>(class_view.interfaces.len() as u16)?;
        for interface in &class_view.interfaces {
            let interface_index = self.get_class_index(interface, &constant_pool_indices)?;
            self.writer.write_u16::<BigEndian>(interface_index)?;
        }

        // 写入字段
        self.write_fields(&class_view.fields, &constant_pool_indices)?;

        // 写入方法
        self.write_methods(&class_view.methods, &constant_pool_indices)?;

        // 写入属性
        self.write_attributes(&class_view.attributes, &constant_pool_indices)?;

        Ok(())
    }

    /// 构建常量池
    fn build_constant_pool(
        &mut self,
        class_view: &ClassView,
        entries: &mut Vec<JvmConstantPoolEntry>,
        indices: &mut HashMap<String, u16>,
    ) -> Result<()> {
        let mut index = 1u16; // 常量池索引从 1 开始

        // 添加类名
        let class_name_utf8 = JvmConstantPoolEntry::Utf8 { value: class_view.this_class.clone() };
        entries.push(class_name_utf8);
        indices.insert(format!("utf8_{}", class_view.this_class), index);
        index += 1;

        let class_entry = JvmConstantPoolEntry::Class { name: class_view.this_class.clone() };
        entries.push(class_entry);
        indices.insert(format!("class_{}", class_view.this_class), index);
        index += 1;

        // 添加超类名
        if let Some(super_class) = &class_view.super_class {
            let super_class_utf8 = JvmConstantPoolEntry::Utf8 { value: super_class.clone() };
            entries.push(super_class_utf8);
            indices.insert(format!("utf8_{}", super_class), index);
            index += 1;

            let super_class_entry = JvmConstantPoolEntry::Class { name: super_class.clone() };
            entries.push(super_class_entry);
            indices.insert(format!("class_{}", super_class), index);
            index += 1;
        }

        // 添加程序常量池中的条目
        for entry in &class_view.constant_pool {
            let key = match entry {
                JvmConstantPoolEntry::Utf8 { value } => format!("utf8_{}", value),
                JvmConstantPoolEntry::Integer { value } => format!("integer_{}", value),
                JvmConstantPoolEntry::Float { value } => format!("float_{}", value),
                JvmConstantPoolEntry::Long { value } => format!("long_{}", value),
                JvmConstantPoolEntry::Double { value } => format!("double_{}", value),
                JvmConstantPoolEntry::Class { name } => {
                    self.add_utf8_entry(name.clone(), entries, indices, &mut index)?;
                    format!("class_{}", name)
                }
                JvmConstantPoolEntry::String { value } => {
                    self.add_utf8_entry(value.clone(), entries, indices, &mut index)?;
                    format!("string_{}", value)
                }
                JvmConstantPoolEntry::Fieldref { class_name, name, descriptor } => {
                    // 确保 Class 和 NameAndType 条目存在
                    let class_key = format!("class_{}", class_name);
                    if !indices.contains_key(&class_key) {
                        self.add_utf8_entry(class_name.clone(), entries, indices, &mut index)?;
                        entries.push(JvmConstantPoolEntry::Class { name: class_name.clone() });
                        indices.insert(class_key, index);
                        index += 1;
                    }

                    let name_and_type_key = format!("nameandtype_{}_{}", name, descriptor);
                    if !indices.contains_key(&name_and_type_key) {
                        self.add_utf8_entry(name.clone(), entries, indices, &mut index)?;
                        self.add_utf8_entry(descriptor.clone(), entries, indices, &mut index)?;
                        entries.push(JvmConstantPoolEntry::NameAndType { name: name.clone(), descriptor: descriptor.clone() });
                        indices.insert(name_and_type_key, index);
                        index += 1;
                    }
                    format!("fieldref_{}_{}_{}", class_name, name, descriptor)
                }
                JvmConstantPoolEntry::Methodref { class_name, name, descriptor } => {
                    // 确保 Class 和 NameAndType 条目存在
                    let class_key = format!("class_{}", class_name);
                    if !indices.contains_key(&class_key) {
                        self.add_utf8_entry(class_name.clone(), entries, indices, &mut index)?;
                        entries.push(JvmConstantPoolEntry::Class { name: class_name.clone() });
                        indices.insert(class_key, index);
                        index += 1;
                    }

                    let name_and_type_key = format!("nameandtype_{}_{}", name, descriptor);
                    if !indices.contains_key(&name_and_type_key) {
                        self.add_utf8_entry(name.clone(), entries, indices, &mut index)?;
                        self.add_utf8_entry(descriptor.clone(), entries, indices, &mut index)?;
                        entries.push(JvmConstantPoolEntry::NameAndType { name: name.clone(), descriptor: descriptor.clone() });
                        indices.insert(name_and_type_key, index);
                        index += 1;
                    }
                    format!("methodref_{}_{}_{}", class_name, name, descriptor)
                }
                JvmConstantPoolEntry::InterfaceMethodref { class_name, name, descriptor } => {
                    // 确保 Class 和 NameAndType 条目存在
                    let class_key = format!("class_{}", class_name);
                    if !indices.contains_key(&class_key) {
                        self.add_utf8_entry(class_name.clone(), entries, indices, &mut index)?;
                        entries.push(JvmConstantPoolEntry::Class { name: class_name.clone() });
                        indices.insert(class_key, index);
                        index += 1;
                    }

                    let name_and_type_key = format!("nameandtype_{}_{}", name, descriptor);
                    if !indices.contains_key(&name_and_type_key) {
                        self.add_utf8_entry(name.clone(), entries, indices, &mut index)?;
                        self.add_utf8_entry(descriptor.clone(), entries, indices, &mut index)?;
                        entries.push(JvmConstantPoolEntry::NameAndType { name: name.clone(), descriptor: descriptor.clone() });
                        indices.insert(name_and_type_key, index);
                        index += 1;
                    }
                    format!("interfacemethodref_{}_{}_{}", class_name, name, descriptor)
                }
                JvmConstantPoolEntry::NameAndType { name, descriptor } => {
                    self.add_utf8_entry(name.clone(), entries, indices, &mut index)?;
                    self.add_utf8_entry(descriptor.clone(), entries, indices, &mut index)?;
                    format!("nameandtype_{}_{}", name, descriptor)
                }
                JvmConstantPoolEntry::Nop => "nop".to_string(),
            };

            if !indices.contains_key(&key) {
                entries.push(entry.clone());
                indices.insert(key, index);
                index += 1;

                // 对于 Long 和 Double，需要占用两个索引位置
                match entry {
                    JvmConstantPoolEntry::Long { .. } | JvmConstantPoolEntry::Double { .. } => {
                        index += 1;
                    }
                    _ => {}
                }
            }
        }

        // 添加方法和字段的名称和描述符
        for method in &class_view.methods {
            // 方法名
            let name_utf8 = JvmConstantPoolEntry::Utf8 { value: method.name.clone() };
            if !entries.iter().any(|e| matches!(e, JvmConstantPoolEntry::Utf8 { value } if value == &method.name)) {
                entries.push(name_utf8);
                indices.insert(format!("utf8_{}", method.name), index);
                index += 1;
            }

            // 方法描述符
            let desc_utf8 = JvmConstantPoolEntry::Utf8 { value: method.descriptor.clone() };
            if !entries.iter().any(|e| matches!(e, JvmConstantPoolEntry::Utf8 { value } if value == &method.descriptor)) {
                entries.push(desc_utf8);
                indices.insert(format!("utf8_{}", method.descriptor), index);
                index += 1;
            }
        }

        for field in &class_view.fields {
            // 字段名
            let name_utf8 = JvmConstantPoolEntry::Utf8 { value: field.name.clone() };
            if !entries.iter().any(|e| matches!(e, JvmConstantPoolEntry::Utf8 { value } if value == &field.name)) {
                entries.push(name_utf8);
                indices.insert(format!("utf8_{}", field.name), index);
                index += 1;
            }

            // 字段描述符
            let desc_utf8 = JvmConstantPoolEntry::Utf8 { value: field.descriptor.clone() };
            if !entries.iter().any(|e| matches!(e, JvmConstantPoolEntry::Utf8 { value } if value == &field.descriptor)) {
                entries.push(desc_utf8);
                indices.insert(format!("utf8_{}", field.descriptor), index);
                index += 1;
            }
        }

        Ok(())
    }

    /// 添加 UTF-8 常量池条目并返回其索引
    fn add_utf8_entry(
        &mut self,
        value: String,
        entries: &mut Vec<JvmConstantPoolEntry>,
        indices: &mut HashMap<String, u16>,
        current_index: &mut u16,
    ) -> Result<u16> {
        let key = format!("utf8_{}", value);
        if let Some(index) = indices.get(&key) {
            Ok(*index)
        }
        else {
            let index = *current_index;
            entries.push(JvmConstantPoolEntry::Utf8 { value: value.clone() });
            indices.insert(key, index);
            *current_index += 1;
            Ok(index)
        }
    }

    /// 写入常量池
    fn write_constant_pool(&mut self, entries: &[JvmConstantPoolEntry]) -> Result<()> {
        // 写入常量池大小（条目数 + 1）
        self.writer.write_u16::<BigEndian>((entries.len() + 1) as u16)?;

        // 写入每个常量池条目
        for entry in entries {
            self.write_constant_pool_entry(entry)?;
        }

        Ok(())
    }

    /// 写入单个常量池条目
    fn write_constant_pool_entry(&mut self, entry: &JvmConstantPoolEntry) -> Result<()> {
        match entry {
            JvmConstantPoolEntry::Utf8 { value } => {
                self.writer.write_u8(1)?; // CONSTANT_Utf8
                self.writer.write_u16::<BigEndian>(value.len() as u16)?;
                self.writer.write_all(value.as_bytes())?;
            }
            JvmConstantPoolEntry::Integer { value } => {
                self.writer.write_u8(3)?; // CONSTANT_Integer
                self.writer.write_i32::<BigEndian>(*value)?;
            }
            JvmConstantPoolEntry::Nop => {
                return Err(GaiaError::custom_error("Nop entry should not be written to constant pool".to_string()));
            }
            JvmConstantPoolEntry::Float { value } => {
                self.writer.write_u8(4)?; // CONSTANT_Float
                self.writer.write_f32::<BigEndian>(*value)?;
            }
            JvmConstantPoolEntry::Long { value } => {
                self.writer.write_u8(5)?; // CONSTANT_Long
                self.writer.write_i64::<BigEndian>(*value)?;
            }
            JvmConstantPoolEntry::Double { value } => {
                self.writer.write_u8(6)?; // CONSTANT_Double
                self.writer.write_f64::<BigEndian>(*value)?;
            }
            JvmConstantPoolEntry::Class { name: _ } => {
                self.writer.write_u8(7)?; // CONSTANT_Class
                                          // 这里需要查找名称的 UTF-8 索引，暂时写 0
                self.writer.write_u16::<BigEndian>(0)?;
            }
            JvmConstantPoolEntry::String { value: _ } => {
                self.writer.write_u8(8)?; // CONSTANT_String
                                          // 这里需要查找字符串的 UTF-8 索引，暂时写 0
                self.writer.write_u16::<BigEndian>(0)?;
            }
            JvmConstantPoolEntry::Fieldref { class_name: _, name: _, descriptor: _ } => {
                self.writer.write_u8(9)?; // CONSTANT_Fieldref
                self.writer.write_u16::<BigEndian>(0)?; // class_index
                self.writer.write_u16::<BigEndian>(0)?; // name_and_type_index
            }
            JvmConstantPoolEntry::Methodref { class_name: _, name: _, descriptor: _ } => {
                self.writer.write_u8(10)?; // CONSTANT_Methodref
                self.writer.write_u16::<BigEndian>(0)?; // class_index
                self.writer.write_u16::<BigEndian>(0)?; // name_and_type_index
            }
            JvmConstantPoolEntry::InterfaceMethodref { class_name: _, name: _, descriptor: _ } => {
                self.writer.write_u8(11)?; // CONSTANT_InterfaceMethodref
                self.writer.write_u16::<BigEndian>(0)?; // class_index
                self.writer.write_u16::<BigEndian>(0)?; // name_and_type_index
            }
            JvmConstantPoolEntry::NameAndType { name: _, descriptor: _ } => {
                self.writer.write_u8(12)?; // CONSTANT_NameAndType
                self.writer.write_u16::<BigEndian>(0)?; // name_index
                self.writer.write_u16::<BigEndian>(0)?; // descriptor_index
            }
        }
        Ok(())
    }

    /// 获取类的常量池索引
    fn get_class_index(&self, class_name: &str, indices: &HashMap<String, u16>) -> Result<u16> {
        indices
            .get(&format!("class_{}", class_name))
            .copied()
            .ok_or_else(|| GaiaError::custom_error(format!("Class not found in constant pool: {}", class_name)))
    }

    /// 获取 UTF-8 常量池索引
    fn get_utf8_index(&self, value: &str, indices: &HashMap<String, u16>) -> Result<u16> {
        indices
            .get(&format!("utf8_{}", value))
            .copied()
            .ok_or_else(|| GaiaError::custom_error(format!("UTF-8 constant not found in constant pool: {}", value)))
    }

    /// 写入字段
    fn write_fields(&mut self, fields: &[JvmField], constant_pool_indices: &HashMap<String, u16>) -> Result<()> {
        self.writer.write_u16::<BigEndian>(fields.len() as u16)?;

        for field in fields {
            // 访问标志
            self.writer.write_u16::<BigEndian>(field.access_flags.to_flags())?;

            // 名称索引
            let name_index = self.get_utf8_index(&field.name, constant_pool_indices)?;
            self.writer.write_u16::<BigEndian>(name_index)?;

            // 描述符索引
            let descriptor_index = self.get_utf8_index(&field.descriptor, constant_pool_indices)?;
            self.writer.write_u16::<BigEndian>(descriptor_index)?;

            // 属性数量
            self.writer.write_u16::<BigEndian>(field.attributes.len() as u16)?;

            // 写入属性
            self.write_attributes_for_field(field.attributes.as_slice(), constant_pool_indices)?;
        }

        Ok(())
    }

    /// 写入字段属性
    fn write_attributes_for_field(
        &mut self,
        attributes: &[JvmAttribute],
        constant_pool_indices: &HashMap<String, u16>,
    ) -> Result<()> {
        for attribute in attributes {
            match attribute {
                JvmAttribute::ConstantValue { value } => {
                    // 属性名索引 ("ConstantValue")
                    let name_index = self.get_utf8_index("ConstantValue", constant_pool_indices)?;
                    self.writer.write_u16::<BigEndian>(name_index)?;

                    // 属性长度
                    self.writer.write_u32::<BigEndian>(2)?;

                    // 常量值索引
                    let value_index = self.get_constant_pool_index(value, constant_pool_indices)?;
                    self.writer.write_u16::<BigEndian>(value_index)?;
                }
                JvmAttribute::Signature { signature } => {
                    // 属性名索引 ("Signature")
                    let name_index = self.get_utf8_index("Signature", constant_pool_indices)?;
                    self.writer.write_u16::<BigEndian>(name_index)?;

                    // 属性长度
                    self.writer.write_u32::<BigEndian>(2)?;

                    // 签名索引
                    let signature_index = self.get_utf8_index(signature, constant_pool_indices)?;
                    self.writer.write_u16::<BigEndian>(signature_index)?;
                }
                _ => {
                    // 其他属性暂时跳过
                }
            }
        }
        Ok(())
    }

    /// 写入方法
    fn write_methods(&mut self, methods: &[JvmMethod], constant_pool_indices: &HashMap<String, u16>) -> Result<()> {
        self.writer.write_u16::<BigEndian>(methods.len() as u16)?;

        for method in methods {
            // 访问标志
            self.writer.write_u16::<BigEndian>(method.access_flags.to_flags())?;

            // 名称索引
            let name_index = self.get_utf8_index(&method.name, constant_pool_indices)?;
            self.writer.write_u16::<BigEndian>(name_index)?;

            // 描述符索引
            let descriptor_index = self.get_utf8_index(&method.descriptor, constant_pool_indices)?;
            self.writer.write_u16::<BigEndian>(descriptor_index)?;

            // 属性数量
            self.writer.write_u16::<BigEndian>(method.attributes.len() as u16)?;

            // 写入属性
            self.write_attributes_for_method(method.attributes.as_slice(), constant_pool_indices)?;
        }

        Ok(())
    }

    /// 写入方法属性
    fn write_attributes_for_method(
        &mut self,
        attributes: &[JvmAttribute],
        constant_pool_indices: &HashMap<String, u16>,
    ) -> Result<()> {
        for attribute in attributes {
            match attribute {
                JvmAttribute::Code { max_stack, max_locals, code, exception_table, attributes } => {
                    // 属性名索引 ("Code")
                    let name_index = self.get_utf8_index("Code", constant_pool_indices)?;
                    self.writer.write_u16::<BigEndian>(name_index)?;

                    // 计算代码长度
                    let mut code_bytes = Vec::new();
                    code_bytes.extend_from_slice(&code);

                    // 属性长度 = 2 + 2 + 4 + code_length + 2 + exception_table.len() * 8 + 2
                    let attribute_length = 2 + 2 + 4 + code_bytes.len() + 2 + exception_table.len() * 8 + 2;
                    self.writer.write_u32::<BigEndian>(attribute_length as u32)?;

                    // max_stack
                    self.writer.write_u16::<BigEndian>(*max_stack)?;

                    // max_locals
                    self.writer.write_u16::<BigEndian>(*max_locals)?;

                    // code_length
                    self.writer.write_u32::<BigEndian>(code_bytes.len() as u32)?;

                    // code
                    self.writer.write_all(&code_bytes)?;

                    // exception_table_length
                    self.writer.write_u16::<BigEndian>(exception_table.len() as u16)?;

                    // exception_table
                    for entry in exception_table {
                        self.writer.write_u16::<BigEndian>(entry.start_pc)?;
                        self.writer.write_u16::<BigEndian>(entry.end_pc)?;
                        self.writer.write_u16::<BigEndian>(entry.handler_pc)?;
                        let catch_type_index = match &entry.catch_type {
                            Some(catch_type) => self.get_class_index(catch_type, constant_pool_indices)?,
                            None => 0,
                        };
                        self.writer.write_u16::<BigEndian>(catch_type_index)?;
                    }

                    // attributes_count
                    self.writer.write_u16::<BigEndian>(attributes.len() as u16)?;

                    // 写入 Code 属性的属性
                    self.write_attributes_for_code(attributes.as_slice(), constant_pool_indices)?;
                }
                JvmAttribute::Exceptions { exceptions } => {
                    // 属性名索引 ("Exceptions")
                    let name_index = self.get_utf8_index("Exceptions", constant_pool_indices)?;
                    self.writer.write_u16::<BigEndian>(name_index)?;

                    // 属性长度
                    self.writer.write_u32::<BigEndian>((2 + exceptions.len() * 2) as u32)?;

                    // 异常数量
                    self.writer.write_u16::<BigEndian>(exceptions.len() as u16)?;

                    // 异常索引
                    for exception in exceptions {
                        let exception_index = self.get_class_index(exception, constant_pool_indices)?;
                        self.writer.write_u16::<BigEndian>(exception_index)?;
                    }
                }
                JvmAttribute::Signature { signature } => {
                    // 属性名索引 ("Signature")
                    let name_index = self.get_utf8_index("Signature", constant_pool_indices)?;
                    self.writer.write_u16::<BigEndian>(name_index)?;

                    // 属性长度
                    self.writer.write_u32::<BigEndian>(2)?;

                    // 签名索引
                    let signature_index = self.get_utf8_index(signature, constant_pool_indices)?;
                    self.writer.write_u16::<BigEndian>(signature_index)?;
                }
                _ => {
                    // 其他属性暂时跳过
                }
            }
        }
        Ok(())
    }

    /// 写入 Code 属性的属性
    fn write_attributes_for_code(
        &mut self,
        attributes: &[JvmAttribute],
        constant_pool_indices: &HashMap<String, u16>,
    ) -> Result<()> {
        for attribute in attributes {
            match attribute {
                JvmAttribute::LineNumberTable { entries } => {
                    // 属性名索引 ("LineNumberTable")
                    let name_index = self.get_utf8_index("LineNumberTable", constant_pool_indices)?;
                    self.writer.write_u16::<BigEndian>(name_index)?;

                    // 属性长度
                    self.writer.write_u32::<BigEndian>((2 + entries.len() * 4) as u32)?;

                    // 行号表长度
                    self.writer.write_u16::<BigEndian>(entries.len() as u16)?;

                    // 行号表
                    for entry in entries {
                        self.writer.write_u16::<BigEndian>(entry.0)?;
                        self.writer.write_u16::<BigEndian>(entry.1)?;
                    }
                }
                JvmAttribute::LocalVariableTable { entries: local_variable_table } => {
                    // 属性名索引 ("LocalVariableTable")
                    let name_index = self.get_utf8_index("LocalVariableTable", constant_pool_indices)?;
                    self.writer.write_u16::<BigEndian>(name_index)?;

                    // 属性长度
                    self.writer.write_u32::<BigEndian>((2 + local_variable_table.len() * 10) as u32)?;

                    // 局部变量表长度
                    self.writer.write_u16::<BigEndian>(local_variable_table.len() as u16)?;

                    // 局部变量表
                    for entry in local_variable_table {
                        self.writer.write_u16::<BigEndian>(entry.start_pc)?;
                        self.writer.write_u16::<BigEndian>(entry.length)?;
                        // 名称索引
                        let name_index = self.get_utf8_index(&entry.name, constant_pool_indices)?;
                        self.writer.write_u16::<BigEndian>(name_index)?;
                        // 描述符索引
                        let descriptor_index = self.get_utf8_index(&entry.descriptor, constant_pool_indices)?;
                        self.writer.write_u16::<BigEndian>(descriptor_index)?;
                        self.writer.write_u16::<BigEndian>(entry.index)?;
                    }
                }
                _ => {
                    // 其他属性暂时跳过
                }
            }
        }
        Ok(())
    }

    /// 获取常量池索引
    fn get_constant_pool_index(
        &self,
        entry: &JvmConstantPoolEntry,
        constant_pool_indices: &HashMap<String, u16>,
    ) -> Result<u16> {
        let key = match entry {
            JvmConstantPoolEntry::Utf8 { value } => format!("utf8_{}", value),
            JvmConstantPoolEntry::Integer { value } => format!("integer_{}", value),
            JvmConstantPoolEntry::Float { value } => format!("float_{}", value),
            JvmConstantPoolEntry::Long { value } => format!("long_{}", value),
            JvmConstantPoolEntry::Double { value } => format!("double_{}", value),
            JvmConstantPoolEntry::Class { name } => format!("class_{}", name),
            JvmConstantPoolEntry::String { value } => format!("string_{}", value),
            JvmConstantPoolEntry::Fieldref { class_name, name, descriptor } => {
                format!("fieldref_{}_{}_{}", class_name, name, descriptor)
            }
            JvmConstantPoolEntry::Methodref { class_name, name, descriptor } => {
                format!("methodref_{}_{}_{}", class_name, name, descriptor)
            }
            JvmConstantPoolEntry::InterfaceMethodref { class_name, name, descriptor } => {
                format!("interfacemethodref_{}_{}_{}", class_name, name, descriptor)
            }
            JvmConstantPoolEntry::NameAndType { name, descriptor } => {
                format!("nameandtype_{}_{}", name, descriptor)
            }
            JvmConstantPoolEntry::Nop => {
                return Err(GaiaError::custom_error("Nop entry does not have a valid index".to_string()))
            }
        };

        constant_pool_indices
            .get(&key)
            .copied()
            .ok_or_else(|| GaiaError::custom_error(format!("Constant pool entry not found: {}", key)))
    }

    /// 写入属性
    fn write_attributes(&mut self, attributes: &[JvmAttribute], constant_pool_indices: &HashMap<String, u16>) -> Result<()> {
        self.writer.write_u16::<BigEndian>(attributes.len() as u16)?;

        self.write_attributes_for_class(attributes, constant_pool_indices)?;

        Ok(())
    }

    /// 写入类属性
    fn write_attributes_for_class(
        &mut self,
        attributes: &[JvmAttribute],
        constant_pool_indices: &HashMap<String, u16>,
    ) -> Result<()> {
        for attribute in attributes {
            match attribute {
                JvmAttribute::SourceFile { filename } => {
                    // 属性名索引（"SourceFile"）
                    let name_index = self.get_utf8_index("SourceFile", constant_pool_indices)?;
                    self.writer.write_u16::<BigEndian>(name_index)?;

                    // 属性长度
                    self.writer.write_u32::<BigEndian>(2)?;

                    // 源文件名索引
                    let filename_index = self.get_utf8_index(filename, constant_pool_indices)?;
                    self.writer.write_u16::<BigEndian>(filename_index)?;
                }
                JvmAttribute::Signature { signature } => {
                    // 属性名索引 ("Signature")
                    let name_index = self.get_utf8_index("Signature", constant_pool_indices)?;
                    self.writer.write_u16::<BigEndian>(name_index)?;

                    // 属性长度
                    self.writer.write_u32::<BigEndian>(2)?;

                    // 签名索引
                    let signature_index = self.get_utf8_index(signature, constant_pool_indices)?;
                    self.writer.write_u16::<BigEndian>(signature_index)?;
                }
                _ => {
                    // 其他属性暂时跳过
                }
            }
        }
        Ok(())
    }
}
