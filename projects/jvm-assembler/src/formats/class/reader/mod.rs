use crate::{
    formats::class::{view::ClassInfo, ClassReadConfig},
    program::JvmProgram,
};
use byteorder::BigEndian;
use gaia_types::{BinaryReader, GaiaDiagnostics, GaiaError};
use std::{
    cell::{OnceCell, RefCell},
    io::{Read, Seek},
};

/// jvm class lazy reader
///
/// 可以只读取必要的部分
pub struct ClassReader<'config, R: Read + Seek> {
    _config: &'config ClassReadConfig,
    reader: RefCell<BinaryReader<R, BigEndian>>,
    /// 缓存的程序
    program: OnceCell<JvmProgram>,
    /// 缓存的类信息
    info: OnceCell<ClassInfo>,
}

impl ClassReadConfig {
    pub fn as_reader<R: Read + Seek>(&self, reader: R) -> ClassReader<'_, R> {
        ClassReader::new(reader, self)
    }
}

impl<'config, R: Read + Seek> ClassReader<'config, R> {
    pub fn new(reader: R, config: &'config ClassReadConfig) -> Self {
        Self { _config: config, reader: RefCell::new(BinaryReader::new(reader)), program: Default::default(), info: Default::default() }
    }

    pub fn get_program(&self) -> Result<&JvmProgram, GaiaError> {
        self.program.get_or_try_init(|| self.read_program())
    }
    pub fn get_info(&self) -> Result<&ClassInfo, GaiaError> {
        self.info.get_or_try_init(|| self.read_view())
    }
}

impl<'config, R: Read + Seek> ClassReader<'config, R> {
    pub fn read(mut self) -> GaiaDiagnostics<JvmProgram> {
        match self.get_program() {
            Ok(_) => {
                let errors = self.reader.borrow_mut().take_errors();
                GaiaDiagnostics { result: self.program.take().ok_or(GaiaError::unreachable()), diagnostics: errors }
            }
            Err(e) => {
                let errors = self.reader.borrow_mut().take_errors();
                GaiaDiagnostics { result: Err(e), diagnostics: errors }
            }
        }
    }
    fn read_program(&self) -> Result<JvmProgram, GaiaError> {
        let mut reader = self.reader.borrow_mut();

        // 从文件开头开始读取
        reader.seek(std::io::SeekFrom::Start(0))?;

        // 读取类文件头
        let (minor_version, major_version) = self.read_class_header(&mut reader)?;
        
        // 读取常量池
        let constant_pool_entries = self.read_constant_pool(&mut reader)?;

        // 读取类信息
        let (access_flags, this_class, super_class) = self.read_class_info(&mut reader)?;
        
        // 解析类名和超类名
        let class_name = self.resolve_class_name(&constant_pool_entries, this_class);
        let super_class_name = if super_class == 0 {
            None
        } else {
            Some(self.resolve_class_name(&constant_pool_entries, super_class))
        };

        // 跳过接口
        self.skip_interfaces(&mut reader)?;

        // 读取字段
        let fields = self.read_fields(&mut reader, &constant_pool_entries)?;

        // 读取方法
        let methods = self.read_methods(&mut reader, &constant_pool_entries)?;

        // 创建常量池
        let mut constant_pool = crate::program::JvmConstantPool::new();
        for entry in constant_pool_entries.iter() {
            if let Some(value) = entry {
                let cp_entry = crate::program::JvmConstantPoolEntry::Utf8 {
                    value: value.clone(),
                };
                constant_pool.entries.push(cp_entry);
            } else {
                constant_pool.entries.push(crate::program::JvmConstantPoolEntry::Nop);
            }
        }

        // 创建 JvmProgram
        let mut program = crate::program::JvmProgram::new(class_name);
        program.version = crate::program::JvmVersion { major: major_version, minor: minor_version };
        program.access_flags = crate::program::JvmAccessFlags::from_flags(access_flags);
        program.super_class = super_class_name;
        program.fields = fields;
        program.methods = methods;
        program.constant_pool = constant_pool;

        Ok(program)
    }

