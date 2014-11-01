//! PE 文件写入器模块
//!
//! 此模块提供将 PE 结构体写入二进制文件的功能，与 reader 模块相对应。

use crate::types::{DataDirectory, DosHeader, NtHeader, OptionalHeader, PeHeader, PeProgram, PeSection, SubsystemType};
use byteorder::{LittleEndian, WriteBytesExt};
use gaia_types::{BinaryAssembler, GaiaError};
use std::io::{Seek, Write};

/// PE 文件生成器的通用接口
#[derive(Debug)]
pub struct PeAssembler<W: Write> {
    writer: BinaryAssembler<W, LittleEndian>,
}

impl<W: Write> PeAssembler<W> {
    /// 创建一个新的 PE 写入器
    pub fn new(writer: W) -> Self {
        Self { writer: BinaryAssembler::new(writer) }
    }

    /// 将 PE 程序写入字节数组
    pub fn write_program(&mut self, program: &PeProgram) -> Result<(), GaiaError>
    where
        W: Seek,
    {
        // 写入 DOS 头
        self.write_dos_header(&program.header.dos_header)?;

        // 写入 DOS stub（简单的 DOS 程序）
        self.write_dos_stub()?;

        // 对齐到 PE 头位置
        let pe_header_offset = program.header.dos_header.e_lfanew as u64;
        self.pad_to_offset(pe_header_offset)?;

        // 写入 NT 头
        self.write_nt_header(&program.header.nt_header)?;

        // 写入 COFF 头
        self.write_coff_header(&program.header.coff_header)?;

        // 写入可选头
        self.write_optional_header(&program.header.optional_header)?;

        // 写入节头
        for section in &program.sections {
            self.write_section_header(section)?;
        }

        // 对齐整个头部区域到文件对齐边界
        let file_alignment = program.header.optional_header.file_alignment;
        self.align_to_boundary(file_alignment)?;

        // 写入节数据
        for section in &program.sections {
            if !section.data.is_empty() {
                // 对齐到节的文件偏移
                self.pad_to_offset(section.pointer_to_raw_data as u64)?;
                self.writer.write_all(&section.data)?;

                // 对齐到文件对齐边界
                self.align_to_boundary(file_alignment)?;
            }
        }

        Ok(())
    }

    /// 写入 DOS 头
    fn write_dos_header(&mut self, dos_header: &DosHeader) -> Result<(), GaiaError> {
        self.writer.write_u16(dos_header.e_magic)?;
        self.writer.write_u16(dos_header.e_cblp)?;
        self.writer.write_u16(dos_header.e_cp)?;
        self.writer.write_u16(dos_header.e_crlc)?;
        self.writer.write_u16(dos_header.e_cparhdr)?;
        self.writer.write_u16(dos_header.e_minalloc)?;
        self.writer.write_u16(dos_header.e_maxalloc)?;
        self.writer.write_u16(dos_header.e_ss)?;
        self.writer.write_u16(dos_header.e_sp)?;
        self.writer.write_u16(dos_header.e_csum)?;
        self.writer.write_u16(dos_header.e_ip)?;
        self.writer.write_u16(dos_header.e_cs)?;
        self.writer.write_u16(dos_header.e_lfarlc)?;
        self.writer.write_u16(dos_header.e_ovno)?;

        // 写入保留字段
        for &res in &dos_header.e_res {
            self.writer.write_u16(res)?;
        }

        self.writer.write_u16(dos_header.e_oemid)?;
        self.writer.write_u16(dos_header.e_oeminfo)?;

        // 写入第二个保留字段
        for &res in &dos_header.e_res2 {
            self.writer.write_u16(res)?;
        }

        self.writer.write_u32(dos_header.e_lfanew)?;

        Ok(())
    }

