#![doc = include_str!("readme.md")]
use gaia_types::{GaiaError, Result};
use serde::{Deserialize, Serialize};

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
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
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
    /// 是否为 volatile
    pub is_volatile: bool,
    /// 是否为 transient
    pub is_transient: bool,
}

impl JvmAccessFlags {
    /// 访问标志常量
    pub const PUBLIC: JvmAccessFlags = JvmAccessFlags {
        is_public: true,
        is_final: false,
        is_super: false,
        is_interface: false,
        is_abstract: false,
        is_synthetic: false,
        is_annotation: false,
        is_enum: false,
        is_static: false,
        is_private: false,
        is_protected: false,
        is_volatile: false,
        is_transient: false,
    };

    pub const PRIVATE: JvmAccessFlags = JvmAccessFlags {
        is_public: false,
        is_final: false,
        is_super: false,
        is_interface: false,
        is_abstract: false,
        is_synthetic: false,
        is_annotation: false,
        is_enum: false,
        is_static: false,
        is_private: true,
        is_protected: false,
        is_volatile: false,
        is_transient: false,
    };

    pub const PROTECTED: JvmAccessFlags = JvmAccessFlags {
        is_public: false,
        is_final: false,
        is_super: false,
        is_interface: false,
        is_abstract: false,
        is_synthetic: false,
        is_annotation: false,
        is_enum: false,
        is_static: false,
        is_private: false,
        is_protected: true,
        is_volatile: false,
        is_transient: false,
    };

    pub const STATIC: JvmAccessFlags = JvmAccessFlags {
        is_public: false,
        is_final: false,
        is_super: false,
        is_interface: false,
        is_abstract: false,
        is_synthetic: false,
        is_annotation: false,
        is_enum: false,
        is_static: true,
        is_private: false,
        is_protected: false,
        is_volatile: false,
        is_transient: false,
    };

    pub const FINAL: JvmAccessFlags = JvmAccessFlags {
        is_public: false,
        is_final: true,
        is_super: false,
        is_interface: false,
        is_abstract: false,
        is_synthetic: false,
        is_annotation: false,
        is_enum: false,
        is_static: false,
        is_private: false,
        is_protected: false,
        is_volatile: false,
        is_transient: false,
    };

    pub const ABSTRACT: JvmAccessFlags = JvmAccessFlags {
        is_public: false,
        is_final: false,
        is_super: false,
        is_interface: false,
        is_abstract: true,
        is_synthetic: false,
        is_annotation: false,
        is_enum: false,
        is_static: false,
        is_private: false,
        is_protected: false,
        is_volatile: false,
        is_transient: false,
    };

    pub const VOLATILE: JvmAccessFlags = JvmAccessFlags {
        is_public: false,
        is_final: false,
        is_super: false,
        is_interface: false,
        is_abstract: false,
        is_synthetic: false,
        is_annotation: false,
        is_enum: false,
        is_static: false,
        is_private: false,
        is_protected: false,
        is_volatile: true,
        is_transient: false,
    };

    pub const TRANSIENT: JvmAccessFlags = JvmAccessFlags {
        is_public: false,
        is_final: false,
        is_super: false,
        is_interface: false,
        is_abstract: false,
        is_synthetic: false,
        is_annotation: false,
        is_enum: false,
        is_static: false,
        is_private: false,
        is_protected: false,
        is_volatile: false,
        is_transient: true,
    };
}

use std::ops::BitOrAssign;

