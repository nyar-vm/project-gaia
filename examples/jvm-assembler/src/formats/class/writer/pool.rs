use crate::program::*;
use std::io::Write;
use super::{ClassWriter, CpEntry};

impl<W: Write> ClassWriter<W> {
    /// 添加 Utf8 常量
    pub fn add_utf8(&mut self, s: String) -> u16 {
        self.add_cp_entry(CpEntry::Utf8(s))
    }

    /// 添加 Class 常量
    pub fn add_class(&mut self, name: String) -> u16 {
        let name_index = self.add_utf8(name);
        self.add_cp_entry(CpEntry::Class(name_index))
    }

    /// 添加 String 常量
    pub fn add_string(&mut self, value: String) -> u16 {
        let utf8_index = self.add_utf8(value);
        self.add_cp_entry(CpEntry::String(utf8_index))
    }

    /// 添加 NameAndType 常量
    pub fn add_name_and_type(&mut self, name: String, descriptor: String) -> u16 {
        let name_index = self.add_utf8(name);
        let descriptor_index = self.add_utf8(descriptor);
        self.add_cp_entry(CpEntry::NameAndType(name_index, descriptor_index))
    }

    /// 添加 Fieldref 常量
    pub fn add_field_ref(&mut self, class_name: String, name: String, descriptor: String) -> u16 {
        let class_index = self.add_class(class_name);
        let name_and_type_index = self.add_name_and_type(name, descriptor);
        self.add_cp_entry(CpEntry::Fieldref(class_index, name_and_type_index))
    }

    /// 添加 Methodref 常量
    pub fn add_method_ref(&mut self, class_name: String, name: String, descriptor: String) -> u16 {
        let class_index = self.add_class(class_name);
        let name_and_type_index = self.add_name_and_type(name, descriptor);
        self.add_cp_entry(CpEntry::Methodref(class_index, name_and_type_index))
    }

    /// 添加 InterfaceMethodref 常量
    pub fn add_interface_method_ref(&mut self, class_name: String, name: String, descriptor: String) -> u16 {
        let class_index = self.add_class(class_name);
        let name_and_type_index = self.add_name_and_type(name, descriptor);
        self.add_cp_entry(CpEntry::InterfaceMethodref(class_index, name_and_type_index))
    }

    /// 添加 Integer 常量
    pub fn add_int(&mut self, val: i32) -> u16 {
        self.add_cp_entry(CpEntry::Integer(val))
    }

    /// 添加 Float 常量
    pub fn add_float(&mut self, val: f32) -> u16 {
        self.add_cp_entry(CpEntry::Float(val.to_bits()))
    }

    /// 添加 Long 常量
    pub fn add_long(&mut self, val: i64) -> u16 {
        self.add_cp_entry(CpEntry::Long(val))
    }

    /// 添加 Double 常量
    pub fn add_double(&mut self, val: f64) -> u16 {
        self.add_cp_entry(CpEntry::Double(val.to_bits()))
    }

    pub fn add_cp_entry(&mut self, entry: CpEntry) -> u16 {
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
}
