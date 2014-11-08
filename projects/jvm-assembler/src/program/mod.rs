//! JVM 程序高层次抽象
//!
//! 这个模块定义了 JVM 程序的高层次表示，用于在 JASM AST 和二进制 Class 文件之间提供中间抽象层。
//! 支持 JVM 字节码指令、常量池、方法、字段等核心概念。

#![doc = include_str!("readme.md")]

use gaia_types::{GaiaError, Result};
use std::collections::HashMap;

/// JVM 程序的高层次表示
#[derive(Debug, Clone)]
pub struct JvmProgram {
    /// 程序名称（类名）
    pub name: String,
    /// 访问标志
    pub access_flags: JvmAccessFlags,
    /// 超类名称
    pub super_class: Option<String>,
    /// 实现的接口列表
    pub interfaces: Vec<String>,
    /// 字段列表
    pub fields: Vec<JvmField>,
    /// 方法列表
    pub methods: Vec<JvmMethod>,
    /// 属性列表
    pub attributes: Vec<JvmAttribute>,
    /// 常量池（高层表示）
    pub constant_pool: JvmConstantPool,
    /// 版本信息
    pub version: JvmVersion,
    /// 源文件信息
    pub source_file: Option<String>,
}

/// JVM 版本信息
#[derive(Debug, Clone)]
pub struct JvmVersion {
    /// 主版本号
    pub major: u16,
    /// 次版本号
    pub minor: u16,
}

/// JVM 访问标志
#[derive(Debug, Clone)]
pub struct JvmAccessFlags {
    /// 是否为 public
    pub is_public: bool,
    /// 是否为 final
    pub is_final: bool,
    /// 是否为 super
    pub is_super: bool,
    /// 是否为 interface
    pub is_interface: bool,
    /// 是否为 abstract
    pub is_abstract: bool,
    /// 是否为 synthetic
    pub is_synthetic: bool,
    /// 是否为 annotation
    pub is_annotation: bool,
    /// 是否为 enum
    pub is_enum: bool,
    /// 是否为 static
    pub is_static: bool,
    /// 是否为 private
    pub is_private: bool,
    /// 是否为 protected
    pub is_protected: bool,
}

/// JVM 常量池（高层表示）
#[derive(Debug, Clone)]
pub struct JvmConstantPool {
    /// 符号表，用于名称到索引的映射
    pub symbol_table: HashMap<String, u16>,
    /// 常量池条目
    pub entries: Vec<JvmConstantPoolEntry>,
}

/// JVM 常量池条目（高层表示）
#[derive(Debug, Clone, PartialEq)]
pub enum JvmConstantPoolEntry {
    /// UTF-8 字符串
    Utf8 { value: String },
    /// 整数常量
    Integer { value: i32 },
    /// 浮点数常量
    Float { value: f32 },
    /// 长整数常量
    Long { value: i64 },
    /// 双精度浮点数常量
    Double { value: f64 },
    /// 类引用
    Class { name: String },
    /// 字符串引用
    String { value: String },
    /// 字段引用
    Fieldref { class_name: String, name: String, descriptor: String },
    /// 方法引用
    Methodref { class_name: String, name: String, descriptor: String },
    /// 接口方法引用
    InterfaceMethodref { class_name: String, name: String, descriptor: String },
    /// 名称和类型
    NameAndType { name: String, descriptor: String },
}

/// JVM 方法信息（高层表示）
#[derive(Debug, Clone)]
pub struct JvmMethod {
    /// 方法名称
    pub name: String,
    /// 方法描述符
    pub descriptor: String,
    /// 访问标志
    pub access_flags: JvmAccessFlags,
    /// 指令列表
    pub instructions: Vec<JvmInstruction>,
    /// 最大栈深度
    pub max_stack: u16,
    /// 最大局部变量数
    pub max_locals: u16,
    /// 异常表
    pub exception_table: Vec<JvmExceptionHandler>,
    /// 属性列表
    pub attributes: Vec<JvmAttribute>,
}

/// JVM 字段信息（高层表示）
#[derive(Debug, Clone)]
pub struct JvmField {
    /// 字段名称
    pub name: String,
    /// 字段描述符
    pub descriptor: String,
    /// 访问标志
    pub access_flags: JvmAccessFlags,
    /// 常量值（如果是常量字段）
    pub constant_value: Option<JvmConstantPoolEntry>,
    /// 属性列表
    pub attributes: Vec<JvmAttribute>,
}

/// JVM 指令（高层表示）
#[derive(Debug, Clone, PartialEq)]
pub enum JvmInstruction {
    /// 无操作数指令
    Simple { opcode: JvmOpcode },
    /// 带立即数的指令
    WithImmediate { opcode: JvmOpcode, value: i32 },
    /// 带局部变量索引的指令
    WithLocalVar { opcode: JvmOpcode, index: u16 },
    /// 带常量池引用的指令
    WithConstantPool { opcode: JvmOpcode, symbol: String },
    /// 方法调用指令
    MethodCall { opcode: JvmOpcode, class_name: String, method_name: String, descriptor: String },
    /// 字段访问指令
    FieldAccess { opcode: JvmOpcode, class_name: String, field_name: String, descriptor: String },
    /// 跳转指令
    Branch { opcode: JvmOpcode, target: String },
    /// 类型转换指令
    TypeCast { opcode: JvmOpcode, target_type: String },
}

