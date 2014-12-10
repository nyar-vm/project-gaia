use crate::types::{
    CoffHeader, DosHeader, ExportTable, ImportTable, NtHeader, OptionalHeader, PeHeader, PeInfo, PeProgram, PeSection,
    SectionHeader,
};
use byteorder::{LittleEndian, ReadBytesExt};
use gaia_types::{helpers::Architecture, GaiaError};
use std::io::{Read, Seek, SeekFrom};

/// PE 文件读取器的通用 trait
pub trait PeReader<R: Read + Seek> {
    /// 获取二进制读取器的可变引用
    fn get_viewer(&mut self) -> &mut R;

    /// 获取诊断信息的可变引用
    fn add_diagnostics(&mut self, error: impl Into<GaiaError>);

    fn get_position(&mut self) -> Result<u64, GaiaError>
    where
        R: Seek,
    {
        let position = self.get_viewer().stream_position()?;
        Ok(position)
    }

    fn set_position(&mut self, offset: u64) -> Result<u64, GaiaError>
    where
        R: Seek,
    {
        let position = self.get_viewer().seek(SeekFrom::Start(offset))?;
        Ok(position)
    }

    /// 获取缓存的节头信息
    fn get_section_headers(&mut self) -> Result<&[SectionHeader], GaiaError>;

    /// 读取 PE 头部信息（通用实现）
    fn get_pe_header(&mut self) -> Result<&PeHeader, GaiaError>;

    /// 将 RVA 转换为文件偏移（通用实现）
    fn rva_to_file_offset(&self, rva: u32, sections: &[PeSection]) -> Result<u32, GaiaError> {
        for section in sections {
            if rva >= section.virtual_address && rva < section.virtual_address + section.virtual_size {
                let offset_in_section = rva - section.virtual_address;
                return Ok(section.pointer_to_raw_data + offset_in_section);
            }
        }
        Err(GaiaError::invalid_data(&format!("无法将 RVA 0x{:08X} 转换为文件偏移", rva)))
    }

