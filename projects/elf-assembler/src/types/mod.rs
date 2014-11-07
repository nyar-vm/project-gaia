use serde::{Deserialize, Serialize};
use std::fmt;

/// ELF 文件类型枚举
///
/// 定义了 ELF 文件的不同类型，决定了文件的用途和加载方式。
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum ElfType {
    /// 无类型文件
    None,
    /// 可重定位文件（目标文件）
    Relocatable,
    /// 可执行文件
    Executable,
    /// 共享目标文件（动态库）
    SharedObject,
    /// 核心转储文件
    Core,
}

impl fmt::Display for ElfType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ElfType::None => write!(f, "无类型文件"),
            ElfType::Relocatable => write!(f, "可重定位文件"),
            ElfType::Executable => write!(f, "可执行文件"),
            ElfType::SharedObject => write!(f, "共享目标文件"),
            ElfType::Core => write!(f, "核心转储文件"),
        }
    }
}

impl From<u16> for ElfType {
    fn from(value: u16) -> Self {
        match value {
            0 => ElfType::None,
            1 => ElfType::Relocatable,
            2 => ElfType::Executable,
            3 => ElfType::SharedObject,
            4 => ElfType::Core,
            _ => ElfType::None,
        }
    }
}

/// ELF 机器类型枚举
///
/// 定义了 ELF 文件支持的处理器架构类型。
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum ElfMachine {
    /// 无机器类型
    None,
    /// Intel 80386
    I386,
    /// AMD x86-64
    X86_64,
    /// ARM
    Arm,
    /// AArch64
    AArch64,
    /// RISC-V
    RiscV,
}

impl From<u16> for ElfMachine {
    fn from(value: u16) -> Self {
        match value {
            0 => ElfMachine::None,
            3 => ElfMachine::I386,
            62 => ElfMachine::X86_64,
            40 => ElfMachine::Arm,
            183 => ElfMachine::AArch64,
            243 => ElfMachine::RiscV,
            _ => ElfMachine::None,
        }
    }
}

/// ELF 头结构（64位）
///
/// ELF 文件的主要头部结构，包含文件的基本信息和元数据。
/// 这是 ELF 文件的第一个结构，用于标识文件格式和基本属性。
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct ElfHeader64 {
    /// ELF 魔数和标识信息 (16 字节)
    pub e_ident: [u8; 16],
    /// 文件类型
    pub e_type: u16,
    /// 机器架构
    pub e_machine: u16,
    /// 文件版本
    pub e_version: u32,
    /// 程序入口点地址
    pub e_entry: u64,
    /// 程序头表偏移
    pub e_phoff: u64,
    /// 节头表偏移
    pub e_shoff: u64,
    /// 处理器特定标志
    pub e_flags: u32,
    /// ELF 头大小
    pub e_ehsize: u16,
    /// 程序头表项大小
    pub e_phentsize: u16,
    /// 程序头表项数量
    pub e_phnum: u16,
    /// 节头表项大小
    pub e_shentsize: u16,
    /// 节头表项数量
    pub e_shnum: u16,
    /// 字符串表索引
    pub e_shstrndx: u16,
}

/// ELF 程序头结构（64位）
///
/// 描述程序段的信息，用于程序加载时的内存布局。
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct ProgramHeader64 {
    /// 段类型
    pub p_type: u32,
    /// 段标志
    pub p_flags: u32,
    /// 段在文件中的偏移
    pub p_offset: u64,
    /// 段的虚拟地址
    pub p_vaddr: u64,
    /// 段的物理地址
    pub p_paddr: u64,
    /// 段在文件中的大小
    pub p_filesz: u64,
    /// 段在内存中的大小
    pub p_memsz: u64,
    /// 段对齐
    pub p_align: u64,
}

/// ELF 节头结构（64位）
///
/// 描述文件中各个节的信息，用于链接和调试。
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct SectionHeader64 {
    /// 节名称（字符串表索引）
    pub sh_name: u32,
    /// 节类型
    pub sh_type: u32,
    /// 节标志
    pub sh_flags: u64,
    /// 节的虚拟地址
    pub sh_addr: u64,
    /// 节在文件中的偏移
    pub sh_offset: u64,
    /// 节的大小
    pub sh_size: u64,
    /// 节头表索引链接
    pub sh_link: u32,
    /// 附加信息
    pub sh_info: u32,
    /// 节对齐
    pub sh_addralign: u64,
    /// 节项大小
    pub sh_entsize: u64,
}

