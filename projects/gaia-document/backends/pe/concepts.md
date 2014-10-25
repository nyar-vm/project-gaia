# PE 基础概念

本文档介绍 PE (Portable Executable) 格式的基础概念和核心知识，帮助您理解 Windows 可执行文件的内部结构。

## 什么是 PE 格式

PE (Portable Executable) 是 Microsoft Windows 操作系统使用的可执行文件格式。它是 COFF (Common Object File Format) 的扩展，用于：

- **可执行文件** (.exe)
- **动态链接库** (.dll)
- **设备驱动程序** (.sys)
- **对象文件** (.obj)

### PE 格式的历史

- **1993年**: 随 Windows NT 3.1 引入
- **基于**: UNIX COFF 格式
- **设计目标**: 跨平台可移植性 (虽然主要用于 Windows)
- **演进**: 支持 32位和64位架构

## 核心特性

### 1. 统一格式

PE 格式为不同类型的可执行文件提供统一的结构：

```rust
use gaia_assembler::pe::{PeType, PeAssembler};

// 创建不同类型的 PE 文件
let exe_assembler = PeAssembler::new(PeType::Executable);
let dll_assembler = PeAssembler::new(PeType::DynamicLibrary);
let sys_assembler = PeAssembler::new(PeType::Driver);
```

### 2. 虚拟内存映射

PE 文件设计为可以直接映射到虚拟内存：

```rust
// 文件中的布局
struct FileLayout {
    dos_header: DosHeader,      // 文件偏移 0x0
    nt_headers: NtHeaders,      // 文件偏移 dos_header.e_lfanew
    sections: Vec<Section>,     // 紧跟 NT 头之后
}

// 内存中的布局
struct MemoryLayout {
    image_base: u64,           // 默认加载地址
    sections: Vec<VirtualSection>, // 按虚拟地址排列
}
```

### 3. 模块化设计

PE 文件由多个独立的节区组成：

```rust
use gaia_assembler::pe::{SectionCharacteristics, Section};

// 代码节区
let text_section = Section::new(".text")
.with_characteristics(
SectionCharacteristics::CODE |
SectionCharacteristics::EXECUTE |
SectionCharacteristics::READ
);

// 数据节区
let data_section = Section::new(".data")
.with_characteristics(
SectionCharacteristics::INITIALIZED_DATA |
SectionCharacteristics::READ |
SectionCharacteristics::WRITE
);
```

## 关键概念

### 1. 镜像基址 (Image Base)

镜像基址是 PE 文件在内存中的首选加载地址：

```rust
use gaia_assembler::pe::PeAssembler;

let mut assembler = PeAssembler::new_console_app();

// 设置镜像基址
assembler.set_image_base(0x400000);  // 32位默认值
// 或
assembler.set_image_base(0x140000000); // 64位默认值
```

**重要性**:

- 影响绝对地址的计算
- 与 ASLR (地址空间布局随机化) 相关
- 决定重定位的需要

### 2. 相对虚拟地址 (RVA)

RVA 是相对于镜像基址的偏移量：

```rust
// RVA 计算示例
fn calculate_rva(virtual_address: u64, image_base: u64) -> u32 {
    (virtual_address - image_base) as u32
}

// 使用示例
let image_base = 0x400000;
let function_address = 0x401000;
let rva = calculate_rva(function_address, image_base); // 0x1000
```

### 3. 文件对齐与节区对齐

PE 文件使用两种不同的对齐方式：

```rust
use gaia_assembler::pe::WriterConfig;

let config = WriterConfig {
file_alignment: 0x200,    // 文件中的对齐 (512 字节)
section_alignment: 0x1000, // 内存中的对齐 (4096 字节)
..Default::default ()
};
```

**文件对齐**: 优化磁盘存储空间
**节区对齐**: 符合内存页面大小

### 4. 虚拟地址与文件偏移

理解地址转换是关键：

