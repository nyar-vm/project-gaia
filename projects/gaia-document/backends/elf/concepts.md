# ELF 基础概念

本文档介绍 ELF (Executable and Linkable Format) 格式的核心概念和基础知识，帮助您理解 Gaia ELF 后端的工作原理。

## 什么是 ELF

ELF (Executable and Linkable Format) 是 Unix 和类 Unix 系统（包括 Linux）的标准二进制文件格式。它定义了可执行文件、目标文件、共享库和核心转储文件的结构。

### ELF 的历史

- **1990年代初**: 由 Unix System Laboratories (USL) 开发
- **1995年**: 成为 System V Release 4 (SVR4) 的标准
- **现在**: Linux、FreeBSD、Solaris 等系统的默认格式

### 文件类型

```rust
use gaia_assembler::elf::{ElfAssembler, FileType};

// 可执行文件
let mut exec_assembler = ElfAssembler::new(FileType::Executable);

// 共享库
let mut so_assembler = ElfAssembler::new(FileType::SharedObject);

// 可重定位文件 (目标文件)
let mut obj_assembler = ElfAssembler::new(FileType::Relocatable);

// 核心转储文件
let mut core_assembler = ElfAssembler::new(FileType::Core);
```

## 核心特性

### 1. 统一格式

ELF 为不同类型的二进制文件提供统一的格式：

```rust
use gaia_assembler::elf::{ElfHeader, FileType, Machine};

let header = ElfHeader {
    file_type: FileType::Executable,
    machine: Machine::X86_64,
    version: 1,
    entry_point: 0x401000,
    // ... 其他字段
};
```

### 2. 虚拟内存映射

ELF 文件设计为可以直接映射到虚拟内存：

```rust
use gaia_assembler::elf::{ProgramHeader, SegmentType, SegmentFlags};

// 创建可执行段
let text_segment = ProgramHeader {
    segment_type: SegmentType::Load,
    flags: SegmentFlags::READ | SegmentFlags::EXECUTE,
    virtual_address: 0x401000,
    physical_address: 0x401000,
    file_size: 0x1000,
    memory_size: 0x1000,
    alignment: 0x1000,
};

// 创建数据段
let data_segment = ProgramHeader {
    segment_type: SegmentType::Load,
    flags: SegmentFlags::READ | SegmentFlags::WRITE,
    virtual_address: 0x402000,
    physical_address: 0x402000,
    file_size: 0x1000,
    memory_size: 0x2000,  // BSS 段在内存中更大
    alignment: 0x1000,
};
```

### 3. 模块化设计

ELF 支持模块化的程序设计和动态链接：

```rust
use gaia_assembler::elf::{DynamicTag, DynamicEntry};

// 添加动态库依赖
assembler.add_dynamic_entry(DynamicEntry {
    tag: DynamicTag::Needed,
    value: "libc.so.6".into(),
});

// 设置共享库名称
assembler.add_dynamic_entry(DynamicEntry {
    tag: DynamicTag::SoName,
    value: "libmylib.so.1".into(),
});
```

## 关键概念

### 1. 虚拟地址 (Virtual Address)

虚拟地址是程序在内存中的逻辑地址：

```rust
use gaia_assembler::elf::VirtualAddress;

// 代码段通常从 0x400000 开始
const TEXT_BASE: VirtualAddress = 0x401000;

// 数据段紧跟代码段
const DATA_BASE: VirtualAddress = 0x402000;

// 堆栈通常在高地址
const STACK_BASE: VirtualAddress = 0x7fffffffe000;

// 设置节区虚拟地址
text_section.set_virtual_address(TEXT_BASE);
data_section.set_virtual_address(DATA_BASE);
```

### 2. 文件偏移 (File Offset)

文件偏移是数据在 ELF 文件中的位置：

```rust
use gaia_assembler::elf::FileOffset;

// ELF 头总是从偏移 0 开始
const ELF_HEADER_OFFSET: FileOffset = 0;

// 程序头表紧跟 ELF 头
const PROGRAM_HEADER_OFFSET: FileOffset = 64;  // 64位 ELF 头大小

// 计算节区在文件中的偏移
fn calculate_section_offset(previous_offset: FileOffset, previous_size: usize) -> FileOffset {
    previous_offset + previous_size as FileOffset
}
```

### 3. 对齐 (Alignment)

ELF 使用对齐来优化内存访问和页面映射：

```rust
use gaia_assembler::elf::Alignment;

// 页面对齐 (通常 4KB)
const PAGE_ALIGNMENT: Alignment = 0x1000;

// 缓存行对齐 (64字节)
const CACHE_LINE_ALIGNMENT: Alignment = 64;

// 设置节区对齐
text_section.set_alignment(PAGE_ALIGNMENT);
data_section.set_alignment(PAGE_ALIGNMENT);

// 函数对齐优化
function_section.set_alignment(16);  // 16字节对齐提高性能
```

### 4. 权限和标志

ELF 使用标志控制内存权限和行为：

