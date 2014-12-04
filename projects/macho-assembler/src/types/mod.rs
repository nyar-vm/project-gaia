#![doc = include_str!("readme.md")]

use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use gaia_types::helpers::Architecture;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Display, Formatter},
    io::Read,
};

use gaia_types::GaiaError;

/// Mach-O 文件类型枚举
///
/// 定义了 Mach-O 文件的不同类型，决定了文件的用途和加载方式。
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum MachoType {
    /// 目标文件
    Object,
    /// 可执行文件
    Execute,
    /// 固定虚拟内存共享库
    FvmLib,
    /// 核心转储文件
    Core,
    /// 预加载的可执行文件
    PreLoad,
    /// 动态共享库
    Dylib,
    /// 动态链接器
    Dylinker,
    /// 动态加载的包
    Bundle,
    /// 动态共享库存根
    DylibStub,
    /// 配套的调试符号文件
    Dsym,
    /// x86_64 kext 包
    KextBundle,
}

impl Display for MachoType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MachoType::Object => write!(f, "目标文件"),
            MachoType::Execute => write!(f, "可执行文件"),
            MachoType::FvmLib => write!(f, "固定虚拟内存共享库"),
            MachoType::Core => write!(f, "核心转储文件"),
            MachoType::PreLoad => write!(f, "预加载的可执行文件"),
            MachoType::Dylib => write!(f, "动态共享库"),
            MachoType::Dylinker => write!(f, "动态链接器"),
            MachoType::Bundle => write!(f, "动态加载的包"),
            MachoType::DylibStub => write!(f, "动态共享库存根"),
            MachoType::Dsym => write!(f, "配套的调试符号文件"),
            MachoType::KextBundle => write!(f, "x86_64 kext 包"),
        }
    }
}

impl From<u32> for MachoType {
    fn from(value: u32) -> Self {
        match value {
            1 => MachoType::Object,
            2 => MachoType::Execute,
            3 => MachoType::FvmLib,
            4 => MachoType::Core,
            5 => MachoType::PreLoad,
            6 => MachoType::Dylib,
            7 => MachoType::Dylinker,
            8 => MachoType::Bundle,
            9 => MachoType::DylibStub,
            10 => MachoType::Dsym,
            11 => MachoType::KextBundle,
            _ => MachoType::Object,
        }
    }
}

/// Mach-O CPU 类型枚举
///
/// 定义了 Mach-O 文件支持的处理器架构类型。
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum CpuType {
    /// x86_64 架构
    X86_64,
    /// ARM64 架构
    Arm64,
    /// ARM64e 架构（带指针认证）
    Arm64e,
    /// i386 架构
    I386,
    /// ARM 架构
    Arm,
}

impl From<u32> for CpuType {
    fn from(value: u32) -> Self {
        match value {
            0x01000007 => CpuType::X86_64,
            0x0100000c => CpuType::Arm64,
            0x0200000c => CpuType::Arm64e,
            0x00000007 => CpuType::I386,
            0x0000000c => CpuType::Arm,
            _ => CpuType::X86_64,
        }
    }
}

/// Mach-O 文件头结构
///
/// 包含了 Mach-O 文件的基本信息，如魔数、CPU 类型、文件类型等。
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct MachoHeader {
    /// 魔数，用于标识文件格式
    pub magic: u32,
    /// CPU 类型
    pub cpu_type: u32,
    /// CPU 子类型
    pub cpu_subtype: u32,
    /// 文件类型
    pub file_type: u32,
    /// 加载命令数量
    pub ncmds: u32,
    /// 加载命令总大小
    pub sizeofcmds: u32,
    /// 标志位
    pub flags: u32,
    /// 保留字段（仅在 64 位格式中存在）
    pub reserved: Option<u32>,
}

impl MachoHeader {
    /// 创建一个新的 Mach-O 文件头
    pub fn new(cpu_type: CpuType, file_type: MachoType) -> Self {
        let (magic, reserved) = match cpu_type {
            CpuType::X86_64 | CpuType::Arm64 | CpuType::Arm64e => (0xfeedfacf, Some(0)),
            _ => (0xfeedface, None),
        };

        Self {
            magic,
            cpu_type: cpu_type as u32,
            cpu_subtype: 0,
            file_type: file_type as u32,
            ncmds: 0,
            sizeofcmds: 0,
            flags: 0,
            reserved,
        }
    }

    /// 检查是否为 64 位格式
    pub fn is_64bit(&self) -> bool {
        self.magic == 0xfeedfacf || self.magic == 0xcffaedfe
    }

    /// 检查字节序
    pub fn is_little_endian(&self) -> bool {
        self.magic == 0xfeedfacf || self.magic == 0xfeedface
    }
}

