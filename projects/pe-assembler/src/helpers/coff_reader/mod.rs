use crate::types::coff::{
    CoffFileType, CoffHeader, CoffInfo, CoffObject, CoffRelocation, CoffSection, CoffSymbol, SectionHeader,
};
use byteorder::{LittleEndian, ReadBytesExt};
use gaia_types::{helpers::Architecture, BinaryReader, GaiaError};
use std::io::{Read, Seek};

/// COFF 文件查看器，用于读取 COFF 对象
#[derive(Debug)]
pub struct CoffViewer<W> {
    pub viewer: BinaryReader<W, LittleEndian>,
}

impl<W> CoffViewer<W> {
    pub fn new(reader: W) -> Self {
        Self { viewer: BinaryReader::new(reader) }
    }
}

/// COFF 文件读取器的通用 trait
pub trait CoffReader<R: Read + Seek> {
    /// 获取二进制读取器的可变引用
    fn get_viewer(&mut self) -> &mut BinaryReader<R, LittleEndian>;

    /// 获取诊断信息的可变引用
    fn add_diagnostics(&mut self, error: impl Into<GaiaError>);

    /// 获取缓存的节头信息
    fn get_cached_section_headers(&self) -> Option<&Vec<SectionHeader>>;

    /// 设置缓存的节头信息
    fn set_cached_section_headers(&mut self, headers: Vec<SectionHeader>);

    /// 读取 COFF 头部信息（通用实现）
    fn read_header_once(&mut self) -> Result<&CoffHeader, GaiaError>;

    /// 强制读取 COFF 头部（通用实现）
    fn read_header_force(&mut self) -> Result<CoffHeader, GaiaError> {
        // 保存当前位置
        let original_pos = self.get_viewer().get_position();

        // 重置到文件开头
        self.get_viewer().set_position(0)?;

        // 读取 COFF 头
        let coff_header = CoffHeader::read(self.get_viewer())?;

        // 验证机器类型
        match coff_header.machine {
            0x014c | 0x8664 | 0x01c0 | 0xaa64 => {} // 支持的架构
            unknown => {
                let error = GaiaError::invalid_data(&format!("不支持的机器类型: 0x{:04x}", unknown));
                self.add_diagnostics(error);
            }
        }

        // 验证节数量
        if coff_header.number_of_sections == 0 {
            let error = GaiaError::invalid_data("COFF 文件必须至少有一个节");
            self.add_diagnostics(error);
        }

        // 恢复原始位置
        self.get_viewer().set_position(original_pos)?;

        Ok(coff_header)
    }

    /// 读取节头信息（通用实现）
    fn read_section_headers(&mut self) -> Result<Vec<SectionHeader>, GaiaError> {
        if let Some(sections) = self.get_cached_section_headers() {
            return Ok(sections.clone());
        }

        // 先读取主头部
        let header = self.read_header_once()?.clone();
        let original_pos = self.get_viewer().get_position();

        // 读取节头部
        let mut section_headers = Vec::with_capacity(header.number_of_sections as usize);

        // 定位到节头部位置（COFF 头后面）
        let section_header_offset = std::mem::size_of::<CoffHeader>() as u64 + header.size_of_optional_header as u64;

        self.get_viewer().set_position(section_header_offset)?;

        for _ in 0..header.number_of_sections {
            let section_header = SectionHeader::read(self.get_viewer())?;
            section_headers.push(section_header);
        }

        // 恢复原始位置
        self.get_viewer().set_position(original_pos)?;

        self.set_cached_section_headers(section_headers.clone());
        Ok(section_headers)
    }

    /// 从节头读取节数据（通用实现）
    fn read_section_from_header(&mut self, header: &SectionHeader) -> Result<CoffSection, GaiaError> {
        let viewer = self.get_viewer();

        let mut data = Vec::new();
        let mut relocations = Vec::new();

        // 读取节数据
        if header.size_of_raw_data > 0 && header.pointer_to_raw_data > 0 {
            let original_pos = viewer.get_position();
            viewer.set_position(header.pointer_to_raw_data as u64)?;
            data.resize(header.size_of_raw_data as usize, 0);
            viewer.read_exact(&mut data)?;
            viewer.set_position(original_pos)?;
        }

        // 读取重定位信息
        if header.number_of_relocations > 0 && header.pointer_to_relocations > 0 {
            let original_pos = viewer.get_position();
            viewer.set_position(header.pointer_to_relocations as u64)?;

            for _ in 0..header.number_of_relocations {
                let mut coff_viewer = CoffViewer::new(&mut *viewer);
                let relocation = CoffRelocation::read(&mut coff_viewer)?;
                relocations.push(relocation);
            }

            viewer.set_position(original_pos)?;
        }

        Ok(CoffSection { header: *header, data, relocations })
    }