impl BitOrAssign for JvmAccessFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.is_public |= rhs.is_public;
        self.is_final |= rhs.is_final;
        self.is_super |= rhs.is_super;
        self.is_interface |= rhs.is_interface;
        self.is_abstract |= rhs.is_abstract;
        self.is_synthetic |= rhs.is_synthetic;
        self.is_annotation |= rhs.is_annotation;
        self.is_enum |= rhs.is_enum;
        self.is_static |= rhs.is_static;
        self.is_private |= rhs.is_private;
        self.is_protected |= rhs.is_protected;
        self.is_volatile |= rhs.is_volatile;
        self.is_transient |= rhs.is_transient;
    }
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
    Nop,
    Utf8 {
        value: String,
    },
    /// 整数常量
    Integer {
        value: i32,
    },
    /// 浮点数常量
    Float {
        value: f32,
    },
    /// 长整数常量
    Long {
        value: i64,
    },
    /// 双精度浮点数常量
    Double {
        value: f64,
    },
    /// 类引用
    Class {
        name: String,
    },
    /// 字符串引用
    String {
        value: String,
    },
    /// 字段引用
    Fieldref {
        class_name: String,
        name: String,
        descriptor: String,
    },
    /// 方法引用
    Methodref {
        class_name: String,
        name: String,
        descriptor: String,
    },
    /// 接口方法引用
    InterfaceMethodref {
        class_name: String,
        name: String,
        descriptor: String,
    },
    /// 名称和类型
    NameAndType {
        name: String,
        descriptor: String,
    },
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
    
    // 带立即数的指令
    Bipush { value: i8 },
    Sipush { value: i16 },
    
    // 常量池引用指令
    Ldc { symbol: String },
    LdcW { symbol: String },
    Ldc2W { symbol: String },

    // 局部变量加载指令
    Iload { index: u16 },
    Lload { index: u16 },
    Fload { index: u16 },
    Dload { index: u16 },
    Aload { index: u16 },
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
    Istore { index: u16 },
    Lstore { index: u16 },
    Fstore { index: u16 },
    Dstore { index: u16 },
    Astore { index: u16 },
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
    Ifeq { target: String },
    Ifne { target: String },
    Iflt { target: String },
    Ifge { target: String },
    Ifgt { target: String },
    Ifle { target: String },
    IfIcmpeq { target: String },
    IfIcmpne { target: String },
    IfIcmplt { target: String },
    IfIcmpge { target: String },
    IfIcmpgt { target: String },
    IfIcmple { target: String },
    IfAcmpeq { target: String },
    IfAcmpne { target: String },
    Goto { target: String },
    Jsr { target: String },
    Ret { index: u16 },

    // 返回指令
    Ireturn,
    Lreturn,
    Freturn,
    Dreturn,
    Areturn,
    Return,

    // 字段访问指令
    Getstatic { class_name: String, field_name: String, descriptor: String },
    Putstatic { class_name: String, field_name: String, descriptor: String },
    Getfield { class_name: String, field_name: String, descriptor: String },
    Putfield { class_name: String, field_name: String, descriptor: String },

    // 方法调用指令
    Invokevirtual { class_name: String, method_name: String, descriptor: String },
    Invokespecial { class_name: String, method_name: String, descriptor: String },
    Invokestatic { class_name: String, method_name: String, descriptor: String },
    Invokeinterface { class_name: String, method_name: String, descriptor: String },
    Invokedynamic { class_name: String, method_name: String, descriptor: String },

    // 对象操作指令
    New { class_name: String },
    Newarray { atype: u8 },
    Anewarray { class_name: String },
    Arraylength,
    Athrow,
    Checkcast { class_name: String },
    Instanceof { class_name: String },
    Monitorenter,
    Monitorexit,

    // 其他指令
    Wide,
    Multianewarray { class_name: String, dimensions: u8 },
    Ifnull { target: String },
    Ifnonnull { target: String },
    GotoW { target: String },
    JsrW { target: String },
}

