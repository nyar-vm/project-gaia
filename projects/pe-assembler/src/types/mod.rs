#![doc = include_str!("readme.md")]

pub use self::{
    dos::DosHeader,
    nt::NtHeader,
    tables::{ExportTable, ImportTable},
};
use byteorder::{LittleEndian, ReadBytesExt};
use gaia_types::helpers::Architecture;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Display, Formatter},
    io::{Read},
};

pub mod coff;
mod dos;
mod nt;
pub mod tables;

pub use coff::*;
use gaia_types::GaiaError;

/// PE 子系统类型枚举
///
/// 定义了 Windows PE 文件可以使用的各种子系统类型，
/// 这些类型决定了程序运行时的环境和依赖。
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum SubsystemType {
    /// 控制台应用程序，运行在控制台窗口中
    Console,
    /// Windows GUI 应用程序，具有图形界面
    Windows,
    /// 原生驱动程序，运行在核心态
    Native,
    /// POSIX 子系统应用程序
    Posix,
    /// Windows CE 子系统
    WindowsCe,
    /// EFI 应用程序
    Efi,
    /// EFI 启动服务驱动程序
    EfiBootServiceDriver,
    /// EFI 运行时驱动程序
    EfiRuntimeDriver,
    /// EFI ROM 映像
    EfiRom,
    /// Xbox 应用程序
    Xbox,
    /// Windows 启动应用程序
    WindowsBootApplication,
}

impl Display for SubsystemType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SubsystemType::Console => write!(f, "控制台应用程序"),
            SubsystemType::Windows => write!(f, "Windows GUI 应用程序"),
            SubsystemType::Native => write!(f, "原生驱动程序"),
            SubsystemType::Posix => write!(f, "POSIX 子系统应用程序"),
            SubsystemType::WindowsCe => write!(f, "Windows CE 子系统"),
            SubsystemType::Efi => write!(f, "EFI 应用程序"),
            SubsystemType::EfiBootServiceDriver => write!(f, "EFI 启动服务驱动程序"),
            SubsystemType::EfiRuntimeDriver => write!(f, "EFI 运行时驱动程序"),
            SubsystemType::EfiRom => write!(f, "EFI ROM 映像"),
            SubsystemType::Xbox => write!(f, "Xbox 应用程序"),
            SubsystemType::WindowsBootApplication => write!(f, "Windows 启动应用程序"),
        }
    }
}

impl From<u16> for SubsystemType {
    /// 从 u16 值创建子系统类型
    ///
    /// # Arguments
    /// * `value` - 子系统类型的数值
    ///
    /// # Returns
    /// 返回对应的子系统类型，未知类型返回 Console
    fn from(value: u16) -> Self {
        match value {
            1 => SubsystemType::Native,
            2 => SubsystemType::Windows,
            3 => SubsystemType::Console,
            7 => SubsystemType::Posix,
            9 => SubsystemType::WindowsCe,
            10 => SubsystemType::Efi,
            11 => SubsystemType::EfiBootServiceDriver,
            12 => SubsystemType::EfiRuntimeDriver,
            13 => SubsystemType::EfiRom,
            14 => SubsystemType::Xbox,
            16 => SubsystemType::WindowsBootApplication,
            _ => SubsystemType::Console, // 默认值
        }
    }
}

impl Default for DataDirectory {
    fn default() -> Self {
        Self { virtual_address: 0, size: 0 }
    }
}

