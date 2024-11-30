# ELF 文件结构

本文档详细介绍 ELF 文件的内部结构、各个组成部分的作用以及如何使用 Gaia 汇编器构建完整的 ELF 文件。

## 文件结构概览

ELF 文件由以下主要部分组成：

```
┌─────────────────┐
│   ELF Header    │  ← 文件开头，包含基本信息
├─────────────────┤
│ Program Headers │  ← 描述段 (Segments)，用于加载
├─────────────────┤
│                 │
│   Segment 1     │  ← 实际的程序数据
│                 │
├─────────────────┤
│                 │
│   Segment 2     │
│                 │
├─────────────────┤
│      ...        │
├─────────────────┤
│ Section Headers │  ← 描述节区 (Sections)，用于链接
└─────────────────┘
```

## ELF 头 (ELF Header)

ELF 头位于文件开头，包含文件的基本信息：

### 结构定义

```rust
use gaia_assembler::elf::{ElfHeader, ElfClass, ElfData, ElfVersion, FileType, Machine};

#[repr(C)]
pub struct ElfHeader64 {
    pub e_ident: [u8; 16],      // ELF 标识
    pub e_type: u16,            // 文件类型
    pub e_machine: u16,         // 目标架构
    pub e_version: u32,         // ELF 版本
    pub e_entry: u64,           // 程序入口点
    pub e_phoff: u64,           // 程序头表偏移
    pub e_shoff: u64,           // 节区头表偏移
    pub e_flags: u32,           // 处理器特定标志
    pub e_ehsize: u16,          // ELF 头大小
    pub e_phentsize: u16,       // 程序头表项大小
    pub e_phnum: u16,           // 程序头表项数量
    pub e_shentsize: u16,       // 节区头表项大小
    pub e_shnum: u16,           // 节区头表项数量
    pub e_shstrndx: u16,        // 字符串表索引
}
```

### 创建 ELF 头

```rust
use gaia_assembler::elf::{ElfAssembler, ElfHeaderBuilder};

let mut assembler = ElfAssembler::new_executable();

// 设置基本信息
assembler.set_class(ElfClass::Class64);
assembler.set_data_encoding(ElfData::Data2LSB);
assembler.set_version(ElfVersion::Current);
assembler.set_file_type(FileType::Executable);
assembler.set_machine(Machine::X86_64);
assembler.set_entry_point(0x401000);

// 设置处理器特定标志
assembler.set_flags(0);  // x86-64 通常为 0
```

### ELF 标识 (e_ident)

```rust
// ELF 魔数和基本信息
const ELF_MAGIC: [u8; 4] = [0x7f, b'E', b'L', b'F'];

fn create_elf_ident() -> [u8; 16] {
    let mut ident = [0u8; 16];
    
    // 魔数
    ident[0..4].copy_from_slice(&ELF_MAGIC);
    
    // 文件类别 (32位或64位)
    ident[4] = 2;  // ELFCLASS64
    
    // 数据编码 (大端或小端)
    ident[5] = 1;  // ELFDATA2LSB (小端)
    
    // ELF 版本
    ident[6] = 1;  // EV_CURRENT
    
    // OS/ABI 标识
    ident[7] = 0;  // ELFOSABI_SYSV (System V)
    
    // ABI 版本
    ident[8] = 0;
    
    // 填充字节 (保留)
    // ident[9..16] 保持为 0
    
    ident
}
```

## 程序头表 (Program Header Table)

程序头表描述了文件中的段 (Segments)，告诉加载器如何将文件映射到内存：

### 程序头结构

```rust
use gaia_assembler::elf::{ProgramHeader, SegmentType, SegmentFlags};

#[repr(C)]
pub struct ProgramHeader64 {
    pub p_type: u32,        // 段类型
    pub p_flags: u32,       // 段标志
    pub p_offset: u64,      // 文件偏移
    pub p_vaddr: u64,       // 虚拟地址
    pub p_paddr: u64,       // 物理地址
    pub p_filesz: u64,      // 文件中的大小
    pub p_memsz: u64,       // 内存中的大小
    pub p_align: u64,       // 对齐
}
```

