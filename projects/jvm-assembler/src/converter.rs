//! JASM AST 到 JVM 程序的转换器

use crate::helpers::*;
use gaia_types::{GaiaError, Result};
use jvm_jasm::ast::{JasmClass, JasmField, JasmInstruction, JasmMethod, JasmRoot};
use std::collections::HashMap;

/// JASM 到 JVM 转换器
pub struct JasmToJvmConverter {
    /// 常量池映射
    constant_pool_map: HashMap<String, u16>,
    /// 常量池
    constant_pool: Vec<ConstantPoolEntry>,
}

impl JasmToJvmConverter {
    /// 创建新的转换器
    pub fn new() -> Self {
        Self { constant_pool_map: HashMap::new(), constant_pool: Vec::new() }
    }

    /// 转换 JASM 根节点到 JVM 类
    pub fn convert(&mut self, jasm_root: &JasmRoot) -> Result<JvmClass> {
        let mut jvm_class = JvmClass::default();

        // 预先添加必要的常量池条目
        self.add_utf8_constant("Code"); // 索引 1，用于 Code 属性

        // 转换类信息
        self.convert_class_info(&jasm_root.class, &mut jvm_class)?;

        // 转换字段
        for field in &jasm_root.class.fields {
            let jvm_field = self.convert_field(field)?;
            jvm_class.fields.push(jvm_field);
        }

        // 转换方法
        for method in &jasm_root.class.methods {
            let jvm_method = self.convert_method(method)?;
            jvm_class.methods.push(jvm_method);
        }

        // 设置常量池
        jvm_class.constant_pool = self.constant_pool.clone();

        Ok(jvm_class)
    }

    /// 转换类基本信息
    fn convert_class_info(&mut self, jasm_class: &JasmClass, jvm_class: &mut JvmClass) -> Result<()> {
        // 解析版本信息
        if let Some(version_str) = &jasm_class.version {
            if let Some((major, minor)) = self.parse_version(version_str) {
                jvm_class.major_version = major;
                jvm_class.minor_version = minor;
            }
        }

        // 添加类名到常量池
        let class_name_index = self.add_utf8_constant(&jasm_class.name);
        let this_class_index = self.add_class_constant(class_name_index);
        jvm_class.this_class = this_class_index;

        // 添加父类到常量池（默认为 Object）
        let super_class_name_index = self.add_utf8_constant("java/lang/Object");
        let super_class_index = self.add_class_constant(super_class_name_index);
        jvm_class.super_class = super_class_index;

        // 设置访问标志
        jvm_class.access_flags = self.convert_class_access_flags(&jasm_class.modifiers);

        Ok(())
    }

    /// 转换字段
    fn convert_field(&mut self, jasm_field: &JasmField) -> Result<JvmField> {
        // 解析字段名和描述符
        let (name, descriptor) = self.parse_name_and_descriptor(&jasm_field.name_and_descriptor)?;

        let name_index = self.add_utf8_constant(&name);
        let descriptor_index = self.add_utf8_constant(&descriptor);

        Ok(JvmField { access_flags: self.convert_field_access_flags(&jasm_field.modifiers), name_index, descriptor_index })
    }

    /// 转换方法
    fn convert_method(&mut self, jasm_method: &JasmMethod) -> Result<JvmMethod> {
        // 解析方法名和描述符
        let (name, descriptor) = self.parse_name_and_descriptor(&jasm_method.name_and_descriptor)?;

        let name_index = self.add_utf8_constant(&name);
        let descriptor_index = self.add_utf8_constant(&descriptor);

        let mut jvm_method = JvmMethod::new(name_index, descriptor_index);
        jvm_method.access_flags = self.convert_method_access_flags(&jasm_method.modifiers);

        // 设置栈和局部变量信息
        if let Some(stack_size) = jasm_method.stack_size {
            jvm_method.max_stack = stack_size as u16;
        }
        if let Some(locals_count) = jasm_method.locals_count {
            jvm_method.max_locals = locals_count as u16;
        }

        // 转换指令
        for jasm_instruction in &jasm_method.instructions {
            let jvm_instruction = self.convert_instruction(jasm_instruction)?;
            jvm_method.add_instruction(jvm_instruction);
        }

        Ok(jvm_method)
    }

