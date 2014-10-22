use crate::{ConstantPoolEntry, JvmClass, JvmField, JvmInstruction, JvmMethod, JvmWriter};
use byteorder::{BigEndian, WriteBytesExt};
use std::io::{Cursor, Write};

impl JvmWriter {
    /// 创建新的写入器
    pub fn new() -> Self {
        Self { buffer: Cursor::new(Vec::new()) }
    }

    /// 写入 JVM 类文件
    pub fn write_class(&mut self, class: &JvmClass) -> gaia_types::Result<()> {
        // 写入魔数
        self.buffer.write_u32::<BigEndian>(class.magic)?;

        // 写入版本号
        self.buffer.write_u16::<BigEndian>(class.minor_version)?;
        self.buffer.write_u16::<BigEndian>(class.major_version)?;

        // 写入常量池
        self.write_constant_pool(&class.constant_pool)?;

        // 写入访问标志
        self.buffer.write_u16::<BigEndian>(class.access_flags)?;

        // 写入类索引
        self.buffer.write_u16::<BigEndian>(class.this_class)?;
        self.buffer.write_u16::<BigEndian>(class.super_class)?;

        // 写入接口
        self.write_interfaces(&class.interfaces)?;

        // 写入字段
        self.write_fields(&class.fields)?;

        // 写入方法
        self.write_methods(&class.methods)?;

        // 写入类属性（暂时为空）
        self.buffer.write_u16::<BigEndian>(0)?; // attributes_count

        Ok(())
    }

    /// 写入常量池
    fn write_constant_pool(&mut self, pool: &[ConstantPoolEntry]) -> gaia_types::Result<()> {
        // 常量池计数（包含占位符）
        self.buffer.write_u16::<BigEndian>((pool.len() + 1) as u16)?;

        for entry in pool {
            self.write_constant_pool_entry(entry)?;
        }

        Ok(())
    }

    /// 写入常量池条目
    fn write_constant_pool_entry(&mut self, entry: &ConstantPoolEntry) -> gaia_types::Result<()> {
        match entry {
            ConstantPoolEntry::Utf8(s) => {
                self.buffer.write_u8(1)?; // CONSTANT_Utf8
                self.buffer.write_u16::<BigEndian>(s.len() as u16)?;
                self.buffer.write_all(s.as_bytes())?;
            }
            ConstantPoolEntry::Integer(value) => {
                self.buffer.write_u8(3)?; // CONSTANT_Integer
                self.buffer.write_i32::<BigEndian>(*value)?;
            }
            ConstantPoolEntry::Float(value) => {
                self.buffer.write_u8(4)?; // CONSTANT_Float
                self.buffer.write_f32::<BigEndian>(*value)?;
            }
            ConstantPoolEntry::Long(value) => {
                self.buffer.write_u8(5)?; // CONSTANT_Long
                self.buffer.write_i64::<BigEndian>(*value)?;
            }
            ConstantPoolEntry::Double(value) => {
                self.buffer.write_u8(6)?; // CONSTANT_Double
                self.buffer.write_f64::<BigEndian>(*value)?;
            }
            ConstantPoolEntry::Class(name_index) => {
                self.buffer.write_u8(7)?; // CONSTANT_Class
                self.buffer.write_u16::<BigEndian>(*name_index)?;
            }
            ConstantPoolEntry::String(string_index) => {
                self.buffer.write_u8(8)?; // CONSTANT_String
                self.buffer.write_u16::<BigEndian>(*string_index)?;
            }
            ConstantPoolEntry::Fieldref { class_index, name_and_type_index } => {
                self.buffer.write_u8(9)?; // CONSTANT_Fieldref
                self.buffer.write_u16::<BigEndian>(*class_index)?;
                self.buffer.write_u16::<BigEndian>(*name_and_type_index)?;
            }
            ConstantPoolEntry::Methodref { class_index, name_and_type_index } => {
                self.buffer.write_u8(10)?; // CONSTANT_Methodref
                self.buffer.write_u16::<BigEndian>(*class_index)?;
                self.buffer.write_u16::<BigEndian>(*name_and_type_index)?;
            }
            ConstantPoolEntry::InterfaceMethodref { class_index, name_and_type_index } => {
                self.buffer.write_u8(11)?; // CONSTANT_InterfaceMethodref
                self.buffer.write_u16::<BigEndian>(*class_index)?;
                self.buffer.write_u16::<BigEndian>(*name_and_type_index)?;
            }
            ConstantPoolEntry::NameAndType { name_index, descriptor_index } => {
                self.buffer.write_u8(12)?; // CONSTANT_NameAndType
                self.buffer.write_u16::<BigEndian>(*name_index)?;
                self.buffer.write_u16::<BigEndian>(*descriptor_index)?;
            }
        }
        Ok(())
    }

