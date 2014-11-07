use crate::{
    assembler::{ExportTable, ImportTable},
    types::{
        DataDirectory, DosHeader, NtHeader, OptionalHeader, PeHeader, PeInfo, PeProgram, PeSection, ReadConfig, SectionHeader,
    },
};
use byteorder::{LittleEndian, ReadBytesExt};
use gaia_types::{helpers::Architecture, reader::BinaryReader, GaiaError};
use pe_coff::types::CoffHeader;
use std::{
    fs::File,
    io::{Cursor, Read, Seek},
    path::Path,
};

/// PE 视图结构
///
/// 轻量级视图，只持有 BinaryReader 与解析后的关键信息。
#[derive(Debug)]
pub struct PeReader<R> {
    /// 二进制读取器（已定位到 DOS 头起始位置）
    viewer: BinaryReader<R, LittleEndian>,
    headers_read: bool,
    lazy_header: Option<PeHeader>,
    lazy_section_headers: Option<Vec<SectionHeader>>,
    lazy_program: Option<PeProgram>,
    lazy_info: Option<PeInfo>,
}

impl<R> PeReader<R> {
    pub fn new(reader: R) -> Self {
        Self {
            viewer: BinaryReader::new(reader),
            headers_read: false,
            lazy_header: None,
            lazy_section_headers: None,
            lazy_program: None,
            lazy_info: None,
        }
    }
}

impl<W: Read> PeReader<W> {
    /// 读取 PE 头部信息（惰性读取，会缓存结果）
    fn read_headers(&mut self) -> Result<PeHeader, GaiaError>
    where
        W: Seek,
    {
        if self.headers_read {
            return Ok(self.lazy_header.clone().unwrap());
        }

        // 读取完整的头部信息（包含完整的PE验证）
        let header = PeHeader::read(self)?;

        self.lazy_header = Some(header.clone());
        self.headers_read = true;
        Ok(header)
    }

    /// 读取节头信息（惰性读取，会缓存结果）
    fn read_section_headers(&mut self) -> Result<Vec<SectionHeader>, GaiaError>
    where
        W: Seek,
    {
        if let Some(ref sections) = self.lazy_section_headers {
            return Ok(sections.clone());
        }

        // 先读取主头部
        let header = self.read_headers()?;
        let original_pos = self.viewer.get_position();

        // 读取节头部
        let mut section_headers = Vec::with_capacity(header.coff_header.number_of_sections as usize);

        // 定位到节头部位置
        let section_header_offset = header.dos_header.e_lfanew as u64
            + 4 // PE signature
            + std::mem::size_of::<CoffHeader>() as u64
            + header.coff_header.size_of_optional_header as u64;

        self.viewer.set_position(section_header_offset)?;

        for _ in 0..header.coff_header.number_of_sections {
            let section_header = SectionHeader::read(self)?;
            section_headers.push(section_header);
        }

        // 恢复原始位置
        self.viewer.set_position(original_pos)?;

        self.lazy_section_headers = Some(section_headers.clone());
        Ok(section_headers)
    }

    /// 读取完整的 PE 程序（惰性读取，会缓存结果）
    pub fn read_program(&mut self) -> Result<PeProgram, GaiaError>
    where
        W: Seek,
    {
        if let Some(ref program) = self.lazy_program {
            return Ok(program.clone());
        }

        // 读取头部信息
        let header = self.read_headers()?;

        // 读取节信息
        let section_headers = self.read_section_headers()?;
        let mut sections = Vec::new();

        for section_header in section_headers {
            let mut section = PeSection {
                name: String::from_utf8_lossy(&section_header.name).trim_matches('\0').to_string(),
                virtual_size: section_header.virtual_size,
                virtual_address: section_header.virtual_address,
                size_of_raw_data: section_header.size_of_raw_data,
                pointer_to_raw_data: section_header.pointer_to_raw_data,
                pointer_to_relocations: section_header.pointer_to_relocations,
                pointer_to_line_numbers: section_header.pointer_to_line_numbers,
                number_of_relocations: section_header.number_of_relocations,
                number_of_line_numbers: section_header.number_of_line_numbers,
                characteristics: section_header.characteristics,
                data: Vec::new(),
            };

            // 读取节数据
            if section_header.size_of_raw_data > 0 && section_header.pointer_to_raw_data > 0 {
                self.viewer.set_position(section_header.pointer_to_raw_data as u64)?;
                section.data = self.viewer.read_bytes(section_header.size_of_raw_data as usize)?;
            }

            sections.push(section);
        }

        // 解析导入表和导出表
        let imports = self.parse_import_table(&header, &sections)?;
        let exports = self.parse_export_table(&header, &sections)?;

        let program = PeProgram { header, sections, imports, exports };
        self.lazy_program = Some(program.clone());
        Ok(program)
    }

