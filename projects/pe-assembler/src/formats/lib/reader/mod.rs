use crate::{
    helpers::CoffReader,
    types::{
        coff::{ArchiveMember, ArchiveMemberHeader, CoffFileType, CoffInfo, StaticLibrary},
        CoffHeader, CoffObject, SectionHeader,
    },
};
use byteorder::ReadBytesExt;
use gaia_types::{
    helpers::{Architecture, Url},
    GaiaDiagnostics, GaiaError,
};
use std::io::{Read, Seek};

/// LIB 结构，惰性读取器
#[derive(Debug)]
pub struct LibReader<R> {
    reader: R,
    url: Option<Url>,
    lazy_library: Option<StaticLibrary>,
    lazy_info: Option<CoffInfo>,
    errors: Vec<GaiaError>,
}

impl<R> LibReader<R> {
    pub fn new(reader: R) -> Self {
        Self { reader, url: None, lazy_library: None, lazy_info: None, errors: vec![] }
    }
    pub fn with_url(mut self, url: Url) -> Self {
        self.url = Some(url);
        self
    }
    pub fn finish(mut self) -> GaiaDiagnostics<StaticLibrary>
    where
        R: Read + Seek,
    {
        if self.lazy_library.is_none() {
            if let Err(e) = self.read_library() {
                return GaiaDiagnostics { result: Err(e), diagnostics: self.errors };
            }
        }
        match self.lazy_library {
            Some(s) => GaiaDiagnostics { result: Ok(s), diagnostics: self.errors },
            None => unreachable!(),
        }
    }
}

impl<R: Read + Seek> CoffReader<R> for LibReader<R> {
    fn get_viewer(&mut self) -> &mut R {
        &mut self.reader
    }

    fn add_diagnostics(&mut self, error: impl Into<GaiaError>) {
        self.errors.push(error.into())
    }

    fn get_coff_header(&mut self) -> Result<&CoffHeader, GaiaError> {
        Err(GaiaError::not_implemented("LibReader 不支持直接读取 COFF 头，请使用成员对象"))
    }

    fn set_coff_header(&mut self, _head: CoffHeader) -> Option<CoffHeader> {
        None // LibReader 不支持设置 COFF 头
    }

    fn get_section_headers(&mut self) -> Result<&[SectionHeader], GaiaError> {
        Err(GaiaError::not_implemented("LibReader 不支持直接读取节头，请使用成员对象"))
    }

    fn set_section_headers(&mut self, _headers: Vec<SectionHeader>) -> Vec<SectionHeader> {
        Vec::new() // LibReader 不支持设置节头
    }

    fn get_coff_object(&mut self) -> Result<&CoffObject, GaiaError> {
        Err(GaiaError::not_implemented("LibReader 不支持直接读取 COFF 对象，请使用成员对象"))
    }

    fn set_coff_object(&mut self, _object: CoffObject) -> Option<CoffObject> {
        None // LibReader 不支持设置 COFF 对象
    }

    fn get_coff_info(&mut self) -> Result<&CoffInfo, GaiaError> {
        if self.lazy_info.is_none() {
            let info = self.create_lib_info()?;
            self.lazy_info = Some(info);
        }
        Ok(self.lazy_info.as_ref().unwrap())
    }

    fn set_coff_info(&mut self, info: CoffInfo) -> Option<CoffInfo> {
        self.lazy_info.replace(info)
    }
}

impl<R: Read + Seek> LibReader<R> {
    /// 检测是否为有效的静态库文件
    pub fn is_valid_lib(&mut self) -> Result<bool, GaiaError> {
        let mut magic = [0u8; 8];
        self.reader.read_exact(&mut magic)?;
        self.reader.seek(std::io::SeekFrom::Start(0))?;
        Ok(&magic == b"!<arch>\n")
    }

    /// 查看静态库文件信息
    pub fn view(&mut self) -> Result<CoffInfo, GaiaError> {
        if let Some(ref info) = self.lazy_info {
            return Ok(info.clone());
        }

        let info = self.create_lib_info()?;
        self.lazy_info = Some(info.clone());
        Ok(info)
    }

    /// 读取静态库
    pub fn read_library(&mut self) -> Result<&StaticLibrary, GaiaError> {
        if self.lazy_library.is_none() {
            self.lazy_library = Some(self.read_library_force()?);
        }
        match self.lazy_library.as_ref() {
            Some(s) => Ok(s),
            None => unreachable!(),
        }
    }