    /// 读取符号表（通用实现）
    fn read_symbols(&mut self) -> Result<Vec<CoffSymbol>, GaiaError> {
        let header = self.read_header_once()?.clone();

        if header.number_of_symbols == 0 || header.pointer_to_symbol_table == 0 {
            return Ok(Vec::new());
        }

        let original_pos = self.get_viewer().get_position();
        self.get_viewer().set_position(header.pointer_to_symbol_table as u64)?;

        let mut symbols = Vec::new();
        for _ in 0..header.number_of_symbols {
            let mut coff_viewer = CoffViewer::new(&mut *self.get_viewer());
            let symbol = CoffSymbol::read(&mut coff_viewer)?;
            symbols.push(symbol);
        }

        self.get_viewer().set_position(original_pos)?;
        Ok(symbols)
    }

    /// 创建 COFF 信息视图（通用实现）
    fn create_coff_info(&mut self) -> Result<CoffInfo, GaiaError> {
        let header = self.read_header_once()?.clone();
        let viewer = self.get_viewer();

        // 根据机器类型确定架构
        let target_arch = match header.machine {
            0x014c => Architecture::X86,
            0x8664 => Architecture::X86_64,
            0x01c0 => Architecture::ARM32,
            0xaa64 => Architecture::ARM64,
            _ => Architecture::Unknown,
        };

        // 获取当前文件大小
        let current_pos = viewer.get_position();
        viewer.seek(std::io::SeekFrom::End(0))?;
        let file_size = viewer.get_position();
        viewer.set_position(current_pos)?;

        Ok(CoffInfo {
            file_type: CoffFileType::Object,
            target_arch,
            section_count: header.number_of_sections,
            symbol_count: header.number_of_symbols,
            file_size,
            timestamp: header.time_date_stamp,
        })
    }

    /// 强制读取完整的 COFF 对象，并缓存结果
    fn read_object_once(&mut self) -> Result<&CoffObject, GaiaError>;

    /// 强制读取完整的 COFF 对象
    fn read_object_force(&mut self) -> Result<CoffObject, GaiaError> {
        let header = self.read_header_once()?.clone();
        let section_headers = self.read_section_headers()?;

        // 读取节数据
        let mut sections = Vec::new();
        for section_header in section_headers {
            let section = self.read_section_from_header(&section_header)?;
            sections.push(section);
        }

        // 读取符号表
        let symbols = self.read_symbols()?;

        // 读取字符串表（简化实现）
        let string_table = Vec::new();

        Ok(CoffObject { header, sections, symbols, string_table })
    }
}

impl CoffRelocation {
    /// 从 CoffViewer 读取重定位项
    pub fn read<R: ReadBytesExt>(reader: &mut CoffViewer<R>) -> Result<Self, GaiaError> {
        Ok(CoffRelocation {
            virtual_address: reader.viewer.read_u32()?,
            symbol_table_index: reader.viewer.read_u32()?,
            relocation_type: reader.viewer.read_u16()?,
        })
    }
}

impl CoffSymbol {
    /// 从 CoffViewer 读取符号
    pub fn read<R: ReadBytesExt>(reader: &mut CoffViewer<R>) -> Result<Self, GaiaError> {
        let mut name_bytes = [0u8; 8];
        reader.viewer.read_exact(&mut name_bytes)?;

        let name = if name_bytes[0..4] == [0, 0, 0, 0] {
            // 长名称，存储在字符串表中
            format!("symbol_{}", u32::from_le_bytes([name_bytes[4], name_bytes[5], name_bytes[6], name_bytes[7]]))
        }
        else {
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
