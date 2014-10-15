use gaia_types::helpers::Architecture;

/// PE 子系统类型枚举
///
/// 定义了 Windows PE 文件可以使用的各种子系统类型，
/// 这些类型决定了程序运行时的环境和依赖。
#[derive(Debug, Clone, Copy)]
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

/// DOS 头结构
///
/// 包含 DOS 可执行文件的基本信息，是 PE 文件的第一个结构。
/// 虽然现代 Windows 程序不运行在 DOS 模式下，但 PE 格式仍保留这个结构用于兼容性。
#[derive(Debug, Clone)]
pub struct DosHeader {
    /// DOS 签名，通常为 0x5A4D（"MZ"）
    pub e_magic: u16,
    /// 文件最后页的字节数
    pub e_cblp: u16,
    /// 文件的总页数
    pub e_cp: u16,
    /// 重定位项的数量
    pub e_crlc: u16,
    /// 段落中头部的大小
    pub e_cparhdr: u16,
    /// 所需的最小段落数
    pub e_minalloc: u16,
    /// 所需的最大段落数
    pub e_maxalloc: u16,
    /// 初始的 SS 寄存器值
    pub e_ss: u16,
    /// 初始的 SP 寄存器值
    pub e_sp: u16,
    /// 校验和
    pub e_csum: u16,
    /// 初始的 IP 寄存器值
    pub e_ip: u16,
    /// 初始的 CS 寄存器值
    pub e_cs: u16,
    /// 重定位表的文件偏移
    pub e_lfarlc: u16,
    /// 覆盖号
    pub e_ovno: u16,
    /// 保留字段，通常为 0
    pub e_res: [u16; 4],
    /// OEM 标识符
    pub e_oemid: u16,
    /// OEM 信息
    pub e_oeminfo: u16,
    /// 保留字段，通常为 0
    pub e_res2: [u16; 10],
    /// PE 头的文件偏移，指向真正的 PE 结构
    pub e_lfanew: u32,
}

/// NT 头结构
///
/// 包含 PE 文件的主要签名和基本信息，标识这是一个有效的 PE 文件。
/// signature 字段必须为 0x00004550（"PE\0\0"）。
#[derive(Debug, Clone)]
pub struct NtHeader {
    /// PE 签名，必须为 0x00004550（"PE\0\0"）
    pub signature: u32,
}

/// COFF 头结构
///
/// 包含 COFF（Common Object File Format）格式的基本信息，
/// 定义了目标机器类型、节的数量和时间戳等关键信息。
#[derive(Debug, Clone)]
pub struct CoffHeader {
    /// 目标机器类型，如 x86、x64、ARM 等
    pub machine: u16,
    /// 节的数量，表示文件中包含多少个节
    pub number_of_sections: u16,
    /// 时间戳，表示文件创建或链接的时间
    pub time_date_stamp: u32,
    /// 符号表的文件偏移，如果没有符号表则为 0
    pub pointer_to_symbol_table: u32,
    /// 符号表中的符号数量
    pub number_of_symbols: u32,
    /// 可选头的大小（以字节为单位）
    pub size_of_optional_header: u16,
    /// 文件特征标志，描述文件的各种属性
    pub characteristics: u16,
}

/// 可选头结构
///
/// 包含 PE 文件的加载和运行时信息，如入口点地址、内存布局、
/// 版本信息等。这个结构对于 Windows 加载器正确加载和执行程序至关重要。
#[derive(Debug, Clone)]
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
}

/// PE 头结构
///
/// 包含 PE 文件的所有头部信息，是 DOS 头、NT 头、COFF 头和可选头的组合。
/// 这个结构提供了完整的 PE 文件元数据信息。
#[derive(Debug, Clone)]
pub struct PeHeader {
    /// DOS 头，包含 DOS 兼容性信息
    pub dos_header: DosHeader,
    /// NT 头，包含 PE 签名
    pub nt_header: NtHeader,
    /// COFF 头，包含机器类型和节信息
    pub coff_header: CoffHeader,
    /// 可选头，包含加载和运行时信息
    pub optional_header: OptionalHeader,
}

/// PE 节结构
///
/// 表示 PE 文件中的一个节（Section），包含节的元数据和实际数据。
/// 常见的节包括 .text（代码）、.data（数据）、.rdata（只读数据）等。
#[derive(Debug, Clone)]
pub struct PeSection {
    /// 节名称，如 ".text"、".data" 等
    pub name: String,
    /// 节在内存中的虚拟大小
    pub virtual_size: u32,
    /// 节在内存中的虚拟地址（RVA）
    pub virtual_address: u32,
    /// 节在文件中的原始数据大小
    pub size_of_raw_data: u32,
    /// 节在文件中的偏移地址
    pub pointer_to_raw_data: u32,
    /// 重定位表在文件中的偏移地址
    pub pointer_to_relocations: u32,
    /// 行号表在文件中的偏移地址
    pub pointer_to_line_numbers: u32,
    /// 重定位项的数量
    pub number_of_relocations: u16,
    /// 行号项的数量
    pub number_of_line_numbers: u16,
    /// 节的特征标志，描述节的属性（可读、可写、可执行等）
    pub characteristics: u32,
    /// 节的实际数据内容
    pub data: Vec<u8>,
}

/// PE 文件读取配置
///
/// 控制 PE 文件解析过程的行为，允许选择性解析不同的部分。
/// 通过调整这些配置，可以在性能和功能之间取得平衡。
#[derive(Debug, Clone)]
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
        Self { 
            include_sections: true, 
            validate_checksum: false, 
            parse_imports: true, 
            parse_exports: true 
        }
    }
}

/// PE 程序结构
///
/// 表示一个完整的 PE（Portable Executable）程序，包含所有头部信息和节数据。
/// 这是 PE 文件的最高级别抽象，包含了文件解析后的完整内容。
#[derive(Debug, Clone)]
pub struct PeProgram {
    /// PE 头部信息，包含 DOS 头、NT 头、COFF 头和可选头
    pub header: PeHeader,
    /// 所有节的集合，包含代码、数据、资源等
    pub sections: Vec<PeSection>,
}

/// PE 信息结构
///
/// 提供 PE 文件的摘要信息，包含关键属性和统计信息。
/// 这个结构用于快速获取文件的基本信息，而无需解析完整的头部结构。
#[derive(Debug, Clone)]
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