```rust
struct AddressConverter {
    sections: Vec<SectionInfo>,
}

impl AddressConverter {
    fn rva_to_file_offset(&self, rva: u32) -> Option<u32> {
        for section in &self.sections {
            if rva >= section.virtual_address &&
                rva < section.virtual_address + section.virtual_size {
                let offset_in_section = rva - section.virtual_address;
                return Some(section.pointer_to_raw_data + offset_in_section);
            }
        }
        None
    }

    fn file_offset_to_rva(&self, file_offset: u32) -> Option<u32> {
        for section in &self.sections {
            if file_offset >= section.pointer_to_raw_data &&
                file_offset < section.pointer_to_raw_data + section.size_of_raw_data {
                let offset_in_section = file_offset - section.pointer_to_raw_data;
                return Some(section.virtual_address + offset_in_section);
            }
        }
        None
    }
}
```

## PE 文件的生命周期

### 1. 编译时 (Compile Time)

```rust
// 编译器生成对象文件
let object_file = compile_source("main.c") ?;

// 链接器创建 PE 文件
let mut assembler = PeAssembler::new_console_app();
assembler.add_object_file(object_file);
let pe_data = assembler.build(WriterConfig::default ()) ?;
```

### 2. 加载时 (Load Time)

```rust
// Windows 加载器的简化流程
struct PeLoader {
    image_base: u64,
    sections: Vec<LoadedSection>,
}

impl PeLoader {
    fn load_pe(&mut self, pe_data: &[u8]) -> Result<(), LoadError> {
        // 1. 解析 PE 头
        let headers = parse_pe_headers(pe_data)?;

        // 2. 分配虚拟内存
        let memory = allocate_virtual_memory(
            headers.optional_header.image_base,
            headers.optional_header.size_of_image
        )?;

        // 3. 映射节区
        for section in headers.sections {
            map_section(&section, pe_data, memory)?;
        }

        // 4. 处理导入
        resolve_imports(&headers.import_table)?;

        // 5. 应用重定位
        apply_relocations(&headers.relocation_table, memory)?;

        // 6. 设置内存保护
        set_memory_protection(&headers.sections, memory)?;

        Ok(())
    }
}
```

### 3. 运行时 (Runtime)

```rust
// 运行时的内存布局
struct RuntimeImage {
    base_address: *mut u8,
    entry_point: *const fn(),
    import_table: ImportTable,
    export_table: ExportTable,
}

impl RuntimeImage {
    fn call_entry_point(&self) {
        unsafe {
            (self.entry_point)();
        }
    }

    fn get_exported_function(&self, name: &str) -> Option<*const fn()> {
        self.export_table.find_function(name)
    }
}
```

## 节区类型详解

### 1. 代码节区 (.text)

```rust
use gaia_assembler::pe::{Section, SectionCharacteristics};

let mut text_section = Section::new(".text");
text_section.set_characteristics(
SectionCharacteristics::CODE |
SectionCharacteristics::EXECUTE |
SectionCharacteristics::READ
);

// 添加函数代码
text_section.add_function("main", & [
0x48, 0x83, 0xEC, 0x28,  // sub rsp, 40
0x48, 0x31, 0xC0,        // xor rax, rax
0x48, 0x83, 0xC4, 0x28,  // add rsp, 40
0xC3,                    // ret
]);
```

### 2. 数据节区 (.data, .rdata)

```rust
// 可写数据节区
let mut data_section = Section::new(".data");
data_section.set_characteristics(
SectionCharacteristics::INITIALIZED_DATA |
SectionCharacteristics::READ |
SectionCharacteristics::WRITE
);

// 只读数据节区
let mut rdata_section = Section::new(".rdata");
rdata_section.set_characteristics(
SectionCharacteristics::INITIALIZED_DATA |
SectionCharacteristics::READ
);

// 添加字符串常量
rdata_section.add_string("Hello, World!\0");
```

### 3. 未初始化数据节区 (.bss)

```rust
let mut bss_section = Section::new(".bss");
bss_section.set_characteristics(
SectionCharacteristics::UNINITIALIZED_DATA |
SectionCharacteristics::READ |
SectionCharacteristics::WRITE
);

// 预留空间但不占用文件大小
bss_section.reserve_space(1024); // 1KB 未初始化空间
```

