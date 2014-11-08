//! JVM Class 文件写入器
//!
//! 这个模块实现了将 JVM 程序转换为 Class 文件字节码的功能。

use crate::program::*;
use byteorder::{BigEndian, WriteBytesExt};
use gaia_types::{BinaryAssembler, GaiaDiagnostics, GaiaError, Result};
use std::{collections::HashMap, io::Write};

/// Class 文件写入器
pub struct ClassWriter<W> {
    /// 二进制汇编器
    writer: BinaryAssembler<W, BigEndian>,
}

impl<W> ClassWriter<W> {
    /// 创建新的 Class 写入器
    pub fn new(assembler: W) -> Self {
        Self { writer: BinaryAssembler::new(assembler) }
    }

    /// 完成写入并返回底层写入器
    pub fn finish(self) -> W {
        self.writer.finish()
    }
}

impl<W: Write> ClassWriter<W> {
    /// 将 JvmProgram 写入为二进制 Class 格式
    pub fn write(mut self, program: JvmProgram) -> GaiaDiagnostics<W> {
        match self.write_class_file(program) {
            Ok(_) => GaiaDiagnostics::success(self.finish()),
            Err(error) => GaiaDiagnostics::failure(error),
        }
    }

    /// 写入 Class 文件
    fn write_class_file(&mut self, program: JvmProgram) -> Result<()> {
        // 构建常量池索引映射
        let mut constant_pool_indices = HashMap::new();
        let mut constant_pool_entries = Vec::new();

        // 添加必要的常量池条目
        self.build_constant_pool(&program, &mut constant_pool_entries, &mut constant_pool_indices)?;

        // 写入魔数
        self.writer.write_u32(0xCAFEBABE)?;

        // 写入版本号
        self.writer.write_u16(program.version.minor)?;
        self.writer.write_u16(program.version.major)?;

        // 写入常量池
        self.write_constant_pool(&constant_pool_entries)?;

        // 写入访问标志
        self.writer.write_u16(program.access_flags.to_flags())?;

        // 写入类名索引
        let this_class_index = self.get_class_index(&program.name, &constant_pool_indices)?;
        self.writer.write_u16(this_class_index)?;

        // 写入超类名索引
        let super_class_index = if let Some(super_class) = &program.super_class {
            self.get_class_index(super_class, &constant_pool_indices)?
        }
        else {
            0 // 只有 Object 类没有超类
        };
        self.writer.write_u16(super_class_index)?;

        // 写入接口数量和接口索引
        self.writer.write_u16(program.interfaces.len() as u16)?;
        for interface in &program.interfaces {
            let interface_index = self.get_class_index(interface, &constant_pool_indices)?;
            self.writer.write_u16(interface_index)?;
        }

        // 写入字段
        self.write_fields(&program.fields, &constant_pool_indices)?;

        // 写入方法
        self.write_methods(&program.methods, &constant_pool_indices)?;

        // 写入属性
        self.write_attributes(&program.attributes, &constant_pool_indices)?;

        Ok(())
    }