/// 可选头结构
///
/// 包含 PE 文件的加载和运行时信息，如入口点地址、内存布局、
/// 版本信息等。这个结构对于 Windows 加载器正确加载和执行程序至关重要。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OptionalHeader {
    /// 魔数，标识 PE32 或 PE32+ 格式
    pub magic: u16,
    /// 链接器的主版本号
    pub major_linker_version: u8,
    /// 链接器的次版本号
    pub minor_linker_version: u8,
    /// 代码节的总大小（以字节为单位）
    pub size_of_code: u32,
    /// 已初始化数据的总大小
    pub size_of_initialized_data: u32,
    /// 未初始化数据的总大小
    pub size_of_uninitialized_data: u32,
    /// 程序入口点的 RVA（相对虚拟地址）
    pub address_of_entry_point: u32,
    /// 代码节的起始 RVA
    pub base_of_code: u32,
    /// 数据节的起始 RVA，仅 PE32 格式有效
    pub base_of_data: Option<u32>, // Only for PE32
    /// 映像的首选加载地址
    pub image_base: u64,
    /// 节在内存中的对齐粒度
    pub section_alignment: u32,
    /// 节在文件中的对齐粒度
    pub file_alignment: u32,
    /// 所需操作系统的主版本号
    pub major_operating_system_version: u16,
    /// 所需操作系统的次版本号
    pub minor_operating_system_version: u16,
    /// 映像的主版本号
    pub major_image_version: u16,
    /// 映像的次版本号
    pub minor_image_version: u16,
    /// 子系统的主版本号
    pub major_subsystem_version: u16,
    /// 子系统的次版本号
    pub minor_subsystem_version: u16,
    /// 保留字段，必须为 0
    pub win32_version_value: u32,
    /// 映像的总大小，包括所有头文件和节
    pub size_of_image: u32,
    /// 所有头文件的总大小
    pub size_of_headers: u32,
    /// 映像的校验和，用于内核模式和系统 DLL
    pub checksum: u32,
    /// 子系统类型，定义程序运行环境
    pub subsystem: SubsystemType,
    /// DLL 特征标志，描述 DLL 的各种属性
    pub dll_characteristics: u16,
    /// 为线程栈保留的虚拟内存大小
    pub size_of_stack_reserve: u64,
    /// 为线程栈提交的虚拟内存大小
    pub size_of_stack_commit: u64,
    /// 为进程堆保留的虚拟内存大小
    pub size_of_heap_reserve: u64,
    /// 为进程堆提交的虚拟内存大小
    pub size_of_heap_commit: u64,
    /// 保留字段，必须为 0
    pub loader_flags: u32,
    /// 数据目录表的条目数量
    pub number_of_rva_and_sizes: u32,
    /// 数据目录表，包含各种数据目录的信息
    pub data_directories: Vec<DataDirectory>,
}

impl OptionalHeader {
    /// 创建一个标准的可选头，适用于 .NET 程序
    pub fn new(
        entry_point: u32,
        image_base: u64,
        size_of_code: u32,
        size_of_headers: u32,
        size_of_image: u32,
        subsystem: SubsystemType,
    ) -> Self {
        let mut data_directories = Vec::with_capacity(16);
        // 初始化 16 个标准数据目录
        for _ in 0..16 {
            data_directories.push(DataDirectory::default());
        }

        Self {
            magic: 0x020B, // PE32+
            major_linker_version: 14,
            minor_linker_version: 0,
            size_of_code,
            size_of_initialized_data: 0,
            size_of_uninitialized_data: 0,
            address_of_entry_point: entry_point,
            base_of_code: 0x2000,
            base_of_data: None, // PE32+ 不使用
            image_base,
            section_alignment: 0x2000,
            file_alignment: 0x200,
            major_operating_system_version: 6,
            minor_operating_system_version: 0,
            major_image_version: 0,
            minor_image_version: 0,
            major_subsystem_version: 6,
            minor_subsystem_version: 0,
            win32_version_value: 0,
            size_of_image,
            size_of_headers,
            checksum: 0,
            subsystem,
            // 关闭 DYNAMIC_BASE 以确保固定映像基址 (0x400000)，避免绝对地址失效
            // 保留 NX_COMPAT、NO_SEH、TERMINAL_SERVER_AWARE
            dll_characteristics: 0x8500, // NX_COMPAT | NO_SEH | TERMINAL_SERVER_AWARE
            size_of_stack_reserve: 0x100000,
            size_of_stack_commit: 0x1000,
            size_of_heap_reserve: 0x100000,
            size_of_heap_commit: 0x1000,
            loader_flags: 0,
            number_of_rva_and_sizes: 16,
            data_directories,
        }
    }

