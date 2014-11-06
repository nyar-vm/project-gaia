use crate::types::{
    ArchiveMember, ArchiveMemberHeader, CoffFileType, CoffHeader, CoffInfo, CoffObject, CoffRelocation, CoffSection,
    CoffSymbol, SectionHeader, StaticLibrary,
};
use byteorder::{LittleEndian, ReadBytesExt};
use gaia_types::{helpers::Architecture, reader::BinaryReader, GaiaError};
use std::{
    fs::File,
    io::{Cursor, Read, Seek},
    path::Path,
};

/// COFF 读取器配置
///
/// 控制 COFF 对象文件的解析行为和深度。
#[derive(Debug, Copy, Clone)]
pub struct CoffReader {
    /// 是否包含节数据
    pub include_section_data: bool,
    /// 是否解析符号表
    pub parse_symbols: bool,
    /// 是否解析重定位表
    pub parse_relocations: bool,
}

impl Default for CoffReader {
    fn default() -> Self {
        Self::new()
    }
}

impl CoffReader {
    /// 创建新的 COFF 读取器，默认配置
    pub fn new() -> Self {
        Self { include_section_data: true, parse_symbols: true, parse_relocations: true }
    }

    /// 从文件读取 COFF 对象
    pub fn read_file<P: AsRef<Path>>(self, path: P) -> Result<CoffObject, GaiaError> {
        let mut file = File::open(path.as_ref()).map_err(|e| GaiaError::invalid_data(&format!("无法打开文件: {}", e)))?;
        self.read(&mut file)
    }

    /// 从读取器读取 COFF 对象
    pub fn read<R: Read + Seek>(self, reader: R) -> Result<CoffObject, GaiaError> {
        let mut viewer = CoffViewer::new(reader);
        viewer.read_object(self)
    }

    /// 检测文件类型
    pub fn detect_file_type<P: AsRef<Path>>(path: P) -> Result<CoffFileType, GaiaError> {
        let mut file = File::open(path.as_ref()).map_err(|e| GaiaError::invalid_data(&format!("无法打开文件: {}", e)))?;

        let mut magic = [0u8; 8];
        file.read_exact(&mut magic).map_err(|e| GaiaError::invalid_data(&format!("读取文件头失败: {}", e)))?;

        // 检查是否为静态库文件
        if &magic == b"!<arch>\n" {
            return Ok(CoffFileType::StaticLibrary);
        }

        // 检查是否为 PE 文件
        if magic[0] == 0x4D && magic[1] == 0x5A {
            // DOS 头签名 "MZ"
            return Ok(CoffFileType::Executable);
        }

        // 默认为对象文件
        Ok(CoffFileType::Object)
    }

    /// 获取文件信息
    pub fn get_file_info<P: AsRef<Path>>(path: P) -> Result<CoffInfo, GaiaError> {
        let file_type = Self::detect_file_type(&path)?;
        let metadata = std::fs::metadata(&path).map_err(|e| GaiaError::invalid_data(&format!("获取文件元数据失败: {}", e)))?;

        match file_type {
            CoffFileType::Object => {
                let obj = Self::new().read_file(&path)?;
                Ok(CoffInfo {
                    file_type,
                    target_arch: Self::machine_to_arch(obj.header.machine),
                    section_count: obj.header.number_of_sections,
                    symbol_count: obj.header.number_of_symbols,
                    file_size: metadata.len(),
                    timestamp: obj.header.time_date_stamp,
                })
            }
            CoffFileType::StaticLibrary => {
                let lib = read_lib_from_file(&path)?;
                Ok(CoffInfo {
                    file_type,
                    target_arch: Architecture::Unknown,
                    section_count: 0,
                    symbol_count: lib.symbol_index.len() as u32,
                    file_size: metadata.len(),
                    timestamp: 0,
                })
            }
            _ => Err(GaiaError::invalid_data("不支持的文件类型")),
        }
    }

    fn machine_to_arch(machine: u16) -> Architecture {
        match machine {
            0x014c => Architecture::X86,
            0x8664 => Architecture::X86_64,
            0x01c0 => Architecture::ARM32,
            0xaa64 => Architecture::ARM64,
            unknown => {
                tracing::warn!("未知的机器类型: {:04x}", unknown);
                Architecture::Unknown
            }
        }
    }
}

/// COFF 视图器
///
/// 用于读取和解析 COFF 对象文件的低级接口。
#[derive(Debug)]
pub struct CoffViewer<W> {
    /// 二进制读取器
    viewer: BinaryReader<W, LittleEndian>,
}

impl<W> CoffViewer<W> {
    /// 创建新的 COFF 视图器
    pub fn new(reader: W) -> Self {
        Self { viewer: BinaryReader::new(reader) }
    }
}