    /// 读取并验证 class 文件头
    fn read_class_header(&self, reader: &mut gaia_types::BinaryReader<impl Read + Seek, BigEndian>) -> Result<(u16, u16), GaiaError> {
        let magic = reader.read_u32()?;
        if magic != 0xCAFEBABE {
            return Err(GaiaError::invalid_data("Invalid class file magic number"));
        }

        let minor_version = reader.read_u16()?;
        let major_version = reader.read_u16()?;
        
        Ok((minor_version, major_version))
    }

    /// 读取常量池
    fn read_constant_pool(&self, reader: &mut gaia_types::BinaryReader<impl Read + Seek, BigEndian>) -> Result<Vec<Option<String>>, GaiaError> {
        let constant_pool_count = reader.read_u16()?;
        let mut constant_pool_entries: Vec<Option<String>> = vec![None; constant_pool_count as usize];

        // 解析常量池，特别关注 UTF8 和 Class 条目
        let mut i = 1;
        while i < constant_pool_count {
            let tag = reader.read_u8()?;
            match tag {
                0 => {
                    // 占位符条目（通常在 Long/Double 之后）
                    // 不需要读取任何数据，只是跳过
                }
                1 => {
                    // UTF8
                    let length = reader.read_u16()?;
                    let mut bytes = vec![0u8; length as usize];
                    reader.read_exact(&mut bytes)?;
                    let utf8_string = String::from_utf8_lossy(&bytes).to_string();
                    constant_pool_entries[i as usize] = Some(utf8_string);
                }
                3 => { reader.read_u32()?; }, // Integer
                4 => { reader.read_u32()?; }, // Float
                5 => { 
                    // Long - 占用两个常量池位置
                    reader.read_u64()?; 
                    i += 1; // 跳过下一个位置
                }, 
                6 => { 
                    // Double - 占用两个常量池位置
                    reader.read_u64()?; 
                    i += 1; // 跳过下一个位置
                }, 
                7 => {
                    // Class - 存储对 UTF8 条目的引用
                    let name_index = reader.read_u16()?;
                    constant_pool_entries[i as usize] = Some(format!("CLASS_REF:{}", name_index));
                }
                8 => { reader.read_u16()?; }, // String
                9 | 10 | 11 => {
                    // Fieldref, Methodref, InterfaceMethodref
                    reader.read_u16()?;
                    reader.read_u16()?;
                }
                12 => {
                    // NameAndType
                    reader.read_u16()?;
                    reader.read_u16()?;
                }
                15 => {
                    // MethodHandle
                    reader.read_u8()?;
                    reader.read_u16()?;
                }
                16 => { reader.read_u16()?; }, // MethodType
                17 | 18 => {
                    // Dynamic, InvokeDynamic
                    reader.read_u16()?;
                    reader.read_u16()?;
                }
                19 | 20 => { reader.read_u16()?; }, // Module, Package
                _ => {
                    return Err(GaiaError::invalid_data(&format!("Unknown constant pool tag: {}", tag)));
                }
            };
            i += 1;
        }

        Ok(constant_pool_entries)
    }

    /// 读取类基本信息
    fn read_class_info(&self, reader: &mut gaia_types::BinaryReader<impl Read + Seek, BigEndian>) -> Result<(u16, u16, u16), GaiaError> {
        let access_flags = reader.read_u16()?;
        let this_class = reader.read_u16()?;
        let super_class = reader.read_u16()?;
        
        Ok((access_flags, this_class, super_class))
    }

    /// 解析类名
    fn resolve_class_name(&self, constant_pool_entries: &[Option<String>], this_class: u16) -> String {
        if let Some(Some(class_ref)) = constant_pool_entries.get(this_class as usize) {
            if let Some(class_ref_str) = class_ref.strip_prefix("CLASS_REF:") {
                if let Ok(name_index) = class_ref_str.parse::<u16>() {
                    if let Some(Some(name)) = constant_pool_entries.get(name_index as usize) {
                        return name.clone();
                    }
                }
            }
        }
        "UnknownClass".to_string()
    }

    /// 跳过接口
    fn skip_interfaces(&self, reader: &mut gaia_types::BinaryReader<impl Read + Seek, BigEndian>) -> Result<(), GaiaError> {
        let interfaces_count = reader.read_u16()?;
        for _ in 0..interfaces_count {
            reader.read_u16()?;
        }
        Ok(())
    }

