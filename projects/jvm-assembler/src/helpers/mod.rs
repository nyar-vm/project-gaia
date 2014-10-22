//! JVM 指令类型定义和辅助功能

/// JVM 指令类型
#[derive(Debug, Clone, PartialEq)]
pub struct JvmInstruction {
    /// 操作码
    pub opcode: u8,
    /// 操作数
    pub operands: Vec<u8>,
    /// 元数据（用于调试和注释）
    pub metadata: Option<String>,
}

impl JvmInstruction {
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

/// JVM 常量池条目类型
#[derive(Debug, Clone, PartialEq)]
pub enum ConstantPoolEntry {
    /// UTF-8 字符串
    Utf8(String),
    /// 整数常量
    Integer(i32),
    /// 浮点数常量
    Float(f32),
    /// 长整数常量
    Long(i64),
    /// 双精度浮点数常量
    Double(f64),
    /// 类引用
    Class(u16),
    /// 字符串引用
    String(u16),
    /// 字段引用
    Fieldref { class_index: u16, name_and_type_index: u16 },
    /// 方法引用
    Methodref { class_index: u16, name_and_type_index: u16 },
    /// 接口方法引用
    InterfaceMethodref { class_index: u16, name_and_type_index: u16 },
    /// 名称和类型
    NameAndType { name_index: u16, descriptor_index: u16 },
}

/// JVM 方法信息
#[derive(Debug, Clone)]
pub struct JvmMethod {
    /// 访问标志
    pub access_flags: u16,
    /// 方法名索引
    pub name_index: u16,
    /// 描述符索引
    pub descriptor_index: u16,
    /// 指令列表
    pub instructions: Vec<JvmInstruction>,
    /// 最大栈深度
    pub max_stack: u16,
    /// 最大局部变量数
    pub max_locals: u16,
}

impl JvmMethod {
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
    pub fn add_instruction(&mut self, instruction: JvmInstruction) {
        self.instructions.push(instruction);
    }

    /// 获取代码长度
    pub fn code_length(&self) -> u32 {
        self.instructions.iter().map(|i| i.len() as u32).sum()
    }
}

/// JVM 类文件结构
#[derive(Debug, Clone)]
pub struct JvmClass {
    /// 魔数 (0xCAFEBABE)
    pub magic: u32,
    /// 次版本号
    pub minor_version: u16,
    /// 主版本号
    pub major_version: u16,
    /// 常量池
    pub constant_pool: Vec<ConstantPoolEntry>,
    /// 访问标志
    pub access_flags: u16,
    /// 当前类索引
    pub this_class: u16,
    /// 父类索引
    pub super_class: u16,
    /// 接口列表
    pub interfaces: Vec<u16>,
    /// 字段列表
    pub fields: Vec<JvmField>,
    /// 方法列表
    pub methods: Vec<JvmMethod>,
}

impl Default for JvmClass {
    fn default() -> Self {
        Self {
            magic: 0xCAFEBABE,
            minor_version: 0,
            major_version: 61, // Java 17
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

/// JVM 字段信息
#[derive(Debug, Clone)]
pub struct JvmField {
    /// 访问标志
    pub access_flags: u16,
    /// 字段名索引
    pub name_index: u16,
    /// 描述符索引
    pub descriptor_index: u16,
}

/// JVM 操作码常量
pub mod opcodes {
    // 常量加载指令
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

    // 其他指令
    pub const NOP: u8 = 0x00;
}
