use crate::{
    formats::class::view::ClassView,
    program::{JvmAccessFlags, JvmAttribute, JvmConstantPoolEntry, JvmField, JvmMethod, JvmVersion},
};
use byteorder::{BigEndian, ReadBytesExt};
use gaia_types::{BinaryReader, GaiaDiagnostics, GaiaError, Result};
use std::io::{Read, Seek};

/// jvm class lazy reader
///
/// 可以只读取必要的部分
pub struct ClassReader<R: Read + Seek> {
    reader: BinaryReader<R, BigEndian>,
}

impl<R: Read + Seek> ClassReader<R> {
    pub fn new(reader: R) -> Self {
        Self { reader: BinaryReader::new(reader) }
    }

    pub fn finish(self) -> R {
        self.reader.finish()
    }
}

impl<R: Read + Seek> ClassReader<R> {
    pub fn read(&mut self) -> GaiaDiagnostics<ClassView>
    where
        R: Seek,
    {
        match self.read_class_file() {
            Ok(class_view) => GaiaDiagnostics::success(class_view),
            Err(error) => GaiaDiagnostics::failure(error),
        }
    }

    fn read_class_file(&mut self) -> Result<ClassView> {
        // 读取魔数
        let magic = self.reader.read_u32()?;
        if magic != 0xCAFEBABE {
            return Err(GaiaError::custom_error(format!("Invalid magic number: 0x{:X}", magic)));
        }

        // 读取版本号
        let minor_version = self.reader.read_u16()?;
        let major_version = self.reader.read_u16()?;
        let version = JvmVersion { major: major_version, minor: minor_version };

        // 读取常量池
        let constant_pool_count = self.reader.read_u16()?;
        let mut constant_pool = Vec::with_capacity(constant_pool_count as usize);
        constant_pool.push(JvmConstantPoolEntry::Nop); // 常量池索引从 1 开始，0 处放置一个 Nop
        for _i in 1..constant_pool_count {
            let entry = self.read_constant_pool_entry()?;
            constant_pool.push(entry.clone());
            // 对于 Long 和 Double，需要占用两个索引位置
            if matches!(entry, JvmConstantPoolEntry::Long { .. } | JvmConstantPoolEntry::Double { .. }) {
                constant_pool.push(JvmConstantPoolEntry::Nop);
            }
        }

        // 读取访问标志
        let access_flags_value = self.reader.read_u16()?;
        let access_flags = JvmAccessFlags::from_flags(access_flags_value);

        // 读取类名索引
        let this_class_index = self.reader.read_u16()?;
        let this_class = self.resolve_class_name(&constant_pool, this_class_index)?;

        // 读取超类名索引
        let super_class_index = self.reader.read_u16()?;
        let super_class =
            if super_class_index != 0 { Some(self.resolve_class_name(&constant_pool, super_class_index)?) } else { None };

        // 读取接口
        let interfaces_count = self.reader.read_u16()?;
        let mut interfaces = Vec::with_capacity(interfaces_count as usize);
        for _ in 0..interfaces_count {
            let interface_index = self.reader.read_u16()?;
            interfaces.push(self.resolve_class_name(&constant_pool, interface_index)?);
        }

        // 读取字段
        let fields = self.read_fields(&constant_pool)?;

        // 读取方法
        let methods = self.read_methods(&constant_pool)?;

        // 读取属性
        let attributes = self.read_attributes(&constant_pool)?;

        Ok(ClassView {
            magic,
            version,
            constant_pool,
            access_flags,
            this_class,
            super_class,
            interfaces,
            fields,
            methods,
            attributes,
        })
    }

    fn read_constant_pool_entry(&mut self) -> Result<JvmConstantPoolEntry> {
        let tag = self.reader.read_u8()?;
        match tag {
            1 => {
                let length = self.reader.read_u16()?;
                let mut bytes = vec![0; length as usize];
                self.reader.read_exact(&mut bytes)?;
                Ok(JvmConstantPoolEntry::Utf8 {
                    value: match String::from_utf8(bytes) {
                        Ok(s) => s,
                        Err(e) => return Err(GaiaError::custom_error(format!("Failed to decode UTF8 string: {}", e))),
                    },
                })
            }
            3 => Ok(JvmConstantPoolEntry::Integer { value: self.reader.read_i32::<BigEndian>()? }),
            4 => Ok(JvmConstantPoolEntry::Float { value: self.reader.read_f32::<BigEndian>()? }),
            5 => Ok(JvmConstantPoolEntry::Long { value: self.reader.read_i64::<BigEndian>()? }),
            6 => Ok(JvmConstantPoolEntry::Double { value: self.reader.read_f64::<BigEndian>()? }),
            7 => {
                let name_index = self.reader.read_u16()?;
                Ok(JvmConstantPoolEntry::Class { name: name_index.to_string() })
                // 临时存储索引，后续解析
            }
            8 => {
                let string_index = self.reader.read_u16()?;
                Ok(JvmConstantPoolEntry::String { value: string_index.to_string() })
                // 临时存储索引，后续解析
            }
            9 => {
                let class_index = self.reader.read_u16()?;
                let name_and_type_index = self.reader.read_u16()?;
                Ok(JvmConstantPoolEntry::Fieldref {
                    class_name: class_index.to_string(),
                    name: name_and_type_index.to_string(),
                    descriptor: String::new(),
                }) // 临时存储索引，后续解析
            }
            10 => {
                let class_index = self.reader.read_u16()?;
                let name_and_type_index = self.reader.read_u16()?;
                Ok(JvmConstantPoolEntry::Methodref {
                    class_name: class_index.to_string(),
                    name: name_and_type_index.to_string(),
                    descriptor: String::new(),
                }) // 临时存储索引，后续解析
            }
            11 => {
                let class_index = self.reader.read_u16()?;
                let name_and_type_index = self.reader.read_u16()?;
                Ok(JvmConstantPoolEntry::InterfaceMethodref {
                    class_name: class_index.to_string(),
                    name: name_and_type_index.to_string(),
                    descriptor: String::new(),
                }) // 临时存储索引，后续解析
            }
            12 => {
                let name_index = self.reader.read_u16()?;
                let descriptor_index = self.reader.read_u16()?;
                Ok(JvmConstantPoolEntry::NameAndType { name: name_index.to_string(), descriptor: descriptor_index.to_string() })
                // 临时存储索引，后续解析
            }
            _ => Err(GaiaError::custom_error(format!("Unsupported constant pool tag: {}", tag))),
        }
    }

