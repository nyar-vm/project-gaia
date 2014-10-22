use crate::{
    opcodes::{
        ACONST_NULL, ARETURN, BIPUSH, DADD, DCONST_0, DCONST_1, DDIV, DMUL, DRETURN, DSUB, DUP, DUP2, DUP2_X1, DUP2_X2, DUP_X1,
        DUP_X2, FADD, FDIV, FMUL, FRETURN, FSUB, IADD, IDIV, IMUL, INVOKEINTERFACE, INVOKESPECIAL, INVOKESTATIC, INVOKEVIRTUAL,
        IRETURN, ISUB, LADD, LCONST_0, LCONST_1, LDC, LDC2_W, LDC_W, LDIV, LMUL, LRETURN, LSUB, NOP, POP, POP2, RETURN, SIPUSH,
        SWAP,
    },
    ConstantPoolEntry, JvmClass, JvmField, JvmInstruction, JvmMethod, JvmReader,
};
use byteorder::{BigEndian, ReadBytesExt};
use gaia_types::GaiaError;
use std::io::{Cursor, Read};

impl JvmReader {
    /// 从字节数组创建读取器
    pub fn new(data: Vec<u8>) -> Self {
        Self { buffer: Cursor::new(data) }
    }

    /// 读取 JVM 类文件
    pub fn read_class(&mut self) -> gaia_types::Result<JvmClass> {
        let mut class = JvmClass::default();

        // 读取魔数
        class.magic = self.buffer.read_u32::<BigEndian>()?;
        if class.magic != 0xCAFEBABE {
            return Err(GaiaError::invalid_data("Invalid JVM class file magic number"));
        }

        // 读取版本号
        class.minor_version = self.buffer.read_u16::<BigEndian>()?;
        class.major_version = self.buffer.read_u16::<BigEndian>()?;

        // 读取常量池
        class.constant_pool = self.read_constant_pool()?;

        // 读取访问标志
        class.access_flags = self.buffer.read_u16::<BigEndian>()?;

        // 读取类索引
        class.this_class = self.buffer.read_u16::<BigEndian>()?;
        class.super_class = self.buffer.read_u16::<BigEndian>()?;

        // 读取接口
        class.interfaces = self.read_interfaces()?;

        // 读取字段
        class.fields = self.read_fields()?;

        // 读取方法
        class.methods = self.read_methods()?;

        // 跳过类属性
        let attributes_count = self.buffer.read_u16::<BigEndian>()?;
        for _ in 0..attributes_count {
            self.skip_attribute()?;
        }

        Ok(class)
    }

    /// 读取常量池
    fn read_constant_pool(&mut self) -> gaia_types::Result<Vec<ConstantPoolEntry>> {
        let count = self.buffer.read_u16::<BigEndian>()? as usize;
        let mut pool = Vec::with_capacity(count.saturating_sub(1));

        // 常量池索引从 1 开始，所以读取 count-1 个条目
        for _ in 1..count {
            let entry = self.read_constant_pool_entry()?;
            pool.push(entry);
        }

        Ok(pool)
    }

    /// 读取常量池条目
    fn read_constant_pool_entry(&mut self) -> gaia_types::Result<ConstantPoolEntry> {
        let tag = self.buffer.read_u8()?;

        match tag {
            1 => {
                // CONSTANT_Utf8
                let length = self.buffer.read_u16::<BigEndian>()? as usize;
                let mut bytes = vec![0u8; length];
                self.buffer.read_exact(&mut bytes)?;
                let string = String::from_utf8(bytes).map_err(|_| GaiaError::invalid_data("Invalid UTF-8 in constant pool"))?;
                Ok(ConstantPoolEntry::Utf8(string))
            }
            3 => {
                // CONSTANT_Integer
                let value = self.buffer.read_i32::<BigEndian>()?;
                Ok(ConstantPoolEntry::Integer(value))
            }
            4 => {
                // CONSTANT_Float
                let value = self.buffer.read_f32::<BigEndian>()?;
                Ok(ConstantPoolEntry::Float(value))
            }
            5 => {
                // CONSTANT_Long
                let value = self.buffer.read_i64::<BigEndian>()?;
                Ok(ConstantPoolEntry::Long(value))
            }
            6 => {
                // CONSTANT_Double
                let value = self.buffer.read_f64::<BigEndian>()?;
                Ok(ConstantPoolEntry::Double(value))
            }
            7 => {
                // CONSTANT_Class
                let name_index = self.buffer.read_u16::<BigEndian>()?;
                Ok(ConstantPoolEntry::Class(name_index))
            }
            8 => {
                // CONSTANT_String
                let string_index = self.buffer.read_u16::<BigEndian>()?;
                Ok(ConstantPoolEntry::String(string_index))
            }
            9 => {
                // CONSTANT_Fieldref
                let class_index = self.buffer.read_u16::<BigEndian>()?;
                let name_and_type_index = self.buffer.read_u16::<BigEndian>()?;
                Ok(ConstantPoolEntry::Fieldref { class_index, name_and_type_index })
            }
            10 => {
                // CONSTANT_Methodref
                let class_index = self.buffer.read_u16::<BigEndian>()?;
                let name_and_type_index = self.buffer.read_u16::<BigEndian>()?;
                Ok(ConstantPoolEntry::Methodref { class_index, name_and_type_index })
            }
            11 => {
                // CONSTANT_InterfaceMethodref
                let class_index = self.buffer.read_u16::<BigEndian>()?;
                let name_and_type_index = self.buffer.read_u16::<BigEndian>()?;
                Ok(ConstantPoolEntry::InterfaceMethodref { class_index, name_and_type_index })
            }
            12 => {
                // CONSTANT_NameAndType
                let name_index = self.buffer.read_u16::<BigEndian>()?;
                let descriptor_index = self.buffer.read_u16::<BigEndian>()?;
                Ok(ConstantPoolEntry::NameAndType { name_index, descriptor_index })
            }
            _ => Err(GaiaError::invalid_data(&format!("Unknown constant pool tag: {}", tag))),
        }
    }

