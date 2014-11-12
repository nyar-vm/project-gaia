#![doc = include_str!("readme.md")]

use crate::{
    formats::pyc::{view::PycView, PycReadConfig},
    program::PycHeader,
};
use byteorder::{LittleEndian, ReadBytesExt};
use gaia_types::GaiaError;
use std::io::{Read, Seek, SeekFrom};

mod marshal;
use marshal::MarshalParser;

/// 负责解析 .pyc 文件到 [PycView] 的读取器
#[derive(Debug)]
/// PycReader 结构体用于从输入流中读取字节码。
pub struct PycReader<'config, R: Read + Seek> {
    pub(crate) reader: R,
    pub(crate) view: PycView,
    // week errors
    pub(crate) errors: Vec<GaiaError>,
    pub(crate) _config: &'config PycReadConfig,
    pub(crate) offset: u64,
}

impl PycReadConfig {
    /// 将 PycReadConfig 转换为 PycReader。
    pub fn as_reader<R: Read + Seek>(&self, reader: R) -> PycReader<'_, R> {
        PycReader::new(reader, self)
    }
}

impl<'config, R: Read + Seek> PycReader<'config, R> {
    /// 创建一个新的 PycReader 实例。
    pub fn new(reader: R, config: &'config PycReadConfig) -> Self {
        Self { reader, view: PycView::default(), errors: vec![], _config: config, offset: 0 }
    }

    /// 获取当前读取器在输入流中的偏移量。
    pub fn get_offset(&self) -> u64 {
        self.offset
    }

    /// 设置当前读取器在输入流中的偏移量。
    pub fn set_offset(&mut self, offset: u64) -> Result<(), GaiaError>
    where
        R: Seek,
    {
        self.reader.seek(SeekFrom::Start(offset))?;
        Ok(self.offset = offset)
    }
}

impl<'config, R: Read + Seek> PycReader<'config, R> {
    /// 从输入流中读取 PycView。
    pub fn read(mut self) -> Result<PycView, GaiaError> {
        self.set_offset(0)?;
        self.read_to_end()?;
        Ok(self.view)
    }

    /// 读取到输入流的末尾。
    pub fn read_to_end(&mut self) -> Result<(), GaiaError> {
        let mut magic = [0u8; 4];
        self.reader.read_exact(&mut magic)?;
        let flags = self.reader.read_u32::<LittleEndian>()?;
        let timestamp = self.reader.read_u32::<LittleEndian>()?;
        let size = self.reader.read_u32::<LittleEndian>()?;

        let mut code_object_bytes = Vec::new();
        self.reader.read_to_end(&mut code_object_bytes)?;

        self.view.header = PycHeader { magic, flags, timestamp, size };
        self.view.code_object_bytes = code_object_bytes.clone();

        // 尝试解析 marshal 数据
        if !self.view.code_object_bytes.is_empty() {
            let mut parser = MarshalParser::new(&self.view.code_object_bytes);
            match parser.parse_code_object() {
                Ok(code_object) => {
                    // 直接从 PythonCodeObject 提取数据到 PycView
                    self.view.constants = code_object.co_consts;
                    self.view.filename = code_object.source_name;
                    self.view.argcount = code_object.co_argcount as u32;
                    self.view.nlocals = code_object.co_nlocal as u32;
                    self.view.stacksize = code_object.co_stacks as u32;
                    self.view.firstlineno = code_object.first_line;
                    self.view.lnotab = code_object.line_info;

                    // 从 upvalues 和 local_vars 提取名称列表
                    self.view.freevars = code_object.upvalues.iter().map(|uv| uv.name.clone()).collect();
                    self.view.varnames = code_object.local_vars.iter().map(|lv| lv.name.clone()).collect();

                    // 暂时保留原始字节码，后续可能需要重新解析指令
                }
                Err(e) => {
                    // 如果解析失败，记录错误但不中断
                    self.errors.push(e);
                }
            }
        }

        Ok(())
    }
}