/// JVM 操作码枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JvmOpcode {
    // 常量加载指令
    Nop,
    AconstNull,
    IconstM1,
    Iconst0,
    Iconst1,
    Iconst2,
    Iconst3,
    Iconst4,
    Iconst5,
    Lconst0,
    Lconst1,
    Fconst0,
    Fconst1,
    Fconst2,
    Dconst0,
    Dconst1,
    Bipush,
    Sipush,
    Ldc,
    LdcW,
    Ldc2W,

    // 局部变量加载指令
    Iload,
    Lload,
    Fload,
    Dload,
    Aload,
    Iload0,
    Iload1,
    Iload2,
    Iload3,
    Lload0,
    Lload1,
    Lload2,
    Lload3,
    Fload0,
    Fload1,
    Fload2,
    Fload3,
    Dload0,
    Dload1,
    Dload2,
    Dload3,
    Aload0,
    Aload1,
    Aload2,
    Aload3,

    // 局部变量存储指令
    Istore,
    Lstore,
    Fstore,
    Dstore,
    Astore,
    Istore0,
    Istore1,
    Istore2,
    Istore3,
    Lstore0,
    Lstore1,
    Lstore2,
    Lstore3,
    Fstore0,
    Fstore1,
    Fstore2,
    Fstore3,
    Dstore0,
    Dstore1,
    Dstore2,
    Dstore3,
    Astore0,
    Astore1,
    Astore2,
    Astore3,

    // 栈操作指令
    Pop,
    Pop2,
    Dup,
    DupX1,
    DupX2,
    Dup2,
    Dup2X1,
    Dup2X2,
    Swap,

    // 算术指令
    Iadd,
    Ladd,
    Fadd,
    Dadd,
    Isub,
    Lsub,
    Fsub,
    Dsub,
    Imul,
    Lmul,
    Fmul,
    Dmul,
    Idiv,
    Ldiv,
    Fdiv,
    Ddiv,
    Irem,
    Lrem,
    Frem,
    Drem,
    Ineg,
    Lneg,
    Fneg,
    Dneg,

    // 位运算指令
    Ishl,
    Lshl,
    Ishr,
    Lshr,
    Iushr,
    Lushr,
    Iand,
    Land,
    Ior,
    Lor,
    Ixor,
    Lxor,

    // 比较指令
    Lcmp,
    Fcmpl,
    Fcmpg,
    Dcmpl,
    Dcmpg,

    // 条件跳转指令
    Ifeq,
    Ifne,
    Iflt,
    Ifge,
    Ifgt,
    Ifle,
    IfIcmpeq,
    IfIcmpne,
    IfIcmplt,
    IfIcmpge,
    IfIcmpgt,
    IfIcmple,
    IfAcmpeq,
    IfAcmpne,
    Goto,
    Jsr,
    Ret,

    // 返回指令
    Ireturn,
    Lreturn,
    Freturn,
    Dreturn,
    Areturn,
    Return,

    // 字段访问指令
    Getstatic,
    Putstatic,
    Getfield,
    Putfield,

    // 方法调用指令
    Invokevirtual,
    Invokespecial,
    Invokestatic,
    Invokeinterface,
    Invokedynamic,

    // 对象操作指令
    New,
    Newarray,
    Anewarray,
    Arraylength,
    Athrow,
    Checkcast,
    Instanceof,
    Monitorenter,
    Monitorexit,

    // 其他指令
    Wide,
    Multianewarray,
    Ifnull,
    Ifnonnull,
    GotoW,
    JsrW,
}

/// JVM 异常处理器
#[derive(Debug, Clone)]
pub struct JvmExceptionHandler {
    /// 起始 PC
    pub start_pc: u16,
    /// 结束 PC
    pub end_pc: u16,
    /// 处理器 PC
    pub handler_pc: u16,
    /// 异常类型（None 表示 finally）
    pub catch_type: Option<String>,
}

/// JVM 属性
#[derive(Debug, Clone)]
pub enum JvmAttribute {
    /// 源文件属性
    SourceFile { filename: String },
    /// 代码属性
    Code {
        max_stack: u16,
        max_locals: u16,
        code: Vec<u8>,
        exception_table: Vec<JvmExceptionHandler>,
        attributes: Vec<JvmAttribute>,
    },
    /// 常量值属性
    ConstantValue { value: JvmConstantPoolEntry },
    /// 行号表属性
    LineNumberTable { entries: Vec<(u16, u16)> },
    /// 局部变量表属性
    LocalVariableTable { entries: Vec<JvmLocalVariable> },
    /// 其他属性
    Other { name: String, data: Vec<u8> },
}

