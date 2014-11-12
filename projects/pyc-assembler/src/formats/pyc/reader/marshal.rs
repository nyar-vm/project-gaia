use crate::program::{PythonCodeObject, PythonObject};
use byteorder::{LittleEndian, ReadBytesExt};
use gaia_types::GaiaError;
use std::io::{Cursor, Read};

// Marshal 类型常量
#[allow(dead_code)]
const TYPE_NULL: u8 = b'0';
#[allow(dead_code)]
const TYPE_NONE: u8 = b'N';
#[allow(dead_code)]
const TYPE_FALSE: u8 = b'F';
#[allow(dead_code)]
const TYPE_TRUE: u8 = b'T';
#[allow(dead_code)]
const TYPE_STOPITER: u8 = b'S';
#[allow(dead_code)]
const TYPE_ELLIPSIS: u8 = b'.';
#[allow(dead_code)]
const TYPE_INT: u8 = b'i';
#[allow(dead_code)]
const TYPE_INT64: u8 = b'I';
#[allow(dead_code)]
const TYPE_FLOAT: u8 = b'f';
#[allow(dead_code)]
const TYPE_BINARY_FLOAT: u8 = b'g';
#[allow(dead_code)]
const TYPE_COMPLEX: u8 = b'x';
#[allow(dead_code)]
const TYPE_BINARY_COMPLEX: u8 = b'y';
#[allow(dead_code)]
const TYPE_LONG: u8 = b'l';
const TYPE_STRING: u8 = b's';
const TYPE_INTERNED: u8 = b't';
#[allow(dead_code)]
const TYPE_REF: u8 = b'r';
const TYPE_TUPLE: u8 = b'(';
const TYPE_LIST: u8 = b'[';
#[allow(dead_code)]
const TYPE_DICT: u8 = b'{';
const TYPE_CODE: u8 = b'c';
const TYPE_UNICODE: u8 = b'u';
#[allow(dead_code)]
const TYPE_UNKNOWN: u8 = b'?';
#[allow(dead_code)]
const TYPE_SET: u8 = b'<';
#[allow(dead_code)]
const TYPE_FROZENSET: u8 = b'>';
const TYPE_ASCII: u8 = b'a';
const TYPE_ASCII_INTERNED: u8 = b'A';
const TYPE_SMALL_TUPLE: u8 = b')';
const TYPE_SHORT_ASCII: u8 = b'z';
const TYPE_SHORT_ASCII_INTERNED: u8 = b'Z';

pub struct MarshalParser<'a> {
    cursor: Cursor<&'a [u8]>,
    #[allow(dead_code)]
    refs: Vec<PythonObject>,
}

