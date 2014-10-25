# PE 文件结构详解

本文档深入分析 PE (Portable Executable) 文件的内部结构，包括各个头部、节区和数据目录的详细格式。

## 文件结构概览

PE 文件采用分层结构设计，从文件开头到结尾依次包含：

```
┌─────────────────────┐ ← 文件偏移 0x0
│    DOS 头部         │
├─────────────────────┤
│    DOS 存根         │
├─────────────────────┤ ← dos_header.e_lfanew
│    NT 头部          │
│  ┌─────────────────┐│
│  │   PE 签名       ││
│  ├─────────────────┤│
│  │   COFF 头       ││
│  ├─────────────────┤│
│  │   可选头        ││
│  └─────────────────┘│
├─────────────────────┤
│    节区头表         │
├─────────────────────┤
│    节区数据         │
│  ┌─────────────────┐│
│  │    .text        ││
│  ├─────────────────┤│
│  │    .data        ││
│  ├─────────────────┤│
│  │    .rdata       ││
│  ├─────────────────┤│
│  │    .rsrc        ││
│  └─────────────────┘│
└─────────────────────┘
```

## DOS 头部 (DOS Header)

DOS 头部保持与 MS-DOS 的兼容性，位于文件开头：

```rust
use gaia_assembler::pe::DosHeader;

#[repr(C)]
pub struct DosHeader {
    pub e_magic: u16,      // 0x5A4D ("MZ")
    pub e_cblp: u16,       // 最后页的字节数
    pub e_cp: u16,         // 文件页数
    pub e_crlc: u16,       // 重定位项数
    pub e_cparhdr: u16,    // 头部段落数
    pub e_minalloc: u16,   // 最小额外段落
    pub e_maxalloc: u16,   // 最大额外段落
    pub e_ss: u16,         // 初始 SS 值
    pub e_sp: u16,         // 初始 SP 值
    pub e_csum: u16,       // 校验和
    pub e_ip: u16,         // 初始 IP 值
    pub e_cs: u16,         // 初始 CS 值
    pub e_lfarlc: u16,     // 重定位表偏移
    pub e_ovno: u16,       // 覆盖号
    pub e_res: [u16; 4],   // 保留字段
    pub e_oemid: u16,      // OEM 标识符
    pub e_oeminfo: u16,    // OEM 信息
    pub e_res2: [u16; 10], // 保留字段
    pub e_lfanew: u32,     // NT 头偏移 ★
}

impl DosHeader {
    pub fn new() -> Self {
        Self {
            e_magic: 0x5A4D,  // "MZ"
            e_cblp: 0x90,
            e_cp: 0x03,
            e_crlc: 0x00,
            e_cparhdr: 0x04,
            e_minalloc: 0x00,
            e_maxalloc: 0xFFFF,
            e_ss: 0x00,
            e_sp: 0xB8,
            e_csum: 0x00,
            e_ip: 0x00,
            e_cs: 0x00,
            e_lfarlc: 0x40,
            e_ovno: 0x00,
            e_res: [0; 4],
            e_oemid: 0x00,
            e_oeminfo: 0x00,
            e_res2: [0; 10],
            e_lfanew: 0x80,   // 通常指向 0x80
        }
    }
}
```

### DOS 存根 (DOS Stub)

DOS 存根是一个小的 16 位程序，在非 Windows 系统上运行时显示错误信息：

```rust
const DOS_STUB: &[u8] = &[
    0x0E, 0x1F, 0xBA, 0x0E, 0x00, 0xB4, 0x09, 0xCD,
    0x21, 0xB8, 0x01, 0x4C, 0xCD, 0x21, 0x54, 0x68,
    0x69, 0x73, 0x20, 0x70, 0x72, 0x6F, 0x67, 0x72,
    0x61, 0x6D, 0x20, 0x63, 0x61, 0x6E, 0x6E, 0x6F,
    0x74, 0x20, 0x62, 0x65, 0x20, 0x72, 0x75, 0x6E,
    0x20, 0x69, 0x6E, 0x20, 0x44, 0x4F, 0x53, 0x20,
    0x6D, 0x6F, 0x64, 0x65, 0x2E, 0x0D, 0x0D, 0x0A,
    0x24, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
// 输出: "This program cannot be run in DOS mode."
```

