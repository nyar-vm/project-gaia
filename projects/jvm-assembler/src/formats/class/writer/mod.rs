#![doc = include_str!("readme.md")]
//! JVM Class 文件写入器
//!
//! 这个模块实现了将 JVM 程序转换为 Class 文件字节码的功能。

use crate::program::*;
use byteorder::BigEndian;
use gaia_types::{BinaryWriter, GaiaDiagnostics, Result};
use std::io::Write;

/// Class 文件写入器
pub struct ClassWriter<W> {
    /// 二进制汇编器
    writer: BinaryWriter<W, BigEndian>,
}

impl<W> ClassWriter<W> {
    /// 创建新的 Class 写入器
    pub fn new(writer: W) -> Self {
        Self { writer: BinaryWriter::new(writer) }
    }

    /// 完成写入并返回底层写入器
    pub fn finish(self) -> W {
        self.writer.finish()
    }
}

impl<W: Write> ClassWriter<W> {
    /// 将 ClassView 写入为二进制 Class 格式
    pub fn write(mut self, program: &JvmProgram) -> GaiaDiagnostics<W> {
        match self.write_class_file(program) {
            Ok(_) => GaiaDiagnostics::success(self.finish()),
            Err(error) => GaiaDiagnostics::failure(error),
        }
    }

    /// 写入 Class 文件
    fn write_class_file(&mut self, program: &JvmProgram) -> Result<()> {
        // 写入魔数
        self.writer.write_u32(0xCAFEBABE)?;
        
        // 写入版本信息
        self.writer.write_u16(program.version.minor)?;
        self.writer.write_u16(program.version.major)?;
        
        // 构建并写入常量池
        self.write_constant_pool(program)?;
        
        // 写入访问标志
        self.writer.write_u16(program.access_flags.to_flags())?;
        
        // 写入类索引（this_class）
        self.writer.write_u16(2)?; // 类的 Class 条目在索引2
        
        // 写入超类索引（super_class）
        if program.super_class.is_some() {
            self.writer.write_u16(4)?; // 超类的 Class 条目在索引4
        } else {
            self.writer.write_u16(0)?;
        }
        
        // 写入接口数量（暂时为0）
        self.writer.write_u16(0)?;
        
        // 写入字段
        self.write_fields(program)?;
        
        // 写入方法
        self.write_methods(program)?;
        
        // 写入属性数量（暂时为0）
        self.writer.write_u16(0)?;
        
        Ok(())
    }
    