impl<'a> MarshalParser<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            cursor: Cursor::new(data),
            refs: Vec::new(), // Explicitly use refs
        }
    }

    pub fn parse_object(&mut self) -> Result<PythonObject, GaiaError> {
        let type_byte =
            self.cursor.read_u8().map_err(|_| GaiaError::custom_error("Failed to read marshal type".to_string()))?;
        self.parse_object_with_type(type_byte)
    }

    fn parse_object_with_type(&mut self, type_byte: u8) -> Result<PythonObject, GaiaError> {
        match type_byte {
            TYPE_NONE => Ok(PythonObject::None),
            TYPE_TRUE => Ok(PythonObject::Bool(true)),
            TYPE_FALSE => Ok(PythonObject::Bool(false)),
            TYPE_INT => {
                let value = self
                    .cursor
                    .read_i32::<LittleEndian>()
                    .map_err(|_| GaiaError::custom_error("Failed to read integer".to_string()))?;
                Ok(PythonObject::Integer(value as i64))
            }
            TYPE_INT64 => {
                let value = self
                    .cursor
                    .read_i64::<LittleEndian>()
                    .map_err(|_| GaiaError::custom_error("Failed to read int64".to_string()))?;
                Ok(PythonObject::Integer(value))
            }
            TYPE_STRING | TYPE_INTERNED => {
                let length = self
                    .cursor
                    .read_u32::<LittleEndian>()
                    .map_err(|_| GaiaError::custom_error("Failed to read string length".to_string()))?;
                let mut buffer = vec![0u8; length as usize];
                self.cursor
                    .read_exact(&mut buffer)
                    .map_err(|_| GaiaError::custom_error("Failed to read string data".to_string()))?;
                let string = String::from_utf8_lossy(&buffer).to_string();
                Ok(PythonObject::String(string))
            }
            TYPE_UNICODE | TYPE_ASCII | TYPE_ASCII_INTERNED => {
                let length = self
                    .cursor
                    .read_u32::<LittleEndian>()
                    .map_err(|_| GaiaError::custom_error("Failed to read unicode length".to_string()))?;
                let mut buffer = vec![0u8; length as usize];
                self.cursor
                    .read_exact(&mut buffer)
                    .map_err(|_| GaiaError::custom_error("Failed to read unicode data".to_string()))?;
                let string = String::from_utf8_lossy(&buffer).to_string();
                Ok(PythonObject::String(string))
            }
            TYPE_SHORT_ASCII | TYPE_SHORT_ASCII_INTERNED => {
                let length = self
                    .cursor
                    .read_u8()
                    .map_err(|_| GaiaError::custom_error("Failed to read short ascii length".to_string()))?;
                let mut buffer = vec![0u8; length as usize];
                self.cursor
                    .read_exact(&mut buffer)
                    .map_err(|_| GaiaError::custom_error("Failed to read short ascii data".to_string()))?;
                let string = String::from_utf8_lossy(&buffer).to_string();
                Ok(PythonObject::String(string))
            }
            TYPE_TUPLE | TYPE_SMALL_TUPLE => {
                let length = if type_byte == TYPE_SMALL_TUPLE {
                    self.cursor
                        .read_u8()
                        .map_err(|_| GaiaError::custom_error("Failed to read small tuple length".to_string()))?
                        as u32
                }
                else {
                    self.cursor
                        .read_u32::<LittleEndian>()
                        .map_err(|_| GaiaError::custom_error("Failed to read tuple length".to_string()))?
                };

                let mut items = Vec::new();
                for _ in 0..length {
                    items.push(self.parse_object()?);
                }
                Ok(PythonObject::Tuple(items))
            }
            TYPE_LIST => {
                let length = self
                    .cursor
                    .read_u32::<LittleEndian>()
                    .map_err(|_| GaiaError::custom_error("Failed to read list length".to_string()))?;

                let mut items = Vec::new();
                for _ in 0..length {
                    items.push(self.parse_object()?);
                }
                Ok(PythonObject::List(items))
            }
            TYPE_CODE => {
                // 跳过 code object，这里我们只需要解析常量
                // 实际的 code object 解析会在后面处理
                Ok(PythonObject::None)
            }
            _ => {
                // 对于未知类型，返回 None
                Ok(PythonObject::None)
            }
        }
    }

    pub fn parse_code_object(&mut self) -> Result<PythonCodeObject, GaiaError> {
        let argcount = self
            .cursor
            .read_u32::<LittleEndian>()
            .map_err(|_| GaiaError::custom_error("Failed to read argcount".to_string()))?;
        let _posonlyargcount = self
            .cursor
            .read_u32::<LittleEndian>()
            .map_err(|_| GaiaError::custom_error("Failed to read posonlyargcount".to_string()))?;
        let _kwonlyargcount = self
            .cursor
            .read_u32::<LittleEndian>()
            .map_err(|_| GaiaError::custom_error("Failed to read kwonlyargcount".to_string()))?;
        let nlocals = self
            .cursor
            .read_u32::<LittleEndian>()
            .map_err(|_| GaiaError::custom_error("Failed to read nlocals".to_string()))?;
        let stacksize = self
            .cursor
            .read_u32::<LittleEndian>()
            .map_err(|_| GaiaError::custom_error("Failed to read stacksize".to_string()))?;
        let flags =
            self.cursor.read_u32::<LittleEndian>().map_err(|_| GaiaError::custom_error("Failed to read flags".to_string()))?;

        // 读取字节码
        let code = self.parse_object()?;
        let _code_bytes = match code {
            PythonObject::String(s) => s.into_bytes(),
            _ => return Err(GaiaError::custom_error("Expected string for code".to_string())),
        };

        // 读取常量
        let constants_obj = self.parse_object()?;
        let constants = match constants_obj {
            PythonObject::Tuple(items) | PythonObject::List(items) => items,
            _ => vec![constants_obj],
        };

        // 读取名称
        let names_obj = self.parse_object()?;
        let _names = self.extract_string_list(names_obj)?;

        // 读取变量名
        let varnames_obj = self.parse_object()?;
        let varnames = self.extract_string_list(varnames_obj)?;

        // 读取自由变量
        let freevars_obj = self.parse_object()?;
        let freevars = self.extract_string_list(freevars_obj)?;

        // 读取单元变量
        let cellvars_obj = self.parse_object()?;
        let cellvars = self.extract_string_list(cellvars_obj)?;

        // 读取文件名
        let filename_obj = self.parse_object()?;
        let filename = match filename_obj {
            PythonObject::String(s) => s,
            _ => String::new(),
        };

        // 读取函数名
        let name_obj = self.parse_object()?;
        let _name = match name_obj {
            PythonObject::String(s) => s,
            _ => String::new(),
        };

        let firstlineno = self
            .cursor
            .read_u32::<LittleEndian>()
            .map_err(|_| GaiaError::custom_error("Failed to read firstlineno".to_string()))?;

        // 读取行号表
        let lnotab_obj = self.parse_object()?;
        let lnotab = match lnotab_obj {
            PythonObject::String(s) => s.into_bytes(),
            _ => Vec::new(),
        };

        // 将解析的数据转换为 PythonCodeObject
        Ok(PythonCodeObject {
            source_name: filename,
            first_line: firstlineno,
            last_line: firstlineno, // 暂时使用 firstlineno，后续可以从 lnotab 计算
            num_params: argcount as u8,
            is_vararg: if flags & 0x04 != 0 { 1 } else { 0 }, // CO_VARARGS
            max_stack_size: stacksize as u8,
            nested_functions: Vec::new(), // 暂时为空，后续可以从 constants 中提取
            upvalues: freevars
                .iter()
                .chain(cellvars.iter())
                .enumerate()
                .map(|(_i, name)| {
                    use crate::program::Upvalue;
                    Upvalue { in_stack: 0, idx: _i as u8, name: name.clone() }
                })
                .collect(),
            local_vars: varnames
                .iter()
                .enumerate()
                .map(|(_i, name)| {
                    use crate::program::LocalVar;
                    LocalVar { name: name.clone(), start_pc: 0, end_pc: 0 }
                })
                .collect(),
            line_info: lnotab,
            co_argcount: argcount as u8,
            co_nlocal: nlocals as u8,
            co_stacks: stacksize as u8,
            num_upval: (freevars.len() + cellvars.len()) as u8,
            co_code: Vec::new(), // 暂时为空，需要从 code_bytes 解析指令
            co_consts: constants,
            upvalue_n: (freevars.len() + cellvars.len()) as u8,
        })
    }

    fn extract_string_list(&self, obj: PythonObject) -> Result<Vec<String>, GaiaError> {
        match obj {
            PythonObject::Tuple(items) | PythonObject::List(items) => {
                let mut strings = Vec::new();
                for item in items {
                    match item {
                        PythonObject::String(s) => strings.push(s),
                        _ => strings.push(String::new()),
                    }
                }
                Ok(strings)
            }
            _ => Ok(Vec::new()),
        }
    }
}