impl<W: ReadBytesExt + Seek> CoffViewer<W> {
    /// 读取 COFF 对象文件
    pub fn read_object(&mut self, config: CoffReader) -> Result<CoffObject, GaiaError> {
        // 读取文件头
        let header = self.read_file_header()?;

        // 读取节头
        let mut sections = Vec::new();
        for _ in 0..header.number_of_sections {
            let section_header = self.read_section_header()?;
            let mut section = CoffSection { header: section_header, data: Vec::new(), relocations: Vec::new() };

            // 读取节数据
            if config.include_section_data && section_header.size_of_raw_data > 0 {
                let current_pos = self.viewer.get_position();
                self.viewer.set_position(section_header.pointer_to_raw_data as u64)?;
                section.data = self.viewer.read_bytes(section_header.size_of_raw_data as usize)?;
                self.viewer.set_position(current_pos)?;
            }

            // 读取重定位表
            if config.parse_relocations && section_header.number_of_relocations > 0 {
                let current_pos = self.viewer.get_position();
                self.viewer.set_position(section_header.pointer_to_relocations as u64)?;
                for _ in 0..section_header.number_of_relocations {
                    section.relocations.push(CoffRelocation::read(self)?);
                }
                self.viewer.set_position(current_pos)?;
            }

            sections.push(section);
        }

        // 读取符号表
        let mut symbols = Vec::new();
        let mut string_table = Vec::new();

        if config.parse_symbols && header.number_of_symbols > 0 {
            self.viewer.set_position(header.pointer_to_symbol_table as u64)?;

            // 读取符号
            for _ in 0..header.number_of_symbols {
                symbols.push(CoffSymbol::read(self)?);
            }

            // 读取字符串表
            let string_table_size = self.viewer.read_u32()?;
            if string_table_size > 4 {
                string_table = self.viewer.read_bytes((string_table_size - 4) as usize)?;
            }
        }

        Ok(CoffObject { header, sections, symbols, string_table })
    }

    fn read_file_header(&mut self) -> Result<CoffHeader, GaiaError> {
        Ok(CoffHeader {
            machine: self.viewer.read_u16()?,
            number_of_sections: self.viewer.read_u16()?,
            time_date_stamp: self.viewer.read_u32()?,
            pointer_to_symbol_table: self.viewer.read_u32()?,
            number_of_symbols: self.viewer.read_u32()?,
            size_of_optional_header: self.viewer.read_u16()?,
            characteristics: self.viewer.read_u16()?,
        })
    }

    fn read_section_header(&mut self) -> Result<SectionHeader, GaiaError> {
        let mut name = [0u8; 8];
        self.viewer.read_exact(&mut name)?;

        Ok(SectionHeader {
            name,
            virtual_size: self.viewer.read_u32()?,
            virtual_address: self.viewer.read_u32()?,
            size_of_raw_data: self.viewer.read_u32()?,
            pointer_to_raw_data: self.viewer.read_u32()?,
            pointer_to_relocations: self.viewer.read_u32()?,
            pointer_to_line_numbers: self.viewer.read_u32()?,
            number_of_relocations: self.viewer.read_u16()?,
            number_of_line_numbers: self.viewer.read_u16()?,
            characteristics: self.viewer.read_u32()?,
        })
    }
}

impl CoffSymbol {
    /// 从读取器读取符号
    pub fn read<R: ReadBytesExt>(reader: &mut CoffViewer<R>) -> Result<Self, GaiaError> {
        let mut name_bytes = [0u8; 8];
        reader.viewer.read_exact(&mut name_bytes)?;

        let name = if name_bytes[0..4] == [0, 0, 0, 0] {
            // 名称在字符串表中
            format!("@{}", u32::from_le_bytes([name_bytes[4], name_bytes[5], name_bytes[6], name_bytes[7]]))
        }
        else {
            // 名称直接存储
            String::from_utf8_lossy(&name_bytes).trim_end_matches('\0').to_string()
        };

        Ok(CoffSymbol {
            name,
            value: reader.viewer.read_u32()?,
            section_number: reader.viewer.read_i16()?,
            symbol_type: reader.viewer.read_u16()?,
            storage_class: reader.viewer.read_u8()?,
            number_of_aux_symbols: reader.viewer.read_u8()?,
        })
    }
}

impl CoffRelocation {
    /// 从读取器读取重定位项
    pub fn read<R: ReadBytesExt>(reader: &mut CoffViewer<R>) -> Result<Self, GaiaError> {
        Ok(CoffRelocation {
            virtual_address: reader.viewer.read_u32()?,
            symbol_table_index: reader.viewer.read_u32()?,
            relocation_type: reader.viewer.read_u16()?,
        })
    }
}

/// 库文件读取器配置
#[derive(Debug, Clone, Copy)]
pub struct LibReader {
    pub read_members: bool,
    pub read_symbols: bool,
}