/// JVM 局部变量信息
#[derive(Debug, Clone)]
pub struct JvmLocalVariable {
    /// 起始 PC
    pub start_pc: u16,
    /// 长度
    pub length: u16,
    /// 变量名
    pub name: String,
    /// 描述符
    pub descriptor: String,
    /// 索引
    pub index: u16,
}

impl JvmProgram {
    /// 创建新的 JVM 程序
    pub fn new(name: String) -> Self {
        Self {
            name,
            access_flags: JvmAccessFlags::default(),
            super_class: Some("java/lang/Object".to_string()),
            interfaces: Vec::new(),
            fields: Vec::new(),
            methods: Vec::new(),
            attributes: Vec::new(),
            constant_pool: JvmConstantPool::new(),
            version: JvmVersion { major: 65, minor: 0 }, // Java 21
            source_file: None,
        }
    }

    /// 添加方法
    pub fn add_method(&mut self, method: JvmMethod) {
        self.methods.push(method);
    }

    /// 添加字段
    pub fn add_field(&mut self, field: JvmField) {
        self.fields.push(field);
    }

    /// 设置源文件
    pub fn set_source_file(&mut self, filename: String) {
        self.source_file = Some(filename.clone());
        self.attributes.push(JvmAttribute::SourceFile { filename });
    }

    /// 验证程序的完整性
    pub fn validate(&self) -> Result<()> {
        // 验证类名不为空
        if self.name.is_empty() {
            return Err(GaiaError::custom_error("Class name cannot be empty".to_string()));
        }

        // 验证方法
        for method in &self.methods {
            method.validate()?;
        }

        // 验证字段
        for field in &self.fields {
            field.validate()?;
        }

        Ok(())
    }
}

impl JvmConstantPool {
    /// 创建新的常量池
    pub fn new() -> Self {
        Self { symbol_table: HashMap::new(), entries: Vec::new() }
    }

    /// 添加常量池条目
    pub fn add_entry(&mut self, entry: JvmConstantPoolEntry) -> u16 {
        // 检查是否已存在相同的条目
        for (index, existing_entry) in self.entries.iter().enumerate() {
            if existing_entry == &entry {
                return (index + 1) as u16; // 常量池索引从 1 开始
            }
        }

        let index = (self.entries.len() + 1) as u16;
        self.entries.push(entry);
        index
    }

    /// 根据符号名称查找索引
    pub fn find_symbol(&self, symbol: &str) -> Option<u16> {
        self.symbol_table.get(symbol).copied()
    }

    /// 添加符号到符号表
    pub fn add_symbol(&mut self, symbol: String, index: u16) {
        self.symbol_table.insert(symbol, index);
    }
}

impl JvmMethod {
    /// 创建新的方法
    pub fn new(name: String, descriptor: String) -> Self {
        Self {
            name,
            descriptor,
            access_flags: JvmAccessFlags::default(),
            instructions: Vec::new(),
            max_stack: 0,
            max_locals: 0,
            exception_table: Vec::new(),
            attributes: Vec::new(),
        }
    }

    /// 添加指令
    pub fn add_instruction(&mut self, instruction: JvmInstruction) {
        self.instructions.push(instruction);
    }

    /// 验证方法的完整性
    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(GaiaError::custom_error("Method name cannot be empty".to_string()));
        }
        if self.descriptor.is_empty() {
            return Err(GaiaError::custom_error("Method descriptor cannot be empty".to_string()));
        }
        Ok(())
    }
}

impl JvmField {
    /// 创建新的字段
    pub fn new(name: String, descriptor: String) -> Self {
        Self { name, descriptor, access_flags: JvmAccessFlags::default(), constant_value: None, attributes: Vec::new() }
    }

    /// 验证字段的完整性
    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(GaiaError::custom_error("Field name cannot be empty".to_string()));
        }
        if self.descriptor.is_empty() {
            return Err(GaiaError::custom_error("Field descriptor cannot be empty".to_string()));
        }
        Ok(())
    }
}

impl Default for JvmAccessFlags {
    fn default() -> Self {
        Self {
            is_public: false,
            is_final: false,
            is_super: true, // 默认为 super
            is_interface: false,
            is_abstract: false,
            is_synthetic: false,
            is_annotation: false,
            is_enum: false,
            is_static: false,
            is_private: false,
            is_protected: false,
        }
    }
}

impl JvmAccessFlags {
    /// 从修饰符字符串列表创建访问标志
    pub fn from_modifiers(modifiers: &[String]) -> Self {
        let mut flags = Self::default();

        for modifier in modifiers {
            match modifier.as_str() {
                "public" => flags.is_public = true,
                "private" => flags.is_private = true,
                "protected" => flags.is_protected = true,
                "static" => flags.is_static = true,
                "final" => flags.is_final = true,
                "super" => flags.is_super = true,
                "interface" => flags.is_interface = true,
                "abstract" => flags.is_abstract = true,
                "synthetic" => flags.is_synthetic = true,
                "annotation" => flags.is_annotation = true,
                "enum" => flags.is_enum = true,
                _ => {} // 忽略未知修饰符
            }
        }

        flags
    }