    /// 解析导入表
    fn parse_import_table(&mut self, header: &PeHeader, sections: &[PeSection]) -> Result<ImportTable, GaiaError>
    where
        W: Seek,
    {
        // 检查数据目录表是否包含导入表信息
        if header.optional_header.data_directories.len() <= 1 {
            return Ok(ImportTable { dll_name: String::new(), functions: Vec::new() });
        }

        let import_dir = &header.optional_header.data_directories[1]; // 导入表是第2个数据目录
        if import_dir.virtual_address == 0 || import_dir.size == 0 {
            return Ok(ImportTable { dll_name: String::new(), functions: Vec::new() });
        }

        // 将 RVA 转换为文件偏移
        let file_offset = self.rva_to_file_offset(import_dir.virtual_address, sections)?;

        // 保存当前位置
        let current_pos = self.viewer.get_position();

        // 定位到导入表
        self.viewer.set_position(file_offset as u64)?;

        let mut functions = Vec::new();
        let mut dll_name = String::new();

        // 读取导入描述符
        loop {
            let import_lookup_table = self.viewer.read_u32()?;
            let time_date_stamp = self.viewer.read_u32()?;
            let forwarder_chain = self.viewer.read_u32()?;
            let name_rva = self.viewer.read_u32()?;
            let import_address_table = self.viewer.read_u32()?;

            // 如果所有字段都为0，表示导入表结束
            if import_lookup_table == 0
                && time_date_stamp == 0
                && forwarder_chain == 0
                && name_rva == 0
                && import_address_table == 0
            {
                break;
            }

            // 读取 DLL 名称
            if name_rva != 0 && dll_name.is_empty() {
                let name_offset = self.rva_to_file_offset(name_rva, sections)?;
                let saved_pos = self.viewer.get_position();
                self.viewer.set_position(name_offset as u64)?;

                let mut name_bytes = Vec::new();
                loop {
                    let byte = self.viewer.read_u8()?;
                    if byte == 0 {
                        break;
                    }
                    name_bytes.push(byte);
                }
                dll_name = String::from_utf8_lossy(&name_bytes).to_string();
                self.viewer.set_position(saved_pos)?;
            }

            // 读取函数名称（从导入查找表）
            if import_lookup_table != 0 {
                let lookup_offset = self.rva_to_file_offset(import_lookup_table, sections)?;
                let saved_pos = self.viewer.get_position();
                self.viewer.set_position(lookup_offset as u64)?;

                loop {
                    let entry = if header.optional_header.magic == 0x20b {
                        // PE32+
                        self.viewer.read_u64()?
                    }
                    else {
                        // PE32
                        self.viewer.read_u32()? as u64
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
                        let func_pos = self.viewer.get_position();
                        self.viewer.set_position(hint_name_offset as u64)?;

                        // 跳过 hint（2字节）
                        self.viewer.read_u16()?;

                        // 读取函数名
                        let mut func_name_bytes = Vec::new();
                        loop {
                            let byte = self.viewer.read_u8()?;
                            if byte == 0 {
                                break;
                            }
                            func_name_bytes.push(byte);
                        }
                        let func_name = String::from_utf8_lossy(&func_name_bytes).to_string();
                        functions.push(func_name);

                        self.viewer.set_position(func_pos)?;
                    }
                    else {
                        // 按序号导入
                        let ordinal = entry & 0xFFFF;
                        functions.push(format!("Ordinal_{}", ordinal));
                    }
                }

                self.viewer.set_position(saved_pos)?;
            }
        }

        // 恢复位置
        self.viewer.set_position(current_pos)?;

        Ok(ImportTable { dll_name, functions })
    }

