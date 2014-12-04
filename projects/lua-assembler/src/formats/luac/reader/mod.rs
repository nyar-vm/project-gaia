#![doc = include_str!("readme.md")]

use crate::{
    formats::luac::LuacReadConfig,
    program::{LuaProgram, LuaVersion, LuacHeader, LuacCodeObject},
};
use byteorder::{LittleEndian, ReadBytesExt};
use gaia_types::{BinaryReader, GaiaDiagnostics, GaiaError};
use std::{
    cell::RefCell,
    io::{Read, Seek},
    sync::OnceLock,
};

/// LuacInfo 表示 .luac 文件的基本信息视图
#[derive(Debug, Clone, Copy)]
pub struct LuacInfo {
    /// .luac 文件头信息
    pub header: LuacHeader,
    /// Lua 版本信息
    pub version: LuaVersion,
}

/// 现代化的惰性 .luac 文件读取器
/// 
/// 采用惰性加载模式，只在需要时才解析数据，提高性能。
/// 使用 `OnceLock` 确保数据只被解析一次。
#[derive(Debug)]
pub struct LuacReader<'config, R> {
    config: &'config LuacReadConfig,
    reader: RefCell<BinaryReader<R, LittleEndian>>,
    info: OnceLock<LuacInfo>,
    program: OnceLock<LuaProgram>,
}

impl LuacReadConfig {
    /// 创建一个新的 LuacReader 实例
    /// 
    /// # 参数
    /// 
    /// * `reader` - 实现了 Read + Seek 的数据源
    /// 
    /// # 返回值
    /// 
    /// 返回一个新的 LuacReader 实例
    pub fn as_reader<R: Read + Seek>(&self, reader: R) -> LuacReader<'_, R> {
        LuacReader::new(reader, self)
    }
}

impl<'config, R> LuacReader<'config, R> {
    /// 创建一个新的 LuacReader 实例
    /// 
    /// # 参数
    /// 
    /// * `reader` - 实现了 Read + Seek 的数据源
    /// * `config` - 读取配置
    /// 
    /// # 返回值
    /// 
    /// 返回一个新的 LuacReader 实例
    pub fn new(reader: R, config: &'config LuacReadConfig) -> Self {
        Self {
            config,
            reader: RefCell::new(BinaryReader::new(reader)),
            info: Default::default(),
            program: Default::default(),
        }
    }

    /// 完成读取并返回 LuaProgram 结果
    /// 
    /// # 返回值
    /// 
    /// 返回包含 LuaProgram 和诊断信息的 GaiaDiagnostics
    pub fn finish(self) -> GaiaDiagnostics<LuaProgram>
    where
        R: Read + Seek,
    {
        match self.get_program() {
            Ok(program) => {
                let errors = self.reader.borrow_mut().take_errors();
                GaiaDiagnostics {
                    result: Ok(program.clone()),
                    diagnostics: errors,
                }
            }
            Err(e) => {
                let errors = self.reader.borrow_mut().take_errors();
                GaiaDiagnostics {
                    result: Err(e),
                    diagnostics: errors,
                }
            }
        }
    }
}

impl<'config, R: Read + Seek> LuacReader<'config, R> {
    /// 获取解析后的 LuaProgram
    /// 
    /// # 返回值
    /// 
    /// 返回 LuaProgram 的引用，如果解析失败则返回错误
    pub fn get_program(&self) -> Result<&LuaProgram, GaiaError> {
        match self.program.get() {
            Some(program) => Ok(program),
            None => {
                let program = self.read_program()?;
                Ok(self.program.get_or_init(|| program))
            }
        }
    }

    /// 获取 .luac 文件的基本信息
    /// 
    /// # 返回值
    /// 
    /// 返回 LuacInfo 的引用，如果解析失败则返回错误
    pub fn get_info(&self) -> Result<&LuacInfo, GaiaError> {
        match self.info.get() {
            Some(info) => Ok(info),
            None => {
                let info = self.read_info()?;
                Ok(self.info.get_or_init(|| info))
            }
        }
    }

    fn read_info(&self) -> Result<LuacInfo, GaiaError> {
        let mut reader = self.reader.borrow_mut();
        
        // 重新定位到文件开头
        reader.seek(std::io::SeekFrom::Start(0))?;
        
        // 检查文件是否为空
        let current_pos = reader.stream_position()?;
        reader.seek(std::io::SeekFrom::End(0))?;
        let file_size = reader.stream_position()?;
        reader.seek(std::io::SeekFrom::Start(current_pos))?;
        
        if file_size == 0 {
            return Err(GaiaError::custom_error("File is empty".to_string()));
        }
        
        // 读取 .luac 文件头
        let header = self.read_header(&mut reader)?;
        
        // 从配置或头部确定版本
        let version = if self.config.version != LuaVersion::Unknown {
            self.config.version
        } else {
            LuaVersion::from_byte(header.version.to_byte())
        };
        
        Ok(LuacInfo { header, version })
    }