    /// 转换指令
    pub fn convert_instruction(&mut self, jasm_instruction: &JasmInstruction) -> Result<JvmInstruction> {
        match jasm_instruction {
            JasmInstruction::Simple(instruction) => {
                let opcode = self.instruction_to_opcode(instruction)?;
                Ok(JvmInstruction::new(opcode, vec![]))
            }
            JasmInstruction::WithArgument { instruction, argument } => {
                let opcode = self.instruction_to_opcode(instruction)?;
                let operands = self.convert_instruction_argument(instruction, argument)?;
                Ok(JvmInstruction::new(opcode, operands))
            }
            JasmInstruction::MethodCall { instruction, method_ref } => {
                let opcode = self.instruction_to_opcode(instruction)?;
                let method_ref_index = self.add_method_ref(method_ref)?;
                let operands = vec![(method_ref_index >> 8) as u8, (method_ref_index & 0xFF) as u8];
                Ok(JvmInstruction::new(opcode, operands))
            }
            JasmInstruction::FieldAccess { instruction, field_ref } => {
                let opcode = self.instruction_to_opcode(instruction)?;
                let field_ref_index = self.add_field_ref(field_ref)?;
                let operands = vec![(field_ref_index >> 8) as u8, (field_ref_index & 0xFF) as u8];
                Ok(JvmInstruction::new(opcode, operands))
            }
        }
    }

    /// 指令名称转换为操作码
    fn instruction_to_opcode(&self, instruction: &str) -> Result<u8> {
        match instruction {
            "aload_0" => Ok(opcodes::ALOAD_0),
            "aload_1" => Ok(opcodes::ALOAD_1),
            "aload_2" => Ok(opcodes::ALOAD_2),
            "aload_3" => Ok(opcodes::ALOAD_3),
            "return" => Ok(opcodes::RETURN),
            "ireturn" => Ok(opcodes::IRETURN),
            "areturn" => Ok(opcodes::ARETURN),
            "ldc" => Ok(opcodes::LDC),
            "invokespecial" => Ok(opcodes::INVOKESPECIAL),
            "invokevirtual" => Ok(opcodes::INVOKEVIRTUAL),
            "invokestatic" => Ok(opcodes::INVOKESTATIC),
            "getstatic" => Ok(0xB2), // GETSTATIC
            "putstatic" => Ok(0xB3), // PUTSTATIC
            "getfield" => Ok(0xB4),  // GETFIELD
            "putfield" => Ok(0xB5),  // PUTFIELD
            _ => Err(GaiaError::invalid_data(&format!("Unknown instruction: {}", instruction))),
        }
    }

    /// 转换指令参数
    fn convert_instruction_argument(&mut self, instruction: &str, argument: &str) -> Result<Vec<u8>> {
        match instruction {
            "ldc" => {
                // 处理字符串常量
                if argument.starts_with("String ") {
                    let string_value = argument.strip_prefix("String ").unwrap().trim_matches('"');
                    let string_index = self.add_string_constant(string_value);
                    Ok(vec![string_index as u8])
                }
                else {
                    Err(GaiaError::invalid_data(&format!("Unsupported ldc argument: {}", argument)))
                }
            }
            _ => Err(GaiaError::invalid_data(&format!("Unsupported instruction with argument: {}", instruction))),
        }
    }