### 段类型

```rust
// 常见段类型
pub enum SegmentType {
    Null = 0,           // 未使用的条目
    Load = 1,           // 可加载段
    Dynamic = 2,        // 动态链接信息
    Interp = 3,         // 解释器路径
    Note = 4,           // 辅助信息
    ShLib = 5,          // 保留
    Phdr = 6,           // 程序头表本身
    Tls = 7,            // 线程本地存储
    GnuEhFrame = 0x6474e550,  // GNU 异常处理框架
    GnuStack = 0x6474e551,    // GNU 栈权限
    GnuRelRo = 0x6474e552,    // GNU 只读重定位
}
```

### 创建程序段

```rust
use gaia_assembler::elf::{SegmentBuilder, PAGE_SIZE};

// 创建代码段
let text_segment = SegmentBuilder::new()
    .segment_type(SegmentType::Load)
    .flags(SegmentFlags::READ() | SegmentFlags::execute())
    .virtual_address(0x401000)
    .physical_address(0x401000)
    .file_offset(0x1000)
    .file_size(0x1000)
    .memory_size(0x1000)
    .alignment(PAGE_SIZE)
    .build();

assembler.add_program_header(text_segment);

// 创建数据段
let data_segment = SegmentBuilder::new()
    .segment_type(SegmentType::Load)
    .flags(SegmentFlags::read() | SegmentFlags::write())
    .virtual_address(0x402000)
    .physical_address(0x402000)
    .file_offset(0x2000)
    .file_size(0x1000)
    .memory_size(0x2000)  // 包含 BSS
    .alignment(PAGE_SIZE)
    .build();

assembler.add_program_header(data_segment);
```

### 特殊段

```rust
// 动态段 (用于动态链接)
let dynamic_segment = SegmentBuilder::new()
    .segment_type(SegmentType::Dynamic)
    .flags(SegmentFlags::read() | SegmentFlags::write())
    .virtual_address(0x403000)
    .file_offset(0x3000)
    .file_size(dynamic_section.size())
    .memory_size(dynamic_section.size())
    .alignment(8)
    .build();

// 解释器段 (指定动态链接器)
let interp_segment = SegmentBuilder::new()
    .segment_type(SegmentType::Interp)
    .flags(SegmentFlags::read())
    .virtual_address(0x400200)
    .file_offset(0x200)
    .file_size(28)  // "/lib64/ld-linux-x86-64.so.2" 的长度
    .memory_size(28)
    .alignment(1)
    .build();

// GNU 栈段 (控制栈权限)
let stack_segment = SegmentBuilder::new()
    .segment_type(SegmentType::GnuStack)
    .flags(SegmentFlags::read() | SegmentFlags::write())
    .virtual_address(0)
    .file_offset(0)
    .file_size(0)
    .memory_size(0)
    .alignment(16)
    .build();
```

## 节区头表 (Section Header Table)

节区头表描述了文件中的节区 (Sections)，主要用于链接和调试：

### 节区头结构

```rust
use gaia_assembler::elf::{SectionHeader, SectionType, SectionFlags};

#[repr(C)]
pub struct SectionHeader64 {
    pub sh_name: u32,       // 名称索引
    pub sh_type: u32,       // 节区类型
    pub sh_flags: u64,      // 节区标志
    pub sh_addr: u64,       // 虚拟地址
    pub sh_offset: u64,     // 文件偏移
    pub sh_size: u64,       // 节区大小
    pub sh_link: u32,       // 链接信息
    pub sh_info: u32,       // 附加信息
    pub sh_addralign: u64,  // 地址对齐
    pub sh_entsize: u64,    // 表项大小
}
```

### 节区类型