    /// 根据架构创建可选头
    pub fn new_for_architecture(
        architecture: &Architecture,
        entry_point: u32,
        image_base: u64,
        size_of_code: u32,
        size_of_headers: u32,
        size_of_image: u32,
        subsystem: SubsystemType,
    ) -> Self {
        let mut data_directories = Vec::with_capacity(16);
        // 初始化 16 个标准数据目录
        for _ in 0..16 {
            data_directories.push(DataDirectory::default());
        }

        let (magic, base_of_data) = match architecture {
            Architecture::X86 => (0x010B, Some(0x2000)), // PE32
            Architecture::X86_64 => (0x020B, None),      // PE32+
            _ => (0x010B, Some(0x2000)),                 // 默认为 PE32
        };

        Self {
            magic,
            major_linker_version: 14,
            minor_linker_version: 0,
            size_of_code,
            size_of_initialized_data: 0,
            size_of_uninitialized_data: 0,
            address_of_entry_point: entry_point,
            base_of_code: 0x1000,
            base_of_data,
            image_base,
            section_alignment: 0x1000,
            file_alignment: 0x200,
            major_operating_system_version: 6,
            minor_operating_system_version: 0,
            major_image_version: 0,
            minor_image_version: 0,
            major_subsystem_version: 6,
            minor_subsystem_version: 0,
            win32_version_value: 0,
            size_of_image,
            size_of_headers,
            checksum: 0,
            subsystem,
            dll_characteristics: 0x8160, // DYNAMIC_BASE | NX_COMPAT | NO_SEH | TERMINAL_SERVER_AWARE
            size_of_stack_reserve: 0x100000,
            size_of_stack_commit: 0x1000,
            size_of_heap_reserve: 0x100000,
            size_of_heap_commit: 0x1000,
            loader_flags: 0,
            number_of_rva_and_sizes: 16,
            data_directories,
        }
    }
}

/// PE 头结构
///
/// 包含 PE 文件的所有头部信息，是 DOS 头、NT 头、COFF 头和可选头的组合。
/// 这个结构提供了完整的 PE 文件元数据信息。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PeHeader {
    /// DOS 头，包含 DOS 兼容性信息
    pub dos_header: DosHeader,
    /// NT 头，包含 PE 签名
    pub nt_header: NtHeader,
    /// COFF 头，包含目标文件信息
    pub coff_header: CoffHeader,
    /// 可选头，包含加载和运行时信息
    pub optional_header: OptionalHeader,
}

/// PE 节结构
///
/// 包含 PE 文件中每个节的详细信息，包括节名、虚拟地址、大小等属性。
/// 节是 PE 文件中的基本组织单位，不同的节包含不同类型的数据（如代码、数据、资源等）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeSection {
    /// 节名称，最多8个字符
    pub name: String,
    /// 节在内存中的虚拟大小
    pub virtual_size: u32,
    /// 节在内存中的虚拟地址
    pub virtual_address: u32,
    /// 节在文件中的原始数据大小
    pub size_of_raw_data: u32,
    /// 节在文件中的原始数据指针
    pub pointer_to_raw_data: u32,
    /// 重定位表指针
    pub pointer_to_relocations: u32,
    /// 行号表指针
    pub pointer_to_line_numbers: u32,
    /// 重定位项数量
    pub number_of_relocations: u16,
    /// 行号数量
    pub number_of_line_numbers: u16,
    /// 节特征标志
    pub characteristics: u32,
    /// 节的原始数据
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub data: Vec<u8>,
}

/// PE 文件读取配置
///
/// 控制 PE 文件解析过程的行为，允许选择性解析不同的部分。
/// 通过调整这些配置，可以在性能和功能之间取得平衡。
#[derive(Debug, Clone, Copy)]
pub struct ReadConfig {
    /// 是否包含节数据，如果为 false 则只解析头部信息
    pub include_sections: bool,
    /// 是否验证校验和，验证会增加解析时间
    pub validate_checksum: bool,
    /// 是否解析导入表，导入表包含依赖的 DLL 信息
    pub parse_imports: bool,
    /// 是否解析导出表，导出表包含对外提供的函数信息
    pub parse_exports: bool,
}

impl Default for ReadConfig {
    /// 创建默认的读取配置
    ///
    /// 默认配置包含节数据、解析导入和导出表，但不验证校验和。
    /// 这种配置在大多数情况下提供了良好的性能和功能平衡。
    fn default() -> Self {
        Self { include_sections: true, validate_checksum: false, parse_imports: true, parse_exports: true }
    }
}

/// PE 程序结构
///
/// 表示一个完整的 PE（Portable Executable）程序，包含所有头部信息和节数据。
/// 这是 PE 文件的最高级别抽象，包含了文件解析后的完整内容。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeProgram {
    /// PE 头部信息，包含 DOS 头、NT 头、COFF 头和可选头
    pub header: PeHeader,
    /// 所有节的集合，包含代码、数据、资源等
    pub sections: Vec<PeSection>,
    /// 导入表，包含程序依赖的外部函数和库
    pub imports: ImportTable,
    /// 导出表，包含程序向外提供的函数和符号
    pub exports: ExportTable,
}

