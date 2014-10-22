//! JVM 字节码读取器

use crate::helpers::*;
use byteorder::{BigEndian, ReadBytesExt};
use gaia_types::{reader::BinaryReader, GaiaError, Result};
use std::io::{Cursor, Read, Seek};

mod jvm_reader;

/// JVM 字节码读取器（传统实现）
pub struct JvmReader {
    /// 输入缓冲区
    buffer: Cursor<Vec<u8>>,
}

/// JVM /// 基于BinaryReader的JVM字节码读取器（泛型实现）
///
/// 提供更广泛的读取支持，可以处理任何实现了Read和ReadBytesExt的类型
pub struct JvmReader2<R> {
    reader: BinaryReader<R, BigEndian>,
}

/// 便捷函数：从字节数组读取 JVM 类
pub fn read_class_from_bytes(data: Vec<u8>) -> Result<JvmClass> {
    let mut reader = JvmReader::new(data);
    reader.read_class()
}

/// 便捷函数：从字节数组读取指令列表
pub fn read_instructions_from_bytes(data: Vec<u8>) -> Result<Vec<JvmInstruction>> {
    let data_len = data.len();
    let mut reader = JvmReader::new(data);
    reader.read_instructions(data_len)
}

impl<R: ReadBytesExt> JvmReader2<R> {
    /// 创建新的JVM读取器2
    ///
    /// # 参数
    /// * `viewer` - 实现了ReadBytesExt trait的读取器
    ///
    /// # 示例
    /// ```rust
    /// use byteorder::ReadBytesExt;
    /// use jvm_assembler::viewer::JvmReader2;
    /// use std::io::Cursor;
    ///
    /// let data = vec![0xCA, 0xFE, 0xBA, 0xBE]; // JVM魔数
    /// let cursor = Cursor::new(data);
    /// let viewer = JvmReader2::new(cursor);
    /// ```
    pub fn new(reader: R) -> Self {
        Self { reader: BinaryReader::new(reader) }
    }

    /// 读取JVM类文件
    ///
    /// # 返回值
    /// 返回解析后的JvmClass结构体
    ///
    /// # 错误
    /// 当数据格式无效时返回GaiaError
    pub fn read_class(&mut self) -> Result<JvmClass>
    where
        R: Seek,
    {
        let mut class = JvmClass::default();

        // 读取魔数
        class.magic = self.reader.read_u32()?;
        if class.magic != 0xCAFEBABE {
            return Err(GaiaError::invalid_data("Invalid JVM class file magic number"));
        }

        // 读取版本号
        class.minor_version = self.reader.read_u16()?;
        class.major_version = self.reader.read_u16()?;

        // 读取常量池
        class.constant_pool = self.read_constant_pool()?;

        // 读取访问标志
        class.access_flags = self.reader.read_u16()?;

        // 读取类索引
        class.this_class = self.reader.read_u16()?;
        class.super_class = self.reader.read_u16()?;

        // 读取接口
        class.interfaces = self.read_interfaces()?;

        // 读取字段
        class.fields = self.read_fields()?;

        // 读取方法
        class.methods = self.read_methods()?;

        // 跳过类属性
        let attributes_count = self.reader.read_u16()?;
        for _ in 0..attributes_count {
            self.skip_attribute()?;
        }

        Ok(class)
    }

    /// 读取常量池
    fn read_constant_pool(&mut self) -> Result<Vec<ConstantPoolEntry>> {
        let count = self.reader.read_u16()? as usize;
        let mut pool = Vec::with_capacity(count.saturating_sub(1));

        // 常量池索引从1开始，所以读取count-1个条目
        for _ in 1..count {
            let entry = self.read_constant_pool_entry()?;
            pool.push(entry);
        }

        Ok(pool)
    }