    /// 转换为 JVM 字节码中的访问标志值
    pub fn to_flags(&self) -> u16 {
        let mut flags = 0u16;

        if self.is_public {
            flags |= 0x0001;
        }
        if self.is_private {
            flags |= 0x0002;
        }
        if self.is_protected {
            flags |= 0x0004;
        }
        if self.is_static {
            flags |= 0x0008;
        }
        if self.is_final {
            flags |= 0x0010;
        }
        if self.is_super {
            flags |= 0x0020;
        }
        if self.is_interface {
            flags |= 0x0200;
        }
        if self.is_abstract {
            flags |= 0x0400;
        }
        if self.is_synthetic {
            flags |= 0x1000;
        }
        if self.is_annotation {
            flags |= 0x2000;
        }
        if self.is_enum {
            flags |= 0x4000;
        }

        flags
    }
}

impl JvmOpcode {
    /// 转换为字节码值
    pub fn to_byte(&self) -> u8 {
        match self {
            JvmOpcode::Nop => 0x00,
            JvmOpcode::AconstNull => 0x01,
            JvmOpcode::IconstM1 => 0x02,
            JvmOpcode::Iconst0 => 0x03,
            JvmOpcode::Iconst1 => 0x04,
            JvmOpcode::Iconst2 => 0x05,
            JvmOpcode::Iconst3 => 0x06,
            JvmOpcode::Iconst4 => 0x07,
            JvmOpcode::Iconst5 => 0x08,
            JvmOpcode::Lconst0 => 0x09,
            JvmOpcode::Lconst1 => 0x0A,
            JvmOpcode::Fconst0 => 0x0B,
            JvmOpcode::Fconst1 => 0x0C,
            JvmOpcode::Fconst2 => 0x0D,
            JvmOpcode::Dconst0 => 0x0E,
            JvmOpcode::Dconst1 => 0x0F,
            JvmOpcode::Bipush => 0x10,
            JvmOpcode::Sipush => 0x11,
            JvmOpcode::Ldc => 0x12,
            JvmOpcode::LdcW => 0x13,
            JvmOpcode::Ldc2W => 0x14,
            JvmOpcode::Iload => 0x15,
            JvmOpcode::Lload => 0x16,
            JvmOpcode::Fload => 0x17,
            JvmOpcode::Dload => 0x18,
            JvmOpcode::Aload => 0x19,
            JvmOpcode::Iload0 => 0x1A,
            JvmOpcode::Iload1 => 0x1B,
            JvmOpcode::Iload2 => 0x1C,
            JvmOpcode::Iload3 => 0x1D,
            JvmOpcode::Lload0 => 0x1E,
            JvmOpcode::Lload1 => 0x1F,
            JvmOpcode::Lload2 => 0x20,
            JvmOpcode::Lload3 => 0x21,
            JvmOpcode::Fload0 => 0x22,
            JvmOpcode::Fload1 => 0x23,
            JvmOpcode::Fload2 => 0x24,
            JvmOpcode::Fload3 => 0x25,
            JvmOpcode::Dload0 => 0x26,
            JvmOpcode::Dload1 => 0x27,
            JvmOpcode::Dload2 => 0x28,
            JvmOpcode::Dload3 => 0x29,
            JvmOpcode::Aload0 => 0x2A,
            JvmOpcode::Aload1 => 0x2B,
            JvmOpcode::Aload2 => 0x2C,
            JvmOpcode::Aload3 => 0x2D,
            JvmOpcode::Istore => 0x36,
            JvmOpcode::Lstore => 0x37,
            JvmOpcode::Fstore => 0x38,
            JvmOpcode::Dstore => 0x39,
            JvmOpcode::Astore => 0x3A,
            JvmOpcode::Istore0 => 0x3B,
            JvmOpcode::Istore1 => 0x3C,
            JvmOpcode::Istore2 => 0x3D,
            JvmOpcode::Istore3 => 0x3E,
            JvmOpcode::Lstore0 => 0x3F,
            JvmOpcode::Lstore1 => 0x40,
            JvmOpcode::Lstore2 => 0x41,
            JvmOpcode::Lstore3 => 0x42,
            JvmOpcode::Fstore0 => 0x43,
            JvmOpcode::Fstore1 => 0x44,
            JvmOpcode::Fstore2 => 0x45,
            JvmOpcode::Fstore3 => 0x46,
            JvmOpcode::Dstore0 => 0x47,
            JvmOpcode::Dstore1 => 0x48,
            JvmOpcode::Dstore2 => 0x49,
            JvmOpcode::Dstore3 => 0x4A,
            JvmOpcode::Astore0 => 0x4B,
            JvmOpcode::Astore1 => 0x4C,
            JvmOpcode::Astore2 => 0x4D,
            JvmOpcode::Astore3 => 0x4E,
            JvmOpcode::Pop => 0x57,
            JvmOpcode::Pop2 => 0x58,
            JvmOpcode::Dup => 0x59,
            JvmOpcode::DupX1 => 0x5A,
            JvmOpcode::DupX2 => 0x5B,
            JvmOpcode::Dup2 => 0x5C,
            JvmOpcode::Dup2X1 => 0x5D,
            JvmOpcode::Dup2X2 => 0x5E,
            JvmOpcode::Swap => 0x5F,
            JvmOpcode::Iadd => 0x60,
            JvmOpcode::Ladd => 0x61,
            JvmOpcode::Fadd => 0x62,
            JvmOpcode::Dadd => 0x63,
            JvmOpcode::Isub => 0x64,
            JvmOpcode::Lsub => 0x65,
            JvmOpcode::Fsub => 0x66,
            JvmOpcode::Dsub => 0x67,
            JvmOpcode::Imul => 0x68,
            JvmOpcode::Lmul => 0x69,
            JvmOpcode::Fmul => 0x6A,
            JvmOpcode::Dmul => 0x6B,
            JvmOpcode::Idiv => 0x6C,
            JvmOpcode::Ldiv => 0x6D,
            JvmOpcode::Fdiv => 0x6E,
            JvmOpcode::Ddiv => 0x6F,
            JvmOpcode::Irem => 0x70,
            JvmOpcode::Lrem => 0x71,
            JvmOpcode::Frem => 0x72,
            JvmOpcode::Drem => 0x73,
            JvmOpcode::Ineg => 0x74,
            JvmOpcode::Lneg => 0x75,
            JvmOpcode::Fneg => 0x76,
            JvmOpcode::Dneg => 0x77,
            JvmOpcode::Ishl => 0x78,
            JvmOpcode::Lshl => 0x79,
            JvmOpcode::Ishr => 0x7A,
            JvmOpcode::Lshr => 0x7B,
            JvmOpcode::Iushr => 0x7C,
            JvmOpcode::Lushr => 0x7D,
            JvmOpcode::Iand => 0x7E,
            JvmOpcode::Land => 0x7F,
            JvmOpcode::Ior => 0x80,
            JvmOpcode::Lor => 0x81,
            JvmOpcode::Ixor => 0x82,
            JvmOpcode::Lxor => 0x83,
            JvmOpcode::Lcmp => 0x94,
            JvmOpcode::Fcmpl => 0x95,
            JvmOpcode::Fcmpg => 0x96,
            JvmOpcode::Dcmpl => 0x97,
            JvmOpcode::Dcmpg => 0x98,
            JvmOpcode::Ifeq => 0x99,
            JvmOpcode::Ifne => 0x9A,
            JvmOpcode::Iflt => 0x9B,
            JvmOpcode::Ifge => 0x9C,
            JvmOpcode::Ifgt => 0x9D,
            JvmOpcode::Ifle => 0x9E,
            JvmOpcode::IfIcmpeq => 0x9F,
            JvmOpcode::IfIcmpne => 0xA0,
            JvmOpcode::IfIcmplt => 0xA1,
            JvmOpcode::IfIcmpge => 0xA2,
            JvmOpcode::IfIcmpgt => 0xA3,
            JvmOpcode::IfIcmple => 0xA4,
            JvmOpcode::IfAcmpeq => 0xA5,
            JvmOpcode::IfAcmpne => 0xA6,
            JvmOpcode::Goto => 0xA7,
            JvmOpcode::Jsr => 0xA8,
            JvmOpcode::Ret => 0xA9,
            JvmOpcode::Ireturn => 0xAC,
            JvmOpcode::Lreturn => 0xAD,
            JvmOpcode::Freturn => 0xAE,
            JvmOpcode::Dreturn => 0xAF,
            JvmOpcode::Areturn => 0xB0,
            JvmOpcode::Return => 0xB1,
            JvmOpcode::Getstatic => 0xB2,
            JvmOpcode::Putstatic => 0xB3,
            JvmOpcode::Getfield => 0xB4,
            JvmOpcode::Putfield => 0xB5,
            JvmOpcode::Invokevirtual => 0xB6,
            JvmOpcode::Invokespecial => 0xB7,
            JvmOpcode::Invokestatic => 0xB8,
            JvmOpcode::Invokeinterface => 0xB9,
            JvmOpcode::Invokedynamic => 0xBA,
            JvmOpcode::New => 0xBB,
            JvmOpcode::Newarray => 0xBC,
            JvmOpcode::Anewarray => 0xBD,
            JvmOpcode::Arraylength => 0xBE,
            JvmOpcode::Athrow => 0xBF,
            JvmOpcode::Checkcast => 0xC0,
            JvmOpcode::Instanceof => 0xC1,
            JvmOpcode::Monitorenter => 0xC2,
            JvmOpcode::Monitorexit => 0xC3,
            JvmOpcode::Wide => 0xC4,
            JvmOpcode::Multianewarray => 0xC5,
            JvmOpcode::Ifnull => 0xC6,
            JvmOpcode::Ifnonnull => 0xC7,
            JvmOpcode::GotoW => 0xC8,
            JvmOpcode::JsrW => 0xC9,
        }
    }