    /// 读取接口列表
    fn read_interfaces(&mut self) -> gaia_types::Result<Vec<u16>> {
        let count = self.buffer.read_u16::<BigEndian>()? as usize;
        let mut interfaces = Vec::with_capacity(count);

        for _ in 0..count {
            interfaces.push(self.buffer.read_u16::<BigEndian>()?);
        }

        Ok(interfaces)
    }

    /// 读取字段列表
    fn read_fields(&mut self) -> gaia_types::Result<Vec<JvmField>> {
        let count = self.buffer.read_u16::<BigEndian>()? as usize;
        let mut fields = Vec::with_capacity(count);

        for _ in 0..count {
            fields.push(self.read_field()?);
        }

        Ok(fields)
    }

    /// 读取单个字段
    fn read_field(&mut self) -> gaia_types::Result<JvmField> {
        let access_flags = self.buffer.read_u16::<BigEndian>()?;
        let name_index = self.buffer.read_u16::<BigEndian>()?;
        let descriptor_index = self.buffer.read_u16::<BigEndian>()?;

        // 跳过字段属性
        let attributes_count = self.buffer.read_u16::<BigEndian>()?;
        for _ in 0..attributes_count {
            self.skip_attribute()?;
        }

        Ok(JvmField { access_flags, name_index, descriptor_index })
    }

    /// 读取方法列表
    fn read_methods(&mut self) -> gaia_types::Result<Vec<JvmMethod>> {
        let count = self.buffer.read_u16::<BigEndian>()? as usize;
        let mut methods = Vec::with_capacity(count);

        for _ in 0..count {
            methods.push(self.read_method()?);
        }

        Ok(methods)
    }

    /// 读取单个方法
    fn read_method(&mut self) -> gaia_types::Result<JvmMethod> {
        let access_flags = self.buffer.read_u16::<BigEndian>()?;
        let name_index = self.buffer.read_u16::<BigEndian>()?;
        let descriptor_index = self.buffer.read_u16::<BigEndian>()?;

        let mut method = JvmMethod::new(name_index, descriptor_index);
        method.access_flags = access_flags;

        // 读取方法属性
        let attributes_count = self.buffer.read_u16::<BigEndian>()?;
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
    fn try_read_code_attribute(&mut self) -> gaia_types::Result<Option<(u16, u16, Vec<JvmInstruction>)>> {
        let _attribute_name_index = self.buffer.read_u16::<BigEndian>()?;
        let _attribute_length = self.buffer.read_u32::<BigEndian>()?;

        // 简化处理：假设这是 Code 属性
        let max_stack = self.buffer.read_u16::<BigEndian>()?;
        let max_locals = self.buffer.read_u16::<BigEndian>()?;
        let code_length = self.buffer.read_u32::<BigEndian>()? as usize;

        // 读取指令
        let instructions = self.read_instructions(code_length)?;

        // 跳过异常表
        let exception_table_length = self.buffer.read_u16::<BigEndian>()?;
        for _ in 0..exception_table_length {
            self.buffer.read_u64::<BigEndian>()?; // 跳过异常表条目
        }

        // 跳过代码属性的属性
        let code_attributes_count = self.buffer.read_u16::<BigEndian>()?;
        for _ in 0..code_attributes_count {
            self.skip_attribute()?;
        }

        Ok(Some((max_stack, max_locals, instructions)))
    }

    /// 读取指令列表
    pub fn read_instructions(&mut self, code_length: usize) -> gaia_types::Result<Vec<JvmInstruction>> {
        let mut instructions = Vec::new();
        let mut bytes_read = 0;

        while bytes_read < code_length {
            let opcode = self.buffer.read_u8()?;
            bytes_read += 1;

            let operands = self.read_instruction_operands(opcode, code_length - bytes_read)?;
            bytes_read += operands.len();

            instructions.push(JvmInstruction::new(opcode, operands));
        }

        Ok(instructions)
    }

    /// 读取指令操作数
    fn read_instruction_operands(&mut self, opcode: u8, remaining_bytes: usize) -> gaia_types::Result<Vec<u8>> {
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
        let mut operands = vec![0u8; actual_count];

        if actual_count > 0 {
            self.buffer.read_exact(&mut operands)?;
        }

        Ok(operands)
    }

    /// 跳过属性
    fn skip_attribute(&mut self) -> gaia_types::Result<()> {
        let _attribute_name_index = self.buffer.read_u16::<BigEndian>()?;
        let attribute_length = self.buffer.read_u32::<BigEndian>()? as usize;

        let mut buffer = vec![0u8; attribute_length];
        self.buffer.read_exact(&mut buffer)?;

        Ok(())
    }
}