/// ELF 程序段类型常量
pub mod segment_type {
    /// 空段
    pub const PT_NULL: u32 = 0;
    /// 可加载段
    pub const PT_LOAD: u32 = 1;
    /// 动态链接信息
    pub const PT_DYNAMIC: u32 = 2;
    /// 解释器路径
    pub const PT_INTERP: u32 = 3;
    /// 注释信息
    pub const PT_NOTE: u32 = 4;
    /// 保留
    pub const PT_SHLIB: u32 = 5;
    /// 程序头表
    pub const PT_PHDR: u32 = 6;
    /// 线程本地存储
    pub const PT_TLS: u32 = 7;
}

/// ELF 节类型常量
pub mod section_type {
    /// 空节
    pub const SHT_NULL: u32 = 0;
    /// 程序数据
    pub const SHT_PROGBITS: u32 = 1;
    /// 符号表
    pub const SHT_SYMTAB: u32 = 2;
    /// 字符串表
    pub const SHT_STRTAB: u32 = 3;
    /// 重定位表（带加数）
    pub const SHT_RELA: u32 = 4;
    /// 哈希表
    pub const SHT_HASH: u32 = 5;
    /// 动态链接信息
    pub const SHT_DYNAMIC: u32 = 6;
    /// 注释
    pub const SHT_NOTE: u32 = 7;
    /// 无数据的程序空间
    pub const SHT_NOBITS: u32 = 8;
    /// 重定位表（不带加数）
    pub const SHT_REL: u32 = 9;
    /// 保留
    pub const SHT_SHLIB: u32 = 10;
    /// 动态符号表
    pub const SHT_DYNSYM: u32 = 11;
}

/// ELF 段标志常量
pub mod segment_flags {
    /// 可执行
    pub const PF_X: u32 = 1;
    /// 可写
    pub const PF_W: u32 = 2;
    /// 可读
    pub const PF_R: u32 = 4;
}

/// ELF 节标志常量
pub mod section_flags {
    /// 可写
    pub const SHF_WRITE: u64 = 1;
    /// 占用内存
    pub const SHF_ALLOC: u64 = 2;
    /// 可执行
    pub const SHF_EXECINSTR: u64 = 4;
}

impl ElfHeader64 {
    /// 创建一个标准的 ELF64 头
    pub fn new() -> Self {
        let mut e_ident = [0u8; 16];
        // ELF 魔数
        e_ident[0] = 0x7f;
        e_ident[1] = b'E';
        e_ident[2] = b'L';
        e_ident[3] = b'F';
        // 64位
        e_ident[4] = 2;
        // 小端序
        e_ident[5] = 1;
        // 当前版本
        e_ident[6] = 1;
        // System V ABI
        e_ident[7] = 0;

        Self {
            e_ident,
            e_type: 2,     // ET_EXEC
            e_machine: 62, // EM_X86_64
            e_version: 1,
            e_entry: 0x401000, // 默认入口点
            e_phoff: 64,       // 程序头表偏移
            e_shoff: 0,        // 节头表偏移（暂时为0）
            e_flags: 0,
            e_ehsize: 64,    // ELF头大小
            e_phentsize: 56, // 程序头表项大小
            e_phnum: 1,      // 程序头表项数量
            e_shentsize: 64, // 节头表项大小
            e_shnum: 0,      // 节头表项数量
            e_shstrndx: 0,   // 字符串表索引
        }
    }
}