    /// 解析导入表（通用实现）
    fn parse_import_table(&mut self, header: &PeHeader, sections: &[PeSection]) -> Result<ImportTable, GaiaError> {
        // 检查数据目录表是否包含导入表信息
        if header.optional_header.data_directories.len() < 2 {
            return Ok(ImportTable::new());
        }

        let import_dir = &header.optional_header.data_directories[1]; // 导入表是第2个数据目录
        if import_dir.virtual_address == 0 || import_dir.size == 0 {
            return Ok(ImportTable::new());
        }

        // 将 RVA 转换为文件偏移
        let file_offset = self.rva_to_file_offset(import_dir.virtual_address, sections)?;

        // 保存当前位置
        let current_pos = self.get_position()?;

        // 定位到导入表
        self.set_position(file_offset as u64)?;

        let mut import_table = ImportTable::new();

        // 读取导入描述符
        loop {
            let import_lookup_table = self.get_viewer().read_u32::<LittleEndian>()?;
            let time_date_stamp = self.get_viewer().read_u32::<LittleEndian>()?;
            let forwarder_chain = self.get_viewer().read_u32::<LittleEndian>()?;
            let name_rva = self.get_viewer().read_u32::<LittleEndian>()?;
            let import_address_table = self.get_viewer().read_u32::<LittleEndian>()?;

            // 如果所有字段都为0，表示导入表结束
            if import_lookup_table == 0
                && time_date_stamp == 0
                && forwarder_chain == 0
                && name_rva == 0
                && import_address_table == 0
            {
                break;
            }

            let mut dll_name = String::new();
            let mut functions = Vec::new();

            // 读取 DLL 名称
            if name_rva != 0 {
                let name_offset = self.rva_to_file_offset(name_rva, sections)?;
                let saved_pos = self.get_position()?;
                self.set_position(name_offset as u64)?;

                let mut name_bytes = Vec::new();
                loop {
                    let byte = self.get_viewer().read_u8()?;
                    if byte == 0 {
                        break;
                    }
                    name_bytes.push(byte);
                }
                dll_name = String::from_utf8_lossy(&name_bytes).to_string();
                self.set_position(saved_pos)?;
            }

            // 读取函数名称（从导入查找表）
            if import_lookup_table != 0 {
                let lookup_offset = self.rva_to_file_offset(import_lookup_table, sections)?;
                let saved_pos = self.get_position()?;
                self.set_position(lookup_offset as u64)?;

                loop {
                    let entry = if header.optional_header.magic == 0x20b {
                        // PE32+
                        self.get_viewer().read_u64::<LittleEndian>()?
                    }
                    else {
                        // PE32
                        self.get_viewer().read_u32::<LittleEndian>()? as u64
                    };

                    if entry == 0 {
                        break;
                    }

                    // 检查是否是按名称导入（最高位为0）
                    let is_ordinal = if header.optional_header.magic == 0x20b {
                        (entry & 0x8000000000000000) != 0
                    }
                    else {
                        (entry & 0x80000000) != 0
                    };

                    if !is_ordinal {
                        let hint_name_rva =
                            entry & if header.optional_header.magic == 0x20b { 0x7FFFFFFFFFFFFFFF } else { 0x7FFFFFFF };
                        let hint_name_offset = self.rva_to_file_offset(hint_name_rva as u32, sections)?;
                        let func_pos = self.get_position()?;
                        self.set_position(hint_name_offset as u64)?;

                        // 跳过 hint（2字节）
                        self.get_viewer().read_u16::<LittleEndian>()?;

                        // 读取函数名
                        let mut func_name_bytes = Vec::new();
                        loop {
                            let byte = self.get_viewer().read_u8()?;
                            if byte == 0 {
                                break;
                            }
                            func_name_bytes.push(byte);
                        }
                        let func_name = String::from_utf8_lossy(&func_name_bytes).to_string();
                        functions.push(func_name);

                        self.set_position(func_pos)?;
                    }
                    else {
                        // 按序号导入
                        let ordinal = entry & 0xFFFF;
                        functions.push(format!("Ordinal_{}", ordinal));
                    }
                }

                self.set_position(saved_pos)?;
            }

            // 添加导入条目
            if !dll_name.is_empty() {
                use crate::types::tables::ImportEntry;
                let entry = ImportEntry { dll_name, functions };
                import_table.entries.push(entry);
            }
        }

        // 恢复位置
        self.set_position(current_pos)?;

        Ok(import_table)
    }

