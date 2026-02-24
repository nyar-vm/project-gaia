use super::types::ClrTypeReference;

/// CLR 操作码枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClrOpcode {
    Nop,
    LdcI4M1,
    LdcI40,
    LdcI41,
    LdcI42,
    LdcI43,
    LdcI44,
    LdcI45,
    LdcI46,
    LdcI47,
    LdcI48,
    LdcI4S,
    LdcI4,
    LdcI8,
    LdcR4,
    LdcR8,
    Ldnull,
    Ldstr,
    Ldarg0,
    Ldarg1,
    Ldarg2,
    Ldarg3,
    Ldloc0,
    Ldloc1,
    Ldloc2,
    Ldloc3,
    Stloc0,
    Stloc1,
    Stloc2,
    Stloc3,
    Call,
    Callvirt,
    Ret,
    Newobj,
    Pop,
    Dup,
    // 其他操作码...
}

impl ClrOpcode {
    /// 将操作码转换为字节
    pub fn to_byte(&self) -> u8 {
        match self {
            ClrOpcode::Nop => 0x00,
            ClrOpcode::LdcI4M1 => 0x15,
            ClrOpcode::LdcI40 => 0x16,
            ClrOpcode::LdcI41 => 0x17,
            ClrOpcode::LdcI42 => 0x18,
            ClrOpcode::LdcI43 => 0x19,
            ClrOpcode::LdcI44 => 0x1A,
            ClrOpcode::LdcI45 => 0x1B,
            ClrOpcode::LdcI46 => 0x1C,
            ClrOpcode::LdcI47 => 0x1D,
            ClrOpcode::LdcI48 => 0x1E,
            ClrOpcode::LdcI4S => 0x1F,
            ClrOpcode::LdcI4 => 0x20,
            ClrOpcode::LdcI8 => 0x21,
            ClrOpcode::LdcR4 => 0x22,
            ClrOpcode::LdcR8 => 0x23,
            ClrOpcode::Ldnull => 0x14,
            ClrOpcode::Ldstr => 0x72,
            ClrOpcode::Ldarg0 => 0x02,
            ClrOpcode::Ldarg1 => 0x03,
            ClrOpcode::Ldarg2 => 0x04,
            ClrOpcode::Ldarg3 => 0x05,
            ClrOpcode::Ldloc0 => 0x06,
            ClrOpcode::Ldloc1 => 0x07,
            ClrOpcode::Ldloc2 => 0x08,
            ClrOpcode::Ldloc3 => 0x09,
            ClrOpcode::Stloc0 => 0x0A,
            ClrOpcode::Stloc1 => 0x0B,
            ClrOpcode::Stloc2 => 0x0C,
            ClrOpcode::Stloc3 => 0x0D,
            ClrOpcode::Call => 0x28,
            ClrOpcode::Callvirt => 0x6F,
            ClrOpcode::Ret => 0x2A,
            ClrOpcode::Newobj => 0x73,
            ClrOpcode::Pop => 0x25,
            ClrOpcode::Dup => 0x25,
            _ => 0x00, // 默认值，实际使用时需要完善所有操作码
        }
    }

    /// 从字符串解析操作码
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "nop" => Some(ClrOpcode::Nop),
            "ldc.i4.m1" => Some(ClrOpcode::LdcI4M1),
            "ldc.i4.0" => Some(ClrOpcode::LdcI40),
            "ldc.i4.1" => Some(ClrOpcode::LdcI41),
            "ldc.i4.2" => Some(ClrOpcode::LdcI42),
            "ldc.i4.3" => Some(ClrOpcode::LdcI43),
            "ldc.i4.4" => Some(ClrOpcode::LdcI44),
            "ldc.i4.5" => Some(ClrOpcode::LdcI45),
            "ldc.i4.6" => Some(ClrOpcode::LdcI46),
            "ldc.i4.7" => Some(ClrOpcode::LdcI47),
            "ldc.i4.8" => Some(ClrOpcode::LdcI48),
            "ldc.i4.s" => Some(ClrOpcode::LdcI4S),
            "ldc.i4" => Some(ClrOpcode::LdcI4),
            "ldc.i8" => Some(ClrOpcode::LdcI8),
            "ldc.r4" => Some(ClrOpcode::LdcR4),
            "ldc.r8" => Some(ClrOpcode::LdcR8),
            "ldnull" => Some(ClrOpcode::Ldnull),
            "ldstr" => Some(ClrOpcode::Ldstr),
            "ldarg.0" => Some(ClrOpcode::Ldarg0),
            "ldarg.1" => Some(ClrOpcode::Ldarg1),
            "ldarg.2" => Some(ClrOpcode::Ldarg2),
            "ldarg.3" => Some(ClrOpcode::Ldarg3),
            "ldloc.0" => Some(ClrOpcode::Ldloc0),
            "ldloc.1" => Some(ClrOpcode::Ldloc1),
            "ldloc.2" => Some(ClrOpcode::Ldloc2),
            "ldloc.3" => Some(ClrOpcode::Ldloc3),
            "stloc.0" => Some(ClrOpcode::Stloc0),
            "stloc.1" => Some(ClrOpcode::Stloc1),
            "stloc.2" => Some(ClrOpcode::Stloc2),
            "stloc.3" => Some(ClrOpcode::Stloc3),
            "call" => Some(ClrOpcode::Call),
            "callvirt" => Some(ClrOpcode::Callvirt),
            "ret" => Some(ClrOpcode::Ret),
            "newobj" => Some(ClrOpcode::Newobj),
            "pop" => Some(ClrOpcode::Pop),
            "dup" => Some(ClrOpcode::Dup),
            _ => None,
        }
    }
}

/// CLR 指令结构
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClrInstruction {
    pub opcode: ClrOpcode,
    pub operand: Option<ClrInstructionOperand>,
    pub offset: u32,
}

/// CLR 指令操作数
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClrInstructionOperand {
    Int32(i32),
    Int64(i64),
    Float32(f32),
    Float64(f64),
    String(String),
    Type(ClrTypeReference),
    Method(String, Option<String>, Vec<ClrTypeReference>, Box<ClrTypeReference>), // (name, class, params, return)
    Field(String, Option<String>, Box<ClrTypeReference>), // (name, class, type)
    Branch(i32), // 偏移量
    Switch(Vec<i32>), // 多个偏移量
}
