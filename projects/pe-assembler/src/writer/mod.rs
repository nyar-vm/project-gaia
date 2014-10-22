//! PE 文件写入器模块
//!
//! 此模块提供将 PE 结构体写入二进制文件的功能，与 reader 模块相对应。

use crate::types::{DataDirectory, DosHeader, NtHeader, OptionalHeader, PeHeader, PeProgram, PeSection, SubsystemType};
use byteorder::{LittleEndian, WriteBytesExt};
use gaia_types::{BinaryAssembler, GaiaError};
use std::io::{Cursor, Seek, Write};

/// PE 文件生成器的通用接口
#[derive(Debug)]
pub struct PeAssembler<W: WriteBytesExt> {
    writer: BinaryAssembler<W, LittleEndian>,
}

impl<W: WriteBytesExt> PeAssembler<W> {
    /// 将 PE 程序写入字节数组
    pub fn write_program(program: &PeProgram) -> Result<Vec<u8>, GaiaError> {
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);

        // 写入 DOS 头
        Self::write_dos_header(&mut cursor, &program.header.dos_header)?;

        // 写入 DOS stub（简单的 DOS 程序）
        Self::write_dos_stub(&mut cursor)?;

        // 对齐到 PE 头位置
        let pe_header_offset = program.header.dos_header.e_lfanew as u64;
        Self::pad_to_offset(&mut cursor, pe_header_offset)?;

        // 写入 NT 头
        Self::write_nt_header(&mut cursor, &program.header.nt_header)?;

        // 写入 COFF 头
        Self::write_coff_header(&mut cursor, &program.header.coff_header)?;

        // 写入可选头
        Self::write_optional_header(&mut cursor, &program.header.optional_header)?;

        // 写入节头
        for section in &program.sections {
            Self::write_section_header(&mut cursor, section)?;
        }

        // 对齐整个头部区域到文件对齐边界
        let file_alignment = program.header.optional_header.file_alignment;
        Self::align_to_boundary(&mut cursor, file_alignment)?;

        // 写入节数据
        for section in &program.sections {
            if !section.data.is_empty() {
                // 对齐到节的文件偏移
                Self::pad_to_offset(&mut cursor, section.pointer_to_raw_data as u64)?;
                cursor.write_all(&section.data)?;

                // 对齐到文件对齐边界
                Self::align_to_boundary(&mut cursor, file_alignment)?;
            }
        }