impl JvmInstruction {
    /// 获取指令的字节码值
    pub fn to_byte(&self) -> u8 {
        match self {
            JvmInstruction::Nop => 0x00,
            JvmInstruction::AconstNull => 0x01,
            JvmInstruction::IconstM1 => 0x02,
            JvmInstruction::Iconst0 => 0x03,
            JvmInstruction::Iconst1 => 0x04,
            JvmInstruction::Iconst2 => 0x05,
            JvmInstruction::Iconst3 => 0x06,
            JvmInstruction::Iconst4 => 0x07,
            JvmInstruction::Iconst5 => 0x08,
            JvmInstruction::Lconst0 => 0x09,
            JvmInstruction::Lconst1 => 0x0A,
            JvmInstruction::Fconst0 => 0x0B,
            JvmInstruction::Fconst1 => 0x0C,
            JvmInstruction::Fconst2 => 0x0D,
            JvmInstruction::Dconst0 => 0x0E,
            JvmInstruction::Dconst1 => 0x0F,
            JvmInstruction::Bipush { .. } => 0x10,
            JvmInstruction::Sipush { .. } => 0x11,
            JvmInstruction::Ldc { .. } => 0x12,
            JvmInstruction::LdcW { .. } => 0x13,
            JvmInstruction::Ldc2W { .. } => 0x14,
            JvmInstruction::Iload { .. } => 0x15,
            JvmInstruction::Lload { .. } => 0x16,
            JvmInstruction::Fload { .. } => 0x17,
            JvmInstruction::Dload { .. } => 0x18,
            JvmInstruction::Aload { .. } => 0x19,
            JvmInstruction::Iload0 => 0x1A,
            JvmInstruction::Iload1 => 0x1B,
            JvmInstruction::Iload2 => 0x1C,
            JvmInstruction::Iload3 => 0x1D,
            JvmInstruction::Lload0 => 0x1E,
            JvmInstruction::Lload1 => 0x1F,
            JvmInstruction::Lload2 => 0x20,
            JvmInstruction::Lload3 => 0x21,
            JvmInstruction::Fload0 => 0x22,
            JvmInstruction::Fload1 => 0x23,
            JvmInstruction::Fload2 => 0x24,
            JvmInstruction::Fload3 => 0x25,
            JvmInstruction::Dload0 => 0x26,
            JvmInstruction::Dload1 => 0x27,
            JvmInstruction::Dload2 => 0x28,
            JvmInstruction::Dload3 => 0x29,
            JvmInstruction::Aload0 => 0x2A,
            JvmInstruction::Aload1 => 0x2B,
            JvmInstruction::Aload2 => 0x2C,
            JvmInstruction::Aload3 => 0x2D,
            JvmInstruction::Istore { .. } => 0x36,
            JvmInstruction::Lstore { .. } => 0x37,
            JvmInstruction::Fstore { .. } => 0x38,
            JvmInstruction::Dstore { .. } => 0x39,
            JvmInstruction::Astore { .. } => 0x3A,
            JvmInstruction::Istore0 => 0x3B,
            JvmInstruction::Istore1 => 0x3C,
            JvmInstruction::Istore2 => 0x3D,
            JvmInstruction::Istore3 => 0x3E,
            JvmInstruction::Lstore0 => 0x3F,
            JvmInstruction::Lstore1 => 0x40,
            JvmInstruction::Lstore2 => 0x41,
            JvmInstruction::Lstore3 => 0x42,
            JvmInstruction::Fstore0 => 0x43,
            JvmInstruction::Fstore1 => 0x44,
            JvmInstruction::Fstore2 => 0x45,
            JvmInstruction::Fstore3 => 0x46,
            JvmInstruction::Dstore0 => 0x47,
            JvmInstruction::Dstore1 => 0x48,
            JvmInstruction::Dstore2 => 0x49,
            JvmInstruction::Dstore3 => 0x4A,
            JvmInstruction::Astore0 => 0x4B,
            JvmInstruction::Astore1 => 0x4C,
            JvmInstruction::Astore2 => 0x4D,
            JvmInstruction::Astore3 => 0x4E,
            JvmInstruction::Pop => 0x57,
            JvmInstruction::Pop2 => 0x58,
            JvmInstruction::Dup => 0x59,
            JvmInstruction::DupX1 => 0x5A,
            JvmInstruction::DupX2 => 0x5B,
            JvmInstruction::Dup2 => 0x5C,
            JvmInstruction::Dup2X1 => 0x5D,
            JvmInstruction::Dup2X2 => 0x5E,
            JvmInstruction::Swap => 0x5F,
            JvmInstruction::Iadd => 0x60,
            JvmInstruction::Ladd => 0x61,
            JvmInstruction::Fadd => 0x62,
            JvmInstruction::Dadd => 0x63,
            JvmInstruction::Isub => 0x64,
            JvmInstruction::Lsub => 0x65,
            JvmInstruction::Fsub => 0x66,
            JvmInstruction::Dsub => 0x67,
            JvmInstruction::Imul => 0x68,
            JvmInstruction::Lmul => 0x69,
            JvmInstruction::Fmul => 0x6A,
            JvmInstruction::Dmul => 0x6B,
            JvmInstruction::Idiv => 0x6C,
            JvmInstruction::Ldiv => 0x6D,
            JvmInstruction::Fdiv => 0x6E,
            JvmInstruction::Ddiv => 0x6F,
            JvmInstruction::Irem => 0x70,
            JvmInstruction::Lrem => 0x71,
            JvmInstruction::Frem => 0x72,
            JvmInstruction::Drem => 0x73,
            JvmInstruction::Ineg => 0x74,
            JvmInstruction::Lneg => 0x75,
            JvmInstruction::Fneg => 0x76,
            JvmInstruction::Dneg => 0x77,
            JvmInstruction::Ishl => 0x78,
            JvmInstruction::Lshl => 0x79,
            JvmInstruction::Ishr => 0x7A,
            JvmInstruction::Lshr => 0x7B,
            JvmInstruction::Iushr => 0x7C,
            JvmInstruction::Lushr => 0x7D,
            JvmInstruction::Iand => 0x7E,
            JvmInstruction::Land => 0x7F,
            JvmInstruction::Ior => 0x80,
            JvmInstruction::Lor => 0x81,
            JvmInstruction::Ixor => 0x82,
            JvmInstruction::Lxor => 0x83,
            JvmInstruction::Lcmp => 0x94,
            JvmInstruction::Fcmpl => 0x95,
            JvmInstruction::Fcmpg => 0x96,
            JvmInstruction::Dcmpl => 0x97,
            JvmInstruction::Dcmpg => 0x98,
            JvmInstruction::Ifeq { .. } => 0x99,
            JvmInstruction::Ifne { .. } => 0x9A,
            JvmInstruction::Iflt { .. } => 0x9B,
            JvmInstruction::Ifge { .. } => 0x9C,
            JvmInstruction::Ifgt { .. } => 0x9D,
            JvmInstruction::Ifle { .. } => 0x9E,
            JvmInstruction::IfIcmpeq { .. } => 0x9F,
            JvmInstruction::IfIcmpne { .. } => 0xA0,
            JvmInstruction::IfIcmplt { .. } => 0xA1,
            JvmInstruction::IfIcmpge { .. } => 0xA2,
            JvmInstruction::IfIcmpgt { .. } => 0xA3,
            JvmInstruction::IfIcmple { .. } => 0xA4,
            JvmInstruction::IfAcmpeq { .. } => 0xA5,
            JvmInstruction::IfAcmpne { .. } => 0xA6,
            JvmInstruction::Goto { .. } => 0xA7,
            JvmInstruction::Jsr { .. } => 0xA8,
            JvmInstruction::Ret { .. } => 0xA9,
            JvmInstruction::Ireturn => 0xAC,
            JvmInstruction::Lreturn => 0xAD,
            JvmInstruction::Freturn => 0xAE,
            JvmInstruction::Dreturn => 0xAF,
            JvmInstruction::Areturn => 0xB0,
            JvmInstruction::Return => 0xB1,
            JvmInstruction::Getstatic { .. } => 0xB2,
            JvmInstruction::Putstatic { .. } => 0xB3,
            JvmInstruction::Getfield { .. } => 0xB4,
            JvmInstruction::Putfield { .. } => 0xB5,
            JvmInstruction::Invokevirtual { .. } => 0xB6,
            JvmInstruction::Invokespecial { .. } => 0xB7,
            JvmInstruction::Invokestatic { .. } => 0xB8,
            JvmInstruction::Invokeinterface { .. } => 0xB9,
            JvmInstruction::Invokedynamic { .. } => 0xBA,
            JvmInstruction::New { .. } => 0xBB,
            JvmInstruction::Newarray { .. } => 0xBC,
            JvmInstruction::Anewarray { .. } => 0xBD,
            JvmInstruction::Arraylength => 0xBE,
            JvmInstruction::Athrow => 0xBF,
            JvmInstruction::Checkcast { .. } => 0xC0,
            JvmInstruction::Instanceof { .. } => 0xC1,
            JvmInstruction::Monitorenter => 0xC2,
            JvmInstruction::Monitorexit => 0xC3,
            JvmInstruction::Wide => 0xC4,
            JvmInstruction::Multianewarray { .. } => 0xC5,
            JvmInstruction::Ifnull { .. } => 0xC6,
            JvmInstruction::Ifnonnull { .. } => 0xC7,
            JvmInstruction::GotoW { .. } => 0xC8,
            JvmInstruction::JsrW { .. } => 0xC9,
        }
    }
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
    /// Signature 属性
    Signature { signature: String },
    /// 异常属性
    Exceptions { exceptions: Vec<String> },
    /// 行号表属性
    LineNumberTable { entries: Vec<(u16, u16)> },
    /// 局部变量表属性
    LocalVariableTable { entries: Vec<JvmLocalVariable> },
    /// 未知属性
    Unknown { name: String, data: Vec<u8> },
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

