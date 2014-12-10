#![doc = include_str!("readme.md")]

use crate::{
    formats::pyc::PycReadConfig,
    program::{PycHeader, PythonCodeObject, PythonObject, PythonProgram, PythonVersion},
};
use byteorder::{LittleEndian, ReadBytesExt};
use gaia_types::{BinaryReader, GaiaDiagnostics, GaiaError};
use std::{
    cell::RefCell,
    io::{Cursor, Read, Seek},
    sync::OnceLock,
};

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

/// PycInfo 表示 .pyc 文件的基本信息视图
#[derive(Debug, Clone, Copy)]
pub struct PycInfo {
    /// .pyc 文件头信息
    pub header: PycHeader,
    /// Python 版本信息
    pub version: PythonVersion,
}

/// 现代化的惰性 .pyc 文件读取器
#[derive(Debug)]
pub struct PycReader<'config, R> {
    config: &'config PycReadConfig,
    reader: RefCell<BinaryReader<R, LittleEndian>>,
    info: OnceLock<PycInfo>,
    program: OnceLock<PythonProgram>,
}

impl PycReadConfig {
    /// 创建一个新的 PycReader 实例
    pub fn as_reader<R: Read + Seek>(&self, reader: R) -> PycReader<'_, R> {
        PycReader::new(reader, self)
    }
}

impl<'config, R> PycReader<'config, R> {
    /// 创建一个新的 PycReader 实例
    pub fn new(reader: R, config: &'config PycReadConfig) -> Self {
        Self { config, reader: RefCell::new(BinaryReader::new(reader)), info: Default::default(), program: Default::default() }
    }

    /// 完成读取并返回 PythonProgram 结果
    pub fn finish(self) -> GaiaDiagnostics<PythonProgram>
    where
        R: Read + Seek,
    {
        match self.get_program() {
            Ok(program) => {
                let errors = self.reader.borrow_mut().take_errors();
                GaiaDiagnostics { result: Ok(program.clone()), diagnostics: errors }
            }
            Err(e) => {
                let errors = self.reader.borrow_mut().take_errors();
                GaiaDiagnostics { result: Err(e), diagnostics: errors }
            }
        }
    }
}

impl<'config, R: Read + Seek> PycReader<'config, R> {
    /// 获取解析后的 PythonProgram
    pub fn get_program(&self) -> Result<&PythonProgram, GaiaError> {
        Ok(self.program.get_or_init(|| self.read_program().unwrap_or_else(|_| PythonProgram::default())))
    }

    /// 获取 .pyc 文件的基本信息
    pub fn get_info(&self) -> Result<&PycInfo, GaiaError> {
        Ok(self.info.get_or_init(|| {
            self.read_info().unwrap_or_else(|_| PycInfo { header: PycHeader::default(), version: PythonVersion::Unknown })
        }))
    }

    fn read_info(&self) -> Result<PycInfo, GaiaError> {
        let mut reader = self.reader.borrow_mut();

        // 重新定位到文件开头
        reader.seek(std::io::SeekFrom::Start(0))?;

        // 读取 .pyc 文件头
        let header = self.read_header(&mut reader)?;

        // 从配置或头部确定版本
        let version = if self.config.version != PythonVersion::Unknown {
            self.config.version
        }
        else {
            PythonVersion::from_magic(header.magic)
        };

        Ok(PycInfo { header, version })
    }

    fn read_program(&self) -> Result<PythonProgram, GaiaError> {
        let mut reader = self.reader.borrow_mut();

        // 重新定位到文件开头
        reader.seek(std::io::SeekFrom::Start(0))?;

        // 读取头部信息
        let header = self.read_header(&mut reader)?;

        // 确定 Python 版本
        let version = if self.config.version != PythonVersion::Unknown {
            self.config.version
        }
        else {
            PythonVersion::from_magic(header.magic)
        };

        // 读取 marshal 数据
        let mut code_object_bytes = Vec::new();
        reader.read_to_end(&mut code_object_bytes)?;

        // 解析 marshal 数据
        let code_object = if !code_object_bytes.is_empty() {
            self.parse_code_object(&code_object_bytes)?
        }
        else {
            PythonCodeObject::default()
        };

        // 构建 PythonProgram
        let program = PythonProgram { header, code_object, version };

        Ok(program)
    }

    fn read_header(&self, reader: &mut BinaryReader<R, LittleEndian>) -> Result<PycHeader, GaiaError> {
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;

        let flags = reader.read_u32()?;
        let timestamp = reader.read_u32()?;
        let size = reader.read_u32()?;

        Ok(PycHeader { magic, flags, timestamp, size })
    }