## NT 头部 (NT Headers)

NT 头部是 PE 格式的核心，包含三个主要部分：

### 1. PE 签名

```rust
const PE_SIGNATURE: u32 = 0x00004550; // "PE\0\0"
```

### 2. COFF 头部 (File Header)

```rust
#[repr(C)]
pub struct CoffHeader {
    pub machine: u16,              // 目标机器类型
    pub number_of_sections: u16,   // 节区数量
    pub time_date_stamp: u32,      // 时间戳
    pub pointer_to_symbol_table: u32, // 符号表偏移
    pub number_of_symbols: u32,    // 符号数量
    pub size_of_optional_header: u16, // 可选头大小
    pub characteristics: u16,      // 文件特性
}

// 机器类型常量
pub mod MachineType {
    pub const I386: u16 = 0x014c;   // Intel 386
    pub const AMD64: u16 = 0x8664;  // x64
    pub const ARM: u16 = 0x01c0;    // ARM
    pub const ARM64: u16 = 0xaa64;  // ARM64
}

// 文件特性标志
pub mod Characteristics {
    pub const RELOCS_STRIPPED: u16 = 0x0001;
    pub const EXECUTABLE_IMAGE: u16 = 0x0002;
    pub const LINE_NUMBERS_STRIPPED: u16 = 0x0004;
    pub const LOCAL_SYMS_STRIPPED: u16 = 0x0008;
    pub const AGGR_WS_TRIM: u16 = 0x0010;
    pub const LARGE_ADDRESS_AWARE: u16 = 0x0020;
    pub const BYTES_REVERSED_LO: u16 = 0x0080;
    pub const _32BIT_MACHINE: u16 = 0x0100;
    pub const DEBUG_STRIPPED: u16 = 0x0200;
    pub const REMOVABLE_RUN_FROM_SWAP: u16 = 0x0400;
    pub const NET_RUN_FROM_SWAP: u16 = 0x0800;
    pub const SYSTEM: u16 = 0x1000;
    pub const DLL: u16 = 0x2000;
    pub const UP_SYSTEM_ONLY: u16 = 0x4000;
    pub const BYTES_REVERSED_HI: u16 = 0x8000;
}
```

### 3. 可选头部 (Optional Header)

可选头部有 32 位和 64 位两种版本：

```rust
// 32位可选头
#[repr(C)]
pub struct OptionalHeader32 {
    // 标准字段
    pub magic: u16,                    // 0x010b (PE32)
    pub major_linker_version: u8,
    pub minor_linker_version: u8,
    pub size_of_code: u32,
    pub size_of_initialized_data: u32,
    pub size_of_uninitialized_data: u32,
    pub address_of_entry_point: u32,   // 入口点 RVA
    pub base_of_code: u32,
    pub base_of_data: u32,
    
    // NT 特定字段
    pub image_base: u32,               // 镜像基址
    pub section_alignment: u32,        // 节区对齐
    pub file_alignment: u32,           // 文件对齐
    pub major_operating_system_version: u16,
    pub minor_operating_system_version: u16,
    pub major_image_version: u16,
    pub minor_image_version: u16,
    pub major_subsystem_version: u16,
    pub minor_subsystem_version: u16,
    pub win32_version_value: u32,
    pub size_of_image: u32,            // 镜像大小
    pub size_of_headers: u32,          // 头部大小
    pub checksum: u32,                 // 校验和
    pub subsystem: u16,                // 子系统
    pub dll_characteristics: u16,      // DLL 特性
    pub size_of_stack_reserve: u32,
    pub size_of_stack_commit: u32,
    pub size_of_heap_reserve: u32,
    pub size_of_heap_commit: u32,
    pub loader_flags: u32,
    pub number_of_rva_and_sizes: u32,  // 数据目录项数
    pub data_directory: [DataDirectory; 16], // 数据目录
}

// 64位可选头
#[repr(C)]
pub struct OptionalHeader64 {
    // 标准字段
    pub magic: u16,                    // 0x020b (PE32+)
    pub major_linker_version: u8,
    pub minor_linker_version: u8,
    pub size_of_code: u32,
    pub size_of_initialized_data: u32,
    pub size_of_uninitialized_data: u32,
    pub address_of_entry_point: u32,
    pub base_of_code: u32,
    
    // NT 特定字段 (64位版本)
    pub image_base: u64,               // 64位镜像基址
    pub section_alignment: u32,
    pub file_alignment: u32,
    pub major_operating_system_version: u16,
    pub minor_operating_system_version: u16,
    pub major_image_version: u16,
    pub minor_image_version: u16,
    pub major_subsystem_version: u16,
    pub minor_subsystem_version: u16,
    pub win32_version_value: u32,
    pub size_of_image: u32,
    pub size_of_headers: u32,
    pub checksum: u32,
    pub subsystem: u16,
    pub dll_characteristics: u16,
    pub size_of_stack_reserve: u64,    // 64位栈大小
    pub size_of_stack_commit: u64,
    pub size_of_heap_reserve: u64,     // 64位堆大小
    pub size_of_heap_commit: u64,
    pub loader_flags: u32,
    pub number_of_rva_and_sizes: u32,
    pub data_directory: [DataDirectory; 16],
}
```