    /// 解析导出表（通用实现）
    fn parse_export_table(&mut self, header: &PeHeader, sections: &[PeSection]) -> Result<ExportTable, GaiaError> {
        // 检查数据目录表是否包含导出表信息
        if header.optional_header.data_directories.is_empty() {
            return Ok(ExportTable { name: String::new(), functions: Vec::new() });
        }

        let export_dir = &header.optional_header.data_directories[0]; // 导出表是第1个数据目录
        if export_dir.virtual_address == 0 || export_dir.size == 0 {
            return Ok(ExportTable { name: String::new(), functions: Vec::new() });
        }

        // 将 RVA 转换为文件偏移
        let file_offset = self.rva_to_file_offset(export_dir.virtual_address, sections)?;

        // 保存当前位置
        let current_pos = self.get_position()?;

        // 定位到导出表
        self.set_position(file_offset as u64)?;

        // 读取导出目录表
        let _export_flags = self.get_viewer().read_u32::<LittleEndian>()?;
        let _time_date_stamp = self.get_viewer().read_u32::<LittleEndian>()?;
        let _major_version = self.get_viewer().read_u16::<LittleEndian>()?;
        let _minor_version = self.get_viewer().read_u16::<LittleEndian>()?;
        let name_rva = self.get_viewer().read_u32::<LittleEndian>()?;
        let _ordinal_base = self.get_viewer().read_u32::<LittleEndian>()?;
        let _number_of_functions = self.get_viewer().read_u32::<LittleEndian>()?;
        let number_of_names = self.get_viewer().read_u32::<LittleEndian>()?;
        let _address_of_functions = self.get_viewer().read_u32::<LittleEndian>()?;
        let address_of_names = self.get_viewer().read_u32::<LittleEndian>()?;
        let _address_of_name_ordinals = self.get_viewer().read_u32::<LittleEndian>()?;

        // 读取模块名称
        let mut name = String::new();
        if name_rva != 0 {
            let name_offset = self.rva_to_file_offset(name_rva, sections)?;
            let saved_pos = self.get_position()?;
            self.set_position(name_offset as u64)?;

            let mut name_bytes = Vec::new();
            loop {
                let byte = self.get_viewer().read_u8()?;
                if byte == 0 {
                    break;
                }
                name_bytes.push(byte);
            }
            name = String::from_utf8_lossy(&name_bytes).to_string();
            self.set_position(saved_pos)?;
        }

        // 读取函数名称
        let mut functions = Vec::new();
        if address_of_names != 0 && number_of_names > 0 {
            let names_offset = self.rva_to_file_offset(address_of_names, sections)?;
            let saved_pos = self.get_position()?;
            self.set_position(names_offset as u64)?;

            for _ in 0..number_of_names {
                let name_rva = self.get_viewer().read_u32::<LittleEndian>()?;
                if name_rva != 0 {
                    let func_name_offset = self.rva_to_file_offset(name_rva, sections)?;
                    let func_pos = self.get_position()?;
                    self.set_position(func_name_offset as u64)?;

                    let mut func_name_bytes = Vec::new();
                    loop {
                        let byte = self.get_viewer().read_u8()?;
                        if byte == 0 {
                            break;
                        }
                        func_name_bytes.push(byte);
                    }
                    let func_name = String::from_utf8_lossy(&func_name_bytes).to_string();
                    functions.push(func_name);

                    self.set_position(func_pos)?;
                }
            }

            self.set_position(saved_pos)?;
        }

        // 恢复位置
        self.set_position(current_pos)?;

        Ok(ExportTable { name, functions })
    }

    /// 创建 PE 信息视图（通用实现）
    fn create_pe_info(&mut self) -> Result<PeInfo, GaiaError> {
        let header = self.get_pe_header()?.clone();

        // 根据机器类型确定架构
        let target_arch = match header.coff_header.machine {
            0x014c => Architecture::X86,
            0x8664 => Architecture::X86_64,
            0x01c0 => Architecture::ARM32,
            0xaa64 => Architecture::ARM64,
            unknown => {
                tracing::warn!("未知的机器类型: {:04x}", unknown);
                Architecture::Unknown
            }
        };

        // 获取当前文件大小
        let current_pos = self.get_position()?;
        self.get_viewer().seek(std::io::SeekFrom::End(0))?;
        let file_size = self.get_position()?;
        self.set_position(current_pos)?;

        Ok(PeInfo {
            target_arch,
            subsystem: header.optional_header.subsystem,
            entry_point: header.optional_header.address_of_entry_point,
            image_base: header.optional_header.image_base,
            section_count: header.coff_header.number_of_sections,
            file_size,
        })
    }
    /// 强制读取完整的 [PeProgram]，并缓存结果
    fn get_program(&mut self) -> Result<&PeProgram, GaiaError>;
}