```rust
pub enum SectionType {
    Null = 0,           // 未使用
    ProgBits = 1,       // 程序数据
    SymTab = 2,         // 符号表
    StrTab = 3,         // 字符串表
    Rela = 4,           // 重定位表 (带加数)
    Hash = 5,           // 符号哈希表
    Dynamic = 6,        // 动态链接信息
    Note = 7,           // 注释
    NoBits = 8,         // 不占文件空间 (BSS)
    Rel = 9,            // 重定位表 (不带加数)
    ShLib = 10,         // 保留
    DynSym = 11,        // 动态符号表
    InitArray = 14,     // 初始化函数数组
    FiniArray = 15,     // 终止函数数组
    PreInitArray = 16,  // 预初始化函数数组
    Group = 17,         // 节区组
    SymTabShndx = 18,   // 扩展节区索引
    GnuHash = 0x6ffffff6,     // GNU 哈希表
    GnuVerSym = 0x6fffffff0,  // GNU 版本符号表
    GnuVerDef = 0x6ffffffd,   // GNU 版本定义
    GnuVerNeed = 0x6ffffffe,  // GNU 版本需求
}
```

### 创建常见节区

```rust
use gaia_assembler::elf::{Section, SectionBuilder};

// 1. 空节区 (索引 0)
let null_section = SectionBuilder::new()
    .name("")
    .section_type(SectionType::Null)
    .build();

// 2. 代码节区
let text_section = SectionBuilder::new()
    .name(".text")
    .section_type(SectionType::ProgBits)
    .flags(SectionFlags::ALLOC | SectionFlags::EXECINSTR)
    .virtual_address(0x401000)
    .file_offset(0x1000)
    .size(0x1000)
    .alignment(16)
    .build();

// 3. 只读数据节区
let rodata_section = SectionBuilder::new()
    .name(".rodata")
    .section_type(SectionType::ProgBits)
    .flags(SectionFlags::ALLOC)
    .virtual_address(0x402000)
    .file_offset(0x2000)
    .size(0x500)
    .alignment(8)
    .build();

// 4. 数据节区
let data_section = SectionBuilder::new()
    .name(".data")
    .section_type(SectionType::ProgBits)
    .flags(SectionFlags::ALLOC | SectionFlags::WRITE)
    .virtual_address(0x403000)
    .file_offset(0x3000)
    .size(0x200)
    .alignment(8)
    .build();

// 5. BSS 节区
let bss_section = SectionBuilder::new()
    .name(".bss")
    .section_type(SectionType::NoBits)
    .flags(SectionFlags::ALLOC | SectionFlags::WRITE)
    .virtual_address(0x403200)
    .file_offset(0)  // BSS 不占文件空间
    .size(0x1000)
    .alignment(8)
    .build();
```

### 符号表和字符串表

```rust
// 字符串表
let mut strtab = SectionBuilder::new()
    .name(".strtab")
    .section_type(SectionType::StrTab)
    .build();

// 添加字符串
let main_name_offset = strtab.add_string("main");
let printf_name_offset = strtab.add_string("printf");

// 符号表
let mut symtab = SectionBuilder::new()
    .name(".symtab")
    .section_type(SectionType::SymTab)
    .link(strtab.index())  // 链接到字符串表
    .entry_size(24)        // 64位符号表项大小
    .build();

// 添加符号
use gaia_assembler::elf::{Symbol, SymbolBinding, SymbolType, SymbolVisibility};

symtab.add_symbol(Symbol {
    name_offset: main_name_offset,
    value: 0x401000,
    size: 64,
    symbol_type: SymbolType::Function,
    binding: SymbolBinding::Global,
    visibility: SymbolVisibility::Default,
    section_index: text_section.index(),
});

symtab.add_symbol(Symbol {
    name_offset: printf_name_offset,
    value: 0,
    size: 0,
    symbol_type: SymbolType::Function,
    binding: SymbolBinding::Global,
    visibility: SymbolVisibility::Default,
    section_index: 0,  // 未定义符号
});
```