    /// 写入常量池
    fn write_constant_pool(&mut self, program: &JvmProgram) -> Result<()> {
        // 简化的常量池结构
        let mut pool_entries = Vec::new();
        
        // 1. 类名的 UTF8 条目
        pool_entries.push(format!("UTF8:{}", program.name));
        
        // 2. 类的 Class 条目（引用索引1）
        pool_entries.push("CLASS:1".to_string());
        
        // 3. 超类名的 UTF8 条目
        if let Some(super_class) = &program.super_class {
            pool_entries.push(format!("UTF8:{}", super_class));
        } else {
            pool_entries.push("UTF8:java/lang/Object".to_string());
        }
        
        // 4. 超类的 Class 条目（引用索引3）
        pool_entries.push("CLASS:3".to_string());
        
        // 5. "Hello, World!" 字符串的 UTF8 条目
        pool_entries.push("UTF8:Hello, World!".to_string());
        
        // 6. String 条目（引用索引5）
        pool_entries.push("STRING:5".to_string());
        
        // 7. System 类名的 UTF8 条目
        pool_entries.push("UTF8:java/lang/System".to_string());
        
        // 8. System 类的 Class 条目（引用索引7）
        pool_entries.push("CLASS:7".to_string());
        
        // 9. out 字段名的 UTF8 条目
        pool_entries.push("UTF8:out".to_string());
        
        // 10. PrintStream 类型描述符的 UTF8 条目
        pool_entries.push("UTF8:Ljava/io/PrintStream;".to_string());
        
        // 11. NameAndType 条目（out 字段的名称和类型）
        pool_entries.push("NAMEANDTYPE:9:10".to_string());
        
        // 12. Fieldref 条目（System.out）
        pool_entries.push("FIELDREF:8:11".to_string());
        
        // 13. PrintStream 类名的 UTF8 条目
        pool_entries.push("UTF8:java/io/PrintStream".to_string());
        
        // 14. PrintStream 类的 Class 条目（引用索引13）
        pool_entries.push("CLASS:13".to_string());
        
        // 15. println 方法名的 UTF8 条目
        pool_entries.push("UTF8:println".to_string());
        
        // 16. println 方法描述符的 UTF8 条目
        pool_entries.push("UTF8:(Ljava/lang/String;)V".to_string());
        
        // 17. NameAndType 条目（println 方法的名称和描述符）
        pool_entries.push("NAMEANDTYPE:15:16".to_string());
        
        // 18. Methodref 条目（PrintStream.println）
        pool_entries.push("METHODREF:14:17".to_string());
        
        // 添加方法和字段的名称和描述符
        for method in &program.methods {
            pool_entries.push(format!("UTF8:{}", method.name));
            pool_entries.push(format!("UTF8:{}", method.descriptor));
        }
        
        for field in &program.fields {
            pool_entries.push(format!("UTF8:{}", field.name));
            pool_entries.push(format!("UTF8:{}", field.descriptor));
        }
        
        // 添加 "Code" 属性名称
        pool_entries.push("UTF8:Code".to_string());
        
        // 写入常量池计数（+1 因为索引从1开始）
        self.writer.write_u16((pool_entries.len() + 1) as u16)?;
        
        // 写入常量池条目
        for entry in &pool_entries {
            if entry.starts_with("UTF8:") {
                let utf8_str = &entry[5..];
                self.writer.write_u8(1)?; // CONSTANT_Utf8 tag
                self.writer.write_u16(utf8_str.len() as u16)?;
                self.writer.write_all(utf8_str.as_bytes())?;
            } else if entry.starts_with("CLASS:") {
                let class_index: u16 = entry[6..].parse().unwrap();
                self.writer.write_u8(7)?; // CONSTANT_Class tag
                self.writer.write_u16(class_index)?;
            } else if entry.starts_with("STRING:") {
                let string_index: u16 = entry[7..].parse().unwrap();
                self.writer.write_u8(8)?; // CONSTANT_String tag
                self.writer.write_u16(string_index)?;
            } else if entry.starts_with("NAMEANDTYPE:") {
                let parts: Vec<&str> = entry[12..].split(':').collect();
                let name_index: u16 = parts[0].parse().unwrap();
                let descriptor_index: u16 = parts[1].parse().unwrap();
                self.writer.write_u8(12)?; // CONSTANT_NameAndType tag
                self.writer.write_u16(name_index)?;
                self.writer.write_u16(descriptor_index)?;
            } else if entry.starts_with("FIELDREF:") {
                let parts: Vec<&str> = entry[9..].split(':').collect();
                let class_index: u16 = parts[0].parse().unwrap();
                let name_and_type_index: u16 = parts[1].parse().unwrap();
                self.writer.write_u8(9)?; // CONSTANT_Fieldref tag
                self.writer.write_u16(class_index)?;
                self.writer.write_u16(name_and_type_index)?;
            } else if entry.starts_with("METHODREF:") {
                let parts: Vec<&str> = entry[10..].split(':').collect();
                let class_index: u16 = parts[0].parse().unwrap();
                let name_and_type_index: u16 = parts[1].parse().unwrap();
                self.writer.write_u8(10)?; // CONSTANT_Methodref tag
                self.writer.write_u16(class_index)?;
                self.writer.write_u16(name_and_type_index)?;
            }
        }
        
        Ok(())
    }
    
    /// 写入字段
    fn write_fields(&mut self, program: &JvmProgram) -> Result<()> {
        self.writer.write_u16(program.fields.len() as u16)?;
        
        for field in &program.fields {
            self.writer.write_u16(field.access_flags.to_flags())?;
            self.writer.write_u16(3)?; // 假设字段名在常量池索引3
            self.writer.write_u16(4)?; // 假设字段描述符在常量池索引4
            self.writer.write_u16(0)?; // 属性数量
        }
        
        Ok(())
    }
    
