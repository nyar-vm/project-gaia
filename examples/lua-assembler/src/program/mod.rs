use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub enum LuaVersion {
    Unknown,
    Lua51,
    Lua52,
    Lua53,
    Lua54,
    LuaJIT,
    Luau,
}

impl LuaVersion {
    pub fn to_byte(&self) -> u8 {
        match self {
            LuaVersion::Lua51 => 0x51,
            LuaVersion::Lua52 => 0x52,
            LuaVersion::Lua53 => 0x53,
            LuaVersion::Lua54 => 0x54,
            _ => 0x00, // Default or unknown version
        }
    }

    pub fn from_byte(byte: u8) -> Self {
        match byte {
            0x51 => LuaVersion::Lua51,
            0x52 => LuaVersion::Lua52,
            0x53 => LuaVersion::Lua53,
            0x54 => LuaVersion::Lua54,
            _ => LuaVersion::Unknown,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Copy)]
pub struct LuacHeader {
    pub magic: [u8; 4], // "\x1bLua"
    pub version: LuaVersion,
    pub format_version: u8,     // LUAC_FORMAT, usually 0
    pub endianness: u8,         // 0x01 for little-endian, 0x00 for big-endian
    pub int_size: u8,           // sizeof(int)
    pub size_t_size: u8,        // sizeof(size_t)
    pub instruction_size: u8,   // sizeof(Instruction)
    pub lua_number_size: u8,    // sizeof(lua_Number)
    pub integral_flag: u8,      // 0x00 if lua_Number is float, 0x01 if integral
    pub flags: u8,              // 新增字段
    pub timestamp: Option<u32>, // 新增字段
    pub size: Option<u32>,      // 新增字段
    pub hash: Option<[u8; 8]>,  // 新增字段
}

impl Debug for LuacHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LuacHeader")
            .field("version", &self.version)
            .field("magic", &format!("0x{:08X}", u32::from_le_bytes(self.magic)))
            .field("flags", &format!("0x{:02X}", self.flags))
            .field("timestamp", &self.timestamp)
            .field("size", &self.size)
            .field("hash", &self.hash.as_ref().map(|h| format!("{:02X?}", h)))
            .finish()
    }
}

impl LuacHeader {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.magic);
        bytes.push(self.version.to_byte());
        bytes.push(self.format_version);
        bytes.push(self.endianness);
        bytes.push(self.int_size);
        bytes.push(self.size_t_size);
        bytes.push(self.instruction_size);
        bytes.push(self.lua_number_size);
        bytes.push(self.integral_flag);
        bytes.push(self.flags);

        if self.flags & 0x01 != 0 {
            // SOURCE_HASH_MODE
            if let Some(hash_val) = self.hash {
                bytes.extend_from_slice(&hash_val);
            }
            else {
                // This should not happen if flags indicate hash mode
                // For now, let's just write 8 zero bytes as a fallback
                bytes.extend_from_slice(&[0; 8]);
            }
        }
        else {
            // Timestamp and size mode
            if let Some(timestamp_val) = self.timestamp {
                bytes.extend_from_slice(&timestamp_val.to_le_bytes());
            }
            else {
                // This should not happen if flags indicate timestamp mode
                bytes.extend_from_slice(&[0; 4]);
            }
            if let Some(size_val) = self.size {
                bytes.extend_from_slice(&size_val.to_le_bytes());
            }
            else {
                // This should not happen if flags indicate timestamp mode
                bytes.extend_from_slice(&[0; 4]);
            }
        }
        bytes
    }
}

/// Upvalue 信息
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Upvalue {
    pub in_stack: u8, // 1 if in stack, 0 if in outer upvalue
    pub idx: u8,      // register or upvalue index
    pub name: String, // for debug info
}