## 动态链接节区

### 动态节区 (.dynamic)

```rust
use gaia_assembler::elf::{DynamicEntry, DynamicTag};

let mut dynamic_section = SectionBuilder::new()
    .name(".dynamic")
    .section_type(SectionType::Dynamic)
    .flags(SectionFlags::ALLOC | SectionFlags::WRITE)
    .entry_size(16)  // 64位动态条目大小
    .build();

// 添加动态条目
dynamic_section.add_entry(DynamicEntry {
    tag: DynamicTag::Needed,
    value: DynamicValue::String("libc.so.6"),
});

dynamic_section.add_entry(DynamicEntry {
    tag: DynamicTag::SoName,
    value: DynamicValue::String("libmylib.so.1"),
});

dynamic_section.add_entry(DynamicEntry {
    tag: DynamicTag::SymTab,
    value: DynamicValue::Address(dynsym_section.virtual_address()),
});

dynamic_section.add_entry(DynamicEntry {
    tag: DynamicTag::StrTab,
    value: DynamicValue::Address(dynstr_section.virtual_address()),
});

// 结束标记
dynamic_section.add_entry(DynamicEntry {
    tag: DynamicTag::Null,
    value: DynamicValue::Integer(0),
});
```

### PLT 和 GOT

```rust
// 过程链接表 (PLT)
let mut plt_section = SectionBuilder::new()
    .name(".plt")
    .section_type(SectionType::ProgBits)
    .flags(SectionFlags::ALLOC | SectionFlags::EXECINSTR)
    .alignment(16)
    .build();

// PLT[0] - 解析器入口
let plt0_code = vec![
    0xff, 0x35, 0x00, 0x00, 0x00, 0x00,  // push GOT[1]
    0xff, 0x25, 0x00, 0x00, 0x00, 0x00,  // jmp *GOT[2]
    0x0f, 0x1f, 0x40, 0x00,              // nop
];
plt_section.add_code(&plt0_code);

// PLT[1] - printf 条目
let plt_printf_code = vec![
    0xff, 0x25, 0x00, 0x00, 0x00, 0x00,  // jmp *GOT[3]
    0x68, 0x00, 0x00, 0x00, 0x00,        // push 0 (重定位索引)
    0xe9, 0x00, 0x00, 0x00, 0x00,        // jmp PLT[0]
];
plt_section.add_code(&plt_printf_code);

// 全局偏移表 (GOT)
let mut got_section = SectionBuilder::new()
    .name(".got.plt")
    .section_type(SectionType::ProgBits)
    .flags(SectionFlags::ALLOC | SectionFlags::WRITE)
    .alignment(8)
    .build();

// GOT[0] - 动态段地址
got_section.add_address(dynamic_section.virtual_address());
// GOT[1] - 链接映射地址 (运行时填充)
got_section.add_address(0);
// GOT[2] - 解析器地址 (运行时填充)
got_section.add_address(0);
// GOT[3] - printf 地址 (初始指向 PLT[1] 的第二条指令)
got_section.add_address(plt_section.virtual_address() + 22);
```

## 重定位节区

```rust
use gaia_assembler::elf::{RelocationEntry, RelocationType};

// 重定位表 (带加数)
let mut rela_section = SectionBuilder::new()
    .name(".rela.plt")
    .section_type(SectionType::Rela)
    .flags(SectionFlags::ALLOC)
    .link(dynsym_section.index())  // 链接到动态符号表
    .info(plt_section.index())     // 应用到 PLT 节区
    .entry_size(24)                // 64位重定位条目大小
    .build();

// 添加重定位条目
rela_section.add_relocation(RelocationEntry {
    offset: got_section.virtual_address() + 24,  // GOT[3]
    symbol_index: printf_symbol_index,
    relocation_type: RelocationType::X86_64_JumpSlot,
    addend: 0,
});
```