    /// 解析导出表
    fn parse_export_table(&mut self, header: &PeHeader, sections: &[PeSection]) -> Result<ExportTable, GaiaError>
    where
        W: Seek,
    {
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
        let current_pos = self.viewer.get_position();

        // 定位到导出表
        self.viewer.set_position(file_offset as u64)?;

        // 读取导出目录表
        let _export_flags = self.viewer.read_u32()?;
        let _time_date_stamp = self.viewer.read_u32()?;
        let _major_version = self.viewer.read_u16()?;
        let _minor_version = self.viewer.read_u16()?;
        let name_rva = self.viewer.read_u32()?;
        let _ordinal_base = self.viewer.read_u32()?;
        let number_of_functions = self.viewer.read_u32()?;
        let number_of_names = self.viewer.read_u32()?;
        let address_of_functions = self.viewer.read_u32()?;
        let address_of_names = self.viewer.read_u32()?;
        let address_of_name_ordinals = self.viewer.read_u32()?;

        // 读取模块名称
        let mut name = String::new();
        if name_rva != 0 {
            let name_offset = self.rva_to_file_offset(name_rva, sections)?;
            let saved_pos = self.viewer.get_position();
            self.viewer.set_position(name_offset as u64)?;

            let mut name_bytes = Vec::new();
            loop {
                let byte = self.viewer.read_u8()?;
                if byte == 0 {
                    break;
                }
                name_bytes.push(byte);
            }
            name = String::from_utf8_lossy(&name_bytes).to_string();
            self.viewer.set_position(saved_pos)?;
        }

        // 读取函数名称
        let mut functions = Vec::new();
        if address_of_names != 0 && number_of_names > 0 {
            let names_offset = self.rva_to_file_offset(address_of_names, sections)?;
            let saved_pos = self.viewer.get_position();
            self.viewer.set_position(names_offset as u64)?;

            for _ in 0..number_of_names {
                let name_rva = self.viewer.read_u32()?;
                if name_rva != 0 {
                    let func_name_offset = self.rva_to_file_offset(name_rva, sections)?;
                    let func_pos = self.viewer.get_position();
                    self.viewer.set_position(func_name_offset as u64)?;

                    let mut func_name_bytes = Vec::new();
                    loop {
                        let byte = self.viewer.read_u8()?;
                        if byte == 0 {
                            break;
                        }
                        func_name_bytes.push(byte);
                    }
                    let func_name = String::from_utf8_lossy(&func_name_bytes).to_string();
                    functions.push(func_name);

                    self.viewer.set_position(func_pos)?;
                }
            }

            self.viewer.set_position(saved_pos)?;
        }

        // 恢复位置
        self.viewer.set_position(current_pos)?;

        Ok(ExportTable { name, functions })
    }

    /// 将 RVA 转换为文件偏移
    fn rva_to_file_offset(&self, rva: u32, sections: &[PeSection]) -> Result<u32, GaiaError> {
        for section in sections {
            if rva >= section.virtual_address && rva < section.virtual_address + section.virtual_size {
                let offset_in_section = rva - section.virtual_address;
                return Ok(section.pointer_to_raw_data + offset_in_section);
            }
        }
        Err(GaiaError::invalid_data(&format!("无法将 RVA 0x{:08X} 转换为文件偏移", rva)))
    }

    /// 读取基本视图（轻量级）
    pub fn view(&mut self) -> Result<PeInfo, GaiaError>
    where
        W: Seek,
    {
        // 读取 DOS 头
        let dos_header = DosHeader::read(self)?;

        // 验证 DOS 签名
        if dos_header.e_magic != 0x5A4D {
            return Err(GaiaError::invalid_data("无效的 DOS 签名"));
        }

        // 跳转到 NT 头位置
        self.viewer.set_position(dos_header.e_lfanew as u64)?;

        // 读取 NT 头
        let nt_header = NtHeader::read(self)?;

        // 验证 PE 签名
        if nt_header.signature != 0x00004550 {
            return Err(GaiaError::invalid_data("无效的 PE 签名"));
        }

        // 读取 COFF 头
        let machine = self.viewer.read_u16()?;
        let number_of_sections = self.viewer.read_u16()?;
        let time_date_stamp = self.viewer.read_u32()?;
        let pointer_to_symbol_table = self.viewer.read_u32()?;
        let number_of_symbols = self.viewer.read_u32()?;
        let size_of_optional_header = self.viewer.read_u16()?;
        let characteristics = self.viewer.read_u16()?;

        let coff_header = CoffHeader {
            machine,
            number_of_sections,
            time_date_stamp,
            pointer_to_symbol_table,
            number_of_symbols,
            size_of_optional_header,
            characteristics,
        };

        // 读取可选头
        let optional_header = OptionalHeader::read(self)?;

        // 根据机器类型确定架构
        let target_arch = match coff_header.machine {
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
        let current_pos = self.viewer.get_position();
        self.viewer.seek(std::io::SeekFrom::End(0))?;
        let file_size = self.viewer.get_position();
        self.viewer.set_position(current_pos)?;

        // 创建 PeInfo
        let info = PeInfo {
            target_arch,
            subsystem: optional_header.subsystem,
            entry_point: optional_header.address_of_entry_point,
            image_base: optional_header.image_base,
            section_count: coff_header.number_of_sections,
            file_size,
        };

        Ok(info)
    }
}

#[derive(Debug)]
pub struct PeView {
    /// 其他有用字段：已解析的 PE 基本信息
    pub info: PeInfo,
    /// 文件路径，用于重新读取完整数据
    file_path: Option<std::path::PathBuf>,
    /// 字节数据，用于重新读取完整数据
    bytes: Option<Vec<u8>>,
}

impl PeView {
    /// 从文件路径创建 PE 视图
    pub fn view_file(path: &Path) -> Result<Self, GaiaError> {
        let file = File::open(path)?;
        let mut pe_reader = PeReader::new(file);
        let info = pe_reader.view()?;
        Ok(PeView { info, file_path: Some(path.to_path_buf()), bytes: None })
    }
    pub fn view_bytes(bytes: &[u8]) -> Result<Self, GaiaError> {
        let mut pe_reader = PeReader::new(Cursor::new(bytes));
        let info = pe_reader.view()?;
        Ok(PeView { info, file_path: None, bytes: Some(bytes.to_vec()) })
    }
    /// 将视图转换为完整的 PeProgram
    pub fn to_program(&self) -> Result<PeProgram, GaiaError> {
        todo!()
    }