    /// 添加方法引用到常量池
    fn add_method_ref(&mut self, method_ref: &str) -> Result<u16> {
        // 检查是否已经存在
        let key = format!("methodref:{}", method_ref);
        if let Some(&index) = self.constant_pool_map.get(&key) {
            return Ok(index);
        }

        // 解析方法引用格式: java/io/PrintStream.println:"(Ljava/lang/String;)V" 或 java/lang/Object."<init>":"()V"
        // 找到最后一个点来分离类名和方法部分
        if let Some(dot_pos) = method_ref.rfind('.') {
            let class_name = &method_ref[..dot_pos];
            let method_part = &method_ref[dot_pos + 1..];

            // 解析方法名和描述符
            if let Some(colon_pos) = method_part.find(':') {
                let method_name_raw = &method_part[..colon_pos];
                let method_descriptor_raw = &method_part[colon_pos + 1..];

                // 去除方法名和描述符中的双引号
                let method_name = method_name_raw.trim_matches('"');
                let method_descriptor = method_descriptor_raw.trim_matches('"');

                // 添加到常量池
                let class_name_index = self.add_utf8_constant(class_name);
                let class_index = self.add_class_constant(class_name_index);

                let method_name_index = self.add_utf8_constant(method_name);
                let method_descriptor_index = self.add_utf8_constant(method_descriptor);
                let name_and_type_index = self.add_name_and_type_constant(method_name_index, method_descriptor_index);

                let method_ref_index = self.constant_pool.len() as u16 + 1;
                self.constant_pool.push(ConstantPoolEntry::Methodref { class_index, name_and_type_index });

                self.constant_pool_map.insert(key, method_ref_index);
                Ok(method_ref_index)
            }
            else {
                Err(GaiaError::invalid_data(&format!("Invalid method reference format - missing colon: {}", method_ref)))
            }
        }
        else {
            Err(GaiaError::invalid_data(&format!("Invalid method reference format - missing dot: {}", method_ref)))
        }
    }

    /// 添加字段引用到常量池
    fn add_field_ref(&mut self, field_ref: &str) -> Result<u16> {
        // 检查是否已经存在
        let key = format!("fieldref:{}", field_ref);
        if let Some(&index) = self.constant_pool_map.get(&key) {
            return Ok(index);
        }

        // 解析字段引用格式: java/lang/System.out:"Ljava/io/PrintStream;"
        // 找到最后一个点的位置来分离类名和字段名
        if let Some(dot_pos) = field_ref.rfind('.') {
            let class_name = &field_ref[..dot_pos];
            let field_part = &field_ref[dot_pos + 1..];

            // 找到冒号位置来分离字段名和描述符
            if let Some(colon_pos) = field_part.find(':') {
                let field_name = &field_part[..colon_pos];
                let field_descriptor = &field_part[colon_pos + 1..].trim_matches('"');

                // 添加到常量池
                let class_name_index = self.add_utf8_constant(class_name);
                let class_index = self.add_class_constant(class_name_index);

                let field_name_index = self.add_utf8_constant(field_name);
                let field_descriptor_index = self.add_utf8_constant(field_descriptor);
                let name_and_type_index = self.add_name_and_type_constant(field_name_index, field_descriptor_index);

                let field_ref_index = self.constant_pool.len() as u16 + 1;
                self.constant_pool.push(ConstantPoolEntry::Fieldref { class_index, name_and_type_index });

                self.constant_pool_map.insert(key, field_ref_index);
                Ok(field_ref_index)
            }
            else {
                Err(GaiaError::invalid_data(&format!("Invalid field reference format - missing colon: {}", field_ref)))
            }
        }
        else {
            Err(GaiaError::invalid_data(&format!("Invalid field reference format - missing dot: {}", field_ref)))
        }
    }

    /// 解析名称和描述符
    fn parse_name_and_descriptor(&self, input: &str) -> Result<(String, String)> {
        if let Some(colon_pos) = input.find(':') {
            let name = input[..colon_pos].trim_matches('"').to_string();
            let descriptor = input[colon_pos + 1..].trim_matches('"').to_string();
            Ok((name, descriptor))
        }
        else {
            Err(GaiaError::invalid_data(&format!("Invalid name and descriptor format: {}", input)))
        }
    }

    /// 解析版本信息
    fn parse_version(&self, version_str: &str) -> Option<(u16, u16)> {
        if let Some(colon_pos) = version_str.find(':') {
            let major_str = &version_str[..colon_pos];
            let minor_str = &version_str[colon_pos + 1..];
            if let (Ok(major), Ok(minor)) = (major_str.parse::<u16>(), minor_str.parse::<u16>()) {
                return Some((major, minor));
            }
        }
        None
    }