/// 局部变量信息
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalVar {
    pub name: String,
    pub start_pc: u32,
    pub end_pc: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LuaObject {
    Str(String),
    Int(i32),
    Code(LuacCodeObject),
    None,
}

/// Lua 操作码
#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LuaOpCode {
    MOVE,
    LOAD_K,
    LOAD_BOOL,
    LOAD_NIL,
    GET_UPVALUE,
    GET_GLOBAL,
    GET_TABLE,
    SET_TABLE,
    NEW_TABLE,
    SET_GLOBAL,
    SET_UPVALUE,
    SELF,
    ADD,
    SUB,
    MUL,
    DIV,
    MOD,
    POW,
    UNM,
    NOT,
    LEN,
    CONCAT,
    JMP,
    EQ,
    LT,
    LE,
    TEST,
    TESTSET,
    CALL,
    TAILCALL,
    RETURN,
    FORLOOP,
    TFORLOOP,
    SETLIST,
    CLOSE,
    CLOSURE,
    VARARG,
    // Add more opcodes as needed
    UNKNOWN(u8),
}

impl From<u8> for LuaOpCode {
    fn from(byte: u8) -> Self {
        match byte {
            0 => LuaOpCode::MOVE,
            1 => LuaOpCode::LOAD_K,
            2 => LuaOpCode::LOAD_BOOL,
            3 => LuaOpCode::LOAD_NIL,
            4 => LuaOpCode::GET_UPVALUE,
            5 => LuaOpCode::GET_GLOBAL,
            6 => LuaOpCode::GET_TABLE,
            7 => LuaOpCode::SET_GLOBAL,
            8 => LuaOpCode::SET_UPVALUE,
            9 => LuaOpCode::SET_TABLE,
            10 => LuaOpCode::NEW_TABLE,
            11 => LuaOpCode::SELF,
            12 => LuaOpCode::ADD,
            13 => LuaOpCode::SUB,
            14 => LuaOpCode::MUL,
            15 => LuaOpCode::DIV,
            16 => LuaOpCode::MOD,
            17 => LuaOpCode::POW,
            18 => LuaOpCode::UNM,
            19 => LuaOpCode::NOT,
            20 => LuaOpCode::LEN,
            21 => LuaOpCode::CONCAT,
            22 => LuaOpCode::JMP,
            23 => LuaOpCode::EQ,
            24 => LuaOpCode::LT,
            25 => LuaOpCode::LE,
            26 => LuaOpCode::TEST,
            27 => LuaOpCode::TESTSET,
            28 => LuaOpCode::CALL,
            29 => LuaOpCode::TAILCALL,
            30 => LuaOpCode::RETURN,
            31 => LuaOpCode::FORLOOP,
            32 => LuaOpCode::TFORLOOP,
            33 => LuaOpCode::SETLIST,
            34 => LuaOpCode::CLOSE,
            35 => LuaOpCode::CLOSURE,
            36 => LuaOpCode::VARARG,
            _ => LuaOpCode::UNKNOWN(byte),
        }
    }
}

/// Lua 指令结构
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LuaInstruction {
    pub opcode: LuaOpCode,
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub bx: u16,
    pub sbx: i16,
    pub ax: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LuacCodeObject {
    pub source_name: String,
    pub first_line: u32,
    pub last_line: u32,
    pub num_params: u8,
    pub is_vararg: u8,
    pub max_stack_size: u8,
    pub nested_functions: Vec<LuacCodeObject>,
    pub upvalues: Vec<Upvalue>,
    pub local_vars: Vec<LocalVar>,
    pub line_info: Vec<u8>,
    pub co_argcount: u8,
    pub co_nlocal: u8,
    pub co_stacks: u8,
    pub num_upval: u8,
    pub co_code: Vec<u32>,
    pub co_consts: Vec<LuaObject>,
    pub upvalue_n: u8,
}

/// Python .pyc 程序的高层语义定义（以指令序列为核心）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LuaProgram {
    pub header: LuacHeader,
    pub code_object: LuacCodeObject,
}