    /// 读取常量池条目
    fn read_constant_pool_entry(&mut self) -> Result<ConstantPoolEntry> {
        let tag = self.reader.read_u8()?;

        match tag {
            1 => {
                // CONSTANT_Utf8
                let length = self.reader.read_u16()? as usize;
                let bytes = self.reader.read_bytes(length)?;
                let string = String::from_utf8(bytes).map_err(|_| GaiaError::invalid_data("Invalid UTF-8 in constant pool"))?;
                Ok(ConstantPoolEntry::Utf8(string))
            }
            3 => {
                // CONSTANT_Integer
                let value = self.reader.read_i32::<BigEndian>()?;
                Ok(ConstantPoolEntry::Integer(value))
            }
            4 => {
                // CONSTANT_Float
                let value = self.reader.read_f32::<BigEndian>()?;
                Ok(ConstantPoolEntry::Float(value))
            }
            5 => {
                // CONSTANT_Long
                let value = self.reader.read_i64::<BigEndian>()?;
                // Long 和 Double 占用两个常量池槽位，插入一个占位条目
                Ok(ConstantPoolEntry::Long(value))
            }
            6 => {
                // CONSTANT_Double
                let value = self.reader.read_f64::<BigEndian>()?;
                // Long 和 Double 占用两个常量池槽位，插入一个占位条目
                Ok(ConstantPoolEntry::Double(value))
            }
            7 => {
                // CONSTANT_Class
                let name_index = self.reader.read_u16()?;
                Ok(ConstantPoolEntry::Class(name_index))
            }
            8 => {
                // CONSTANT_String
                let string_index = self.reader.read_u16()?;
                Ok(ConstantPoolEntry::String(string_index))
            }
            9 => {
                // CONSTANT_Fieldref
                let class_index = self.reader.read_u16()?;
                let name_and_type_index = self.reader.read_u16()?;
                Ok(ConstantPoolEntry::Fieldref { class_index, name_and_type_index })
            }
            10 => {
                // CONSTANT_Methodref
                let class_index = self.reader.read_u16()?;
                let name_and_type_index = self.reader.read_u16()?;
                Ok(ConstantPoolEntry::Methodref { class_index, name_and_type_index })
            }
            11 => {
                // CONSTANT_InterfaceMethodref
                let class_index = self.reader.read_u16()?;
                let name_and_type_index = self.reader.read_u16()?;
                Ok(ConstantPoolEntry::InterfaceMethodref { class_index, name_and_type_index })
            }
            12 => {
                // CONSTANT_NameAndType
                let name_index = self.reader.read_u16()?;
                let descriptor_index = self.reader.read_u16()?;
                Ok(ConstantPoolEntry::NameAndType { name_index, descriptor_index })
            }
            _ => Err(GaiaError::invalid_data(&format!("Unknown constant pool tag: {}", tag))),
        }
    }

    /// 读取接口列表
    fn read_interfaces(&mut self) -> Result<Vec<u16>> {
        let count = self.reader.read_u16()? as usize;
        let mut interfaces = Vec::with_capacity(count);

        for _ in 0..count {
            interfaces.push(self.reader.read_u16()?);
        }

        Ok(interfaces)
    }

    /// 读取字段列表
    fn read_fields(&mut self) -> Result<Vec<JvmField>>
    where
        R: Seek,
    {
        let count = self.reader.read_u16()? as usize;
        let mut fields = Vec::with_capacity(count);

        for _ in 0..count {
            fields.push(self.read_field()?);
        }

        Ok(fields)
    }

    /// 读取单个字段
    fn read_field(&mut self) -> Result<JvmField>
    where
        R: Seek,
    {
        let access_flags = self.reader.read_u16()?;
        let name_index = self.reader.read_u16()?;
        let descriptor_index = self.reader.read_u16()?;

        // 跳过字段属性
        let attributes_count = self.reader.read_u16()?;
        for _ in 0..attributes_count {
            self.skip_attribute()?;
        }

        Ok(JvmField { access_flags, name_index, descriptor_index })
    }

    /// 读取方法列表
    fn read_methods(&mut self) -> Result<Vec<JvmMethod>>
    where
        R: Seek,
    {
        let count = self.reader.read_u16()? as usize;
        let mut methods = Vec::with_capacity(count);

        for _ in 0..count {
            methods.push(self.read_method()?);
        }

        Ok(methods)
    }

    /// 读取单个方法
    fn read_method(&mut self) -> Result<JvmMethod>
    where
        R: Seek,
    {
        let access_flags = self.reader.read_u16()?;
        let name_index = self.reader.read_u16()?;
        let descriptor_index = self.reader.read_u16()?;

        let mut method = JvmMethod::new(name_index, descriptor_index);
        method.access_flags = access_flags;

        // 读取方法属性
        let attributes_count = self.reader.read_u16()?;
        for _ in 0..attributes_count {
            if let Some((max_stack, max_locals, instructions)) = self.try_read_code_attribute()? {
                method.max_stack = max_stack;
                method.max_locals = max_locals;
                method.instructions = instructions;
            }
            else {
                self.skip_attribute()?;
            }
        }

        Ok(method)
    }