    /// 设置访问标志
    pub fn with_access_flags(mut self, access_flags: JvmAccessFlags) -> Self {
        self.access_flags = access_flags;
        self
    }

    /// 添加 public 访问修饰符
    pub fn with_public(mut self) -> Self {
        self.access_flags |= JvmAccessFlags::PUBLIC;
        self
    }

    /// 添加 private 访问修饰符
    pub fn with_private(mut self) -> Self {
        self.access_flags |= JvmAccessFlags::PRIVATE;
        self
    }

    /// 添加 protected 访问修饰符
    pub fn with_protected(mut self) -> Self {
        self.access_flags |= JvmAccessFlags::PROTECTED;
        self
    }

    /// 添加 static 修饰符
    pub fn with_static(mut self) -> Self {
        self.access_flags |= JvmAccessFlags::STATIC;
        self
    }

    /// 添加 final 修饰符
    pub fn with_final(mut self) -> Self {
        self.access_flags |= JvmAccessFlags::FINAL;
        self
    }

    /// 添加 abstract 修饰符
    pub fn with_abstract(mut self) -> Self {
        self.access_flags |= JvmAccessFlags::ABSTRACT;
        self
    }

    /// 设置最大栈深度
    pub fn with_max_stack(mut self, max_stack: u16) -> Self {
        self.max_stack = max_stack;
        self
    }