impl Default for ElfHeader64 {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgramHeader64 {
    /// 创建一个可加载的代码段
    pub fn new_load_segment(offset: u64, vaddr: u64, size: u64) -> Self {
        Self {
            p_type: segment_type::PT_LOAD,
            p_flags: segment_flags::PF_R | segment_flags::PF_X, // 可读可执行
            p_offset: offset,
            p_vaddr: vaddr,
            p_paddr: vaddr,
            p_filesz: size,
            p_memsz: size,
            p_align: 0x1000, // 4KB 对齐
        }
    }
}

/// ELF 文件构建器
///
/// 用于构建完整的 ELF 文件结构
#[derive(Debug, Clone)]
pub struct ElfFile {
    /// ELF 头
    pub header: ElfHeader64,
    /// 程序头表
    pub program_headers: Vec<ProgramHeader64>,
    /// 节头表
    pub section_headers: Vec<SectionHeader64>,
    /// 文件数据
    pub data: Vec<u8>,
}

impl ElfFile {
    /// 创建新的 ELF 文件
    pub fn new() -> Self {
        Self { header: ElfHeader64::new(), program_headers: Vec::new(), section_headers: Vec::new(), data: Vec::new() }
    }

    /// 添加程序头
    pub fn add_program_header(&mut self, header: ProgramHeader64) {
        self.program_headers.push(header);
        self.header.e_phnum = self.program_headers.len() as u16;
    }

    /// 设置入口点
    pub fn set_entry_point(&mut self, entry: u64) {
        self.header.e_entry = entry;
    }

    /// 生成完整的 ELF 文件字节数组
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // 写入 ELF 头
        bytes.extend_from_slice(&self.header_to_bytes());

        // 写入程序头表
        for ph in &self.program_headers {
            bytes.extend_from_slice(&self.program_header_to_bytes(ph));
        }

        // 对齐到页边界
        while bytes.len() % 0x1000 != 0 {
            bytes.push(0);
        }

        // 写入数据
        bytes.extend_from_slice(&self.data);

        bytes
    }

    /// 将 ELF 头转换为字节数组
    fn header_to_bytes(&self) -> [u8; 64] {
        let mut bytes = [0u8; 64];
        let h = &self.header;

        bytes[0..16].copy_from_slice(&h.e_ident);
        bytes[16..18].copy_from_slice(&h.e_type.to_le_bytes());
        bytes[18..20].copy_from_slice(&h.e_machine.to_le_bytes());
        bytes[20..24].copy_from_slice(&h.e_version.to_le_bytes());
        bytes[24..32].copy_from_slice(&h.e_entry.to_le_bytes());
        bytes[32..40].copy_from_slice(&h.e_phoff.to_le_bytes());
        bytes[40..48].copy_from_slice(&h.e_shoff.to_le_bytes());
        bytes[48..52].copy_from_slice(&h.e_flags.to_le_bytes());
        bytes[52..54].copy_from_slice(&h.e_ehsize.to_le_bytes());
        bytes[54..56].copy_from_slice(&h.e_phentsize.to_le_bytes());
        bytes[56..58].copy_from_slice(&h.e_phnum.to_le_bytes());
        bytes[58..60].copy_from_slice(&h.e_shentsize.to_le_bytes());
        bytes[60..62].copy_from_slice(&h.e_shnum.to_le_bytes());
        bytes[62..64].copy_from_slice(&h.e_shstrndx.to_le_bytes());

        bytes
    }

    /// 将程序头转换为字节数组
    fn program_header_to_bytes(&self, ph: &ProgramHeader64) -> [u8; 56] {
        let mut bytes = [0u8; 56];

        bytes[0..4].copy_from_slice(&ph.p_type.to_le_bytes());
        bytes[4..8].copy_from_slice(&ph.p_flags.to_le_bytes());
        bytes[8..16].copy_from_slice(&ph.p_offset.to_le_bytes());
        bytes[16..24].copy_from_slice(&ph.p_vaddr.to_le_bytes());
        bytes[24..32].copy_from_slice(&ph.p_paddr.to_le_bytes());
        bytes[32..40].copy_from_slice(&ph.p_filesz.to_le_bytes());
        bytes[40..48].copy_from_slice(&ph.p_memsz.to_le_bytes());
        bytes[48..56].copy_from_slice(&ph.p_align.to_le_bytes());

        bytes
    }
}

impl Default for ElfFile {
    fn default() -> Self {
        Self::new()
    }
}