    /// 强制读取静态库（不使用缓存）
    fn read_library_force(&mut self) -> Result<StaticLibrary, GaiaError> {
        // 验证文件头
        if !self.is_valid_lib()? {
            return Err(GaiaError::invalid_data("不是有效的静态库文件"));
        }

        // 跳过文件头 "!<arch>\n" (8字节)
        self.reader.seek(std::io::SeekFrom::Start(8))?;

        let mut members = Vec::new();
        let mut symbol_index = Vec::new();
        let file_size = self.get_file_size()?;

        println!("开始解析库文件，文件大小: {} bytes", file_size);
        println!("跳过文件头后，从位置8开始读取成员");

        // 读取所有成员
        while self.get_position()? < file_size {
            let current_pos = self.get_position()?;
            println!("当前位置: {}, 剩余: {} bytes", current_pos, file_size - current_pos);

            // 检查是否还有足够的数据读取成员头（60字节）
            if current_pos + 60 > file_size {
                println!("剩余数据不足60字节，停止解析");
                break;
            }

            match self.read_member() {
                Ok(member) => {
                    println!("读取到成员: '{}', 大小: {} bytes", member.header.name, member.header.size);

                    // 检查是否是符号表（支持传统格式"/"和现代格式"/<ECSYMBOLS>"）
                    if member.header.name == "/" || member.header.name.starts_with("/<ECSYMBOLS>") {
                        println!("发现符号表: '{}', 开始解析符号", member.header.name);
                        println!("符号表数据大小: {} bytes", member.data.len());
                        if member.data.len() >= 4 {
                            let symbol_count_be =
                                u32::from_be_bytes([member.data[0], member.data[1], member.data[2], member.data[3]]);
                            println!("符号表头部显示符号数量: {}", symbol_count_be);
                            // 打印前16个字节的十六进制内容
                            let preview_len = std::cmp::min(16, member.data.len());
                            let hex_preview: String =
                                member.data[..preview_len].iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");
                            println!("符号表前{}字节内容: {}", preview_len, hex_preview);
                        }
                        // 这是符号表，解析符号
                        match self.parse_symbol_table(&member.data, members.len()) {
                            Ok(symbols) => {
                                println!("成功解析 {} 个符号", symbols.len());
                                if !symbols.is_empty() {
                                    println!("前5个符号: {:?}", &symbols[..std::cmp::min(5, symbols.len())]);
                                }
                                symbol_index.extend(symbols);
                            }
                            Err(e) => {
                                println!("符号表解析失败: {:?}", e);
                            }
                        }
                    }
                    else if member.header.name == "//" {
                        println!("发现扩展名称表，跳过");
                        // 这是扩展名称表，跳过
                    }
                    else {
                        println!("发现普通成员: {}", member.header.name);
                        // 这是普通成员
                    }
                    members.push(member);
                }
                Err(e) => {
                    // 如果读取失败，记录错误但继续
                    println!("读取成员失败: {:?}", e);
                    self.add_diagnostics(e);
                    break;
                }
            }
        }

        println!("解析完成，总成员数: {}, 总符号数: {}", members.len(), symbol_index.len());
        Ok(StaticLibrary { signature: "!<arch>\n".to_string(), members, symbol_index })
    }

    /// 创建库信息
    fn create_lib_info(&mut self) -> Result<CoffInfo, GaiaError> {
        let file_size = self.get_file_size()?;
        let library = self.read_library()?;

        Ok(CoffInfo {
            file_type: CoffFileType::StaticLibrary,
            target_arch: Architecture::Unknown,
            section_count: 0,
            symbol_count: library.symbol_index.len() as u32,
            file_size,
            timestamp: 0,
        })
    }

    /// 获取文件大小
    pub fn get_file_size(&mut self) -> Result<u64, GaiaError> {
        let current_pos = self.get_position()?;
        let size = self.reader.seek(std::io::SeekFrom::End(0))?;
        self.set_position(current_pos)?;
        Ok(size)
    }

    /// 读取成员
    fn read_member(&mut self) -> Result<ArchiveMember, GaiaError> {
        let header = self.read_member_header()?;
        let mut data = vec![0u8; header.size as usize];
        self.reader.read_exact(&mut data)?;

        // 对齐到偶数边界
        if header.size % 2 == 1 {
            self.reader.read_u8()?;
        }

        // 尝试解析 COFF 对象（如果数据是有效的 COFF 格式）
        let coff_object = if data.len() > 20 {
            // 尝试从数据中读取 COFF 对象
            // 这里暂时返回 None，因为需要一个具体的 CoffReader 实现
            // TODO: 实现一个简单的 COFF 对象解析器
            None
        }
        else {
            None
        };

        Ok(ArchiveMember { header, data, coff_object })
    }