    /// 设置最大局部变量数
    pub fn with_max_locals(mut self, max_locals: u16) -> Self {
        self.max_locals = max_locals;
        self
    }

    /// 添加指令
    pub fn with_instruction(mut self, instruction: JvmInstruction) -> Self {
        self.instructions.push(instruction);
        self
    }

    /// 添加多个指令
    pub fn with_instructions(mut self, instructions: Vec<JvmInstruction>) -> Self {
        self.instructions.extend(instructions);
        self
    }

    /// 添加异常处理器
    pub fn with_exception_handler(mut self, handler: JvmExceptionHandler) -> Self {
        self.exception_table.push(handler);
        self
    }

    /// 添加属性
    pub fn with_attribute(mut self, attribute: JvmAttribute) -> Self {
        self.attributes.push(attribute);
        self
    }

    /// 添加多个属性
    pub fn with_attributes(mut self, attributes: Vec<JvmAttribute>) -> Self {
        self.attributes.extend(attributes);
        self
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

    /// 设置访问标志
    pub fn with_access_flags(mut self, access_flags: JvmAccessFlags) -> Self {
        self.access_flags = access_flags;
        self
    }

    /// 添加 public 访问修饰符
    pub fn with_public(mut self) -> Self {
        self.access_flags |= JvmAccessFlags::PUBLIC;
        self
    }

    /// 添加 private 访问修饰符
    pub fn with_private(mut self) -> Self {
        self.access_flags |= JvmAccessFlags::PRIVATE;
        self
    }

    /// 添加 protected 访问修饰符
    pub fn with_protected(mut self) -> Self {
        self.access_flags |= JvmAccessFlags::PROTECTED;
        self
    }

    /// 添加 static 修饰符
    pub fn with_static(mut self) -> Self {
        self.access_flags |= JvmAccessFlags::STATIC;
        self
    }

    /// 添加 final 修饰符
    pub fn with_final(mut self) -> Self {
        self.access_flags |= JvmAccessFlags::FINAL;
        self
    }

    /// 添加 volatile 修饰符
    pub fn with_volatile(mut self) -> Self {
        self.access_flags |= JvmAccessFlags::VOLATILE;
        self
    }

    /// 添加 transient 修饰符
    pub fn with_transient(mut self) -> Self {
        self.access_flags |= JvmAccessFlags::TRANSIENT;
        self
    }

    /// 设置常量值
    pub fn with_constant_value(mut self, value: JvmConstantPoolEntry) -> Self {
        self.constant_value = Some(value);
        self
    }

    /// 添加属性
    pub fn with_attribute(mut self, attribute: JvmAttribute) -> Self {
        self.attributes.push(attribute);
        self
    }

    /// 添加多个属性
    pub fn with_attributes(mut self, attributes: Vec<JvmAttribute>) -> Self {
        self.attributes.extend(attributes);
        self
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
            is_volatile: false,
            is_transient: false,
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
                "volatile" => flags.is_volatile = true,
                "transient" => flags.is_transient = true,
                _ => {} // 忽略未知修饰符
            }
        }