    /// 从字符串解析操作码
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "nop" => Some(JvmOpcode::Nop),
            "aconst_null" => Some(JvmOpcode::AconstNull),
            "iconst_m1" => Some(JvmOpcode::IconstM1),
            "iconst_0" => Some(JvmOpcode::Iconst0),
            "iconst_1" => Some(JvmOpcode::Iconst1),
            "iconst_2" => Some(JvmOpcode::Iconst2),
            "iconst_3" => Some(JvmOpcode::Iconst3),
            "iconst_4" => Some(JvmOpcode::Iconst4),
            "iconst_5" => Some(JvmOpcode::Iconst5),
            "lconst_0" => Some(JvmOpcode::Lconst0),
            "lconst_1" => Some(JvmOpcode::Lconst1),
            "fconst_0" => Some(JvmOpcode::Fconst0),
            "fconst_1" => Some(JvmOpcode::Fconst1),
            "fconst_2" => Some(JvmOpcode::Fconst2),
            "dconst_0" => Some(JvmOpcode::Dconst0),
            "dconst_1" => Some(JvmOpcode::Dconst1),
            "bipush" => Some(JvmOpcode::Bipush),
            "sipush" => Some(JvmOpcode::Sipush),
            "ldc" => Some(JvmOpcode::Ldc),
            "ldc_w" => Some(JvmOpcode::LdcW),
            "ldc2_w" => Some(JvmOpcode::Ldc2W),
            "iload" => Some(JvmOpcode::Iload),
            "lload" => Some(JvmOpcode::Lload),
            "fload" => Some(JvmOpcode::Fload),
            "dload" => Some(JvmOpcode::Dload),
            "aload" => Some(JvmOpcode::Aload),
            "iload_0" => Some(JvmOpcode::Iload0),
            "iload_1" => Some(JvmOpcode::Iload1),
            "iload_2" => Some(JvmOpcode::Iload2),
            "iload_3" => Some(JvmOpcode::Iload3),
            "aload_0" => Some(JvmOpcode::Aload0),
            "aload_1" => Some(JvmOpcode::Aload1),
            "aload_2" => Some(JvmOpcode::Aload2),
            "aload_3" => Some(JvmOpcode::Aload3),
            "istore" => Some(JvmOpcode::Istore),
            "astore" => Some(JvmOpcode::Astore),
            "istore_0" => Some(JvmOpcode::Istore0),
            "istore_1" => Some(JvmOpcode::Istore1),
            "istore_2" => Some(JvmOpcode::Istore2),
            "istore_3" => Some(JvmOpcode::Istore3),
            "astore_0" => Some(JvmOpcode::Astore0),
            "astore_1" => Some(JvmOpcode::Astore1),
            "astore_2" => Some(JvmOpcode::Astore2),
            "astore_3" => Some(JvmOpcode::Astore3),
            "pop" => Some(JvmOpcode::Pop),
            "dup" => Some(JvmOpcode::Dup),
            "iadd" => Some(JvmOpcode::Iadd),
            "isub" => Some(JvmOpcode::Isub),
            "imul" => Some(JvmOpcode::Imul),
            "idiv" => Some(JvmOpcode::Idiv),
            "irem" => Some(JvmOpcode::Irem),
            "ineg" => Some(JvmOpcode::Ineg),
            "ireturn" => Some(JvmOpcode::Ireturn),
            "lreturn" => Some(JvmOpcode::Lreturn),
            "freturn" => Some(JvmOpcode::Freturn),
            "dreturn" => Some(JvmOpcode::Dreturn),
            "areturn" => Some(JvmOpcode::Areturn),
            "return" => Some(JvmOpcode::Return),
            "getstatic" => Some(JvmOpcode::Getstatic),
            "putstatic" => Some(JvmOpcode::Putstatic),
            "getfield" => Some(JvmOpcode::Getfield),
            "putfield" => Some(JvmOpcode::Putfield),
            "invokevirtual" => Some(JvmOpcode::Invokevirtual),
            "invokespecial" => Some(JvmOpcode::Invokespecial),
            "invokestatic" => Some(JvmOpcode::Invokestatic),
            "invokeinterface" => Some(JvmOpcode::Invokeinterface),
            "invokedynamic" => Some(JvmOpcode::Invokedynamic),
            "new" => Some(JvmOpcode::New),
            "newarray" => Some(JvmOpcode::Newarray),
            "anewarray" => Some(JvmOpcode::Anewarray),
            "arraylength" => Some(JvmOpcode::Arraylength),
            "athrow" => Some(JvmOpcode::Athrow),
            "checkcast" => Some(JvmOpcode::Checkcast),
            "instanceof" => Some(JvmOpcode::Instanceof),
            _ => None,
        }
    }
}