    /// 读取字段
    fn read_fields(&self, reader: &mut gaia_types::BinaryReader<impl Read + Seek, BigEndian>, constant_pool_entries: &[Option<String>]) -> Result<Vec<crate::program::JvmField>, GaiaError> {
        let fields_count = reader.read_u16()?;
        let mut fields = Vec::new();
        
        for _ in 0..fields_count {
            let field_access_flags = reader.read_u16()?;
            let name_index = reader.read_u16()?;
            let descriptor_index = reader.read_u16()?;
            
            // 获取字段名和描述符
            let field_name = self.get_string_from_pool(constant_pool_entries, name_index, "UnknownField");
            let field_descriptor = self.get_string_from_pool(constant_pool_entries, descriptor_index, "UnknownDescriptor");
            
            let mut field = crate::program::JvmField::new(field_name, field_descriptor);
            field.access_flags = crate::program::JvmAccessFlags::from_flags(field_access_flags);
            
            // 跳过字段属性
            self.skip_attributes(reader)?;
            
            fields.push(field);
        }
        
        Ok(fields)
    }

    /// 读取方法
    fn read_methods(&self, reader: &mut gaia_types::BinaryReader<impl Read + Seek, BigEndian>, constant_pool_entries: &[Option<String>]) -> Result<Vec<crate::program::JvmMethod>, GaiaError> {
        let methods_count = reader.read_u16()?;
        let mut methods = Vec::new();
        
        for _ in 0..methods_count {
            let method_access_flags = reader.read_u16()?;
            let name_index = reader.read_u16()?;
            let descriptor_index = reader.read_u16()?;
            
            // 获取方法名和描述符
            let method_name = self.get_string_from_pool(constant_pool_entries, name_index, "UnknownMethod");
            let method_descriptor = self.get_string_from_pool(constant_pool_entries, descriptor_index, "UnknownDescriptor");
            
            let mut method = crate::program::JvmMethod::new(method_name, method_descriptor);
            method.access_flags = crate::program::JvmAccessFlags::from_flags(method_access_flags);
            
            // 跳过方法属性
            self.skip_attributes(reader)?;
            
            methods.push(method);
        }
        
        Ok(methods)
    }

    /// 从常量池获取字符串
    fn get_string_from_pool(&self, constant_pool_entries: &[Option<String>], index: u16, default: &str) -> String {
        constant_pool_entries.get(index as usize)
            .and_then(|opt| opt.as_ref())
            .unwrap_or(&default.to_string())
            .clone()
    }

    /// 跳过属性
    fn skip_attributes(&self, reader: &mut gaia_types::BinaryReader<impl Read + Seek, BigEndian>) -> Result<(), GaiaError> {
        let attributes_count = reader.read_u16()?;
        for _ in 0..attributes_count {
            reader.read_u16()?; // attribute_name_index
            let attribute_length = reader.read_u32()?;
            let mut attribute_data = vec![0u8; attribute_length as usize];
            reader.read_exact(&mut attribute_data)?;
        }
        Ok(())
    }

    fn read_view(&self) -> Result<ClassInfo, GaiaError> {
        let mut reader = self.reader.borrow_mut();

        // 重新定位到文件开头
        reader.seek(std::io::SeekFrom::Start(0))?;

        // 读取并验证 class 文件头
        let (minor_version, major_version) = self.read_class_header(&mut reader)?;

        // 读取常量池
        let constant_pool_entries = self.read_constant_pool(&mut reader)?;

        // 读取类基本信息
        let (access_flags, this_class, super_class) = self.read_class_info(&mut reader)?;

        // 解析类名和超类名
        let class_name = self.resolve_class_name(&constant_pool_entries, this_class);
        let super_class_name = if super_class == 0 {
            None
        } else {
            Some(self.resolve_class_name(&constant_pool_entries, super_class))
        };

        Ok(ClassInfo {
            magic: 0xCAFEBABE,
            version: crate::program::JvmVersion { major: major_version, minor: minor_version },
            access_flags: crate::program::JvmAccessFlags::from_flags(access_flags),
            this_class: class_name,
            super_class: super_class_name,
        })
    }
}