```rust
use gaia_assembler::elf::{SectionFlags, SegmentFlags};

// 节区标志
let text_flags = SectionFlags::ALLOC | SectionFlags::EXECINSTR;
let data_flags = SectionFlags::ALLOC | SectionFlags::WRITE;
let rodata_flags = SectionFlags::ALLOC;

// 段标志
let executable_flags = SegmentFlags::READ() | SegmentFlags::execute();
let writable_flags = SegmentFlags::read() | SegmentFlags::write();
let readonly_flags = SegmentFlags::read();
```

## ELF 文件的生命周期

### 1. 编译时 (Compile Time)

编译器生成可重定位的 ELF 目标文件：

```rust
use gaia_assembler::elf::{ObjectFile, RelocationEntry, RelocationInfo};

// 创建目标文件
let mut object = ObjectFile::new();

// 添加未解析的符号引用
object.add_undefined_symbol("printf");
object.add_undefined_symbol("malloc");

// 添加重定位条目
object.add_relocation(RelocationEntry {
    offset: 0x10,
    symbol: "printf",
    relocation_type: RelocationInfo::X86_64_PLT32,
    addend: -4,
});
```

### 2. 链接时 (Link Time)

链接器将多个目标文件合并为可执行文件：

```rust
use gaia_assembler::elf::{Linker, LinkOptions};

let mut linker = Linker::new();

// 添加目标文件
linker.add_object_file("main.o");
linker.add_object_file("utils.o");

// 添加库文件
linker.add_library("libc.so.6");
linker.add_library("libm.so.6");

// 设置链接选项
let options = LinkOptions {
    entry_point: Some("_start"),
    output_type: OutputType::Executable,
    dynamic_linking: true,
    ..Default::default()
};

// 执行链接
let executable = linker.link(options)?;
```

### 3. 加载时 (Load Time)

操作系统加载器将 ELF 文件映射到内存：

```rust
use gaia_assembler::elf::{Loader, LoadOptions};

// 模拟加载过程
let loader = Loader::new();

// 解析 ELF 头
let elf_header = loader.parse_header(&elf_data)?;

// 映射程序段到内存
for program_header in elf_header.program_headers() {
    if program_header.segment_type == SegmentType::Load {
        loader.map_segment(program_header)?;
    }
}

// 执行动态链接
if elf_header.has_dynamic_section() {
    loader.resolve_dynamic_symbols()?;
}

// 跳转到入口点
loader.jump_to_entry_point(elf_header.entry_point);
```

### 4. 运行时 (Runtime)

程序在内存中执行，可能触发延迟绑定：

```rust
use gaia_assembler::elf::{RuntimeLinker, SymbolResolver};

// 运行时符号解析
let runtime_linker = RuntimeLinker::new();

// 延迟绑定 (Lazy Binding)
runtime_linker.register_plt_resolver(|symbol_name| {
    // 查找符号地址
    let symbol_address = runtime_linker.resolve_symbol(symbol_name)?;
    
    // 更新 GOT 表项
    runtime_linker.update_got_entry(symbol_name, symbol_address)?;
    
    Ok(symbol_address)
});
```

## 节区类型

### 1. 代码节区 (.text)

存储可执行的机器代码：

```rust
use gaia_assembler::elf::{Section, SectionType};

let mut text_section = Section::new(".text");
text_section.set_type(SectionType::ProgBits);
text_section.set_flags(SectionFlags::ALLOC | SectionFlags::EXECINSTR);

// 添加函数代码
text_section.add_function("main", &[
    0x48, 0x83, 0xec, 0x08,  // sub rsp, 8
    0x48, 0x83, 0xc4, 0x08,  // add rsp, 8
    0xc3,                    // ret
]);
```

### 2. 数据节区 (.data, .rodata, .bss)

```rust
// 初始化数据
let mut data_section = Section::new(".data");
data_section.set_type(SectionType::ProgBits);
data_section.set_flags(SectionFlags::ALLOC | SectionFlags::WRITE);
data_section.add_data(&[1, 2, 3, 4]);

// 只读数据
let mut rodata_section = Section::new(".rodata");
rodata_section.set_type(SectionType::ProgBits);
rodata_section.set_flags(SectionFlags::ALLOC);
rodata_section.add_string("Hello, World!");

// 未初始化数据
let mut bss_section = Section::new(".bss");
bss_section.set_type(SectionType::NoBits);
bss_section.set_flags(SectionFlags::ALLOC | SectionFlags::WRITE);
bss_section.set_size(1024);  // 1KB 未初始化空间
```

### 3. 符号表 (.symtab, .dynsym)

```rust
use gaia_assembler::elf::{Symbol, SymbolBinding, SymbolType};

// 静态符号表
let mut symtab = Section::new(".symtab");
symtab.set_type(SectionType::SymTab);

// 添加符号
symtab.add_symbol(Symbol {
    name: "main",
    value: 0x401000,
    size: 64,
    symbol_type: SymbolType::Function,
    binding: SymbolBinding::Global,
    visibility: SymbolVisibility::Default,
    section_index: text_section.index(),
});

// 动态符号表
let mut dynsym = Section::new(".dynsym");
dynsym.set_type(SectionType::DynSym);
dynsym.add_symbol(Symbol {
    name: "printf",
    value: 0,
    size: 0,
    symbol_type: SymbolType::Function,
    binding: SymbolBinding::Global,
    visibility: SymbolVisibility::Default,
    section_index: 0,  // 未定义符号
});
```