### 子系统类型

```rust
pub mod Subsystem {
    pub const UNKNOWN: u16 = 0;
    pub const NATIVE: u16 = 1;         // 设备驱动程序
    pub const WINDOWS_GUI: u16 = 2;    // Windows GUI 应用
    pub const WINDOWS_CUI: u16 = 3;    // 控制台应用
    pub const OS2_CUI: u16 = 5;        // OS/2 控制台
    pub const POSIX_CUI: u16 = 7;      // POSIX 控制台
    pub const NATIVE_WINDOWS: u16 = 8; // Win9x 驱动
    pub const WINDOWS_CE_GUI: u16 = 9; // Windows CE
    pub const EFI_APPLICATION: u16 = 10;
    pub const EFI_BOOT_SERVICE_DRIVER: u16 = 11;
    pub const EFI_RUNTIME_DRIVER: u16 = 12;
    pub const EFI_ROM: u16 = 13;
    pub const XBOX: u16 = 14;
    pub const WINDOWS_BOOT_APPLICATION: u16 = 16;
}
```

### DLL 特性标志

```rust
pub mod DllCharacteristics {
    pub const HIGH_ENTROPY_VA: u16 = 0x0020;        // 高熵 ASLR
    pub const DYNAMIC_BASE: u16 = 0x0040;           // ASLR 支持
    pub const FORCE_INTEGRITY: u16 = 0x0080;        // 强制完整性检查
    pub const NX_COMPAT: u16 = 0x0100;              // DEP 兼容
    pub const NO_ISOLATION: u16 = 0x0200;           // 不隔离
    pub const NO_SEH: u16 = 0x0400;                 // 无 SEH
    pub const NO_BIND: u16 = 0x0800;                // 不绑定
    pub const APPCONTAINER: u16 = 0x1000;           // AppContainer
    pub const WDM_DRIVER: u16 = 0x2000;             // WDM 驱动
    pub const GUARD_CF: u16 = 0x4000;               // 控制流保护
    pub const TERMINAL_SERVER_AWARE: u16 = 0x8000;  // 终端服务器感知
}
```

## 数据目录

数据目录指向 PE 文件中的特殊数据结构：

```rust
#[repr(C)]
pub struct DataDirectory {
    pub virtual_address: u32,  // RVA
    pub size: u32,            // 大小
}

// 数据目录索引
pub mod DataDirectoryIndex {
    pub const EXPORT_TABLE: usize = 0;
    pub const IMPORT_TABLE: usize = 1;
    pub const RESOURCE_TABLE: usize = 2;
    pub const EXCEPTION_TABLE: usize = 3;
    pub const CERTIFICATE_TABLE: usize = 4;
    pub const BASE_RELOCATION_TABLE: usize = 5;
    pub const DEBUG: usize = 6;
    pub const ARCHITECTURE: usize = 7;
    pub const GLOBAL_PTR: usize = 8;
    pub const TLS_TABLE: usize = 9;
    pub const LOAD_CONFIG_TABLE: usize = 10;
    pub const BOUND_IMPORT: usize = 11;
    pub const IAT: usize = 12;
    pub const DELAY_IMPORT_DESCRIPTOR: usize = 13;
    pub const COM_PLUS_RUNTIME_HEADER: usize = 14;
    pub const RESERVED: usize = 15;
}
```