    // 集成的 marshal 解析功能
    fn parse_code_object(&self, data: &[u8]) -> Result<PythonCodeObject, GaiaError> {
        let mut cursor = Cursor::new(data);

        // 首先检查是否是 CODE 类型
        let type_byte = cursor.read_u8().map_err(|_| GaiaError::custom_error("Failed to read type byte".to_string()))?;
        if (type_byte & 0x7F) != TYPE_CODE {
            return Err(GaiaError::custom_error(format!("Expected code object, got type {}", type_byte)));
        }

        // Python 3.11+ marshal format
        let argcount =
            cursor.read_u32::<LittleEndian>().map_err(|_| GaiaError::custom_error("Failed to read argcount".to_string()))?;
        let posonlyargcount = cursor
            .read_u32::<LittleEndian>()
            .map_err(|_| GaiaError::custom_error("Failed to read posonlyargcount".to_string()))?;
        let kwonlyargcount = cursor
            .read_u32::<LittleEndian>()
            .map_err(|_| GaiaError::custom_error("Failed to read kwonlyargcount".to_string()))?;
        let stacksize =
            cursor.read_u32::<LittleEndian>().map_err(|_| GaiaError::custom_error("Failed to read stacksize".to_string()))?;
        let flags =
            cursor.read_u32::<LittleEndian>().map_err(|_| GaiaError::custom_error("Failed to read flags".to_string()))?;

        // co_code
        let code_obj = self.parse_object(&mut cursor)?;
        let code_bytes = match code_obj {
            PythonObject::Bytes(b) => b,
            _ => return Err(GaiaError::custom_error("Expected bytes for code".to_string())),
        };

        // co_consts
        let constants_obj = self.parse_object(&mut cursor)?;
        let constants = match constants_obj {
            PythonObject::Tuple(items) | PythonObject::List(items) => items,
            _ => vec![constants_obj],
        };

        // co_names
        let names_obj = self.parse_object(&mut cursor)?;
        let names = self.extract_string_list(names_obj)?;

        // co_localsplusnames
        let localsplusnames_obj = self.parse_object(&mut cursor)?;
        let localsplusnames = self.extract_string_list(localsplusnames_obj)?;

        // co_localspluskinds
        let localspluskinds_obj = self.parse_object(&mut cursor)?;
        let localspluskinds = match localspluskinds_obj {
            PythonObject::Bytes(b) => b,
            _ => Vec::new(),
        };

        // co_filename
        let filename_obj = self.parse_object(&mut cursor)?;
        let filename = match filename_obj {
            PythonObject::String(s) | PythonObject::Str(s) => s,
            _ => String::new(),
        };

        // co_name
        let name_obj = self.parse_object(&mut cursor)?;
        let name = match name_obj {
            PythonObject::String(s) | PythonObject::Str(s) => s,
            _ => String::new(),
        };

        // co_qualname
        let qualname_obj = self.parse_object(&mut cursor)?;
        let qualname = match qualname_obj {
            PythonObject::String(s) | PythonObject::Str(s) => s,
            _ => String::new(),
        };

        let firstlineno =
            cursor.read_u32::<LittleEndian>().map_err(|_| GaiaError::custom_error("Failed to read firstlineno".to_string()))?;

        // co_linetable
        let lnotab_obj = self.parse_object(&mut cursor)?;
        let lnotab = match lnotab_obj {
            PythonObject::Bytes(b) => b,
            _ => Vec::new(),
        };

        // co_exceptiontable
        let exceptiontable_obj = self.parse_object(&mut cursor)?;
        let exceptiontable = match exceptiontable_obj {
            PythonObject::Bytes(b) => b,
            _ => Vec::new(),
        };

        // TODO: 将字节码解析为指令
        let instructions = Vec::new();

        Ok(PythonCodeObject {
            name,
            qualname,
            source_name: filename,
            first_line: firstlineno,
            last_line: firstlineno, // TODO: 从 lnotab 计算
            co_argcount: argcount as u8,
            co_posonlyargcount: posonlyargcount as u8,
            co_kwonlyargcount: kwonlyargcount as u8,
            co_nlocals: localsplusnames.len() as u8, // 估计值
            co_stacksize: stacksize as u8,
            co_flags: flags,
            co_code: instructions,
            co_consts: constants,
            co_names: names,
            co_localsplusnames: localsplusnames,
            co_localspluskinds: localspluskinds,
            co_linetable: lnotab,
            co_exceptiontable: exceptiontable,
        })
    }