## 重定位机制

### 1. 重定位类型

```rust
use gaia_assembler::elf::{RelocationEntry, RelocationType};

// 绝对地址重定位
let abs_reloc = RelocationEntry {
    offset: 0x1000,
    symbol_index: 1,
    relocation_type: RelocationType::X86_64_64,
    addend: 0,
};

// 相对地址重定位
let rel_reloc = RelocationEntry {
    offset: 0x1005,
    symbol_index: 2,
    relocation_type: RelocationType::X86_64_PC32,
    addend: -4,
};

// PLT 重定位
let plt_reloc = RelocationEntry {
    offset: 0x100A,
    symbol_index: 3,
    relocation_type: RelocationType::X86_64_PLT32,
    addend: -4,
};
```

### 2. 重定位计算

```rust
// 重定位计算公式
fn apply_relocation(
    reloc: &RelocationEntry,
    symbol_value: u64,
    section_base: u64,
    page_base: u64,
) -> u64 {
    match reloc.relocation_type {
        // S + A (符号值 + 加数)
        RelocationType::X86_64_64 => {
            symbol_value + reloc.addend as u64
        },
        
        // S + A - P (符号值 + 加数 - 重定位位置)
        RelocationType::X86_64_PC32 => {
            let reloc_address = section_base + reloc.offset;
            (symbol_value + reloc.addend as u64).wrapping_sub(reloc_address)
        },
        
        // L + A - P (PLT 条目 + 加数 - 重定位位置)
        RelocationType::X86_64_PLT32 => {
            let plt_entry = get_plt_entry_address(reloc.symbol_index);
            let reloc_address = section_base + reloc.offset;
            (plt_entry + reloc.addend as u64).wrapping_sub(reloc_address)
        },
        
        // 其他重定位类型...
        _ => unimplemented!(),
    }
}
```

## 安全模型

### 1. 内存保护

ELF 支持现代安全特性：

```rust
use gaia_assembler::elf::{SecurityFeatures, StackProtection};

let mut assembler = ElfAssembler::new_executable();

// 启用栈保护
assembler.enable_stack_protection(StackProtection::Strong);

// 启用 ASLR (地址空间布局随机化)
assembler.enable_aslr(true);

// 启用 NX 位 (不可执行栈)
assembler.enable_nx_bit(true);

// 启用 FORTIFY_SOURCE
assembler.enable_fortify_source(true);
```

### 2. 控制流完整性

```rust
// 添加 CFI 指令
text_section.add_cfi_directive(".cfi_startproc");
text_section.add_code(&function_prologue);
text_section.add_cfi_directive(".cfi_def_cfa_offset 16");
text_section.add_code(&function_body);
text_section.add_cfi_directive(".cfi_endproc");
```

## 性能特性

### 1. 内存效率

ELF 设计为内存高效：

```rust
// 共享只读段
let rodata_segment = ProgramHeader {
    segment_type: SegmentType::Load,
    flags: SegmentFlags::READ,  // 只读，可在进程间共享
    virtual_address: 0x400000,
    file_size: rodata_size,
    memory_size: rodata_size,
    alignment: PAGE_SIZE,
};

// 写时复制数据段
let data_segment = ProgramHeader {
    segment_type: SegmentType::Load,
    flags: SegmentFlags::READ | SegmentFlags::WRITE,
    virtual_address: 0x600000,
    file_size: data_size,
    memory_size: data_size + bss_size,
    alignment: PAGE_SIZE,
};
```

### 2. 加载优化

```rust
// 预链接支持
assembler.enable_prelinking(true);

// 符号版本控制
assembler.add_symbol_version("malloc", "GLIBC_2.2.5");
assembler.add_symbol_version("free", "GLIBC_2.2.5");

// GNU Hash 表 (比传统 Hash 更快)
assembler.use_gnu_hash(true);
```

## 下一步

现在您已经了解了 ELF 的基础概念，可以继续学习：

1. **[文件结构](./file-structure.md)** - 深入了解 ELF 文件的详细结构
2. **[入门指南](./getting-started.md)** - 学习如何开始使用 ELF 后端
3. **[ELF 官方规范](https://refspecs.linuxfoundation.org/elf/elf.pdf)** - 查看 ELF 格式的官方规范
4. **[System V ABI](https://refspecs.linuxfoundation.org/elf/x86_64-abi-0.99.pdf)** - 了解 System V 应用程序二进制接口

## 参考资源

- [ELF 格式规范](https://refspecs.linuxfoundation.org/elf/elf.pdf)
- [System V ABI](https://refspecs.linuxfoundation.org/elf/x86_64-abi-0.99.pdf)
- [Linux 内核 ELF 加载器](https://github.com/torvalds/linux/blob/master/fs/binfmt_elf.c)
- [GNU Binutils 文档](https://sourceware.org/binutils/docs/)

---

*本文档介绍了 ELF 格式的核心概念。如需了解具体实现细节，请参考相关的技术文档。*