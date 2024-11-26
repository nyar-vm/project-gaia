use crate::types::{
    coff::{CoffFileType, CoffHeader, CoffInfo, CoffObject, CoffRelocation, CoffSymbol, SectionHeader},
    CoffSection,
};
use byteorder::{LittleEndian, ReadBytesExt};
use gaia_types::{helpers::Architecture, GaiaError};
use std::io::{Read, Seek, SeekFrom};

/// COFF 文件读取器的通用 trait
pub trait CoffReader<R: Read + Seek> {
    /// 获取二进制读取器的可变引用
    fn get_viewer(&mut self) -> &mut R;

    /// 获取诊断信息的可变引用
    fn add_diagnostics(&mut self, error: impl Into<GaiaError>);

    fn get_position(&mut self) -> Result<u64, GaiaError>
    where
        R: Seek,
    {
        Ok(self.get_viewer().stream_position()?)
    }

    fn set_position(&mut self, offset: u64) -> Result<u64, GaiaError>
    where
        R: Seek,
    {
        Ok(self.get_viewer().seek(SeekFrom::Start(offset))?)
    }

    /// 读取 COFF 头部信息（通用实现）
    fn get_coff_header(&mut self) -> Result<&CoffHeader, GaiaError>;

    /// 读取 COFF 头部信息（通用实现）
    fn set_coff_header(&mut self, head: CoffHeader) -> Option<CoffHeader>;

    /// 获取缓存的节头信息
    fn get_section_headers(&mut self) -> Result<&[SectionHeader], GaiaError>;

    /// 设置缓存的节头信息
    fn set_section_headers(&mut self, headers: Vec<SectionHeader>) -> Vec<SectionHeader>;

    /// 强制读取完整的 COFF 对象，并缓存结果
    fn get_coff_object(&mut self) -> Result<&CoffObject, GaiaError>;

    /// 强制读取完整的 COFF 对象，并缓存结果
    fn set_coff_object(&mut self, object: CoffObject) -> Option<CoffObject>;

    /// 获取缓存的 COFF 信息
    fn get_coff_info(&mut self) -> Result<&CoffInfo, GaiaError>;

    /// 设置缓存的 COFF 信息
    fn set_coff_info(&mut self, info: CoffInfo) -> Option<CoffInfo>;

    /// 创建 COFF 信息视图（通用实现）
    fn create_coff_info(&mut self) -> Result<CoffInfo, GaiaError> {
        let header = self.get_coff_header()?.clone();

        // 根据机器类型确定架构
        let target_arch = match header.machine {
            0x014c => Architecture::X86,
            0x8664 => Architecture::X86_64,
            0x01c0 => Architecture::ARM32,
            0xaa64 => Architecture::ARM64,
            _ => Architecture::Unknown,
        };

        // 获取当前文件大小
        let current_pos = self.get_position()?;
        self.get_viewer().seek(SeekFrom::End(0))?;
        let file_size = self.get_position()?;
        self.set_position(current_pos)?;

        Ok(CoffInfo {
            file_type: CoffFileType::Object,
            target_arch,
            section_count: header.number_of_sections,
            symbol_count: header.number_of_symbols,
            file_size,
            timestamp: header.time_date_stamp,
        })
    }
}

impl CoffRelocation {
    /// 从 CoffViewer 读取重定位项
    pub fn read<R: Read>(mut reader: R) -> Result<Self, GaiaError> {
        Ok(CoffRelocation {
            virtual_address: reader.read_u32::<LittleEndian>()?,
            symbol_table_index: reader.read_u32::<LittleEndian>()?,
            relocation_type: reader.read_u16::<LittleEndian>()?,
        })
    }
}

impl CoffSymbol {
    /// 从 CoffViewer 读取符号
    pub fn read<R: Read>(mut reader: R) -> Result<Self, GaiaError> {
        let mut name_bytes = [0u8; 8];
        reader.read_exact(&mut name_bytes)?;

        let name = if name_bytes[0..4] == [0, 0, 0, 0] {
            // 长名称，存储在字符串表中
            format!("symbol_{}", u32::from_le_bytes([name_bytes[4], name_bytes[5], name_bytes[6], name_bytes[7]]))
        }
        else {
            String::from_utf8_lossy(&name_bytes).trim_end_matches('\0').to_string()
        };

        Ok(CoffSymbol {
            name,
            value: reader.read_u32::<LittleEndian>()?,
            section_number: reader.read_i16::<LittleEndian>()?,
            symbol_type: reader.read_u16::<LittleEndian>()?,
            storage_class: reader.read_u8()?,
            number_of_aux_symbols: reader.read_u8()?,
        })
    }
}

