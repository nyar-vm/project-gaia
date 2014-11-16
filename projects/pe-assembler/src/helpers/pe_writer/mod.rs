//! PE 文件写入器的通用 trait
//!
//! 此模块提供 PE 文件写入的通用接口，类似于 pe_reader 模块的设计

use crate::types::{DataDirectory, DosHeader, NtHeader, OptionalHeader, PeProgram, PeSection, SubsystemType};
use byteorder::{LittleEndian, WriteBytesExt};
use gaia_types::GaiaError;
use std::io::{Seek, Write};

/// PE 文件写入器的通用 trait
pub trait PeWriter<W: Write + Seek> {
    /// 获取写入器的可变引用
    fn get_writer(&mut self) -> &mut W;

    /// 获取当前流位置
    fn stream_position(&mut self) -> Result<u64, GaiaError> {
        Ok(self.get_writer().stream_position()?)
    }

    /// 将 PE 程序写入流（通用实现）
    fn write_program(&mut self, program: &PeProgram) -> Result<(), GaiaError> {
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
                self.get_writer().write_all(&section.data)?;

                // 对齐到文件对齐边界
                self.align_to_boundary(file_alignment)?;
            }
        }

        // 写入导入表（如果存在）
        let pointer_size: usize = if program.header.optional_header.magic == 0x020B { 8 } else { 4 };
        self.write_import_table(&program.imports, &program.sections, pointer_size)?;

        Ok(())
    }

    /// 写入 DOS 头（通用实现）
    fn write_dos_header(&mut self, dos_header: &DosHeader) -> Result<(), GaiaError> {
        let writer = self.get_writer();
        writer.write_u16::<LittleEndian>(dos_header.e_magic)?;
        writer.write_u16::<LittleEndian>(dos_header.e_cblp)?;
        writer.write_u16::<LittleEndian>(dos_header.e_cp)?;
        writer.write_u16::<LittleEndian>(dos_header.e_crlc)?;
        writer.write_u16::<LittleEndian>(dos_header.e_cparhdr)?;
        writer.write_u16::<LittleEndian>(dos_header.e_min_allocate)?;
        writer.write_u16::<LittleEndian>(dos_header.e_max_allocate)?;
        writer.write_u16::<LittleEndian>(dos_header.e_ss)?;
        writer.write_u16::<LittleEndian>(dos_header.e_sp)?;
        writer.write_u16::<LittleEndian>(dos_header.e_check_sum)?;
        writer.write_u16::<LittleEndian>(dos_header.e_ip)?;
        writer.write_u16::<LittleEndian>(dos_header.e_cs)?;
        writer.write_u16::<LittleEndian>(dos_header.e_lfarlc)?;
        writer.write_u16::<LittleEndian>(dos_header.e_ovno)?;
        for &res in &dos_header.e_res {
            writer.write_u16::<LittleEndian>(res)?;
        }
        writer.write_u16::<LittleEndian>(dos_header.e_oem_id)?;
        writer.write_u16::<LittleEndian>(dos_header.e_oem_info)?;
        for &res in &dos_header.e_res2 {
            writer.write_u16::<LittleEndian>(res)?;
        }
        writer.write_u32::<LittleEndian>(dos_header.e_lfanew)?;
        Ok(())
    }

    /// 写入 DOS stub（通用实现）
    fn write_dos_stub(&mut self) -> Result<(), GaiaError> {
        // 简单的 DOS stub 程序
        let dos_stub = b"This program cannot be run in DOS mode.\r\n$";
        self.get_writer().write_all(dos_stub)?;
        // 填充到 PE 头位置
        while self.stream_position()? < 0x80 {
            self.get_writer().write_u8(0)?;
        }
        Ok(())
    }

    /// 写入 NT 头（通用实现）
    fn write_nt_header(&mut self, nt_header: &NtHeader) -> Result<(), GaiaError> {
        self.get_writer().write_u32::<LittleEndian>(nt_header.signature)?;
        Ok(())
    }

    /// 写入 COFF 头（通用实现）
    fn write_coff_header(&mut self, coff_header: &crate::types::coff::CoffHeader) -> Result<(), GaiaError> {
        let writer = self.get_writer();
        writer.write_u16::<LittleEndian>(coff_header.machine)?;
        writer.write_u16::<LittleEndian>(coff_header.number_of_sections)?;
        writer.write_u32::<LittleEndian>(coff_header.time_date_stamp)?;
        writer.write_u32::<LittleEndian>(coff_header.pointer_to_symbol_table)?;
        writer.write_u32::<LittleEndian>(coff_header.number_of_symbols)?;
        writer.write_u16::<LittleEndian>(coff_header.size_of_optional_header)?;
        writer.write_u16::<LittleEndian>(coff_header.characteristics)?;
        Ok(())
    }

    /// 写入可选头（通用实现）
    fn write_optional_header(&mut self, optional_header: &OptionalHeader) -> Result<(), GaiaError> {
        let writer = self.get_writer();
        writer.write_u16::<LittleEndian>(optional_header.magic)?;
        writer.write_u8(optional_header.major_linker_version)?;
        writer.write_u8(optional_header.minor_linker_version)?;
        writer.write_u32::<LittleEndian>(optional_header.size_of_code)?;
        writer.write_u32::<LittleEndian>(optional_header.size_of_initialized_data)?;
        writer.write_u32::<LittleEndian>(optional_header.size_of_uninitialized_data)?;
        writer.write_u32::<LittleEndian>(optional_header.address_of_entry_point)?;
        writer.write_u32::<LittleEndian>(optional_header.base_of_code)?;

        // 根据架构写入不同的字段
        if optional_header.magic == 0x20b {
            // PE32+
            writer.write_u64::<LittleEndian>(optional_header.image_base)?;
        }
        else {
            // PE32
            let base_of_data = optional_header.base_of_data.unwrap_or(0);
            writer.write_u32::<LittleEndian>(base_of_data)?;
            writer.write_u32::<LittleEndian>(optional_header.image_base as u32)?;
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

        // 写入子系统
        let subsystem_value = match optional_header.subsystem {
            SubsystemType::Console => 3,
            SubsystemType::Windows => 2,
            SubsystemType::Native => 1,
            _ => 3, // 默认为控制台
        };
        writer.write_u16::<LittleEndian>(subsystem_value)?;

        writer.write_u16::<LittleEndian>(optional_header.dll_characteristics)?;

        // 根据架构写入不同大小的字段
        if optional_header.magic == 0x20b {
            // PE32+
            writer.write_u64::<LittleEndian>(optional_header.size_of_stack_reserve)?;
            writer.write_u64::<LittleEndian>(optional_header.size_of_stack_commit)?;
            writer.write_u64::<LittleEndian>(optional_header.size_of_heap_reserve)?;
            writer.write_u64::<LittleEndian>(optional_header.size_of_heap_commit)?;
        }
        else {
            // PE32
            writer.write_u32::<LittleEndian>(optional_header.size_of_stack_reserve as u32)?;
            writer.write_u32::<LittleEndian>(optional_header.size_of_stack_commit as u32)?;
            writer.write_u32::<LittleEndian>(optional_header.size_of_heap_reserve as u32)?;
            writer.write_u32::<LittleEndian>(optional_header.size_of_heap_commit as u32)?;
        }

        writer.write_u32::<LittleEndian>(optional_header.loader_flags)?;
        writer.write_u32::<LittleEndian>(optional_header.number_of_rva_and_sizes)?;

        // 写入数据目录
        for data_dir in &optional_header.data_directories {
            self.write_data_directory(data_dir)?;
        }

        Ok(())
    }

    /// 写入数据目录（通用实现）
    fn write_data_directory(&mut self, data_dir: &DataDirectory) -> Result<(), GaiaError> {
        let writer = self.get_writer();
        writer.write_u32::<LittleEndian>(data_dir.virtual_address)?;
        writer.write_u32::<LittleEndian>(data_dir.size)?;
        Ok(())
    }

    /// 写入节头（通用实现）
    fn write_section_header(&mut self, section: &PeSection) -> Result<(), GaiaError> {
        let writer = self.get_writer();
        // 写入节名（8字节，不足补0）
        let mut name_bytes = [0u8; 8];
        let name_len = section.name.len().min(8);
        name_bytes[..name_len].copy_from_slice(&section.name.as_bytes()[..name_len]);
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

    /// 填充到指定偏移（通用实现）
    fn pad_to_offset(&mut self, target_offset: u64) -> Result<(), GaiaError> {
        let current_pos = self.stream_position()?;
        if current_pos < target_offset {
            let padding_size = target_offset - current_pos;
            for _ in 0..padding_size {
                self.get_writer().write_u8(0)?;
            }
        }
        Ok(())
    }

    /// 对齐到边界（通用实现）
    fn align_to_boundary(&mut self, alignment: u32) -> Result<(), GaiaError> {
        let current_pos = self.stream_position()?;
        let remainder = current_pos % alignment as u64;
        if remainder != 0 {
            let padding = alignment as u64 - remainder;
            for _ in 0..padding {
                self.get_writer().write_u8(0)?;
            }
        }
        Ok(())
    }

    /// 写入导入表（通用实现）
    fn write_import_table(
        &mut self,
        imports: &crate::types::tables::ImportTable,
        sections: &[PeSection],
        pointer_size: usize,
    ) -> Result<(), GaiaError> {
        // 如果没有导入，直接返回
        if imports.entries.is_empty() {
            return Ok(());
        }

        // 查找 .idata 节
        let idata_section = sections.iter().find(|s| s.name == ".idata");
        if let Some(section) = idata_section {
            // 移动到 .idata 节的文件偏移
            self.pad_to_offset(section.pointer_to_raw_data as u64)?;

            let base_rva = section.virtual_address;
            let mut current_rva = base_rva + ((imports.entries.len() + 1) * 20) as u32; // 跳过导入描述符表

            // 计算 DLL 名称的 RVA
            let mut dll_name_rvas = Vec::new();
            for entry in &imports.entries {
                dll_name_rvas.push(current_rva);
                current_rva += (entry.dll_name.len() + 1) as u32; // 包括空终止符
            }

            // 对齐到 2 字节边界（名称通常按字对齐）
            if current_rva % 2 != 0 {
                current_rva += 1;
            }

            // 计算函数名称的 RVA（Hint+Name）
            let mut function_name_rvas = Vec::new();
            for entry in &imports.entries {
                let mut entry_function_rvas = Vec::new();
                for function in &entry.functions {
                    entry_function_rvas.push(current_rva);
                    current_rva += (2 + function.len() + 1) as u32; // Hint(2字节) + 函数名 + 空终止符
                }
                function_name_rvas.push(entry_function_rvas);
            }

            // 对齐到 2 字节边界（IMAGE_IMPORT_BY_NAME 推荐字对齐）
            if current_rva % 2 != 0 {
                current_rva += 1;
            }

            // 计算 INT 与 IAT 的 RVA（在函数名称之后）
            // 先分配 INT（OriginalFirstThunk 指向的名称指针数组），再分配 IAT（FirstThunk 指向的地址数组）
            // 对齐到 pointer_size 字节边界
            if current_rva % (pointer_size as u32) != 0 {
                current_rva = (current_rva + (pointer_size as u32) - 1) & !((pointer_size as u32) - 1);
            }

            let mut int_rvas = Vec::new();
            for entry in &imports.entries {
                int_rvas.push(current_rva);
                current_rva += (entry.functions.len() as u32) * (pointer_size as u32) + (pointer_size as u32);
            }

            // 对齐到 pointer_size 字节边界
            if current_rva % (pointer_size as u32) != 0 {
                current_rva = (current_rva + (pointer_size as u32) - 1) & !((pointer_size as u32) - 1);
            }

            let mut iat_rvas = Vec::new();
            for entry in &imports.entries {
                iat_rvas.push(current_rva);
                current_rva += (entry.functions.len() as u32) * (pointer_size as u32) + (pointer_size as u32);
            }

            // 写入导入描述符表
            for (i, _entry) in imports.entries.iter().enumerate() {
                let writer = self.get_writer();
                // x64：使用经典布局（OFT 指向 INT，IAT 初始为 0）
                // x86：为了兼容更多加载器，采用 OFT=0（不使用 INT），IAT 初始填入 Hint/Name 的 RVA
                if pointer_size == 8 {
                    writer.write_u32::<LittleEndian>(int_rvas[i])?; // OriginalFirstThunk (INT)
                }
                else {
                    writer.write_u32::<LittleEndian>(0)?; // OriginalFirstThunk = 0（使用 IAT 作为查找表）
                }
                writer.write_u32::<LittleEndian>(0)?; // TimeDateStamp
                writer.write_u32::<LittleEndian>(0)?; // ForwarderChain
                writer.write_u32::<LittleEndian>(dll_name_rvas[i])?; // Name RVA
                writer.write_u32::<LittleEndian>(iat_rvas[i])?; // FirstThunk 指向 IAT（地址数组）
            }

            // 写入终止符（全零的导入描述符）
            {
                let writer = self.get_writer();
                for _ in 0..5 {
                    writer.write_u32::<LittleEndian>(0)?;
                }
            }

            // 写入 DLL 名称字符串
            for entry in &imports.entries {
                let writer = self.get_writer();
                writer.write_all(entry.dll_name.as_bytes())?;
                writer.write_u8(0)?; // 空终止符
            }

            // 按 2 字节对齐
            if self.stream_position()? % 2 != 0 {
                self.get_writer().write_u8(0)?;
            }

            // 写入函数名称字符串
            for (_i, entry) in imports.entries.iter().enumerate() {
                for (_j, function) in entry.functions.iter().enumerate() {
                    let writer = self.get_writer();
                    writer.write_u16::<LittleEndian>(0)?; // Hint
                    writer.write_all(function.as_bytes())?;
                    writer.write_u8(0)?; // 空终止符
                }
            }

            // 按 2 字节对齐
            if self.stream_position()? % 2 != 0 {
                self.get_writer().write_u8(0)?;
            }

            // 写入 INT（OriginalFirstThunk 指向的名称指针数组）
            // 按 pointer_size 字节对齐
            while self.stream_position()? % (pointer_size as u64) != 0 {
                self.get_writer().write_u8(0)?;
            }
            for (i, entry) in imports.entries.iter().enumerate() {
                for j in 0..entry.functions.len() {
                    let writer = self.get_writer();
                    if pointer_size == 8 {
                        writer.write_u64::<LittleEndian>(function_name_rvas[i][j] as u64)?;
                    }
                    else {
                        writer.write_u32::<LittleEndian>(function_name_rvas[i][j])?;
                    }
                }
                let writer = self.get_writer();
                if pointer_size == 8 {
                    writer.write_u64::<LittleEndian>(0)?;
                }
                else {
                    writer.write_u32::<LittleEndian>(0)?;
                }
            }

            // 写入 IAT（FirstThunk 指向的地址数组）。
            // 经典模式下 IAT 初始为 0，加载器解析后填入实际地址。
            // 按 pointer_size 字节对齐
            while self.stream_position()? % (pointer_size as u64) != 0 {
                self.get_writer().write_u8(0)?;
            }
            for (i, entry) in imports.entries.iter().enumerate() {
                for _j in 0..entry.functions.len() {
                    let writer = self.get_writer();
                    if pointer_size == 8 {
                        // x64：IAT 初始填入 Hint/Name 的 RVA（与 x86 一致）
                        // 加载器解析后会覆盖为实际地址。
                        writer.write_u64::<LittleEndian>(function_name_rvas[i][_j] as u64)?;
                    }
                    else {
                        // x86：IAT 初始填入 Hint/Name 的 RVA，加载器解析后覆盖
                        writer.write_u32::<LittleEndian>(function_name_rvas[i][_j])?;
                    }
                }
                // 终止符 0
                let writer = self.get_writer();
                if pointer_size == 8 {
                    writer.write_u64::<LittleEndian>(0)?;
                }
                else {
                    writer.write_u32::<LittleEndian>(0)?;
                }
            }

            // 对齐到节的大小
            self.align_to_boundary(section.size_of_raw_data)?;
        }

        Ok(())
    }
}