// 为了兼容性，保留原有的类型定义
pub use JvmConstantPoolEntry as ConstantPoolEntry;

/// 向后兼容的 JvmInstruction 结构
#[derive(Debug, Clone, PartialEq)]
pub struct JvmInstructionCompat {
    /// 操作码
    pub opcode: u8,
    /// 操作数
    pub operands: Vec<u8>,
    /// 元数据（用于调试和注释）
    pub metadata: Option<String>,
}

impl JvmInstructionCompat {
    /// 创建新的 JVM 指令
    pub fn new(opcode: u8, operands: Vec<u8>) -> Self {
        Self { opcode, operands, metadata: None }
    }

    /// 创建带元数据的 JVM 指令
    pub fn with_metadata(opcode: u8, operands: Vec<u8>, metadata: String) -> Self {
        Self { opcode, operands, metadata: Some(metadata) }
    }

    /// 获取指令的字节表示
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![self.opcode];
        bytes.extend_from_slice(&self.operands);
        bytes
    }

    /// 获取指令长度
    pub fn len(&self) -> usize {
        1 + self.operands.len()
    }

    /// 检查指令是否为空
    pub fn is_empty(&self) -> bool {
        false // JVM 指令至少有一个操作码
    }
}

/// 向后兼容的 JvmClass 结构
#[derive(Debug, Clone)]
pub struct JvmClassCompat {
    /// 魔数
    pub magic: u32,
    /// 次版本号
    pub minor_version: u16,
    /// 主版本号
    pub major_version: u16,
    /// 常量池
    pub constant_pool: Vec<ConstantPoolEntry>,
    /// 访问标志
    pub access_flags: u16,
    /// 类名索引
    pub this_class: u16,
    /// 超类名索引
    pub super_class: u16,
    /// 接口索引列表
    pub interfaces: Vec<u16>,
    /// 字段列表
    pub fields: Vec<JvmFieldCompat>,
    /// 方法列表
    pub methods: Vec<JvmMethodCompat>,
}