## 节区头表 (Section Headers)

节区头表描述文件中每个节区的属性：

```rust
#[repr(C)]
pub struct SectionHeader {
    pub name: [u8; 8],                // 节区名称 (8字节)
    pub virtual_size: u32,            // 内存中的大小
    pub virtual_address: u32,         // 内存中的 RVA
    pub size_of_raw_data: u32,        // 文件中的大小
    pub pointer_to_raw_data: u32,     // 文件中的偏移
    pub pointer_to_relocations: u32,  // 重定位表偏移
    pub pointer_to_line_numbers: u32, // 行号表偏移
    pub number_of_relocations: u16,   // 重定位项数
    pub number_of_line_numbers: u16,  // 行号数
    pub characteristics: u32,         // 节区特性
}

// 节区特性标志
pub mod SectionCharacteristics {
    pub const TYPE_NO_PAD: u32 = 0x00000008;
    pub const CNT_CODE: u32 = 0x00000020;              // 代码
    pub const CNT_INITIALIZED_DATA: u32 = 0x00000040;  // 初始化数据
    pub const CNT_UNINITIALIZED_DATA: u32 = 0x00000080; // 未初始化数据
    pub const LNK_OTHER: u32 = 0x00000100;
    pub const LNK_INFO: u32 = 0x00000200;
    pub const LNK_REMOVE: u32 = 0x00000800;
    pub const LNK_COMDAT: u32 = 0x00001000;
    pub const NO_DEFER_SPEC_EXC: u32 = 0x00004000;
    pub const GPREL: u32 = 0x00008000;
    pub const MEM_FARDATA: u32 = 0x00008000;
    pub const MEM_PURGEABLE: u32 = 0x00020000;
    pub const MEM_16BIT: u32 = 0x00020000;
    pub const MEM_LOCKED: u32 = 0x00040000;
    pub const MEM_PRELOAD: u32 = 0x00080000;
    
    // 对齐标志
    pub const ALIGN_1BYTES: u32 = 0x00100000;
    pub const ALIGN_2BYTES: u32 = 0x00200000;
    pub const ALIGN_4BYTES: u32 = 0x00300000;
    pub const ALIGN_8BYTES: u32 = 0x00400000;
    pub const ALIGN_16BYTES: u32 = 0x00500000;
    pub const ALIGN_32BYTES: u32 = 0x00600000;
    pub const ALIGN_64BYTES: u32 = 0x00700000;
    pub const ALIGN_128BYTES: u32 = 0x00800000;
    pub const ALIGN_256BYTES: u32 = 0x00900000;
    pub const ALIGN_512BYTES: u32 = 0x00A00000;
    pub const ALIGN_1024BYTES: u32 = 0x00B00000;
    pub const ALIGN_2048BYTES: u32 = 0x00C00000;
    pub const ALIGN_4096BYTES: u32 = 0x00D00000;
    pub const ALIGN_8192BYTES: u32 = 0x00E00000;
    
    pub const LNK_NRELOC_OVFL: u32 = 0x01000000;
    pub const MEM_DISCARDABLE: u32 = 0x02000000;       // 可丢弃
    pub const MEM_NOT_CACHED: u32 = 0x04000000;        // 不缓存
    pub const MEM_NOT_PAGED: u32 = 0x08000000;         // 不分页
    pub const MEM_SHARED: u32 = 0x10000000;            // 共享
    pub const MEM_EXECUTE: u32 = 0x20000000;           // 可执行
    pub const MEM_READ: u32 = 0x40000000;              // 可读
    pub const MEM_WRITE: u32 = 0x80000000;             // 可写
}
```