    fn read_program(&self) -> Result<LuaProgram, GaiaError> {
        let mut reader = self.reader.borrow_mut();
        
        // 重新定位到文件开头
        reader.seek(std::io::SeekFrom::Start(0))?;
        
        // 读取头部信息
        let header = self.read_header(&mut reader)?;
        
        // 确定 Lua 版本
        let version = if self.config.version != LuaVersion::Unknown {
            self.config.version
        } else {
            LuaVersion::from_byte(header.version.to_byte())
        };
        
        // 读取代码对象
        let code_object = self.read_code_object(&mut reader)?;
        
        // 构建 LuaProgram
        let program = LuaProgram {
            header,
            code_object,
        };
        
        Ok(program)
    }

    fn read_header(&self, reader: &mut BinaryReader<R, LittleEndian>) -> Result<LuacHeader, GaiaError> {
        // 读取 Lua 字节码魔数 "\x1bLua"
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic).map_err(|e| {
            GaiaError::custom_error(format!("Failed to read magic bytes: {}", e))
        })?;
        
        // 验证魔数（如果配置要求检查）
        if self.config.check_magic_head {
            let expected_magic = [0x1b, b'L', b'u', b'a'];
            if magic != expected_magic {
                return Err(GaiaError::custom_error(format!(
                    "Invalid Lua bytecode magic: expected {:?}, got {:?}",
                    expected_magic, magic
                )));
            }
        }
        
        // 读取版本信息
        let version_byte = reader.read_u8().map_err(|e| {
            GaiaError::custom_error(format!("Failed to read version byte: {}", e))
        })?;
        let version = LuaVersion::from_byte(version_byte);
        
        // 读取格式版本
        let format_version = reader.read_u8().map_err(|e| {
            GaiaError::custom_error(format!("Failed to read format version: {}", e))
        })?;
        
        // 读取字节序标识
        let endianness = reader.read_u8().map_err(|e| {
            GaiaError::custom_error(format!("Failed to read endianness: {}", e))
        })?;
        
        // 读取各种大小信息
        let int_size = reader.read_u8().map_err(|e| {
            GaiaError::custom_error(format!("Failed to read int_size: {}", e))
        })?;
        let size_t_size = reader.read_u8().map_err(|e| {
            GaiaError::custom_error(format!("Failed to read size_t_size: {}", e))
        })?;
        let instruction_size = reader.read_u8().map_err(|e| {
            GaiaError::custom_error(format!("Failed to read instruction_size: {}", e))
        })?;
        let lua_number_size = reader.read_u8().map_err(|e| {
            GaiaError::custom_error(format!("Failed to read lua_number_size: {}", e))
        })?;
        let integral_flag = reader.read_u8().map_err(|e| {
            GaiaError::custom_error(format!("Failed to read integral_flag: {}", e))
        })?;
        
        Ok(LuacHeader {
            magic,
            version,
            format_version,
            endianness,
            int_size,
            size_t_size,
            instruction_size,
            lua_number_size,
            integral_flag,
            flags: 0,
            timestamp: None,
            size: None,
            hash: None,
        })
    }

    fn read_code_object(&self, _reader: &mut BinaryReader<R, LittleEndian>) -> Result<LuacCodeObject, GaiaError> {
        // 暂时返回默认的代码对象
        // 完整的实现需要解析 Lua 字节码的复杂结构
        Ok(LuacCodeObject {
            source_name: String::new(),
            first_line: 0,
            last_line: 0,
            num_params: 0,
            is_vararg: 0,
            max_stack_size: 0,
            nested_functions: Vec::new(),
            upvalues: Vec::new(),
            local_vars: Vec::new(),
            line_info: Vec::new(),
            co_argcount: 0,
            co_nlocal: 0,
            co_stacks: 0,
            num_upval: 0,
            co_code: Vec::new(),
            co_consts: Vec::new(),
            upvalue_n: 0,
        })
    }
}

impl Default for LuaProgram {
    fn default() -> Self {
        Self {
            header: LuacHeader::default(),
            code_object: LuacCodeObject::default(),
        }
    }
}

impl Default for LuacHeader {
    fn default() -> Self {
        Self {
            magic: [0x1b, b'L', b'u', b'a'],
            version: LuaVersion::Unknown,
            format_version: 0,
            endianness: 1, // 小端序
            int_size: 4,
            size_t_size: 8,
            instruction_size: 4,
            lua_number_size: 8,
            integral_flag: 0,
            flags: 0,
            timestamp: None,
            size: None,
            hash: None,
        }
    }
}

impl Default for LuacCodeObject {
    fn default() -> Self {
        Self {
            source_name: String::new(),
            first_line: 0,
            last_line: 0,
            num_params: 0,
            is_vararg: 0,
            max_stack_size: 0,
            nested_functions: Vec::new(),
            upvalues: Vec::new(),
            local_vars: Vec::new(),
            line_info: Vec::new(),
            co_argcount: 0,
            co_nlocal: 0,
            co_stacks: 0,
            num_upval: 0,
            co_code: Vec::new(),
            co_consts: Vec::new(),
            upvalue_n: 0,
        }
    }
}