    /// 写入 DOS stub
    fn write_dos_stub(&mut self) -> Result<(), GaiaError> {
        // 简单的 DOS 程序，显示 "This program cannot be run in DOS mode."
        let dos_stub = [
            0x0e, 0x1f, 0xba, 0x0e, 0x00, 0xb4, 0x09, 0xcd, 0x21, 0xb8, 0x01, 0x4c, 0xcd, 0x21, 0x54, 0x68, 0x69, 0x73, 0x20,
            0x70, 0x72, 0x6f, 0x67, 0x72, 0x61, 0x6d, 0x20, 0x63, 0x61, 0x6e, 0x6e, 0x6f, 0x74, 0x20, 0x62, 0x65, 0x20, 0x72,
            0x75, 0x6e, 0x20, 0x69, 0x6e, 0x20, 0x44, 0x4f, 0x53, 0x20, 0x6d, 0x6f, 0x64, 0x65, 0x2e, 0x0d, 0x0d, 0x0a, 0x24,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        self.writer.write_all(&dos_stub)?;
        Ok(())
    }

    /// 写入 NT 头
    fn write_nt_header(&mut self, nt_header: &NtHeader) -> Result<(), GaiaError> {
        self.writer.write_u32(nt_header.signature)?;
        Ok(())
    }

    /// 写入 COFF 头
    fn write_coff_header(&mut self, coff_header: &pe_coff::types::CoffHeader) -> Result<(), GaiaError> {
        self.writer.write_u16(coff_header.machine)?;
        self.writer.write_u16(coff_header.number_of_sections)?;
        self.writer.write_u32(coff_header.time_date_stamp)?;
        self.writer.write_u32(coff_header.pointer_to_symbol_table)?;
        self.writer.write_u32(coff_header.number_of_symbols)?;
        self.writer.write_u16(coff_header.size_of_optional_header)?;
        self.writer.write_u16(coff_header.characteristics)?;
        Ok(())
    }

    /// 写入可选头
    fn write_optional_header(&mut self, optional_header: &OptionalHeader) -> Result<(), GaiaError> {
        self.writer.write_u16(optional_header.magic)?;
        self.writer.write_u8(optional_header.major_linker_version)?;
        self.writer.write_u8(optional_header.minor_linker_version)?;
        self.writer.write_u32(optional_header.size_of_code)?;
        self.writer.write_u32(optional_header.size_of_initialized_data)?;
        self.writer.write_u32(optional_header.size_of_uninitialized_data)?;
        self.writer.write_u32(optional_header.address_of_entry_point)?;
        self.writer.write_u32(optional_header.base_of_code)?;

        // base_of_data 只在 PE32 中存在
        if let Some(base_of_data) = optional_header.base_of_data {
            self.writer.write_u32(base_of_data)?;
            // PE32: image_base 是 32 位
            self.writer.write_u32(optional_header.image_base as u32)?;
        }
        else {
            // PE32+: image_base 是 64 位
            self.writer.write_u64(optional_header.image_base)?;
        }
        self.writer.write_u32(optional_header.section_alignment)?;
        self.writer.write_u32(optional_header.file_alignment)?;
        self.writer.write_u16(optional_header.major_operating_system_version)?;
        self.writer.write_u16(optional_header.minor_operating_system_version)?;
        self.writer.write_u16(optional_header.major_image_version)?;
        self.writer.write_u16(optional_header.minor_image_version)?;
        self.writer.write_u16(optional_header.major_subsystem_version)?;
        self.writer.write_u16(optional_header.minor_subsystem_version)?;
        self.writer.write_u32(optional_header.win32_version_value)?;
        self.writer.write_u32(optional_header.size_of_image)?;
        self.writer.write_u32(optional_header.size_of_headers)?;
        self.writer.write_u32(optional_header.checksum)?;

        // 写入子系统类型
        let subsystem_value = match optional_header.subsystem {
            SubsystemType::Native => 1,
            SubsystemType::Windows => 2,
            SubsystemType::Console => 3,
            SubsystemType::Posix => 7,
            SubsystemType::WindowsCe => 9,
            SubsystemType::Efi => 10,
            SubsystemType::EfiBootServiceDriver => 11,
            SubsystemType::EfiRuntimeDriver => 12,
            SubsystemType::EfiRom => 13,
            SubsystemType::Xbox => 14,
            SubsystemType::WindowsBootApplication => 16,
        };
        self.writer.write_u16(subsystem_value)?;

        self.writer.write_u16(optional_header.dll_characteristics)?;

        // 堆栈和堆字段的大小取决于PE格式
        if optional_header.base_of_data.is_some() {
            // PE32: 这些字段是32位
            self.writer.write_u32(optional_header.size_of_stack_reserve as u32)?;
            self.writer.write_u32(optional_header.size_of_stack_commit as u32)?;
            self.writer.write_u32(optional_header.size_of_heap_reserve as u32)?;
            self.writer.write_u32(optional_header.size_of_heap_commit as u32)?;
        }
        else {
            // PE32+: 这些字段是64位
            self.writer.write_u64(optional_header.size_of_stack_reserve)?;
            self.writer.write_u64(optional_header.size_of_stack_commit)?;
            self.writer.write_u64(optional_header.size_of_heap_reserve)?;
            self.writer.write_u64(optional_header.size_of_heap_commit)?;
        }

        self.writer.write_u32(optional_header.loader_flags)?;
        self.writer.write_u32(optional_header.number_of_rva_and_sizes)?;

        // 写入数据目录
        for data_dir in &optional_header.data_directories {
            self.write_data_directory(data_dir)?;
        }

        Ok(())
    }

    /// 写入数据目录
    fn write_data_directory(&mut self, data_dir: &DataDirectory) -> Result<(), GaiaError> {
        self.writer.write_u32(data_dir.virtual_address)?;
        self.writer.write_u32(data_dir.size)?;
        Ok(())
    }

    /// 写入节头
    fn write_section_header(&mut self, section: &PeSection) -> Result<(), GaiaError> {
        // 写入节名（8字节，不足的用0填充）
        let mut name_bytes = [0u8; 8];
        let name_bytes_src = section.name.as_bytes();
        let copy_len = std::cmp::min(name_bytes_src.len(), 8);
        name_bytes[..copy_len].copy_from_slice(&name_bytes_src[..copy_len]);
        self.writer.write_all(&name_bytes)?;

        self.writer.write_u32(section.virtual_size)?;
        self.writer.write_u32(section.virtual_address)?;
        self.writer.write_u32(section.size_of_raw_data)?;
        self.writer.write_u32(section.pointer_to_raw_data)?;
        self.writer.write_u32(section.pointer_to_relocations)?;
        self.writer.write_u32(section.pointer_to_line_numbers)?;
        self.writer.write_u16(section.number_of_relocations)?;
        self.writer.write_u16(section.number_of_line_numbers)?;
        self.writer.write_u32(section.characteristics)?;

        Ok(())
    }

    /// 填充到指定偏移
    fn pad_to_offset(&mut self, target_offset: u64) -> Result<(), GaiaError>
    where
        W: Seek,
    {
        let current_pos = self.writer.stream_position()?;
        if target_offset > current_pos {
            let padding_size = target_offset - current_pos;
            let padding = vec![0u8; padding_size as usize];
            self.writer.write_all(&padding)?;
        }
        Ok(())
    }

    /// 对齐到边界
    fn align_to_boundary(&mut self, alignment: u32) -> Result<(), GaiaError>
    where
        W: Seek,
    {
        let current_pos = self.writer.stream_position()?;
        let remainder = current_pos % alignment as u64;
        if remainder != 0 {
            let padding_size = alignment as u64 - remainder;
            let padding = vec![0u8; padding_size as usize];
            self.writer.write_all(&padding)?;
        }
        Ok(())
    }
}

/// PE 文件构建器
#[derive(Debug)]
pub struct PeBuilder {
    header: Option<PeHeader>,
    sections: Vec<PeSection>,
}

impl PeBuilder {
    /// 创建新的 PE 构建器
    pub fn new() -> Self {
        Self { header: None, sections: Vec::new() }
    }

    /// 设置 PE 头
    pub fn with_header(mut self, header: PeHeader) -> Self {
        self.header = Some(header);
        self
    }

    /// 添加节
    pub fn add_section(mut self, section: PeSection) -> Self {
        self.sections.push(section);
        self
    }

    /// 构建 PE 程序
    pub fn build(self) -> Result<PeProgram, GaiaError> {
        let header = self
            .header
            .ok_or_else(|| GaiaError::syntax_error("PE header is required", gaia_types::SourceLocation::default()))?;

        Ok(PeProgram {
            header,
            sections: self.sections,
            imports: crate::assembler::ImportTable { dll_name: String::new(), functions: Vec::new() },
            exports: crate::assembler::ExportTable { name: String::new(), functions: Vec::new() },
        })
    }
}

impl Default for PeBuilder {
    fn default() -> Self {
        Self::new()
    }
}
