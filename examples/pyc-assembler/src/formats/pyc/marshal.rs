//! # Marshal 序列化模块
//!
//! 本模块提供了对 Python 对象进行 Marshal 序列化的功能，主要用于生成 `.pyc` 文件。
//! 它实现了将 `PythonObject` 和 `PythonCodeObject` 转换为字节流的逻辑。

use crate::{
    instructions::PythonInstruction,
    program::{PythonCodeObject, PythonObject, PythonVersion},
};
use byteorder::{LittleEndian, WriteBytesExt};
use std::io::{self, Write};

/// Marshal 序列化器
#[derive(Debug)]
pub struct MarshalWriter<W> {
    writer: W,
    #[allow(dead_code)]
    version: PythonVersion,
}

impl<W: Write> MarshalWriter<W> {
    /// 创建一个新的 MarshalWriter 实例
    pub fn new(writer: W, version: PythonVersion) -> Self {
        Self { writer, version }
    }

    /// 写入一个 Python 对象
    pub fn write_object(&mut self, obj: &PythonObject) -> io::Result<()> {
        match obj {
            PythonObject::None => {
                self.writer.write_u8(b'N')?;
            }
            PythonObject::Bool(b) => {
                self.writer.write_u8(if *b { b'T' } else { b'F' })?;
            }
            PythonObject::Int(i) => {
                self.writer.write_u8(b'i' | 0x80)?; // TYPE_INT | FLAG_REF
                self.writer.write_i32::<LittleEndian>(*i)?;
            }
            PythonObject::Integer(i) => {
                if *i >= i32::MIN as i64 && *i <= i32::MAX as i64 {
                    self.writer.write_u8(b'i' | 0x80)?;
                    self.writer.write_i32::<LittleEndian>(*i as i32)?;
                }
                else {
                    self.writer.write_u8(b'I' | 0x80)?; // TYPE_INT64 | FLAG_REF
                    self.writer.write_i64::<LittleEndian>(*i)?;
                }
            }
            PythonObject::Float(f) => {
                self.writer.write_u8(b'g' | 0x80)?; // TYPE_BINARY_FLOAT | FLAG_REF
                self.writer.write_f64::<LittleEndian>(*f)?;
            }
            PythonObject::Str(s) | PythonObject::String(s) => {
                if s.len() <= 255 && s.is_ascii() {
                    self.writer.write_u8(b'Z' | 0x80)?; // TYPE_SHORT_ASCII_INTERNED | FLAG_REF
                    self.writer.write_u8(s.len() as u8)?;
                    self.writer.write_all(s.as_bytes())?;
                }
                else {
                    self.writer.write_u8(b'u' | 0x80)?; // TYPE_UNICODE | FLAG_REF
                    self.writer.write_u32::<LittleEndian>(s.len() as u32)?;
                    self.writer.write_all(s.as_bytes())?;
                }
            }
            PythonObject::Bytes(b) => {
                self.writer.write_u8(b's' | 0x80)?; // TYPE_STRING | FLAG_REF
                self.writer.write_u32::<LittleEndian>(b.len() as u32)?;
                self.writer.write_all(b)?;
            }
            PythonObject::List(l) => {
                self.writer.write_u8(b'[' | 0x80)?; // TYPE_LIST | FLAG_REF
                self.writer.write_u32::<LittleEndian>(l.len() as u32)?;
                for item in l {
                    self.write_object(item)?;
                }
            }
            PythonObject::Tuple(t) => {
                if t.len() <= 255 {
                    self.writer.write_u8(b')' | 0x80)?; // TYPE_SMALL_TUPLE | FLAG_REF
                    self.writer.write_u8(t.len() as u8)?;
                }
                else {
                    self.writer.write_u8(b'(' | 0x80)?; // TYPE_TUPLE | FLAG_REF
                    self.writer.write_u32::<LittleEndian>(t.len() as u32)?;
                }
                for item in t {
                    self.write_object(item)?;
                }
            }
            PythonObject::Code(code) => {
                self.write_code_object(code)?;
            }
        }
        Ok(())
    }