    fn resolve_class_name(&self, constant_pool: &[JvmConstantPoolEntry], index: u16) -> Result<String> {
        if let Some(JvmConstantPoolEntry::Class { name }) = constant_pool.get(index as usize) {
            if let Some(JvmConstantPoolEntry::Utf8 { value }) = constant_pool.get(match name.parse::<u16>() {
                Ok(index) => index as usize,
                Err(e) => return Err(GaiaError::custom_error(format!("Failed to parse class name index: {}", e))),
            }) {
                Ok(value.clone())
            }
            else {
                Err(GaiaError::custom_error(format!("Invalid UTF8 index for class name: {}", name)))
            }
        }
        else {
            Err(GaiaError::custom_error(format!("Invalid class index: {}", index)))
        }
    }

    fn read_fields(&mut self, constant_pool: &[JvmConstantPoolEntry]) -> Result<Vec<JvmField>> {
        let fields_count = self.reader.read_u16()?;
        let mut fields = Vec::with_capacity(fields_count as usize);
        for _ in 0..fields_count {
            let access_flags_value = self.reader.read_u16()?;
            let access_flags = JvmAccessFlags::from_flags(access_flags_value);
            let name_index = self.reader.read_u16()?;
            let descriptor_index = self.reader.read_u16()?;
            let attributes_count = self.reader.read_u16()?;
            let mut attributes = Vec::with_capacity(attributes_count as usize);
            for _ in 0..attributes_count {
                attributes.push(self.read_attribute(constant_pool)?);
            }

            let name = self.resolve_utf8_constant(constant_pool, name_index)?;
            let descriptor = self.resolve_utf8_constant(constant_pool, descriptor_index)?;

            fields.push(JvmField { access_flags, name, descriptor, constant_value: None, attributes });
        }
        Ok(fields)
    }

    fn read_methods(&mut self, constant_pool: &[JvmConstantPoolEntry]) -> Result<Vec<JvmMethod>> {
        let methods_count = self.reader.read_u16()?;
        let mut methods = Vec::with_capacity(methods_count as usize);
        for _ in 0..methods_count {
            let access_flags_value = self.reader.read_u16()?;
            let access_flags = JvmAccessFlags::from_flags(access_flags_value);
            let name_index = self.reader.read_u16()?;
            let descriptor_index = self.reader.read_u16()?;
            let attributes_count = self.reader.read_u16()?;
            let mut attributes = Vec::with_capacity(attributes_count as usize);
            for _ in 0..attributes_count {
                attributes.push(self.read_attribute(constant_pool)?);
            }

            let name = self.resolve_utf8_constant(constant_pool, name_index)?;
            let descriptor = self.resolve_utf8_constant(constant_pool, descriptor_index)?;

            // TODO: 解析 Code 属性中的指令
            methods.push(JvmMethod {
                access_flags,
                name,
                descriptor,
                attributes,
                max_stack: 0,
                max_locals: 0,
                instructions: Vec::new(),
                exception_table: Vec::new(),
            });
        }
        Ok(methods)
    }

    fn read_attributes(&mut self, constant_pool: &[JvmConstantPoolEntry]) -> Result<Vec<JvmAttribute>> {
        let attributes_count = self.reader.read_u16()?;
        let mut attributes = Vec::with_capacity(attributes_count as usize);
        for _ in 0..attributes_count {
            attributes.push(self.read_attribute(constant_pool)?);
        }
        Ok(attributes)
    }

    fn read_attribute(&mut self, constant_pool: &[JvmConstantPoolEntry]) -> Result<JvmAttribute> {
        let attribute_name_index = self.reader.read_u16()?;
        let attribute_length = self.reader.read_u32()?;
        let attribute_name = self.resolve_utf8_constant(constant_pool, attribute_name_index)?;

        match attribute_name.as_str() {
            "SourceFile" => {
                let source_file_name_index = self.reader.read_u16()?;
                let source_file_name = self.resolve_utf8_constant(constant_pool, source_file_name_index)?;
                Ok(JvmAttribute::SourceFile { filename: source_file_name })
            }
            _ => {
                // 对于不支持的属性，跳过其内容
                self.reader.seek_relative(attribute_length as i64)?;
                Ok(JvmAttribute::Unknown { name: attribute_name, data: Vec::new() })
            }
        }
    }

    fn resolve_utf8_constant(&self, constant_pool: &[JvmConstantPoolEntry], index: u16) -> Result<String> {
        if let Some(JvmConstantPoolEntry::Utf8 { value }) = constant_pool.get(index as usize) {
            Ok(value.clone())
        }
        else {
            Err(GaiaError::custom_error(format!("Invalid UTF8 constant pool index: {}", index)))
        }
    }
}