## 常见节区类型

### .text 节区 (代码段)

```rust
fn create_text_section() -> SectionHeader {
    SectionHeader {
        name: *b".text\0\0\0",
        virtual_size: 0x1000,
        virtual_address: 0x1000,
        size_of_raw_data: 0x1000,
        pointer_to_raw_data: 0x400,
        pointer_to_relocations: 0,
        pointer_to_line_numbers: 0,
        number_of_relocations: 0,
        number_of_line_numbers: 0,
        characteristics: SectionCharacteristics::CNT_CODE |
                        SectionCharacteristics::MEM_EXECUTE |
                        SectionCharacteristics::MEM_READ,
    }
}
```

### .data 节区 (可写数据段)

```rust
fn create_data_section() -> SectionHeader {
    SectionHeader {
        name: *b".data\0\0\0",
        virtual_size: 0x1000,
        virtual_address: 0x2000,
        size_of_raw_data: 0x1000,
        pointer_to_raw_data: 0x1400,
        pointer_to_relocations: 0,
        pointer_to_line_numbers: 0,
        number_of_relocations: 0,
        number_of_line_numbers: 0,
        characteristics: SectionCharacteristics::CNT_INITIALIZED_DATA |
                        SectionCharacteristics::MEM_READ |
                        SectionCharacteristics::MEM_WRITE,
    }
}
```

### .rdata 节区 (只读数据段)

```rust
fn create_rdata_section() -> SectionHeader {
    SectionHeader {
        name: *b".rdata\0\0",
        virtual_size: 0x1000,
        virtual_address: 0x3000,
        size_of_raw_data: 0x1000,
        pointer_to_raw_data: 0x2400,
        pointer_to_relocations: 0,
        pointer_to_line_numbers: 0,
        number_of_relocations: 0,
        number_of_line_numbers: 0,
        characteristics: SectionCharacteristics::CNT_INITIALIZED_DATA |
                        SectionCharacteristics::MEM_READ,
    }
}
```

### .rsrc 节区 (资源段)

```rust
fn create_rsrc_section() -> SectionHeader {
    SectionHeader {
        name: *b".rsrc\0\0\0",
        virtual_size: 0x1000,
        virtual_address: 0x4000,
        size_of_raw_data: 0x1000,
        pointer_to_raw_data: 0x3400,
        pointer_to_relocations: 0,
        pointer_to_line_numbers: 0,
        number_of_relocations: 0,
        number_of_line_numbers: 0,
        characteristics: SectionCharacteristics::CNT_INITIALIZED_DATA |
                        SectionCharacteristics::MEM_READ,
    }
}
```

## PE 文件构建示例

```rust
use gaia_assembler::pe::{PeBuilder, WriterConfig};

fn build_simple_pe() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut builder = PeBuilder::new();
    
    // 1. 设置 DOS 头
    let dos_header = DosHeader::new();
    builder.set_dos_header(dos_header);
    
    // 2. 设置 NT 头
    let mut nt_headers = NtHeaders64::new();
    nt_headers.file_header.machine = MachineType::AMD64;
    nt_headers.file_header.number_of_sections = 2;
    nt_headers.file_header.characteristics = 
        Characteristics::EXECUTABLE_IMAGE |
        Characteristics::LARGE_ADDRESS_AWARE;
    
    nt_headers.optional_header.magic = 0x020b; // PE32+
    nt_headers.optional_header.address_of_entry_point = 0x1000;
    nt_headers.optional_header.image_base = 0x140000000;
    nt_headers.optional_header.section_alignment = 0x1000;
    nt_headers.optional_header.file_alignment = 0x200;
    nt_headers.optional_header.subsystem = Subsystem::WINDOWS_CUI;
    nt_headers.optional_header.dll_characteristics = 
        DllCharacteristics::DYNAMIC_BASE |
        DllCharacteristics::NX_COMPAT;
    
    builder.set_nt_headers(nt_headers);
    
    // 3. 添加节区
    let text_section = create_text_section();
    let data_section = create_data_section();
    
    builder.add_section(text_section, &generate_code());
    builder.add_section(data_section, &generate_data());
    
    // 4. 构建 PE 文件
    let config = WriterConfig::default();
    builder.build(config)
}

fn generate_code() -> Vec<u8> {
    vec![
        // 简单的退出程序代码
        0x48, 0x31, 0xC9,        // xor rcx, rcx
        0xFF, 0x15, 0x00, 0x00, 0x00, 0x00, // call [ExitProcess]
    ]
}

fn generate_data() -> Vec<u8> {
    b"Hello, World!\0".to_vec()
}
```