    /// 写入一个 Python 代码对象
    pub fn write_code_object(&mut self, code: &PythonCodeObject) -> io::Result<()> {
        self.writer.write_u8(b'c' | 0x80)?; // TYPE_CODE | FLAG_REF

        // Python 3.11+ code object fields
        self.writer.write_u32::<LittleEndian>(code.co_argcount as u32)?;
        self.writer.write_u32::<LittleEndian>(code.co_posonlyargcount as u32)?;
        self.writer.write_u32::<LittleEndian>(code.co_kwonlyargcount as u32)?;
        // nlocals removed in 3.11+ marshal, replaced by stacksize and flags
        self.writer.write_u32::<LittleEndian>(code.co_stacksize as u32)?;
        self.writer.write_u32::<LittleEndian>(code.co_flags)?;

        // co_code
        let bytecode = self.encode_instructions(&code.co_code);
        self.write_object(&PythonObject::Bytes(bytecode))?;

        // co_consts
        self.write_object(&PythonObject::Tuple(code.co_consts.clone()))?;

        // co_names
        self.write_object(&PythonObject::Tuple(code.co_names.iter().map(|s| PythonObject::Str(s.clone())).collect()))?;

        // co_localsplusnames (Python 3.11+)
        self.write_object(&PythonObject::Tuple(
            code.co_localsplusnames.iter().map(|s| PythonObject::Str(s.clone())).collect(),
        ))?;

        // co_localspluskinds
        self.write_object(&PythonObject::Bytes(code.co_localspluskinds.clone()))?;

        // co_filename
        self.write_object(&PythonObject::Str(code.source_name.clone()))?;

        // co_name
        self.write_object(&PythonObject::Str(code.name.clone()))?;

        // co_qualname
        self.write_object(&PythonObject::Str(code.qualname.clone()))?;

        // co_firstlineno
        self.writer.write_u32::<LittleEndian>(code.first_line)?;

        // co_linetable
        self.write_object(&PythonObject::Bytes(code.co_linetable.clone()))?;

        // co_exceptiontable
        self.write_object(&PythonObject::Bytes(code.co_exceptiontable.clone()))?;

        Ok(())
    }

    fn encode_instructions(&self, instrs: &[PythonInstruction]) -> Vec<u8> {
        let mut data = Vec::new();
        for instr in instrs {
            let (opcode, arg) = self.map_instruction(instr);
            data.push(opcode);
            data.push((arg & 0xFF) as u8);
        }
        data
    }

    fn map_instruction(&self, instr: &PythonInstruction) -> (u8, u32) {
        match instr {
            PythonInstruction::RESUME => (151, 0),
            PythonInstruction::RETURN_CONST(idx) => (121, *idx),
            PythonInstruction::LOAD_CONST(idx) => (100, *idx),
            PythonInstruction::STORE_NAME(idx) => (90, *idx),
            PythonInstruction::LOAD_NAME(idx) => (101, *idx),
            PythonInstruction::PUSH_NULL => (2, 0),
            PythonInstruction::CALL(argc) => (171, *argc),
            PythonInstruction::BINARY_OP(op) => (122, *op),
            PythonInstruction::POP_TOP => (1, 0),
            PythonInstruction::RETURN_VALUE => (83, 0),
            PythonInstruction::BINARY_MODULO => (22, 0),
            PythonInstruction::INPLACE_MODULO => (59, 0),
            PythonInstruction::NOP => (9, 0),
            PythonInstruction::UNARY_NEGATIVE => (11, 0),
            PythonInstruction::UNARY_NOT => (12, 0),
            PythonInstruction::UNARY_INVERT => (15, 0),
            PythonInstruction::GET_LEN => (30, 0),
            PythonInstruction::MATCH_MAPPING => (31, 0),
            PythonInstruction::MATCH_SEQUENCE => (32, 0),
            PythonInstruction::MATCH_KEYS => (33, 0),
            PythonInstruction::GET_ITER => (68, 0),
            PythonInstruction::LOAD_BUILD_CLASS => (71, 0),
            PythonInstruction::LOAD_ASSERTION_ERROR => (74, 0),
            PythonInstruction::RETURN_GENERATOR => (75, 0),
            PythonInstruction::YIELD_VALUE => (150, 0),
            PythonInstruction::LOAD_FAST(idx) => (124, *idx),
            PythonInstruction::STORE_FAST(idx) => (125, *idx),
            PythonInstruction::DELETE_FAST(idx) => (126, *idx),
            PythonInstruction::LOAD_GLOBAL(idx) => (116, *idx),
            PythonInstruction::STORE_GLOBAL(idx) => (97, *idx),
            PythonInstruction::DELETE_GLOBAL(idx) => (98, *idx),
            PythonInstruction::LOAD_ATTR(idx) => (106, *idx),
            PythonInstruction::STORE_ATTR(idx) => (95, *idx),
            PythonInstruction::DELETE_ATTR(idx) => (96, *idx),
            PythonInstruction::COMPARE_OP(idx) => (107, *idx),
            PythonInstruction::IMPORT_NAME(idx) => (108, *idx),
            PythonInstruction::IMPORT_FROM(idx) => (109, *idx),
            PythonInstruction::JUMP_FORWARD(idx) => (110, *idx),
            PythonInstruction::JUMP_BACKWARD(idx) => (140, *idx),
            PythonInstruction::POP_JUMP_IF_FALSE(idx) => (114, *idx),
            PythonInstruction::POP_JUMP_IF_TRUE(idx) => (115, *idx),
            PythonInstruction::MAKE_FUNCTION(idx) => (132, *idx),
            PythonInstruction::BUILD_TUPLE(idx) => (102, *idx),
            PythonInstruction::BUILD_LIST(idx) => (103, *idx),
            PythonInstruction::BUILD_SET(idx) => (104, *idx),
            PythonInstruction::BUILD_MAP(idx) => (105, *idx),
            PythonInstruction::LIST_APPEND(idx) => (145, *idx),
            PythonInstruction::SET_ADD(idx) => (146, *idx),
            PythonInstruction::MAP_ADD(idx) => (147, *idx),
            _ => (0, 0), // Default to NOP or handle error
        }
    }
}