    /// 解析符号表
    fn parse_symbol_table(&self, data: &[u8], member_index: usize) -> Result<Vec<(String, usize)>, GaiaError> {
        let mut symbols = Vec::new();

        if data.len() < 4 {
            return Ok(symbols);
        }

        // 读取符号数量（前4个字节，大端序）
        let symbol_count = u32::from_be_bytes([data[0], data[1], data[2], data[3]]) as usize;

        if symbol_count == 0 || symbol_count > 100000 {
            // 符号数量不合理，可能不是标准的符号表格式
            return Ok(symbols);
        }

        // 跳过符号偏移表（每个符号4字节偏移）
        let string_table_start = 4 + symbol_count * 4;

        if string_table_start >= data.len() {
            return Ok(symbols);
        }

        // 解析字符串表
        let string_data = &data[string_table_start..];
        let mut current_pos = 0;

        while current_pos < string_data.len() && symbols.len() < symbol_count {
            // 查找下一个null终止符
            if let Some(null_pos) = string_data[current_pos..].iter().position(|&b| b == 0) {
                if null_pos > 0 {
                    if let Ok(symbol_name) = std::str::from_utf8(&string_data[current_pos..current_pos + null_pos]) {
                        symbols.push((symbol_name.to_string(), member_index));
                    }
                }
                current_pos += null_pos + 1;
            }
            else {
                break;
            }
        }

        Ok(symbols)
    }

    /// 读取成员头
    fn read_member_header(&mut self) -> Result<ArchiveMemberHeader, GaiaError> {
        let mut name = [0u8; 16];
        self.reader.read_exact(&mut name)?;

        let mut date = [0u8; 12];
        self.reader.read_exact(&mut date)?;

        let mut uid = [0u8; 6];
        self.reader.read_exact(&mut uid)?;

        let mut gid = [0u8; 6];
        self.reader.read_exact(&mut gid)?;

        let mut mode = [0u8; 8];
        self.reader.read_exact(&mut mode)?;

        let mut size = [0u8; 10];
        self.reader.read_exact(&mut size)?;

        let mut end_chars = [0u8; 2];
        self.reader.read_exact(&mut end_chars)?;

        println!("成员头结束符: {:02X} {:02X} (期望: 60 0A)", end_chars[0], end_chars[1]);

        if &end_chars != b"`\n" {
            return Err(GaiaError::invalid_data("无效的成员头结束符"));
        }

        // 解析字段 - ar格式的字段都是ASCII字符串，右填充空格
        let name_str = std::str::from_utf8(&name).map_err(|_| GaiaError::invalid_data("无效的名称字段"))?;
        // ar格式中名称以斜杠结尾，然后用空格填充
        let name = name_str.trim_end_matches(' ').trim_end_matches('/').to_string();

        let date_str = std::str::from_utf8(&date).map_err(|_| GaiaError::invalid_data("无效的日期字段"))?;
        let timestamp = date_str.trim_end_matches(' ').parse::<u32>().unwrap_or(0);

        let uid_str = std::str::from_utf8(&uid).map_err(|_| GaiaError::invalid_data("无效的用户ID字段"))?;
        let user_id = uid_str.trim_end_matches(' ').parse::<u16>().unwrap_or(0);

        let gid_str = std::str::from_utf8(&gid).map_err(|_| GaiaError::invalid_data("无效的组ID字段"))?;
        let group_id = gid_str.trim_end_matches(' ').parse::<u16>().unwrap_or(0);

        let mode_str = std::str::from_utf8(&mode).map_err(|_| GaiaError::invalid_data("无效的模式字段"))?;
        let mode = u32::from_str_radix(mode_str.trim_end_matches(' '), 8).unwrap_or(0); // 模式字段是八进制

        let size_str = std::str::from_utf8(&size).map_err(|_| GaiaError::invalid_data("无效的大小字段"))?;
        let size = size_str.trim_end_matches(' ').parse::<u32>().unwrap_or(0);
        Ok(ArchiveMemberHeader { name, timestamp, user_id, group_id, mode, size })
    }
}