/// 加载命令类型枚举
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum LoadCommandType {
    /// 段加载命令
    Segment,
    /// 64位段加载命令
    Segment64,
    /// 符号表
    Symtab,
    /// 动态符号表
    Dysymtab,
    /// 加载动态库
    LoadDylib,
    /// ID 动态库
    IdDylib,
    /// 加载动态链接器
    LoadDylinker,
    /// ID 动态链接器
    IdDylinker,
    /// 预绑定动态库
    PreboundDylib,
    /// 线程
    Thread,
    /// Unix 线程
    UnixThread,
    /// 加载弱动态库
    LoadWeakDylib,
    /// UUID
    Uuid,
    /// 代码签名
    CodeSignature,
    /// 段分割信息
    SegmentSplitInfo,
    /// 重新导出动态库
    ReexportDylib,
    /// 延迟加载动态库
    LazyLoadDylib,
    /// 加密信息
    EncryptionInfo,
    /// 动态库代码签名目录
    DylibCodeSignDrs,
    /// 版本最小 macOS
    VersionMinMacosx,
    /// 版本最小 iOS
    VersionMinIphoneos,
    /// 函数开始
    FunctionStarts,
    /// 动态库环境
    DyldEnvironment,
    /// 主程序
    Main,
    /// 数据在代码中
    DataInCode,
    /// 源版本
    SourceVersion,
    /// 动态库代码签名目录
    DylibCodeSignDrs2,
    /// 加密信息 64
    EncryptionInfo64,
    /// 链接器选项
    LinkerOption,
    /// 链接器优化提示
    LinkerOptimizationHint,
    /// 版本最小 tvOS
    VersionMinTvos,
    /// 版本最小 watchOS
    VersionMinWatchos,
    /// 注释
    Note,
    /// 构建版本
    BuildVersion,
    /// 动态库导出 trie
    DyldExportsTrie,
    /// 动态库链式修复
    DyldChainedFixups,
    /// 文件集入口
    FilesetEntry,
}

impl From<u32> for LoadCommandType {
    fn from(value: u32) -> Self {
        match value {
            0x1 => LoadCommandType::Segment,
            0x19 => LoadCommandType::Segment64,
            0x2 => LoadCommandType::Symtab,
            0xb => LoadCommandType::Dysymtab,
            0xc => LoadCommandType::LoadDylib,
            0xd => LoadCommandType::IdDylib,
            0xe => LoadCommandType::LoadDylinker,
            0xf => LoadCommandType::IdDylinker,
            0x10 => LoadCommandType::PreboundDylib,
            0x4 => LoadCommandType::Thread,
            0x5 => LoadCommandType::UnixThread,
            0x18 => LoadCommandType::LoadWeakDylib,
            0x1b => LoadCommandType::Uuid,
            0x1d => LoadCommandType::CodeSignature,
            0x1e => LoadCommandType::SegmentSplitInfo,
            0x1f => LoadCommandType::ReexportDylib,
            0x20 => LoadCommandType::LazyLoadDylib,
            0x21 => LoadCommandType::EncryptionInfo,
            0x22 => LoadCommandType::DylibCodeSignDrs,
            0x24 => LoadCommandType::VersionMinMacosx,
            0x25 => LoadCommandType::VersionMinIphoneos,
            0x26 => LoadCommandType::FunctionStarts,
            0x27 => LoadCommandType::DyldEnvironment,
            0x28 => LoadCommandType::Main,
            0x29 => LoadCommandType::DataInCode,
            0x2a => LoadCommandType::SourceVersion,
            0x2b => LoadCommandType::DylibCodeSignDrs2,
            0x2c => LoadCommandType::EncryptionInfo64,
            0x2d => LoadCommandType::LinkerOption,
            0x2e => LoadCommandType::LinkerOptimizationHint,
            0x2f => LoadCommandType::VersionMinTvos,
            0x30 => LoadCommandType::VersionMinWatchos,
            0x31 => LoadCommandType::Note,
            0x32 => LoadCommandType::BuildVersion,
            0x33 => LoadCommandType::DyldExportsTrie,
            0x34 => LoadCommandType::DyldChainedFixups,
            0x35 => LoadCommandType::FilesetEntry,
            _ => LoadCommandType::Segment,
        }
    }
}

/// 加载命令基础结构
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoadCommand {
    /// 命令类型
    pub cmd: u32,
    /// 命令大小
    pub cmdsize: u32,
    /// 命令数据
    pub data: Vec<u8>,
}