impl SectionHeader {
    /// 从 ExeReader 读取节头
    pub fn read<R: Read>(mut reader: R) -> Result<Self, GaiaError> {
        let mut name = [0u8; 8];
        reader.read_exact(&mut name)?;

        Ok(SectionHeader {
            name,
            virtual_size: reader.read_u32::<LittleEndian>()?,
            virtual_address: reader.read_u32::<LittleEndian>()?,
            size_of_raw_data: reader.read_u32::<LittleEndian>()?,
            pointer_to_raw_data: reader.read_u32::<LittleEndian>()?,
            pointer_to_relocations: reader.read_u32::<LittleEndian>()?,
            pointer_to_line_numbers: reader.read_u32::<LittleEndian>()?,
            number_of_relocations: reader.read_u16::<LittleEndian>()?,
            number_of_line_numbers: reader.read_u16::<LittleEndian>()?,
            characteristics: reader.read_u32::<LittleEndian>()?,
        })
    }
}

/// 强制读取 COFF 头部（通用实现）
pub(crate) fn read_coff_header<R: Read + Seek>(reader: &mut impl CoffReader<R>) -> Result<CoffHeader, GaiaError> {
    // 保存当前位置
    let original_pos = reader.get_position()?;

    // 重置到文件开头
    reader.set_position(0)?;

    // 读取 COFF 头
    let coff_header = CoffHeader::read(reader.get_viewer())?;

    // 验证机器类型
    match coff_header.machine {
        0x014c | 0x8664 | 0x01c0 | 0xaa64 => {} // 支持的架构
        unknown => {
            let error = GaiaError::invalid_data(&format!("不支持的机器类型: 0x{:04x}", unknown));
            reader.add_diagnostics(error);
        }
    }

    // 验证节数量
    if coff_header.number_of_sections == 0 {
        let error = GaiaError::invalid_data("COFF 文件必须至少有一个节");
        reader.add_diagnostics(error);
    }

    // 恢复原始位置
    reader.set_position(original_pos)?;

    Ok(coff_header)
}

/// 读取节头信息（通用实现）
pub(crate) fn read_section_headers<R: Read + Seek>(reader: &mut impl CoffReader<R>) -> Result<Vec<SectionHeader>, GaiaError> {
    // 先读取主头部
    let header = reader.get_coff_header()?.clone();
    let original_pos = reader.get_position()?;

    // 读取节头部
    let mut section_headers = Vec::with_capacity(header.number_of_sections as usize);

    // 定位到节头部位置（COFF 头后面）
    let section_header_offset = std::mem::size_of::<CoffHeader>() as u64 + header.size_of_optional_header as u64;

    reader.set_position(section_header_offset)?;

    for _ in 0..header.number_of_sections {
        let section_header = SectionHeader::read(reader.get_viewer())?;
        section_headers.push(section_header);
    }

    // 恢复原始位置
    reader.set_position(original_pos)?;

    Ok(section_headers)
}

/// 从节头读取节数据（通用实现）
pub(crate) fn read_section_from_header<R: Read + Seek>(
    reader: &mut impl CoffReader<R>,
    header: &SectionHeader,
) -> Result<CoffSection, GaiaError> {
    let mut data = Vec::new();
    let mut relocations = Vec::new();

    // 读取节数据
    if header.size_of_raw_data > 0 && header.pointer_to_raw_data > 0 {
        let original_pos = reader.get_position()?;
        reader.set_position(header.pointer_to_raw_data as u64)?;
        data.resize(header.size_of_raw_data as usize, 0);
        reader.get_viewer().read_exact(&mut data)?;
        reader.set_position(original_pos)?;
    }

    // 读取重定位信息
    if header.number_of_relocations > 0 && header.pointer_to_relocations > 0 {
        let original_pos = reader.get_position()?;
        reader.set_position(header.pointer_to_relocations as u64)?;

        for _ in 0..header.number_of_relocations {
            let relocation = CoffRelocation::read(reader.get_viewer())?;
            relocations.push(relocation);
        }

        reader.set_position(original_pos)?;
    }

    Ok(CoffSection { header: *header, data, relocations })
}

/// 读取符号表（通用实现）
pub(crate) fn read_symbols<R: Read + Seek>(reader: &mut impl CoffReader<R>) -> Result<Vec<CoffSymbol>, GaiaError> {
    let header = reader.get_coff_header()?.clone();
    let original_pos = reader.get_position()?;
    
    // 计算符号表偏移
    let symbol_table_offset = header.pointer_to_symbol_table as u64;
    reader.set_position(symbol_table_offset)?;
    
    let mut symbols = Vec::new();
    for _ in 0..header.number_of_symbols {
        let symbol = CoffSymbol::read(reader.get_viewer())?;
        symbols.push(symbol);
    }

    reader.set_position(original_pos)?;
    Ok(symbols)
}

/// 强制读取完整的 COFF 对象
pub(crate) fn read_coff_object<R: Read + Seek>(reader: &mut impl CoffReader<R>) -> Result<CoffObject, GaiaError> {
    let header = reader.get_coff_header()?.clone();
    let section_headers = read_section_headers(reader)?;

    // 读取节数据
    let mut sections = Vec::new();
    for section_header in &section_headers {
        let section = read_section_from_header(reader, section_header)?;
        sections.push(section);
    }

    // 读取符号表
    let symbols = read_symbols(reader)?;

    // 读取字符串表（简化实现）
    let string_table = Vec::new();

    Ok(CoffObject { header, sections, symbols, string_table })
}