impl Default for LibReader {
    fn default() -> Self {
        Self { read_members: true, read_symbols: true }
    }
}

/// 库文件视图器
#[derive(Debug)]
pub struct LibViewer<W> {
    viewer: BinaryReader<W, LittleEndian>,
    config: LibReader,
}

impl<W> LibViewer<W> {
    /// 创建新的库文件视图器
    pub fn new(reader: W, config: LibReader) -> Self {
        Self { viewer: BinaryReader::new(reader), config }
    }
}

impl<W: ReadBytesExt + Seek> LibViewer<W> {
    /// 检查是否为有效的库文件
    pub fn is_valid_lib(&mut self) -> Result<bool, GaiaError> {
        let mut signature = [0u8; 8];
        self.viewer.read_exact(&mut signature)?;
        Ok(&signature == b"!<arch>\n")
    }

    /// 读取静态库
    pub fn read_library(&mut self) -> Result<StaticLibrary, GaiaError> {
        if !self.is_valid_lib()? {
            return Err(GaiaError::invalid_data("无效的库文件签名"));
        }

        let mut members = Vec::new();
        let mut symbol_index = Vec::new();

        while self.viewer.get_position() < self.get_file_size()? {
            match self.read_member() {
                Ok(member) => members.push(member),
                Err(_) => break, // 到达文件末尾
            }
        }

        Ok(StaticLibrary { signature: "!<arch>\n".to_string(), members, symbol_index })
    }

    fn get_file_size(&mut self) -> Result<u64, GaiaError> {
        use std::io::SeekFrom;
        let current_pos = self.viewer.get_position();
        let size = self.viewer.seek(SeekFrom::End(0))?;
        self.viewer.set_position(current_pos)?;
        Ok(size)
    }

    fn read_member(&mut self) -> Result<ArchiveMember, GaiaError> {
        let header = self.read_member_header()?;
        let data = if self.config.read_members {
            self.viewer.read_bytes(header.size as usize)?
        }
        else {
            self.viewer.skip(header.size as u64)?;
            Vec::new()
        };

        // 对齐到偶数边界
        if header.size % 2 == 1 {
            self.viewer.skip(1)?;
        }

        Ok(ArchiveMember { header, data, coff_object: None })
    }

    fn read_member_header(&mut self) -> Result<ArchiveMemberHeader, GaiaError> {
        let mut name_bytes = [0u8; 16];
        self.viewer.read_exact(&mut name_bytes)?;
        let name = String::from_utf8_lossy(&name_bytes).trim_end_matches(' ').to_string();

        let mut timestamp_bytes = [0u8; 12];
        self.viewer.read_exact(&mut timestamp_bytes)?;
        let timestamp = String::from_utf8_lossy(&timestamp_bytes).trim().parse().unwrap_or(0);

        let mut user_id_bytes = [0u8; 6];
        self.viewer.read_exact(&mut user_id_bytes)?;
        let user_id = String::from_utf8_lossy(&user_id_bytes).trim().parse().unwrap_or(0);

        let mut group_id_bytes = [0u8; 6];
        self.viewer.read_exact(&mut group_id_bytes)?;
        let group_id = String::from_utf8_lossy(&group_id_bytes).trim().parse().unwrap_or(0);

        let mut mode_bytes = [0u8; 8];
        self.viewer.read_exact(&mut mode_bytes)?;
        let mode = u32::from_str_radix(String::from_utf8_lossy(&mode_bytes).trim(), 8).unwrap_or(0);

        let mut size_bytes = [0u8; 10];
        self.viewer.read_exact(&mut size_bytes)?;
        let size = String::from_utf8_lossy(&size_bytes).trim().parse().unwrap_or(0);

        let mut end_marker = [0u8; 2];
        self.viewer.read_exact(&mut end_marker)?;
        if &end_marker != b"`\n" {
            return Err(GaiaError::invalid_data("无效的成员头结束标记"));
        }

        Ok(ArchiveMemberHeader { name, timestamp, user_id, group_id, mode, size })
    }
}

/// 从字节数组读取静态库
pub fn read_lib_from_bytes(data: &[u8]) -> Result<StaticLibrary, GaiaError> {
    let mut viewer = LibViewer::new(Cursor::new(data), LibReader::default());
    viewer.read_library()
}

/// 从文件读取静态库
pub fn read_lib_from_file<P: AsRef<Path>>(path: P) -> Result<StaticLibrary, GaiaError> {
    let file = File::open(path)?;
    let mut viewer = LibViewer::new(file, LibReader::default());
    viewer.read_library()
}

/// 从文件读取 COFF 对象文件
pub fn read_coff_from_file<P: AsRef<Path>>(path: P) -> Result<CoffObject, GaiaError> {
    CoffReader::new().read_file(path)
}