        Ok(buffer)
    }

    /// 写入 DOS 头
    fn write_dos_header<T: WriteBytesExt>(writer: &mut T, dos_header: &DosHeader) -> Result<(), GaiaError> {
        writer.write_u16::<LittleEndian>(dos_header.e_magic)?;
        writer.write_u16::<LittleEndian>(dos_header.e_cblp)?;
        writer.write_u16::<LittleEndian>(dos_header.e_cp)?;
        writer.write_u16::<LittleEndian>(dos_header.e_crlc)?;
        writer.write_u16::<LittleEndian>(dos_header.e_cparhdr)?;
        writer.write_u16::<LittleEndian>(dos_header.e_minalloc)?;
        writer.write_u16::<LittleEndian>(dos_header.e_maxalloc)?;
        writer.write_u16::<LittleEndian>(dos_header.e_ss)?;
        writer.write_u16::<LittleEndian>(dos_header.e_sp)?;
        writer.write_u16::<LittleEndian>(dos_header.e_csum)?;
        writer.write_u16::<LittleEndian>(dos_header.e_ip)?;
        writer.write_u16::<LittleEndian>(dos_header.e_cs)?;
        writer.write_u16::<LittleEndian>(dos_header.e_lfarlc)?;
        writer.write_u16::<LittleEndian>(dos_header.e_ovno)?;

        // 写入保留字段
        for &res in &dos_header.e_res {
            writer.write_u16::<LittleEndian>(res)?;
        }

        writer.write_u16::<LittleEndian>(dos_header.e_oemid)?;
        writer.write_u16::<LittleEndian>(dos_header.e_oeminfo)?;

        // 写入第二个保留字段
        for &res in &dos_header.e_res2 {
            writer.write_u16::<LittleEndian>(res)?;
        }

        writer.write_u32::<LittleEndian>(dos_header.e_lfanew)?;

        Ok(())
    }

    /// 写入 DOS stub
    fn write_dos_stub<T: Write>(writer: &mut T) -> Result<(), GaiaError> {
        // 简单的 DOS 程序，显示 "This program cannot be run in DOS mode."
        let dos_stub = [
            0x0e, 0x1f, 0xba, 0x0e, 0x00, 0xb4, 0x09, 0xcd, 0x21, 0xb8, 0x01, 0x4c, 0xcd, 0x21, 0x54, 0x68, 0x69, 0x73, 0x20,
            0x70, 0x72, 0x6f, 0x67, 0x72, 0x61, 0x6d, 0x20, 0x63, 0x61, 0x6e, 0x6e, 0x6f, 0x74, 0x20, 0x62, 0x65, 0x20, 0x72,
            0x75, 0x6e, 0x20, 0x69, 0x6e, 0x20, 0x44, 0x4f, 0x53, 0x20, 0x6d, 0x6f, 0x64, 0x65, 0x2e, 0x0d, 0x0d, 0x0a, 0x24,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        writer.write_all(&dos_stub)?;
        Ok(())
    }

    /// 写入 NT 头
    fn write_nt_header<T: WriteBytesExt>(writer: &mut T, nt_header: &NtHeader) -> Result<(), GaiaError> {
        writer.write_u32::<LittleEndian>(nt_header.signature)?;
        Ok(())
    }

    /// 写入 COFF 头
    fn write_coff_header(writer: &mut W, coff_header: &pe_coff::types::CoffHeader) -> Result<(), GaiaError> {
        writer.write_u16::<LittleEndian>(coff_header.machine)?;
        writer.write_u16::<LittleEndian>(coff_header.number_of_sections)?;
        writer.write_u32::<LittleEndian>(coff_header.time_date_stamp)?;
        writer.write_u32::<LittleEndian>(coff_header.pointer_to_symbol_table)?;
        writer.write_u32::<LittleEndian>(coff_header.number_of_symbols)?;
        writer.write_u16::<LittleEndian>(coff_header.size_of_optional_header)?;
        writer.write_u16::<LittleEndian>(coff_header.characteristics)?;
        Ok(())
    }

    /// 写入可选头
    fn write_optional_header(writer: &mut W, optional_header: &OptionalHeader) -> Result<(), GaiaError> {
        writer.write_u16::<LittleEndian>(optional_header.magic)?;
        writer.write_u8(optional_header.major_linker_version)?;
        writer.write_u8(optional_header.minor_linker_version)?;
        writer.write_u32::<LittleEndian>(optional_header.size_of_code)?;
        writer.write_u32::<LittleEndian>(optional_header.size_of_initialized_data)?;
        writer.write_u32::<LittleEndian>(optional_header.size_of_uninitialized_data)?;
        writer.write_u32::<LittleEndian>(optional_header.address_of_entry_point)?;
        writer.write_u32::<LittleEndian>(optional_header.base_of_code)?;

        // base_of_data 只在 PE32 中存在
        if let Some(base_of_data) = optional_header.base_of_data {
            writer.write_u32::<LittleEndian>(base_of_data)?;
            // PE32: image_base 是 32 位
            writer.write_u32::<LittleEndian>(optional_header.image_base as u32)?;
        }
        else {
            // PE32+: image_base 是 64 位
            writer.write_u64::<LittleEndian>(optional_header.image_base)?;
        }

        writer.write_u32::<LittleEndian>(optional_header.section_alignment)?;
        writer.write_u32::<LittleEndian>(optional_header.file_alignment)?;
        writer.write_u16::<LittleEndian>(optional_header.major_operating_system_version)?;
        writer.write_u16::<LittleEndian>(optional_header.minor_operating_system_version)?;
        writer.write_u16::<LittleEndian>(optional_header.major_image_version)?;
        writer.write_u16::<LittleEndian>(optional_header.minor_image_version)?;
        writer.write_u16::<LittleEndian>(optional_header.major_subsystem_version)?;
        writer.write_u16::<LittleEndian>(optional_header.minor_subsystem_version)?;
        writer.write_u32::<LittleEndian>(optional_header.win32_version_value)?;
        writer.write_u32::<LittleEndian>(optional_header.size_of_image)?;
        writer.write_u32::<LittleEndian>(optional_header.size_of_headers)?;
        writer.write_u32::<LittleEndian>(optional_header.checksum)?;

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
        writer.write_u16::<LittleEndian>(subsystem_value)?;

        writer.write_u16::<LittleEndian>(optional_header.dll_characteristics)?;

        // 堆栈和堆字段的大小取决于PE格式
        if optional_header.base_of_data.is_some() {
            // PE32: 这些字段是32位
            writer.write_u32::<LittleEndian>(optional_header.size_of_stack_reserve as u32)?;
            writer.write_u32::<LittleEndian>(optional_header.size_of_stack_commit as u32)?;
            writer.write_u32::<LittleEndian>(optional_header.size_of_heap_reserve as u32)?;
            writer.write_u32::<LittleEndian>(optional_header.size_of_heap_commit as u32)?;
        }
        else {
            // PE32+: 这些字段是64位
            writer.write_u64::<LittleEndian>(optional_header.size_of_stack_reserve)?;
            writer.write_u64::<LittleEndian>(optional_header.size_of_stack_commit)?;
            writer.write_u64::<LittleEndian>(optional_header.size_of_heap_reserve)?;
            writer.write_u64::<LittleEndian>(optional_header.size_of_heap_commit)?;
        }

        writer.write_u32::<LittleEndian>(optional_header.loader_flags)?;
        writer.write_u32::<LittleEndian>(optional_header.number_of_rva_and_sizes)?;

        // 写入数据目录
        for data_dir in &optional_header.data_directories {
            Self::write_data_directory(writer, data_dir)?;
        }

        Ok(())
    }

    /// 写入数据目录
    fn write_data_directory(writer: &mut W, data_dir: &DataDirectory) -> Result<(), GaiaError> {
        writer.write_u32::<LittleEndian>(data_dir.virtual_address)?;
        writer.write_u32::<LittleEndian>(data_dir.size)?;
        Ok(())
    }

    /// 写入节头
    fn write_section_header(writer: &mut W, section: &PeSection) -> Result<(), GaiaError> {
        // 写入节名（8字节，不足的用0填充）
        let mut name_bytes = [0u8; 8];
        let name_bytes_src = section.name.as_bytes();
        let copy_len = std::cmp::min(name_bytes_src.len(), 8);
        name_bytes[..copy_len].copy_from_slice(&name_bytes_src[..copy_len]);
        writer.write_all(&name_bytes)?;

        writer.write_u32::<LittleEndian>(section.virtual_size)?;
        writer.write_u32::<LittleEndian>(section.virtual_address)?;
        writer.write_u32::<LittleEndian>(section.size_of_raw_data)?;
        writer.write_u32::<LittleEndian>(section.pointer_to_raw_data)?;
        writer.write_u32::<LittleEndian>(section.pointer_to_relocations)?;
        writer.write_u32::<LittleEndian>(section.pointer_to_line_numbers)?;
        writer.write_u16::<LittleEndian>(section.number_of_relocations)?;
        writer.write_u16::<LittleEndian>(section.number_of_line_numbers)?;
        writer.write_u32::<LittleEndian>(section.characteristics)?;

        Ok(())
    }

    /// 填充到指定偏移
    fn pad_to_offset<T: Write + Seek>(writer: &mut T, target_offset: u64) -> Result<(), GaiaError> {
        let current_pos = writer.stream_position()?;
        if target_offset > current_pos {
            let padding_size = target_offset - current_pos;
            let padding = vec![0u8; padding_size as usize];
            writer.write_all(&padding)?;
        }
        Ok(())
    }

    /// 对齐到边界
    fn align_to_boundary<T: Write + Seek>(writer: &mut T, alignment: u32) -> Result<(), GaiaError> {
        let current_pos = writer.stream_position()?;
        let remainder = current_pos % alignment as u64;
        if remainder != 0 {
            let padding_size = alignment as u64 - remainder;
            let padding = vec![0u8; padding_size as usize];
            writer.write_all(&padding)?;
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