impl Default for JvmClassCompat {
    fn default() -> Self {
        Self {
            magic: 0xCAFEBABE,
            minor_version: 0,
            major_version: 65, // Java 21
            constant_pool: Vec::new(),
            access_flags: 0x0021, // ACC_PUBLIC | ACC_SUPER
            this_class: 0,
            super_class: 0,
            interfaces: Vec::new(),
            fields: Vec::new(),
            methods: Vec::new(),
        }
    }
}

/// 向后兼容的 JvmMethod 结构
#[derive(Debug, Clone)]
pub struct JvmMethodCompat {
    /// 访问标志
    pub access_flags: u16,
    /// 方法名索引
    pub name_index: u16,
    /// 描述符索引
    pub descriptor_index: u16,
    /// 指令列表
    pub instructions: Vec<JvmInstructionCompat>,
    /// 最大栈深度
    pub max_stack: u16,
    /// 最大局部变量数
    pub max_locals: u16,
}

impl JvmMethodCompat {
    /// 创建新的方法
    pub fn new(name_index: u16, descriptor_index: u16) -> Self {
        Self {
            access_flags: 0x0001, // ACC_PUBLIC
            name_index,
            descriptor_index,
            instructions: Vec::new(),
            max_stack: 2,
            max_locals: 1,
        }
    }

    /// 添加指令
    pub fn add_instruction(&mut self, instruction: JvmInstructionCompat) {
        self.instructions.push(instruction);
    }

    /// 获取代码长度
    pub fn code_length(&self) -> u32 {
        self.instructions.iter().map(|i| i.len() as u32).sum()
    }
}

/// 向后兼容的 JvmField 结构
#[derive(Debug, Clone)]
pub struct JvmFieldCompat {
    /// 访问标志
    pub access_flags: u16,
    /// 字段名索引
    pub name_index: u16,
    /// 描述符索引
    pub descriptor_index: u16,
}

/// 操作码常量模块
pub mod opcodes {
    pub const ACONST_NULL: u8 = 0x01;
    pub const ICONST_M1: u8 = 0x02;
    pub const ICONST_0: u8 = 0x03;
    pub const ICONST_1: u8 = 0x04;
    pub const ICONST_2: u8 = 0x05;
    pub const ICONST_3: u8 = 0x06;
    pub const ICONST_4: u8 = 0x07;
    pub const ICONST_5: u8 = 0x08;
    pub const LCONST_0: u8 = 0x09;
    pub const LCONST_1: u8 = 0x0A;
    pub const FCONST_0: u8 = 0x0B;
    pub const FCONST_1: u8 = 0x0C;
    pub const FCONST_2: u8 = 0x0D;
    pub const DCONST_0: u8 = 0x0E;
    pub const DCONST_1: u8 = 0x0F;
    pub const BIPUSH: u8 = 0x10;
    pub const SIPUSH: u8 = 0x11;
    pub const LDC: u8 = 0x12;
    pub const LDC_W: u8 = 0x13;
    pub const LDC2_W: u8 = 0x14;

