//! ELF 文件写入器模块
//!
//! 此模块提供将 ELF 结构体写入二进制文件的功能。

use crate::types::{ElfFile, ElfHeader64, ProgramHeader64, SectionHeader64};
use byteorder::{LittleEndian, WriteBytesExt};
use gaia_types::GaiaError;
use std::io::{Seek, Write};

/// ELF 文件生成器的通用接口
#[derive(Debug)]
pub struct ElfWriter<W> {
    writer: W,
}

impl<W> ElfWriter<W> {
    /// 创建一个新的 ELF 写入器
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    pub fn finish(self) -> W {
        self.writer
    }
}

impl<W: Write> ElfWriter<W> {
    /// 将 ELF 文件写入字节数组
    pub fn write_elf_file(&mut self, elf_file: &ElfFile) -> Result<(), GaiaError>
    where
        W: Seek,
    {
        // 写入 ELF 头
        self.write_elf_header(&elf_file.header)?;

        // 写入程序头表
        for program_header in &elf_file.program_headers {
            self.write_program_header(program_header)?;
        }

        // 对齐到页边界 (4KB)
        self.align_to_boundary(0x1000)?;

        // 写入数据
        self.writer.write_all(&elf_file.data)?;

        Ok(())
    }

    /// 写入 ELF 头（64位）
    pub fn write_elf_header(&mut self, header: &ElfHeader64) -> Result<(), GaiaError> {
        // 写入 ELF 标识符 (16 字节)
        self.writer.write_all(&header.e_ident)?;

        // 写入文件类型
        self.writer.write_u16::<LittleEndian>(header.e_type)?;

        // 写入机器架构
        self.writer.write_u16::<LittleEndian>(header.e_machine)?;

        // 写入版本
        self.writer.write_u32::<LittleEndian>(header.e_version)?;

        // 写入入口点地址
        self.writer.write_u64::<LittleEndian>(header.e_entry)?;

        // 写入程序头表偏移
        self.writer.write_u64::<LittleEndian>(header.e_phoff)?;

        // 写入节头表偏移
        self.writer.write_u64::<LittleEndian>(header.e_shoff)?;

        // 写入处理器特定标志
        self.writer.write_u32::<LittleEndian>(header.e_flags)?;

        // 写入 ELF 头大小
        self.writer.write_u16::<LittleEndian>(header.e_ehsize)?;

        // 写入程序头表项大小
        self.writer.write_u16::<LittleEndian>(header.e_phentsize)?;

        // 写入程序头表项数量
        self.writer.write_u16::<LittleEndian>(header.e_phnum)?;

        // 写入节头表项大小
        self.writer.write_u16::<LittleEndian>(header.e_shentsize)?;

        // 写入节头表项数量
        self.writer.write_u16::<LittleEndian>(header.e_shnum)?;

        // 写入字符串表索引
        self.writer.write_u16::<LittleEndian>(header.e_shstrndx)?;

        Ok(())
    }

    /// 写入程序头（64位）
    pub fn write_program_header(&mut self, header: &ProgramHeader64) -> Result<(), GaiaError> {
        // 写入段类型
        self.writer.write_u32::<LittleEndian>(header.p_type)?;

        // 写入段标志
        self.writer.write_u32::<LittleEndian>(header.p_flags)?;

        // 写入段在文件中的偏移
        self.writer.write_u64::<LittleEndian>(header.p_offset)?;

        // 写入段的虚拟地址
        self.writer.write_u64::<LittleEndian>(header.p_vaddr)?;

        // 写入段的物理地址
        self.writer.write_u64::<LittleEndian>(header.p_paddr)?;

        // 写入段在文件中的大小
        self.writer.write_u64::<LittleEndian>(header.p_filesz)?;

        // 写入段在内存中的大小
        self.writer.write_u64::<LittleEndian>(header.p_memsz)?;

        // 写入段对齐
        self.writer.write_u64::<LittleEndian>(header.p_align)?;

        Ok(())
    }

    /// 写入节头（64位）
    pub fn write_section_header(&mut self, header: &SectionHeader64) -> Result<(), GaiaError> {
        // 写入节名称索引
        self.writer.write_u32::<LittleEndian>(header.sh_name)?;

        // 写入节类型
        self.writer.write_u32::<LittleEndian>(header.sh_type)?;

        // 写入节标志
        self.writer.write_u64::<LittleEndian>(header.sh_flags)?;

        // 写入节的虚拟地址
        self.writer.write_u64::<LittleEndian>(header.sh_addr)?;

        // 写入节在文件中的偏移
        self.writer.write_u64::<LittleEndian>(header.sh_offset)?;

        // 写入节的大小
        self.writer.write_u64::<LittleEndian>(header.sh_size)?;

        // 写入节头表索引链接
        self.writer.write_u32::<LittleEndian>(header.sh_link)?;

        // 写入附加信息
        self.writer.write_u32::<LittleEndian>(header.sh_info)?;

        // 写入节对齐
        self.writer.write_u64::<LittleEndian>(header.sh_addralign)?;

        // 写入节项大小
        self.writer.write_u64::<LittleEndian>(header.sh_entsize)?;

        Ok(())
    }

    /// 对齐到指定边界
    pub fn align_to_boundary(&mut self, boundary: u32) -> Result<(), GaiaError>
    where
        W: Seek,
    {
        let current_pos = self.writer.stream_position()?;
        let alignment = boundary as u64;
        let padding = (alignment - (current_pos % alignment)) % alignment;

        for _ in 0..padding {
            self.writer.write_u8(0)?;
        }

        Ok(())
    }

    /// 填充到指定偏移
    pub fn pad_to_offset(&mut self, offset: u64) -> Result<(), GaiaError>
    where
        W: Seek,
    {
        let current_pos = self.writer.stream_position()?;
        if current_pos < offset {
            let padding = offset - current_pos;
            for _ in 0..padding {
                self.writer.write_u8(0)?;
            }
        }
        Ok(())
    }
}

/// ELF 文件构建器
///
/// 提供高级接口来构建 ELF 文件
#[derive(Debug)]
pub struct ElfBuilder {
    elf_file: ElfFile,
}

impl ElfBuilder {
    /// 创建新的 ELF 构建器
    pub fn new() -> Self {
        Self { elf_file: ElfFile::new() }
    }

    /// 设置入口点
    pub fn set_entry_point(&mut self, entry: u64) -> &mut Self {
        self.elf_file.set_entry_point(entry);
        self
    }

    /// 添加代码段
    pub fn add_code_segment(&mut self, code: Vec<u8>) -> &mut Self {
        let code_size = code.len() as u64;
        let code_offset = 0x1000; // 4KB 偏移
        let code_vaddr = 0x401000; // 虚拟地址

        // 创建程序头
        let program_header = ProgramHeader64::new_load_segment(code_offset, code_vaddr, code_size);
        self.elf_file.add_program_header(program_header);

        // 设置数据
        self.elf_file.data = code;

        self
    }

    /// 构建 ELF 文件
    pub fn build(self) -> ElfFile {
        self.elf_file
    }

    /// 构建并写入到写入器
    pub fn write_to<W: Write + Seek>(&self, writer: W) -> Result<W, GaiaError> {
        let mut elf_writer = ElfWriter::new(writer);
        elf_writer.write_elf_file(&self.elf_file)?;
        Ok(elf_writer.finish())
    }

    /// 构建并转换为字节数组
    pub fn to_bytes(&self) -> Vec<u8> {
        self.elf_file.to_bytes()
    }
}

impl Default for ElfBuilder {
    fn default() -> Self {
        Self::new()
    }
}