        flags
    }

    /// 从 JVM 字节码中的访问标志值创建访问标志
    pub fn from_flags(flags: u16) -> Self {
        Self {
            is_public: (flags & 0x0001) != 0,
            is_private: (flags & 0x0002) != 0,
            is_protected: (flags & 0x0004) != 0,
            is_static: (flags & 0x0008) != 0,
            is_final: (flags & 0x0010) != 0,
            is_super: (flags & 0x0020) != 0,
            is_volatile: (flags & 0x0040) != 0,
            is_transient: (flags & 0x0080) != 0,
            is_interface: (flags & 0x0200) != 0,
            is_abstract: (flags & 0x0400) != 0,
            is_synthetic: (flags & 0x1000) != 0,
            is_annotation: (flags & 0x2000) != 0,
            is_enum: (flags & 0x4000) != 0,
        }
    }

    /// 将访问标志转换为修饰符字符串列表
    pub fn to_modifiers(&self) -> Vec<String> {
        let mut modifiers = Vec::new();
        if self.is_public {
            modifiers.push("public".to_string());
        }
        if self.is_private {
            modifiers.push("private".to_string());
        }
        if self.is_protected {
            modifiers.push("protected".to_string());
        }
        if self.is_static {
            modifiers.push("static".to_string());
        }
        if self.is_final {
            modifiers.push("final".to_string());
        }
        if self.is_super {
            modifiers.push("super".to_string());
        }
        if self.is_interface {
            modifiers.push("interface".to_string());
        }
        if self.is_abstract {
            modifiers.push("abstract".to_string());
        }
        if self.is_synthetic {
            modifiers.push("synthetic".to_string());
        }
        if self.is_annotation {
            modifiers.push("annotation".to_string());
        }
        if self.is_enum {
            modifiers.push("enum".to_string());
        }
        if self.is_volatile {
            modifiers.push("volatile".to_string());
        }
        if self.is_transient {
            modifiers.push("transient".to_string());
        }
        modifiers
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
        if self.is_volatile {
            flags |= 0x0040;
        }
        if self.is_transient {
            flags |= 0x0080;
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