/// PE 信息结构
///
/// 提供 PE 文件的摘要信息，包含关键属性和统计信息。
/// 这个结构用于快速获取文件的基本信息，而无需解析完整的头部结构。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeInfo {
    /// 目标架构类型，如 x86、x64、ARM 等
    pub target_arch: Architecture,
    /// 子系统类型，定义程序运行环境
    pub subsystem: SubsystemType,
    /// 程序入口点的 RVA（相对虚拟地址）
    pub entry_point: u32,
    /// 映像的首选加载地址
    pub image_base: u64,
    /// 文件中节的数量
    pub section_count: u16,
    /// 文件的总大小（以字节为单位）
    pub file_size: u64,
}

/// 数据目录结构
///
/// 包含 PE 文件中各种数据目录的信息，如导入表、导出表、
/// 资源表等。每个数据目录项包含一个RVA和大小。
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct DataDirectory {
    /// 数据目录的相对虚拟地址（RVA）
    pub virtual_address: u32,
    /// 数据目录的大小（以字节为单位）
    pub size: u32,
}

impl DataDirectory {
    /// 从 ExeReader 读取数据目录
    pub fn read<R: Read>(mut reader: R) -> Result<Self, GaiaError> {
        Ok(DataDirectory { virtual_address: reader.read_u32::<LittleEndian>()?, size: reader.read_u32::<LittleEndian>()? })
    }
}

impl OptionalHeader {
    /// 从 ExeReader 读取可选头
    pub fn read<R: Read>(mut reader: R) -> Result<Self, GaiaError> {
        let magic = reader.read_u16::<LittleEndian>()?;
        let is_pe32_plus = magic == 0x020B;

        let major_linker_version = reader.read_u8()?;
        let minor_linker_version = reader.read_u8()?;
        let size_of_code = reader.read_u32::<LittleEndian>()?;
        let size_of_initialized_data = reader.read_u32::<LittleEndian>()?;
        let size_of_uninitialized_data = reader.read_u32::<LittleEndian>()?;
        let address_of_entry_point = reader.read_u32::<LittleEndian>()?;
        let base_of_code = reader.read_u32::<LittleEndian>()?;

        let (base_of_data, image_base) = if is_pe32_plus {
            (None, reader.read_u64::<LittleEndian>()?)
        }
        else {
            (Some(reader.read_u32::<LittleEndian>()?), reader.read_u32::<LittleEndian>()? as u64)
        };

        let section_alignment = reader.read_u32::<LittleEndian>()?;
        let file_alignment = reader.read_u32::<LittleEndian>()?;
        let major_operating_system_version = reader.read_u16::<LittleEndian>()?;
        let minor_operating_system_version = reader.read_u16::<LittleEndian>()?;
        let major_image_version = reader.read_u16::<LittleEndian>()?;
        let minor_image_version = reader.read_u16::<LittleEndian>()?;
        let major_subsystem_version = reader.read_u16::<LittleEndian>()?;
        let minor_subsystem_version = reader.read_u16::<LittleEndian>()?;
        let win32_version_value = reader.read_u32::<LittleEndian>()?;
        let size_of_image = reader.read_u32::<LittleEndian>()?;
        let size_of_headers = reader.read_u32::<LittleEndian>()?;
        let checksum = reader.read_u32::<LittleEndian>()?;
        let subsystem = reader.read_u16::<LittleEndian>()?.into();
        let dll_characteristics = reader.read_u16::<LittleEndian>()?;

        let (size_of_stack_reserve, size_of_stack_commit, size_of_heap_reserve, size_of_heap_commit) = if is_pe32_plus {
            (
                reader.read_u64::<LittleEndian>()?,
                reader.read_u64::<LittleEndian>()?,
                reader.read_u64::<LittleEndian>()?,
                reader.read_u64::<LittleEndian>()?,
            )
        }
        else {
            (
                reader.read_u32::<LittleEndian>()? as u64,
                reader.read_u32::<LittleEndian>()? as u64,
                reader.read_u32::<LittleEndian>()? as u64,
                reader.read_u32::<LittleEndian>()? as u64,
            )
        };

        let loader_flags = reader.read_u32::<LittleEndian>()?;
        let number_of_rva_and_sizes = reader.read_u32::<LittleEndian>()?;

        let mut data_directories = Vec::new();
        for _ in 0..number_of_rva_and_sizes {
            data_directories.push(DataDirectory::read(&mut reader)?);
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