/// 段命令结构（64位）
#[derive(Clone, Debug, Serialize, Deserialize, Copy)]
pub struct SegmentCommand64 {
    /// 命令类型
    pub cmd: u32,
    /// 命令大小
    pub cmdsize: u32,
    /// 段名称
    pub segname: [u8; 16],
    /// 虚拟内存地址
    pub vmaddr: u64,
    /// 虚拟内存大小
    pub vmsize: u64,
    /// 文件偏移
    pub fileoff: u64,
    /// 文件大小
    pub filesize: u64,
    /// 最大虚拟内存保护
    pub maxprot: u32,
    /// 初始虚拟内存保护
    pub initprot: u32,
    /// 节数量
    pub nsects: u32,
    /// 标志
    pub flags: u32,
}

/// 节结构（64位）
#[derive(Clone, Debug, Serialize, Deserialize, Copy)]
pub struct Section64 {
    /// 节名称
    pub sectname: [u8; 16],
    /// 段名称
    pub segname: [u8; 16],
    /// 虚拟内存地址
    pub addr: u64,
    /// 大小
    pub size: u64,
    /// 文件偏移
    pub offset: u32,
    /// 对齐
    pub align: u32,
    /// 重定位偏移
    pub reloff: u32,
    /// 重定位数量
    pub nreloc: u32,
    /// 标志
    pub flags: u32,
    /// 保留字段1
    pub reserved1: u32,
    /// 保留字段2
    pub reserved2: u32,
    /// 保留字段3
    pub reserved3: u32,
}

/// Mach-O 读取配置
#[derive(Debug, Clone, Copy)]
pub struct MachoReadConfig {
    /// 是否包含节数据
    pub include_sections: bool,
    /// 是否解析符号表
    pub parse_symbols: bool,
    /// 是否解析动态库
    pub parse_dylibs: bool,
}

impl Default for MachoReadConfig {
    fn default() -> Self {
        Self { include_sections: true, parse_symbols: true, parse_dylibs: true }
    }
}

/// Mach-O 程序结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachoProgram {
    /// 文件头
    pub header: MachoHeader,
    /// 加载命令列表
    pub load_commands: Vec<LoadCommand>,
    /// 段列表
    pub segments: Vec<SegmentCommand64>,
    /// 节列表
    pub sections: Vec<Section64>,
}

impl MachoProgram {
    /// 创建一个新的 Mach-O 程序
    pub fn new(cpu_type: CpuType, file_type: MachoType) -> Self {
        Self {
            header: MachoHeader::new(cpu_type, file_type),
            load_commands: Vec::new(),
            segments: Vec::new(),
            sections: Vec::new(),
        }
    }
}

/// Mach-O 文件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachoInfo {
    /// 目标架构
    pub target_arch: Architecture,
    /// 文件类型
    pub file_type: MachoType,
    /// CPU 类型
    pub cpu_type: CpuType,
    /// 加载命令数量
    pub ncmds: u32,
    /// 段数量
    pub nsegments: u32,
    /// 节数量
    pub nsections: u32,
    /// 文件大小
    pub file_size: u64,
}

impl MachoHeader {
    /// 从字节流读取 Mach-O 文件头
    pub fn read<R: Read>(mut reader: R) -> Result<Self, GaiaError> {
        let magic = reader.read_u32::<LittleEndian>()?;
        
        let is_64bit = magic == 0xfeedfacf || magic == 0xcffaedfe;
        let is_little_endian = magic == 0xfeedfacf || magic == 0xfeedface;

        let cpu_type = if is_little_endian {
            reader.read_u32::<LittleEndian>()?
        } else {
            reader.read_u32::<BigEndian>()?
        };

        let cpu_subtype = if is_little_endian {
            reader.read_u32::<LittleEndian>()?
        } else {
            reader.read_u32::<BigEndian>()?
        };

        let file_type = if is_little_endian {
            reader.read_u32::<LittleEndian>()?
        } else {
            reader.read_u32::<BigEndian>()?
        };

        let ncmds = if is_little_endian {
            reader.read_u32::<LittleEndian>()?
        } else {
            reader.read_u32::<BigEndian>()?
        };

        let sizeofcmds = if is_little_endian {
            reader.read_u32::<LittleEndian>()?
        } else {
            reader.read_u32::<BigEndian>()?
        };

        let flags = if is_little_endian {
            reader.read_u32::<LittleEndian>()?
        } else {
            reader.read_u32::<BigEndian>()?
        };

        let reserved = if is_64bit {
            Some(if is_little_endian {
                reader.read_u32::<LittleEndian>()?
            } else {
                reader.read_u32::<BigEndian>()?
            })
        } else {
            None
        };

        Ok(Self {
            magic,
            cpu_type,
            cpu_subtype,
            file_type,
            ncmds,
            sizeofcmds,
            flags,
            reserved,
        })
    }
}