## 导入导出机制

### 导入表 (Import Table)

```rust
use gaia_assembler::pe::ImportDescriptor;

// 添加 DLL 导入
assembler.add_import_dll("kernel32.dll", & [
"GetStdHandle",
"WriteConsoleA",
"ExitProcess"
]);

assembler.add_import_dll("user32.dll", & [
"MessageBoxA",
"GetWindowTextA"
]);
```

### 导出表 (Export Table)

```rust
// 添加函数导出 (用于 DLL)
assembler.add_export("MyFunction", 0x1000);
assembler.add_export("MyVariable", 0x2000);

// 按序号导出
assembler.add_export_by_ordinal("FastFunction", 1, 0x1100);
```

## 重定位机制

### 基址重定位

```rust
use gaia_assembler::pe::{RelocationType, RelocationEntry};

// 添加重定位项
assembler.add_relocation(RelocationEntry {
rva: 0x1005,
reloc_type: RelocationType::Dir64, // 64位绝对地址
});

assembler.add_relocation(RelocationEntry {
rva: 0x1010,
reloc_type: RelocationType::HighLow, // 32位绝对地址
});
```

### 重定位的必要性

```rust
// 原始代码中的绝对地址引用
let original_address = 0x401000;

// 如果加载到不同地址
let actual_base = 0x500000;
let image_base = 0x400000;
let delta = actual_base - image_base;

// 需要修正的地址
let corrected_address = original_address + delta; // 0x501000
```

## 安全特性

### 1. 数据执行保护 (DEP)

```rust
use gaia_assembler::pe::DllCharacteristics;

assembler.set_dll_characteristics(
DllCharacteristics::NX_COMPAT
);
```

### 2. 地址空间布局随机化 (ASLR)

```rust
assembler.set_dll_characteristics(
DllCharacteristics::DYNAMIC_BASE
);
```

### 3. 控制流保护 (CFG)

```rust
assembler.set_dll_characteristics(
DllCharacteristics::GUARD_CF
);
```

## 调试信息

### 调试目录

```rust
use gaia_assembler::pe::{DebugDirectory, DebugType};

// 添加 CodeView 调试信息
assembler.add_debug_directory(DebugDirectory {
debug_type: DebugType::CodeView,
size_of_data: debug_data.len() as u32,
address_of_raw_data: debug_rva,
pointer_to_raw_data: debug_file_offset,
});
```

### 符号表

```rust
// 添加符号信息
assembler.add_symbol("main", 0x1000, SymbolType::Function);
assembler.add_symbol("global_var", 0x2000, SymbolType::Data);
```

## 性能考虑

### 1. 内存布局优化

```rust
// 将相关的代码和数据放在相邻的节区中
// 以提高缓存局部性
assembler.optimize_section_layout();
```

### 2. 导入优化

```rust
// 延迟加载 DLL
assembler.set_delay_load("optional.dll", & ["OptionalFunction"]);
```

### 3. 代码对齐

```rust
// 函数对齐以优化指令缓存
text_section.set_function_alignment(16); // 16字节对齐
```

## 下一步

现在您已经了解了 PE 格式的基础概念，可以继续学习：

1. **[文件结构](./file-structure.md)** - PE 文件的详细结构分析
2. **[代码生成](./code-generation.md)** - 机器码生成和指令集
3. **[内存管理](./memory-management.md)** - 虚拟内存和地址空间管理
4. **[导入导出](./import-export.md)** - 深入了解 DLL 机制

## 参考资源

- [Microsoft PE/COFF 规范](https://docs.microsoft.com/en-us/windows/win32/debug/pe-format)
- [Intel x86-64 架构手册](https://software.intel.com/content/www/us/en/develop/articles/intel-sdm.html)
- [Windows 内核编程](https://docs.microsoft.com/en-us/windows-hardware/drivers/)

---

*本文档涵盖了 PE 格式的核心概念。如需更深入的技术细节，请参考相关的专门文档。*