    fn parse_object(&self, cursor: &mut Cursor<&[u8]>) -> Result<PythonObject, GaiaError> {
        let type_byte = cursor.read_u8().map_err(|_| GaiaError::custom_error("Failed to read marshal type".to_string()))?;
        // 忽略 FLAG_REF (0x80)
        self.parse_object_with_type(cursor, type_byte & 0x7F)
    }

    fn parse_object_with_type(&self, cursor: &mut Cursor<&[u8]>, type_byte: u8) -> Result<PythonObject, GaiaError> {
        match type_byte {
            TYPE_NONE => Ok(PythonObject::None),
            TYPE_TRUE => Ok(PythonObject::Bool(true)),
            TYPE_FALSE => Ok(PythonObject::Bool(false)),
            TYPE_INT => {
                let value = cursor
                    .read_i32::<LittleEndian>()
                    .map_err(|_| GaiaError::custom_error("Failed to read integer".to_string()))?;
                Ok(PythonObject::Int(value))
            }
            TYPE_INT64 => {
                let value = cursor
                    .read_i64::<LittleEndian>()
                    .map_err(|_| GaiaError::custom_error("Failed to read int64".to_string()))?;
                Ok(PythonObject::Integer(value))
            }
            TYPE_STRING | TYPE_INTERNED => {
                let length = cursor
                    .read_u32::<LittleEndian>()
                    .map_err(|_| GaiaError::custom_error("Failed to read string length".to_string()))?;
                let mut buffer = vec![0u8; length as usize];
                cursor
                    .read_exact(&mut buffer)
                    .map_err(|_| GaiaError::custom_error("Failed to read string data".to_string()))?;
                Ok(PythonObject::Bytes(buffer))
            }
            TYPE_UNICODE | TYPE_ASCII | TYPE_ASCII_INTERNED => {
                let length = cursor
                    .read_u32::<LittleEndian>()
                    .map_err(|_| GaiaError::custom_error("Failed to read unicode length".to_string()))?;
                let mut buffer = vec![0u8; length as usize];
                cursor
                    .read_exact(&mut buffer)
                    .map_err(|_| GaiaError::custom_error("Failed to read unicode data".to_string()))?;
                let string = String::from_utf8_lossy(&buffer).to_string();
                Ok(PythonObject::Str(string))
            }
            TYPE_SHORT_ASCII | TYPE_SHORT_ASCII_INTERNED => {
                let length =
                    cursor.read_u8().map_err(|_| GaiaError::custom_error("Failed to read short ascii length".to_string()))?;
                let mut buffer = vec![0u8; length as usize];
                cursor
                    .read_exact(&mut buffer)
                    .map_err(|_| GaiaError::custom_error("Failed to read short ascii data".to_string()))?;
                let string = String::from_utf8_lossy(&buffer).to_string();
                Ok(PythonObject::Str(string))
            }
            TYPE_TUPLE | TYPE_SMALL_TUPLE => {
                let length = if type_byte == TYPE_SMALL_TUPLE {
                    cursor.read_u8().map_err(|_| GaiaError::custom_error("Failed to read small tuple length".to_string()))?
                        as u32
                }
                else {
                    cursor
                        .read_u32::<LittleEndian>()
                        .map_err(|_| GaiaError::custom_error("Failed to read tuple length".to_string()))?
                };

                let mut items = Vec::new();
                for _ in 0..length {
                    items.push(self.parse_object(cursor)?);
                }
                Ok(PythonObject::Tuple(items))
            }
            TYPE_LIST => {
                let length = cursor
                    .read_u32::<LittleEndian>()
                    .map_err(|_| GaiaError::custom_error("Failed to read list length".to_string()))?;

                let mut items = Vec::new();
                for _ in 0..length {
                    items.push(self.parse_object(cursor)?);
                }
                Ok(PythonObject::List(items))
            }
            _ => {
                // 对于未知类型，返回 None
                Ok(PythonObject::None)
            }
        }
    }

    fn extract_string_list(&self, obj: PythonObject) -> Result<Vec<String>, GaiaError> {
        match obj {
            PythonObject::Tuple(items) | PythonObject::List(items) => {
                let mut strings = Vec::new();
                for item in items {
                    match item {
                        PythonObject::String(s) | PythonObject::Str(s) => strings.push(s),
                        _ => strings.push(String::new()),
                    }
                }
                Ok(strings)
            }
            _ => Ok(Vec::new()),
        }
    }
}