## 完整的 ELF 文件构建

```rust
use gaia_assembler::elf::{ElfAssembler, WriterConfig};

fn build_complete_elf() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut assembler = ElfAssembler::new_executable();
    
    // 1. 设置 ELF 头
    assembler.set_machine(Machine::X86_64);
    assembler.set_entry_point(0x401000);
    
    // 2. 添加解释器
    let interp_section = assembler.add_section(".interp")?;
    interp_section.add_string("/lib64/ld-linux-x86-64.so.2")?;
    
    // 3. 添加代码段
    let text_section = assembler.add_section(".text")?;
    text_section.set_virtual_address(0x401000);
    text_section.add_function("main", &[
        // 调用 printf
        0x48, 0x8d, 0x3d, 0x00, 0x00, 0x00, 0x00,  // lea rdi, [rip+message]
        0xe8, 0x00, 0x00, 0x00, 0x00,              // call printf@plt
        
        // 退出
        0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00,  // mov rax, 60
        0x48, 0xc7, 0xc7, 0x00, 0x00, 0x00, 0x00,  // mov rdi, 0
        0x0f, 0x05,                                // syscall
    ])?;
    
    // 4. 添加只读数据
    let rodata_section = assembler.add_section(".rodata")?;
    rodata_section.add_string("Hello, World!\n")?;
    
    // 5. 添加动态链接信息
    assembler.add_needed_library("libc.so.6")?;
    assembler.add_plt_entry("printf")?;
    
    // 6. 构建符号表
    assembler.build_symbol_tables()?;
    
    // 7. 生成文件
    let config = WriterConfig {
        strip_debug: false,
        optimize_size: true,
        ..Default::default()
    };
    
    assembler.build(config)
}
```

## 文件验证

```rust
use gaia_assembler::elf::{ElfValidator, ValidationError};

fn validate_elf_file(elf_data: &[u8]) -> Result<(), ValidationError> {
    let validator = ElfValidator::new();
    
    // 验证 ELF 头
    validator.validate_header(elf_data)?;
    
    // 验证程序头表
    validator.validate_program_headers(elf_data)?;
    
    // 验证节区头表
    validator.validate_section_headers(elf_data)?;
    
    // 验证符号表
    validator.validate_symbol_tables(elf_data)?;
    
    // 验证重定位表
    validator.validate_relocations(elf_data)?;
    
    // 验证动态链接信息
    validator.validate_dynamic_section(elf_data)?;
    
    Ok(())
}
```

## 性能优化

### 内存布局优化

```rust
// 优化节区排列以减少内存碎片
let layout = MemoryLayoutOptimizer::new()
    .add_executable_sections(&[text_section, plt_section])
    .add_readonly_sections(&[rodata_section, eh_frame_section])
    .add_writable_sections(&[data_section, got_section])
    .add_nobits_sections(&[bss_section])
    .optimize();

assembler.apply_layout(layout);
```

### 文件大小优化

```rust
// 启用节区合并
assembler.enable_section_merging(true);

// 移除未使用的符号
assembler.enable_dead_symbol_elimination(true);

// 压缩调试信息
assembler.enable_debug_compression(true);
```

## 下一步

现在您已经了解了 ELF 文件的详细结构，可以继续学习：

1. **[入门指南](./getting-started.md)** - 学习如何开始使用 ELF 后端
2. **[基本概念](./concepts.md)** - 复习 ELF 格式的核心概念
3. **[用户指南](../../user-guide/index.md)** - 了解 Gaia 框架的通用功能

## 参考工具

- **readelf**: 分析 ELF 文件结构
- **objdump**: 反汇编和查看节区内容
- **hexdump**: 查看文件的十六进制内容
- **nm**: 查看符号表
- **ldd**: 查看动态库依赖

---

*本文档详细介绍了 ELF 文件的内部结构。如需了解具体的代码生成和链接过程，请参考相关的技术文档。*