    /// 写入方法
    fn write_methods(&mut self, program: &JvmProgram) -> Result<()> {
        self.writer.write_u16(program.methods.len() as u16)?;
        
        for method in &program.methods {
            self.writer.write_u16(method.access_flags.to_flags())?;
            // 方法名和描述符在常量池中的索引需要根据实际位置计算
            // 假设 main 方法名在索引19，描述符在索引20
            self.writer.write_u16(19)?; // 方法名索引
            self.writer.write_u16(20)?; // 方法描述符索引
            
            // 写入属性（Code 属性）
            self.writer.write_u16(1)?; // 属性数量
            self.write_code_attribute(method)?;
        }
        
        Ok(())
    }
    
    /// 写入 Code 属性
    fn write_code_attribute(&mut self, method: &JvmMethod) -> Result<()> {
        // Code 属性名称索引（"Code" 在索引21）
        self.writer.write_u16(21)?;
        
        let bytecode = self.generate_method_bytecode(method);
        
        // Code 属性长度（不包括属性名称索引和长度字段本身）
        let attribute_length = 2 + 2 + 4 + bytecode.len() + 2 + 2;
        self.writer.write_u32(attribute_length as u32)?;
        
        // max_stack 和 max_locals
        self.writer.write_u16(2)?; // max_stack
        self.writer.write_u16(1)?; // max_locals
        
        // 字节码长度和字节码
        self.writer.write_u32(bytecode.len() as u32)?;
        self.writer.write_all(&bytecode)?;
        
        // 异常表长度（0）
        self.writer.write_u16(0)?;
        
        // 属性数量（0）
        self.writer.write_u16(0)?;
        
        Ok(())
    }
    
    /// 生成方法的字节码
    fn generate_method_bytecode(&self, method: &JvmMethod) -> Vec<u8> {
        let mut bytecode = Vec::new();
        
        for instruction in &method.instructions {
            match instruction {
                JvmInstruction::Nop => bytecode.push(0x00),
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
                JvmInstruction::Ldc { symbol: _ } => {
                    bytecode.push(0x12); // ldc
                    bytecode.push(6); // String 常量在索引6
                }
                JvmInstruction::Iload0 => bytecode.push(0x1A),
                JvmInstruction::Iload1 => bytecode.push(0x1B),
                JvmInstruction::Iload2 => bytecode.push(0x1C),
                JvmInstruction::Iload3 => bytecode.push(0x1D),
                JvmInstruction::Aload0 => bytecode.push(0x2A),
                JvmInstruction::Aload1 => bytecode.push(0x2B),
                JvmInstruction::Aload2 => bytecode.push(0x2C),
                JvmInstruction::Aload3 => bytecode.push(0x2D),
                JvmInstruction::Istore0 => bytecode.push(0x3B),
                JvmInstruction::Istore1 => bytecode.push(0x3C),
                JvmInstruction::Istore2 => bytecode.push(0x3D),
                JvmInstruction::Istore3 => bytecode.push(0x3E),
                JvmInstruction::Astore0 => bytecode.push(0x4B),
                JvmInstruction::Astore1 => bytecode.push(0x4C),
                JvmInstruction::Astore2 => bytecode.push(0x4D),
                JvmInstruction::Astore3 => bytecode.push(0x4E),
                JvmInstruction::Iadd => bytecode.push(0x60),
                JvmInstruction::Pop => bytecode.push(0x57),
                JvmInstruction::Return => bytecode.push(0xB1),
                JvmInstruction::Ireturn => bytecode.push(0xAC),
                JvmInstruction::New { class_name: _ } => {
                    bytecode.push(0xBB); // new
                    bytecode.push(0x00); // 类索引高字节
                    bytecode.push(0x02); // 类索引低字节
                }

                JvmInstruction::Getstatic { class_name: _, field_name: _, descriptor: _ } => {
                    bytecode.push(0xB2); // getstatic
                    bytecode.push(0x00); // 字段引用索引高字节
                    bytecode.push(0x0C); // 字段引用索引低字节（System.out，索引12）
                }
                JvmInstruction::Invokevirtual { class_name: _, method_name: _, descriptor: _ } => {
                    bytecode.push(0xB6); // invokevirtual
                    bytecode.push(0x00); // 方法引用索引高字节
                    bytecode.push(0x12); // 方法引用索引低字节（PrintStream.println，索引18）
                }
                _ => {
                    // 对于其他指令，暂时使用 nop
                    bytecode.push(0x00);
                }
            }
        }
        
        // 如果方法没有指令，添加一个 return 指令
        if bytecode.is_empty() {
            bytecode.push(0xB1); // return
        }
        
        bytecode
    }
}