    /// 构建常量池
    fn build_constant_pool(
        &self,
        program: &JvmProgram,
        entries: &mut Vec<JvmConstantPoolEntry>,
        indices: &mut HashMap<String, u16>,
    ) -> Result<()> {
        let mut index = 1u16; // 常量池索引从 1 开始

        // 添加类名
        let class_name_utf8 = JvmConstantPoolEntry::Utf8 { value: program.name.clone() };
        entries.push(class_name_utf8);
        indices.insert(format!("utf8_{}", program.name), index);
        index += 1;

        let class_entry = JvmConstantPoolEntry::Class { name: program.name.clone() };
        entries.push(class_entry);
        indices.insert(format!("class_{}", program.name), index);
        index += 1;

        // 添加超类名
        if let Some(super_class) = &program.super_class {
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
        for entry in &program.constant_pool.entries {
            entries.push(entry.clone());
            index += 1;

            // 对于 Long 和 Double，需要占用两个索引位置
            match entry {
                JvmConstantPoolEntry::Long { .. } | JvmConstantPoolEntry::Double { .. } => {
                    index += 1;
                }
                _ => {}
            }
        }

        // 添加方法和字段的名称和描述符
        for method in &program.methods {
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

        for field in &program.fields {
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

    /// 写入常量池
    fn write_constant_pool(&mut self, entries: &[JvmConstantPoolEntry]) -> Result<()> {
        // 写入常量池大小（条目数 + 1）
        self.writer.write_u16((entries.len() + 1) as u16)?;

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
                self.writer.write_u16(value.len() as u16)?;
                self.writer.write_all(value.as_bytes())?;
            }
            JvmConstantPoolEntry::Integer { value } => {
                self.writer.write_u8(3)?; // CONSTANT_Integer
                self.writer.write_i32::<BigEndian>(*value)?;
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
                self.writer.write_u16(0)?;
            }
            JvmConstantPoolEntry::String { value: _ } => {
                self.writer.write_u8(8)?; // CONSTANT_String
                                          // 这里需要查找字符串的 UTF-8 索引，暂时写 0
                self.writer.write_u16(0)?;
            }
            JvmConstantPoolEntry::Fieldref { class_name: _, name: _, descriptor: _ } => {
                self.writer.write_u8(9)?; // CONSTANT_Fieldref
                self.writer.write_u16(0)?; // class_index
                self.writer.write_u16(0)?; // name_and_type_index
            }
            JvmConstantPoolEntry::Methodref { class_name: _, name: _, descriptor: _ } => {
                self.writer.write_u8(10)?; // CONSTANT_Methodref
                self.writer.write_u16(0)?; // class_index
                self.writer.write_u16(0)?; // name_and_type_index
            }
            JvmConstantPoolEntry::InterfaceMethodref { class_name: _, name: _, descriptor: _ } => {
                self.writer.write_u8(11)?; // CONSTANT_InterfaceMethodref
                self.writer.write_u16(0)?; // class_index
                self.writer.write_u16(0)?; // name_and_type_index
            }
            JvmConstantPoolEntry::NameAndType { name: _, descriptor: _ } => {
                self.writer.write_u8(12)?; // CONSTANT_NameAndType
                self.writer.write_u16(0)?; // name_index
                self.writer.write_u16(0)?; // descriptor_index
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

    /// 写入字段
    fn write_fields(&mut self, fields: &[JvmField], _indices: &HashMap<String, u16>) -> Result<()> {
        self.writer.write_u16(fields.len() as u16)?;

        for field in fields {
            // 访问标志
            self.writer.write_u16(field.access_flags.to_flags())?;

            // 名称索引（暂时写 0）
            self.writer.write_u16(0)?;

            // 描述符索引（暂时写 0）
            self.writer.write_u16(0)?;

            // 属性数量
            self.writer.write_u16(0)?;
        }

        Ok(())
    }

    /// 写入方法
    fn write_methods(&mut self, methods: &[JvmMethod], _indices: &HashMap<String, u16>) -> Result<()> {
        self.writer.write_u16(methods.len() as u16)?;

        for method in methods {
            // 访问标志
            self.writer.write_u16(method.access_flags.to_flags())?;

            // 名称索引（暂时写 0）
            self.writer.write_u16(0)?;

            // 描述符索引（暂时写 0）
            self.writer.write_u16(0)?;

            // 属性数量（至少有一个 Code 属性）
            self.writer.write_u16(1)?;

            // 写入 Code 属性
            self.write_code_attribute(method)?;
        }

        Ok(())
    }

    /// 写入 Code 属性
    fn write_code_attribute(&mut self, method: &JvmMethod) -> Result<()> {
        // 属性名索引（"Code"，暂时写 0）
        self.writer.write_u16(0)?;

        // 计算代码长度
        let mut code_bytes = Vec::new();
        for instruction in &method.instructions {
            self.write_instruction_bytes(instruction, &mut code_bytes)?;
        }

        // 属性长度 = 2 + 2 + 4 + code_length + 2 + exception_table_length * 8 + 2
        let attribute_length = 2 + 2 + 4 + code_bytes.len() + 2 + 0 + 2;
        self.writer.write_u32(attribute_length as u32)?;

        // max_stack
        self.writer.write_u16(method.max_stack)?;

        // max_locals
        self.writer.write_u16(method.max_locals)?;

        // code_length
        self.writer.write_u32(code_bytes.len() as u32)?;

        // code
        self.writer.write_all(&code_bytes)?;

        // exception_table_length
        self.writer.write_u16(0)?;

        // attributes_count
        self.writer.write_u16(0)?;

        Ok(())
    }

    /// 将指令转换为字节码
    fn write_instruction_bytes(&self, instruction: &JvmInstruction, bytes: &mut Vec<u8>) -> Result<()> {
        match instruction {
            JvmInstruction::Simple { opcode } => {
                bytes.push(opcode.to_byte());
            }
            JvmInstruction::WithImmediate { opcode, value } => {
                bytes.push(opcode.to_byte());
                match opcode {
                    JvmOpcode::Bipush => {
                        bytes.push(*value as u8);
                    }
                    JvmOpcode::Sipush => {
                        bytes.extend_from_slice(&(*value as i16).to_be_bytes());
                    }
                    _ => {
                        return Err(GaiaError::custom_error(format!("Unsupported immediate instruction: {:?}", opcode)));
                    }
                }
            }
            JvmInstruction::WithLocalVar { opcode, index } => {
                bytes.push(opcode.to_byte());
                if *index <= 255 {
                    bytes.push(*index as u8);
                }
                else {
                    return Err(GaiaError::custom_error("Local variable index too large".to_string()));
                }
            }
            JvmInstruction::WithConstantPool { opcode, symbol: _ } => {
                bytes.push(opcode.to_byte());
                // 暂时写入占位符索引
                bytes.extend_from_slice(&1u16.to_be_bytes());
            }
            JvmInstruction::MethodCall { opcode, class_name: _, method_name: _, descriptor: _ } => {
                bytes.push(opcode.to_byte());
                // 暂时写入占位符索引
                bytes.extend_from_slice(&1u16.to_be_bytes());
            }
            JvmInstruction::FieldAccess { opcode, class_name: _, field_name: _, descriptor: _ } => {
                bytes.push(opcode.to_byte());
                // 暂时写入占位符索引
                bytes.extend_from_slice(&1u16.to_be_bytes());
            }
            JvmInstruction::Branch { opcode, target: _ } => {
                bytes.push(opcode.to_byte());
                // 暂时写入占位符偏移
                bytes.extend_from_slice(&0i16.to_be_bytes());
            }
            JvmInstruction::TypeCast { opcode, target_type: _ } => {
                bytes.push(opcode.to_byte());
                // 暂时写入占位符索引
                bytes.extend_from_slice(&1u16.to_be_bytes());
            }
        }
        Ok(())
    }

    /// 写入属性
    fn write_attributes(&mut self, attributes: &[JvmAttribute], _indices: &HashMap<String, u16>) -> Result<()> {
        self.writer.write_u16(attributes.len() as u16)?;

        for attribute in attributes {
            match attribute {
                JvmAttribute::SourceFile { filename } => {
                    // 属性名索引（"SourceFile"，暂时写 0）
                    self.writer.write_u16(0)?;

                    // 属性长度
                    self.writer.write_u32(2)?;

                    // 源文件名索引（暂时写 0）
                    self.writer.write_u16(0)?;
                }
                _ => {
                    // 其他属性暂时跳过
                }
            }
        }

        Ok(())
    }
}