## 文件验证

### 头部验证

```rust
fn validate_pe_headers(pe_data: &[u8]) -> Result<(), String> {
    // 1. 检查 DOS 签名
    if pe_data.len() < 2 || &pe_data[0..2] != b"MZ" {
        return Err("Invalid DOS signature".to_string());
    }
    
    // 2. 获取 NT 头偏移
    if pe_data.len() < 64 {
        return Err("File too small".to_string());
    }
    let nt_offset = u32::from_le_bytes([
        pe_data[60], pe_data[61], pe_data[62], pe_data[63]
    ]) as usize;
    
    // 3. 检查 PE 签名
    if pe_data.len() < nt_offset + 4 {
        return Err("Invalid NT header offset".to_string());
    }
    if &pe_data[nt_offset..nt_offset + 4] != b"PE\0\0" {
        return Err("Invalid PE signature".to_string());
    }
    
    Ok(())
}
```

### 节区验证

```rust
fn validate_sections(sections: &[SectionHeader]) -> Result<(), String> {
    for (i, section) in sections.iter().enumerate() {
        // 检查节区名称
        if section.name[0] == 0 {
            return Err(format!("Section {} has empty name", i));
        }
        
        // 检查虚拟地址对齐
        if section.virtual_address % 0x1000 != 0 {
            return Err(format!("Section {} virtual address not aligned", i));
        }
        
        // 检查文件偏移对齐
        if section.pointer_to_raw_data % 0x200 != 0 {
            return Err(format!("Section {} file offset not aligned", i));
        }
        
        // 检查大小合理性
        if section.virtual_size == 0 && section.size_of_raw_data == 0 {
            return Err(format!("Section {} has zero size", i));
        }
    }
    
    Ok(())
}
```

## 性能优化

### 内存布局优化

```rust
fn optimize_section_layout(sections: &mut Vec<SectionHeader>) {
    // 按访问模式排序节区
    sections.sort_by(|a, b| {
        let a_priority = get_section_priority(a);
        let b_priority = get_section_priority(b);
        a_priority.cmp(&b_priority)
    });
}

fn get_section_priority(section: &SectionHeader) -> u32 {
    if section.characteristics & SectionCharacteristics::CNT_CODE != 0 {
        1 // 代码段优先级最高
    } else if section.characteristics & SectionCharacteristics::CNT_INITIALIZED_DATA != 0 {
        2 // 初始化数据次之
    } else {
        3 // 其他段最低
    }
}
```

### 文件大小优化

```rust
fn optimize_file_size(builder: &mut PeBuilder) {
    // 合并相似特性的节区
    builder.merge_compatible_sections();
    
    // 移除未使用的数据目录项
    builder.remove_empty_data_directories();
    
    // 优化对齐设置
    builder.set_minimal_alignment();
}
```

## 下一步

现在您已经了解了 PE 文件的详细结构，可以继续学习：

1. **[代码生成](./code-generation.md)** - x86/x64 机器码生成技术
2. **[内存管理](./memory-management.md)** - 虚拟内存和地址空间管理
3. **[导入导出](./import-export.md)** - DLL 导入导出机制详解
4. **[重定位处理](./relocations.md)** - 基址重定位和地址修正

## 参考工具

- **dumpbin**: Visual Studio 自带的 PE 分析工具
- **PE Explorer**: 图形化 PE 文件查看器
- **CFF Explorer**: 免费的 PE 编辑器
- **HxD**: 十六进制编辑器，用于查看原始字节

---

*本文档详细介绍了 PE 文件的结构。如需了解具体的实现细节，请参考相关的代码示例和工具文档。*