    /// 获取 PE 基本信息
    pub fn info(&self) -> &PeInfo {
        &self.info
    }
}

impl DosHeader {
    /// 从二进制读取器中读取 DOS 头
    ///
    /// # Arguments
    /// * `lexer` - 二进制读取器
    ///
    /// # Returns
    /// 返回 DOS 头结构或错误
    pub fn read<R: ReadBytesExt>(reader: &mut PeReader<R>) -> Result<Self, GaiaError> {
        let e_magic = reader.viewer.read_u16()?;
        let e_cblp = reader.viewer.read_u16()?;
        let e_cp = reader.viewer.read_u16()?;
        let e_crlc = reader.viewer.read_u16()?;
        let e_cparhdr = reader.viewer.read_u16()?;
        let e_minalloc = reader.viewer.read_u16()?;
        let e_maxalloc = reader.viewer.read_u16()?;
        let e_ss = reader.viewer.read_u16()?;
        let e_sp = reader.viewer.read_u16()?;
        let e_csum = reader.viewer.read_u16()?;
        let e_ip = reader.viewer.read_u16()?;
        let e_cs = reader.viewer.read_u16()?;
        let e_lfarlc = reader.viewer.read_u16()?;
        let e_ovno = reader.viewer.read_u16()?;

        let mut e_res = [0u16; 4];
        for i in 0..4 {
            e_res[i] = reader.viewer.read_u16()?;
        }

        let e_oemid = reader.viewer.read_u16()?;
        let e_oeminfo = reader.viewer.read_u16()?;

        let mut e_res2 = [0u16; 10];
        for i in 0..10 {
            e_res2[i] = reader.viewer.read_u16()?;
        }

        let e_lfanew = reader.viewer.read_u32()?;

        Ok(DosHeader {
            e_magic,
            e_cblp,
            e_cp,
            e_crlc,
            e_cparhdr,
            e_minalloc,
            e_maxalloc,
            e_ss,
            e_sp,
            e_csum,
            e_ip,
            e_cs,
            e_lfarlc,
            e_ovno,
            e_res,
            e_oemid,
            e_oeminfo,
            e_res2,
            e_lfanew,
        })
    }
}

impl NtHeader {
    /// 从二进制读取器中读取 NT 头
    ///
    /// # Arguments
    /// * `lexer` - 二进制读取器
    ///
    /// # Returns
    /// 返回 NT 头结构或错误
    pub fn read<R: ReadBytesExt>(reader: &mut PeReader<R>) -> Result<Self, GaiaError> {
        let signature = reader.viewer.read_u32()?;
        Ok(NtHeader { signature })
    }
}

impl PeHeader {
    /// 从二进制读取器中读取 PE 头
    ///
    /// # Arguments
    /// * `lexer` - 二进制读取器
    ///
    /// # Returns
    /// 返回 PE 头结构或错误
    pub fn read<R: Read + Seek>(reader: &mut PeReader<R>) -> Result<Self, GaiaError> {
        // 保存当前位置
        let original_pos = reader.viewer.get_position();

        // 重置到文件开头
        reader.viewer.set_position(0)?;

        // 读取 DOS 头
        let dos_header = DosHeader::read(reader)?;

        // 验证 DOS 签名 (MZ)
        if dos_header.e_magic != 0x5A4D {
            return Err(GaiaError::invalid_data("无效的 DOS 签名 (MZ)"));
        }

        // 跳转到 NT 头位置
        reader.viewer.set_position(dos_header.e_lfanew as u64)?;

        // 读取 NT 头
        let nt_header = NtHeader::read(reader)?;

        // 验证 PE 签名 (PE\0\0)
        if nt_header.signature != 0x00004550 {
            return Err(GaiaError::invalid_data("无效的 PE 签名 (PE)"));
        }

        // 读取 COFF 头
        let machine = reader.viewer.read_u16()?;
        let number_of_sections = reader.viewer.read_u16()?;
        let time_date_stamp = reader.viewer.read_u32()?;
        let pointer_to_symbol_table = reader.viewer.read_u32()?;
        let number_of_symbols = reader.viewer.read_u32()?;
        let size_of_optional_header = reader.viewer.read_u16()?;
        let characteristics = reader.viewer.read_u16()?;

        let coff_header = CoffHeader {
            machine,
            number_of_sections,
            time_date_stamp,
            pointer_to_symbol_table,
            number_of_symbols,
            size_of_optional_header,
            characteristics,
        };

        // 验证 COFF 头中的节数量
        if coff_header.number_of_sections == 0 {
            return Err(GaiaError::invalid_data("PE 文件必须至少有一个节"));
        }

        // 读取可选头
        let optional_header = OptionalHeader::read(reader)?;

        // 验证可选头的魔数
        match optional_header.magic {
            0x10b => {} // PE32
            0x20b => {} // PE32+
            _ => return Err(GaiaError::invalid_data("无效的可选头魔数")),
        }

        // 恢复原始位置
        reader.viewer.set_position(original_pos)?;

        Ok(PeHeader { dos_header, nt_header, coff_header, optional_header })
    }
}

impl SectionHeader {
    /// 从二进制读取器中读取节头
    ///
    /// # Arguments
    /// * `lexer` - 二进制读取器
    ///
    /// # Returns
    /// 返回节头结构或错误
    pub fn read<R: ReadBytesExt + Seek>(reader: &mut PeReader<R>) -> Result<Self, GaiaError> {
        let name = reader.viewer.read_array::<8>()?;

        let virtual_size = reader.viewer.read_u32()?;
        let virtual_address = reader.viewer.read_u32()?;
        let size_of_raw_data = reader.viewer.read_u32()?;
        let pointer_to_raw_data = reader.viewer.read_u32()?;
        let pointer_to_relocations = reader.viewer.read_u32()?;
        let pointer_to_linenumbers = reader.viewer.read_u32()?;
        let number_of_relocations = reader.viewer.read_u16()?;
        let number_of_linenumbers = reader.viewer.read_u16()?;
        let characteristics = reader.viewer.read_u32()?;

        Ok(SectionHeader {
            name,
            virtual_size,
            virtual_address,
            size_of_raw_data,
            pointer_to_raw_data,
            pointer_to_relocations,
            number_of_relocations,
            characteristics,
            pointer_to_line_numbers: pointer_to_linenumbers,
            number_of_line_numbers: number_of_linenumbers,
        })
    }
}

impl OptionalHeader {
    /// 从二进制读取器中读取可选头
    ///
    /// # Arguments
    /// * `lexer` - 二进制读取器
    ///
    /// # Returns
    /// 返回可选头结构或错误
    pub fn read<R: ReadBytesExt>(reader: &mut PeReader<R>) -> Result<Self, GaiaError> {
        let magic = reader.viewer.read_u16()?;
        let major_linker_version = reader.viewer.read_u8()?;
        let minor_linker_version = reader.viewer.read_u8()?;
        let size_of_code = reader.viewer.read_u32()?;
        let size_of_initialized_data = reader.viewer.read_u32()?;
        let size_of_uninitialized_data = reader.viewer.read_u32()?;
        let address_of_entry_point = reader.viewer.read_u32()?;
        let base_of_code = reader.viewer.read_u32()?;

        // 根据 magic 值判断是 PE32 还是 PE32+
        let (base_of_data, image_base) = if magic == 0x10b {
            // PE32 格式
            let base_of_data = Some(reader.viewer.read_u32()?);
            let image_base = reader.viewer.read_u32()? as u64;
            (base_of_data, image_base)
        }
        else if magic == 0x20b {
            // PE32+ 格式
            let image_base = reader.viewer.read_u64()?;
            (None, image_base)
        }
        else {
            return Err(GaiaError::invalid_data(&format!("不支持的 PE 格式，magic: 0x{:x}", magic)));
        };

        let section_alignment = reader.viewer.read_u32()?;
        let file_alignment = reader.viewer.read_u32()?;
        let major_operating_system_version = reader.viewer.read_u16()?;
        let minor_operating_system_version = reader.viewer.read_u16()?;
        let major_image_version = reader.viewer.read_u16()?;
        let minor_image_version = reader.viewer.read_u16()?;
        let major_subsystem_version = reader.viewer.read_u16()?;
        let minor_subsystem_version = reader.viewer.read_u16()?;
        let win32_version_value = reader.viewer.read_u32()?;
        let size_of_image = reader.viewer.read_u32()?;
        let size_of_headers = reader.viewer.read_u32()?;
        let checksum = reader.viewer.read_u32()?;
        let subsystem = reader.viewer.read_u16()?.into();
        let dll_characteristics = reader.viewer.read_u16()?;

        // 根据格式读取不同大小的字段
        let (size_of_stack_reserve, size_of_stack_commit, size_of_heap_reserve, size_of_heap_commit) = if magic == 0x10b {
            // PE32 格式 - 4字节字段
            (
                reader.viewer.read_u32()? as u64,
                reader.viewer.read_u32()? as u64,
                reader.viewer.read_u32()? as u64,
                reader.viewer.read_u32()? as u64,
            )
        }
        else {
            // PE32+ 格式 - 8字节字段
            (reader.viewer.read_u64()?, reader.viewer.read_u64()?, reader.viewer.read_u64()?, reader.viewer.read_u64()?)
        };

        let loader_flags = reader.viewer.read_u32()?;
        let number_of_rva_and_sizes = reader.viewer.read_u32()?;

        // 读取数据目录表
        let mut data_directories = Vec::new();
        for _ in 0..number_of_rva_and_sizes {
            data_directories.push(DataDirectory::read(reader)?);
        }

        Ok(OptionalHeader {
            magic,
            major_linker_version,
            minor_linker_version,
            size_of_code,
            size_of_initialized_data,
            size_of_uninitialized_data,
            address_of_entry_point,
            base_of_code,
            base_of_data,
            image_base,
            section_alignment,
            file_alignment,
            major_operating_system_version,
            minor_operating_system_version,
            major_image_version,
            minor_image_version,
            major_subsystem_version,
            minor_subsystem_version,
            win32_version_value,
            size_of_image,
            size_of_headers,
            checksum,
            subsystem,
            dll_characteristics,
            size_of_stack_reserve,
            size_of_stack_commit,
            size_of_heap_reserve,
            size_of_heap_commit,
            loader_flags,
            number_of_rva_and_sizes,
            data_directories,
        })
    }
}

impl DataDirectory {
    /// 从二进制读取器中读取数据目录
    ///
    /// # Arguments
    /// * `lexer` - 二进制读取器
    ///
    /// # Returns
    /// 返回数据目录结构或错误
    pub fn read<R: ReadBytesExt>(reader: &mut PeReader<R>) -> Result<Self, GaiaError> {
        let virtual_address = reader.viewer.read_u32()?;
        let size = reader.viewer.read_u32()?;

        Ok(DataDirectory { virtual_address, size })
    }
}

impl PeSection {
    /// 从二进制读取器中读取 PE 节
    ///
    /// # Arguments
    /// * `lexer` - 二进制读取器
    ///
    /// # Returns
    /// 返回 PE 节结构或错误
    pub fn read<R: ReadBytesExt + Seek>(reader: &mut PeReader<R>) -> Result<Self, GaiaError> {
        let header = SectionHeader::read(reader)?;

        // 读取节的数据
        let position = reader.viewer.get_position();
        reader.viewer.set_position(header.pointer_to_raw_data as u64)?;

        let data = reader.viewer.read_bytes(header.size_of_raw_data as usize)?;

        // 恢复原来的位置
        reader.viewer.set_position(position)?;

        // 将SectionHeader的字段映射到PeSection
        let name = String::from_utf8_lossy(&header.name).trim_end_matches('\0').to_string();

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
}
