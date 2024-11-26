use byteorder::{LittleEndian, ReadBytesExt};
use gaia_types::{helpers::Architecture, GaiaError};
use serde::{Deserialize, Serialize};
use std::io::Read;

/// COFF 文件头结构
///
/// 包含 COFF（Common Object File Format）格式的基本信息，
/// 定义了目标机器类型、节的数量和时间戳等关键信息。
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
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

impl CoffHeader {
    /// 创建一个新的 COFF 头，只需要最核心的字段
    pub fn new(machine: u16, number_of_sections: u16) -> Self {
        CoffHeader {
            machine,
            number_of_sections,
            time_date_stamp: 0,
            pointer_to_symbol_table: 0,
            number_of_symbols: 0,
            size_of_optional_header: 0,
            characteristics: 0,
        }
    }

    /// 设置时间戳
    pub fn with_timestamp(mut self, time_date_stamp: u32) -> Self {
        self.time_date_stamp = time_date_stamp;
        self
    }

    /// 设置符号表信息
    pub fn with_symbol_table(mut self, pointer_to_symbol_table: u32, number_of_symbols: u32) -> Self {
        self.pointer_to_symbol_table = pointer_to_symbol_table;
        self.number_of_symbols = number_of_symbols;
        self
    }

    /// 设置可选头大小
    pub fn with_optional_header_size(mut self, size_of_optional_header: u16) -> Self {
        self.size_of_optional_header = size_of_optional_header;
        self
    }

    /// 设置文件特征
    pub fn with_characteristics(mut self, characteristics: u16) -> Self {
        self.characteristics = characteristics;
        self
    }

    pub fn read<R: Read>(mut reader: R) -> Result<Self, GaiaError> {
        Ok(CoffHeader {
            machine: reader.read_u16::<LittleEndian>()?,
            number_of_sections: reader.read_u16::<LittleEndian>()?,
            time_date_stamp: reader.read_u32::<LittleEndian>()?,
            pointer_to_symbol_table: reader.read_u32::<LittleEndian>()?,
            number_of_symbols: reader.read_u32::<LittleEndian>()?,
            size_of_optional_header: reader.read_u16::<LittleEndian>()?,
            characteristics: reader.read_u16::<LittleEndian>()?,
        })
    }

    pub fn get_architecture(&self) -> Architecture {
        match self.machine {
            0x014C => Architecture::X86,
            0x8664 => Architecture::X86_64,
            0x0200 => Architecture::ARM32,
            0xAA64 => Architecture::ARM64,
            _ => Architecture::Unknown,
        }
    }
}

/// 节头结构
///
/// 包含 COFF 文件中一个节（Section）的元数据信息，如名称、大小、
/// 位置和属性等。这个结构不包含节的实际数据，只包含描述信息。
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct SectionHeader {
    /// 节名称，8字节的ASCII字符串，如 ".text"、".data" 等
    pub name: [u8; 8],
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
}

impl SectionHeader {
    pub fn get_name(&self) -> &str {
        unsafe {
            let name = str::from_utf8_unchecked(&self.name);
            name.trim_end_matches('\0')
        }
    }
}

/// COFF 符号表项
///
/// 表示 COFF 对象文件中的一个符号，包含符号名称、值、节号等信息。
/// 符号可以是函数、变量、标签等程序中的标识符。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CoffSymbol {
    /// 符号名称，如果长度超过8字节则存储在字符串表中
    pub name: String,
    /// 符号的值，通常是地址或偏移量
    pub value: u32,
    /// 符号所在的节号，0表示未定义，-1表示绝对符号，-2表示调试符号
    pub section_number: i16,
    /// 符号类型，描述符号的基本类型
    pub symbol_type: u16,
    /// 存储类别，描述符号的作用域和生命周期
    pub storage_class: u8,
    /// 辅助符号的数量
    pub number_of_aux_symbols: u8,
}

/// COFF 重定位项
///
/// 表示需要在链接时进行地址重定位的项目。
/// 重定位是将相对地址转换为绝对地址的过程。
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct CoffRelocation {
    /// 需要重定位的虚拟地址
    pub virtual_address: u32,
    /// 符号表索引，指向相关的符号
    pub symbol_table_index: u32,
    /// 重定位类型，定义如何进行重定位
    pub relocation_type: u16,
}

/// COFF 节结构
///
/// 表示 COFF 对象文件中的一个节，包含节头和数据。
/// 与 PE 节类似，但用于对象文件而非可执行文件。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CoffSection {
    /// 节头信息
    pub header: SectionHeader,
    /// 节的原始数据
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub data: Vec<u8>,
    /// 重定位表，包含该节的所有重定位项
    pub relocations: Vec<CoffRelocation>,
}

/// COFF 对象文件结构
///
/// 表示一个完整的 COFF 对象文件，包含头部、节、符号表等信息。
/// COFF 对象文件是编译器生成的中间文件，包含未链接的代码和数据。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CoffObject {
    /// COFF 头部信息
    pub header: CoffHeader,
    /// 所有节的集合
    pub sections: Vec<CoffSection>,
    /// 符号表，包含所有符号信息
    pub symbols: Vec<CoffSymbol>,
    /// 字符串表，存储长符号名称
    pub string_table: Vec<u8>,
}

/// 库文件成员头
///
/// 表示静态库文件中一个成员文件的头部信息。
/// 静态库是多个对象文件的集合，每个成员都有自己的头部。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArchiveMemberHeader {
    /// 成员文件名称
    pub name: String,
    /// 文件修改时间戳
    pub timestamp: u32,
    /// 用户ID
    pub user_id: u16,
    /// 组ID
    pub group_id: u16,
    /// 文件权限模式
    pub mode: u32,
    /// 文件大小
    pub size: u32,
}

/// 库文件成员
///
/// 表示静态库文件中的一个成员，包含头部和数据。
/// 每个成员通常是一个 COFF 对象文件。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArchiveMember {
    /// 成员头部信息
    pub header: ArchiveMemberHeader,
    /// 成员数据，通常是 COFF 对象文件的内容
    pub data: Vec<u8>,
    /// 解析后的 COFF 对象（如果成功解析）
    pub coff_object: Option<CoffObject>,
}

/// 静态库文件结构
///
/// 表示一个完整的静态库文件（.lib 文件），包含多个对象文件。
/// 静态库用于将多个对象文件打包成一个文件，便于分发和链接。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StaticLibrary {
    /// 库文件签名，通常为 "!<arch>\n"
    pub signature: String,
    /// 所有成员文件的集合
    pub members: Vec<ArchiveMember>,
    /// 符号索引表，用于快速查找符号
    pub symbol_index: Vec<(String, usize)>, // (symbol_name, member_index)
}

/// COFF 文件类型枚举
///
/// 区分不同类型的 COFF 相关文件格式。
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum CoffFileType {
    /// COFF 对象文件 (.obj)
    Object,
    /// 静态库文件 (.lib)
    StaticLibrary,
    /// PE 可执行文件 (.exe)
    Executable,
    /// PE 动态库文件 (.dll)
    DynamicLibrary,
}

/// COFF 文件信息
///
/// 提供 COFF 相关文件的摘要信息。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CoffInfo {
    /// 文件类型
    pub file_type: CoffFileType,
    /// 目标架构
    pub target_arch: Architecture,
    /// 节的数量
    pub section_count: u16,
    /// 符号的数量
    pub symbol_count: u32,
    /// 文件大小
    pub file_size: u64,
    /// 时间戳
    pub timestamp: u32,
}