/// 解析 PE 头部（通用实现）
pub fn read_pe_head<R: Read + Seek>(reader: &mut impl PeReader<R>) -> Result<PeHeader, GaiaError> {
    // 保存当前位置
    let original_pos = reader.get_position()?;

    // 重置到文件开头
    reader.set_position(0)?;

    // 读取 DOS 头
    let dos_header = DosHeader::read(reader.get_viewer())?;

    // 验证 DOS 签名 (MZ)
    if dos_header.e_magic != 0x5A4D {
        let error = GaiaError::invalid_data("无效的 DOS 签名 (MZ)");
        reader.add_diagnostics(error);
    }

    // 跳转到 NT 头位置
    reader.set_position(dos_header.e_lfanew as u64)?;

    // 读取 NT 头
    let nt_header = NtHeader::read(reader.get_viewer())?;

    // 验证 PE 签名 (PE\0\0)
    if nt_header.signature != 0x00004550 {
        let error = GaiaError::invalid_data("无效的 PE 签名 (PE)");
        reader.add_diagnostics(error);
    }

    // 读取 COFF 头
    let coff_header = CoffHeader::read(reader.get_viewer())?;

    // 验证 COFF 头中的节数量
    if coff_header.number_of_sections == 0 {
        let error = GaiaError::invalid_data("PE 文件必须至少有一个节");
        reader.add_diagnostics(error);
    }

    // 读取可选头
    let optional_header = OptionalHeader::read(reader.get_viewer())?;

    // 验证可选头的魔数
    match optional_header.magic {
        0x10b => {} // PE32
        0x20b => {} // PE32+
        _ => {
            let error = GaiaError::invalid_data("无效的可选头魔数");
            reader.add_diagnostics(error);
            return Err(GaiaError::invalid_data("无效的可选头魔数"));
        }
    }

    // 恢复原始位置
    reader.set_position(original_pos)?;

    Ok(PeHeader { dos_header, nt_header, coff_header, optional_header })
}

/// 读取节头信息（通用实现）
pub fn read_pe_section_headers<R: Read + Seek>(reader: &mut impl PeReader<R>) -> Result<Vec<SectionHeader>, GaiaError> {
    // 先读取主头部
    let header = reader.get_pe_header()?.clone();
    let original_pos = reader.get_position()?;

    // 读取节头部
    let mut section_headers = Vec::with_capacity(header.coff_header.number_of_sections as usize);

    // 定位到节头部位置
    let section_header_offset = header.dos_header.e_lfanew as u64
        + 4 // PE signature
        + std::mem::size_of::<CoffHeader>() as u64
        + header.coff_header.size_of_optional_header as u64;

    reader.set_position(section_header_offset)?;

    for _ in 0..header.coff_header.number_of_sections {
        let section_header = SectionHeader::read(reader.get_viewer())?;
        section_headers.push(section_header);
    }

    // 恢复原始位置
    reader.set_position(original_pos)?;

    Ok(section_headers)
}

/// 从节头读取节数据（通用实现）
pub fn read_section_from_header<R: Read + Seek>(
    reader: &mut impl PeReader<R>,
    header: &SectionHeader,
) -> Result<PeSection, GaiaError> {
    let name = String::from_utf8_lossy(&header.name).trim_end_matches('\0').to_string();

    let mut data = Vec::new();
    if header.size_of_raw_data > 0 && header.pointer_to_raw_data > 0 {
        let original_pos = reader.get_position()?;
        reader.set_position(header.pointer_to_raw_data as u64)?;
        data.resize(header.size_of_raw_data as usize, 0);
        reader.get_viewer().read_exact(&mut data)?;
        reader.set_position(original_pos)?;
    }

    Ok(PeSection {
        name,
        virtual_size: header.virtual_size,
        virtual_address: header.virtual_address,
        size_of_raw_data: header.size_of_raw_data,
        pointer_to_raw_data: header.pointer_to_raw_data,
        pointer_to_relocations: header.pointer_to_relocations,
        pointer_to_line_numbers: header.pointer_to_line_numbers,
        number_of_relocations: header.number_of_relocations,
        number_of_line_numbers: header.number_of_line_numbers,
        characteristics: header.characteristics,
        data,
    })
}

pub fn read_pe_program<R: Read + Seek>(reader: &mut impl PeReader<R>) -> Result<PeProgram, GaiaError> {
    let header = reader.get_pe_header()?.clone();
    let section_headers = reader.get_section_headers()?.to_vec();

    // 读取节数据
    let mut sections = Vec::new();
    for section_header in section_headers {
        let section = read_section_from_header(reader, &section_header)?;
        sections.push(section);
    }

    // 解析导入表
    let imports = reader.parse_import_table(&header, &sections)?;

    // 解析导出表（EXE 文件通常没有导出表）
    let exports = reader.parse_export_table(&header, &sections)?;
    Ok(PeProgram { header, sections, imports, exports })
}