    // 局部变量加载指令
    pub const ILOAD: u8 = 0x15;
    pub const LLOAD: u8 = 0x16;
    pub const FLOAD: u8 = 0x17;
    pub const DLOAD: u8 = 0x18;
    pub const ALOAD: u8 = 0x19;
    pub const ILOAD_0: u8 = 0x1A;
    pub const ILOAD_1: u8 = 0x1B;
    pub const ILOAD_2: u8 = 0x1C;
    pub const ILOAD_3: u8 = 0x1D;
    pub const LLOAD_0: u8 = 0x1E;
    pub const LLOAD_1: u8 = 0x1F;
    pub const LLOAD_2: u8 = 0x20;
    pub const LLOAD_3: u8 = 0x21;
    pub const FLOAD_0: u8 = 0x22;
    pub const FLOAD_1: u8 = 0x23;
    pub const FLOAD_2: u8 = 0x24;
    pub const FLOAD_3: u8 = 0x25;
    pub const DLOAD_0: u8 = 0x26;
    pub const DLOAD_1: u8 = 0x27;
    pub const DLOAD_2: u8 = 0x28;
    pub const DLOAD_3: u8 = 0x29;
    pub const ALOAD_0: u8 = 0x2A;
    pub const ALOAD_1: u8 = 0x2B;
    pub const ALOAD_2: u8 = 0x2C;
    pub const ALOAD_3: u8 = 0x2D;

    // 局部变量存储指令
    pub const ISTORE: u8 = 0x36;
    pub const LSTORE: u8 = 0x37;
    pub const FSTORE: u8 = 0x38;
    pub const DSTORE: u8 = 0x39;
    pub const ASTORE: u8 = 0x3A;
    pub const ISTORE_0: u8 = 0x3B;
    pub const ISTORE_1: u8 = 0x3C;
    pub const ISTORE_2: u8 = 0x3D;
    pub const ISTORE_3: u8 = 0x3E;
    pub const LSTORE_0: u8 = 0x3F;
    pub const LSTORE_1: u8 = 0x40;
    pub const LSTORE_2: u8 = 0x41;
    pub const LSTORE_3: u8 = 0x42;
    pub const FSTORE_0: u8 = 0x43;
    pub const FSTORE_1: u8 = 0x44;
    pub const FSTORE_2: u8 = 0x45;
    pub const FSTORE_3: u8 = 0x46;
    pub const DSTORE_0: u8 = 0x47;
    pub const DSTORE_1: u8 = 0x48;
    pub const DSTORE_2: u8 = 0x49;
    pub const DSTORE_3: u8 = 0x4A;
    pub const ASTORE_0: u8 = 0x4B;
    pub const ASTORE_1: u8 = 0x4C;
    pub const ASTORE_2: u8 = 0x4D;
    pub const ASTORE_3: u8 = 0x4E;

    // 栈操作指令
    pub const POP: u8 = 0x57;
    pub const POP2: u8 = 0x58;
    pub const DUP: u8 = 0x59;
    pub const DUP_X1: u8 = 0x5A;
    pub const DUP_X2: u8 = 0x5B;
    pub const DUP2: u8 = 0x5C;
    pub const DUP2_X1: u8 = 0x5D;
    pub const DUP2_X2: u8 = 0x5E;
    pub const SWAP: u8 = 0x5F;

    // 算术指令
    pub const IADD: u8 = 0x60;
    pub const LADD: u8 = 0x61;
    pub const FADD: u8 = 0x62;
    pub const DADD: u8 = 0x63;
    pub const ISUB: u8 = 0x64;
    pub const LSUB: u8 = 0x65;
    pub const FSUB: u8 = 0x66;
    pub const DSUB: u8 = 0x67;
    pub const IMUL: u8 = 0x68;
    pub const LMUL: u8 = 0x69;
    pub const FMUL: u8 = 0x6A;
    pub const DMUL: u8 = 0x6B;
    pub const IDIV: u8 = 0x6C;
    pub const LDIV: u8 = 0x6D;
    pub const FDIV: u8 = 0x6E;
    pub const DDIV: u8 = 0x6F;

    // 方法调用指令
    pub const INVOKEVIRTUAL: u8 = 0xB6;
    pub const INVOKESPECIAL: u8 = 0xB7;
    pub const INVOKESTATIC: u8 = 0xB8;
    pub const INVOKEINTERFACE: u8 = 0xB9;

    // 返回指令
    pub const IRETURN: u8 = 0xAC;
    pub const LRETURN: u8 = 0xAD;
    pub const FRETURN: u8 = 0xAE;
    pub const DRETURN: u8 = 0xAF;
    pub const ARETURN: u8 = 0xB0;
    pub const RETURN: u8 = 0xB1;

    // 字段访问指令
    pub const GETSTATIC: u8 = 0xB2;
    pub const PUTSTATIC: u8 = 0xB3;
    pub const GETFIELD: u8 = 0x84;
    pub const PUTFIELD: u8 = 0xB5;

    // 其他指令
    pub const NOP: u8 = 0x00;
}