    /// 尝试读取 Code 属性
    fn try_read_code_attribute(&mut self) -> Result<Option<(u16, u16, Vec<JvmInstruction>)>>
    where
        R: Seek,
    {
        let _attribute_name_index = self.reader.read_u16()?;
        let _attribute_length = self.reader.read_u32()?;

        // 简化处理：假设这是 Code 属性
        let max_stack = self.reader.read_u16()?;
        let max_locals = self.reader.read_u16()?;
        let code_length = self.reader.read_u32()? as usize;

        // 读取指令
        let instructions = self.read_instructions(code_length)?;

        // 跳过异常表
        let exception_table_length = self.reader.read_u16()?;
        for _ in 0..exception_table_length {
            self.reader.read_u64()?; // 跳过异常表条目
        }

        // 跳过代码属性的属性
        let code_attributes_count = self.reader.read_u16()?;
        for _ in 0..code_attributes_count {
            self.skip_attribute()?;
        }

        Ok(Some((max_stack, max_locals, instructions)))
    }

    /// 读取指令列表
    pub fn read_instructions(&mut self, code_length: usize) -> Result<Vec<JvmInstruction>> {
        let mut instructions = Vec::new();
        let mut bytes_read = 0;

        while bytes_read < code_length {
            let opcode = self.reader.read_u8()?;
            bytes_read += 1;

            let operands = self.read_instruction_operands(opcode, code_length - bytes_read)?;
            bytes_read += operands.len();

            instructions.push(JvmInstruction::new(opcode, operands));
        }

        Ok(instructions)
    }

    /// 读取指令操作数
    fn read_instruction_operands(&mut self, opcode: u8, remaining_bytes: usize) -> Result<Vec<u8>> {
        use crate::helpers::opcodes::*;

        let operand_count = match opcode {
            // 无操作数指令
            NOP
            | ACONST_NULL
            | ICONST_M1..=ICONST_5
            | LCONST_0
            | LCONST_1
            | FCONST_0..=FCONST_2
            | DCONST_0
            | DCONST_1
            | ILOAD_0..=ILOAD_3
            | LLOAD_0..=LLOAD_3
            | FLOAD_0..=FLOAD_3
            | DLOAD_0..=DLOAD_3
            | ALOAD_0..=ALOAD_3
            | ISTORE_0..=ISTORE_3
            | LSTORE_0..=LSTORE_3
            | FSTORE_0..=FSTORE_3
            | DSTORE_0..=DSTORE_3
            | ASTORE_0..=ASTORE_3
            | POP
            | POP2
            | DUP
            | DUP_X1
            | DUP_X2
            | DUP2
            | DUP2_X1
            | DUP2_X2
            | SWAP
            | IADD
            | LADD
            | FADD
            | DADD
            | ISUB
            | LSUB
            | FSUB
            | DSUB
            | IMUL
            | LMUL
            | FMUL
            | DMUL
            | IDIV
            | LDIV
            | FDIV
            | DDIV
            | IRETURN
            | LRETURN
            | FRETURN
            | DRETURN
            | ARETURN
            | RETURN => 0,

            // 单字节操作数指令
            BIPUSH | LDC => 1,

            // 双字节操作数指令
            SIPUSH | LDC_W | LDC2_W | INVOKEVIRTUAL | INVOKESPECIAL | INVOKESTATIC => 2,

            // 特殊指令
            INVOKEINTERFACE => 4, // invokeinterface 有 4 个字节的操作数

            // 其他指令默认无操作数
            _ => 0,
        };

        let actual_count = operand_count.min(remaining_bytes);
        let mut operands = Vec::with_capacity(actual_count);
        for _ in 0..actual_count {
            operands.push(self.reader.read_u8()?);
        }

        Ok(operands)
    }

    /// 跳过属性
    fn skip_attribute(&mut self) -> Result<()>
    where
        R: Seek,
    {
        let _attribute_name_index = self.reader.read_u16()?;
        let attribute_length = self.reader.read_u32()? as usize;
        self.reader.skip(attribute_length as u64)?;
        Ok(())
    }
}

/// 便捷函数：从任意Read类型读取JVM类
pub fn read_class_from_reader<R: Read + std::io::Seek>(reader: R) -> Result<JvmClass> {
    let mut jvm_reader = JvmReader2::new(reader);
    jvm_reader.read_class()
}

/// 便捷函数：从任意Read类型读取指令列表
pub fn read_instructions_from_reader<R: Read + std::io::Seek>(reader: R, code_length: usize) -> Result<Vec<JvmInstruction>> {
    let mut jvm_reader = JvmReader2::new(reader);
    jvm_reader.read_instructions(code_length)
}