    /// 添加 UTF-8 常量到常量池
    pub fn add_utf8_constant(&mut self, value: &str) -> u16 {
        let key = format!("utf8:{}", value);
        if let Some(&index) = self.constant_pool_map.get(&key) {
            return index;
        }

        let index = self.constant_pool.len() as u16 + 1;
        self.constant_pool.push(ConstantPoolEntry::Utf8(value.to_string()));
        self.constant_pool_map.insert(key, index);
        index
    }

    /// 添加类常量到常量池
    fn add_class_constant(&mut self, name_index: u16) -> u16 {
        let key = format!("class:{}", name_index);
        if let Some(&index) = self.constant_pool_map.get(&key) {
            return index;
        }

        let index = self.constant_pool.len() as u16 + 1;
        self.constant_pool.push(ConstantPoolEntry::Class(name_index));
        self.constant_pool_map.insert(key, index);
        index
    }

    /// 添加字符串常量到常量池
    fn add_string_constant(&mut self, value: &str) -> u16 {
        let utf8_index = self.add_utf8_constant(value);
        let key = format!("string:{}", utf8_index);
        if let Some(&index) = self.constant_pool_map.get(&key) {
            return index;
        }

        let index = self.constant_pool.len() as u16 + 1;
        self.constant_pool.push(ConstantPoolEntry::String(utf8_index));
        self.constant_pool_map.insert(key, index);
        index
    }

    /// 添加名称和类型常量到常量池
    fn add_name_and_type_constant(&mut self, name_index: u16, descriptor_index: u16) -> u16 {
        let key = format!("nameandtype:{}:{}", name_index, descriptor_index);
        if let Some(&index) = self.constant_pool_map.get(&key) {
            return index;
        }

        let index = self.constant_pool.len() as u16 + 1;
        self.constant_pool.push(ConstantPoolEntry::NameAndType { name_index, descriptor_index });
        self.constant_pool_map.insert(key, index);
        index
    }

    /// 转换类访问标志
    fn convert_class_access_flags(&self, modifiers: &[String]) -> u16 {
        let mut flags = 0x0020; // ACC_SUPER (默认)

        for modifier in modifiers {
            match modifier.as_str() {
                "public" => flags |= 0x0001,   // ACC_PUBLIC
                "final" => flags |= 0x0010,    // ACC_FINAL
                "super" => flags |= 0x0020,    // ACC_SUPER
                "abstract" => flags |= 0x0400, // ACC_ABSTRACT
                _ => {}
            }
        }

        flags
    }

    /// 转换字段访问标志
    fn convert_field_access_flags(&self, modifiers: &[String]) -> u16 {
        let mut flags = 0;

        for modifier in modifiers {
            match modifier.as_str() {
                "public" => flags |= 0x0001,    // ACC_PUBLIC
                "private" => flags |= 0x0002,   // ACC_PRIVATE
                "protected" => flags |= 0x0004, // ACC_PROTECTED
                "static" => flags |= 0x0008,    // ACC_STATIC
                "final" => flags |= 0x0010,     // ACC_FINAL
                _ => {}
            }
        }

        flags
    }

    /// 转换方法访问标志
    fn convert_method_access_flags(&self, modifiers: &[String]) -> u16 {
        let mut flags = 0;

        for modifier in modifiers {
            match modifier.as_str() {
                "public" => flags |= 0x0001,    // ACC_PUBLIC
                "private" => flags |= 0x0002,   // ACC_PRIVATE
                "protected" => flags |= 0x0004, // ACC_PROTECTED
                "static" => flags |= 0x0008,    // ACC_STATIC
                "final" => flags |= 0x0010,     // ACC_FINAL
                "abstract" => flags |= 0x0400,  // ACC_ABSTRACT
                _ => {}
            }
        }

        flags
    }
}

impl Default for JasmToJvmConverter {
    fn default() -> Self {
        Self::new()
    }
}

/// 便捷函数：从 JASM 文本转换为 JVM 类
pub fn convert_jasm_to_jvm(jasm_text: &str) -> Result<JvmClass> {
    let parser = jvm_jasm::parser::JasmParser::new();
    let parse_result = parser.parse_text(jasm_text);

    match parse_result.result {
        Ok(jasm_root) => {
            let mut converter = JasmToJvmConverter::new();
            converter.convert(&jasm_root)
        }
        Err(error) => Err(error),
    }
}