    /// 写入接口列表
    fn write_interfaces(&mut self, interfaces: &[u16]) -> gaia_types::Result<()> {
        self.buffer.write_u16::<BigEndian>(interfaces.len() as u16)?;
        for interface in interfaces {
            self.buffer.write_u16::<BigEndian>(*interface)?;
        }
        Ok(())
    }

    /// 写入字段列表
    fn write_fields(&mut self, fields: &[JvmField]) -> gaia_types::Result<()> {
        self.buffer.write_u16::<BigEndian>(fields.len() as u16)?;
        for field in fields {
            self.write_field(field)?;
        }
        Ok(())
    }

    /// 写入单个字段
    fn write_field(&mut self, field: &JvmField) -> gaia_types::Result<()> {
        self.buffer.write_u16::<BigEndian>(field.access_flags)?;
        self.buffer.write_u16::<BigEndian>(field.name_index)?;
        self.buffer.write_u16::<BigEndian>(field.descriptor_index)?;
        self.buffer.write_u16::<BigEndian>(0)?; // attributes_count
        Ok(())
    }

    /// 写入方法列表
    fn write_methods(&mut self, methods: &[JvmMethod]) -> gaia_types::Result<()> {
        self.buffer.write_u16::<BigEndian>(methods.len() as u16)?;
        for method in methods {
            self.write_method(method)?;
        }
        Ok(())
    }

    /// 写入单个方法
    fn write_method(&mut self, method: &JvmMethod) -> gaia_types::Result<()> {
        self.buffer.write_u16::<BigEndian>(method.access_flags)?;
        self.buffer.write_u16::<BigEndian>(method.name_index)?;
        self.buffer.write_u16::<BigEndian>(method.descriptor_index)?;

        // 写入方法属性（Code 属性）
        if !method.instructions.is_empty() {
            self.buffer.write_u16::<BigEndian>(1)?; // attributes_count
            self.write_code_attribute(method)?;
        }
        else {
            self.buffer.write_u16::<BigEndian>(0)?; // attributes_count
        }

        Ok(())
    }

    /// 写入 Code 属性
    fn write_code_attribute(&mut self, method: &JvmMethod) -> gaia_types::Result<()> {
        // 属性名索引（假设为 1，指向 "Code" 字符串）
        self.buffer.write_u16::<BigEndian>(1)?;

        // 计算属性长度
        let code_length = method.code_length();
        let attribute_length = 2 + 2 + 4 + code_length + 2 + 2; // max_stack + max_locals + code_length + code + exception_table_length + attributes_count
        self.buffer.write_u32::<BigEndian>(attribute_length)?;

        // 写入栈和局部变量信息
        self.buffer.write_u16::<BigEndian>(method.max_stack)?;
        self.buffer.write_u16::<BigEndian>(method.max_locals)?;

        // 写入代码长度和代码
        self.buffer.write_u32::<BigEndian>(code_length)?;
        for instruction in &method.instructions {
            self.write_instruction(instruction)?;
        }

        // 写入异常表（暂时为空）
        self.buffer.write_u16::<BigEndian>(0)?; // exception_table_length

        // 写入代码属性的属性（暂时为空）
        self.buffer.write_u16::<BigEndian>(0)?; // attributes_count

        Ok(())
    }

    /// 写入单个指令
    pub fn write_instruction(&mut self, instruction: &JvmInstruction) -> gaia_types::Result<()> {
        self.buffer.write_u8(instruction.opcode)?;
        self.buffer.write_all(&instruction.operands)?;
        Ok(())
    }

    /// 获取生成的字节码
    pub fn into_bytes(self) -> Vec<u8> {
        self.buffer.into_inner()
    }

    /// 获取当前字节码的引用
    pub fn as_bytes(&self) -> &[u8] {
        self.buffer.get_ref()
    }
}

impl Default for JvmWriter {
    fn default() -> Self {
        Self::new()
    }
}
