# CLR 汇编器库

用于 PE（可移植可执行文件）文件生成和操作的 IL（中间语言）汇编库的 Rust 实现。

## 概述

该库提供了生成和操作 PE 文件的 Rust 实现，通过中间语言汇编接口实现。它设计用于通过 WIT（Wasm 接口类型）规范与 WebAssembly
协同工作。

## 特性

- **PE 文件生成**: 程序化创建 PE 可执行文件和 DLL
- **汇编接口**: 用于 PE 汇编操作的高级中间语言
- **WebAssembly 集成**: 基于 WIT 的跨平台接口
- **控制台应用程序支持**: 内置 Windows 控制台应用程序支持
- **导入表处理**: 完整的 DLL 导入和函数解析支持

## 使用方法

在您的 `Cargo.toml` 中添加此库：

```toml
[dependencies]
il-assembler = { path = "../il-rust" }
```

### 基本示例

#### 创建简单的退出应用程序

```rust
use il_assembler::assembler;

// 创建一个以特定代码退出的 PE 文件
let exit_code = 42;
let pe_data = assembler::easy_exit_code(exit_code);

// 写入文件
std::fs::write("exit_example.exe", pe_data) ?;
```

#### 创建控制台输出应用程序

```rust
use il_assembler::assembler;

// 创建一个向控制台输出文本的 PE 文件
let text = "Hello, World!";
let pe_data = assembler::easy_console_log(text.to_string());

// 写入文件
std::fs::write("console_example.exe", pe_data) ?;
```

### 高级用法

#### 创建自定义 PE 汇编器

```rust
use il_assembler::assembler::{PeAssembler, PeSection, ImportTable, ImportEntry};

// 创建新的测试
let mut assembler = PeAssembler::new_console_app();

// 添加导入表
let kernel32_import = ImportTable {
dll_name: "kernel32.dll".to_string(),
imports: vec![
    ImportEntry {
        function_name: "ExitProcess".to_string(),
        ordinal: None,
        iat_offset: 0x2000,
    },
    // 根据需要添加更多导入
],
import_lookup_table_rva: 0x2010,
time_date_stamp: 0,
forwarder_chain: 0,
name_rva: 0x2030,
import_address_table_rva: 0x2000,
};

assembler.import_tables.push(kernel32_import);

// 添加节区
let text_section = PeSection {
name: ".text".to_string(),
virtual_size: 0x1000,
virtual_address: 0x1000,
size_of_raw_data: 0x200,
pointer_to_raw_data: 0x200,
pointer_to_relocations: 0,
pointer_to_line_numbers: 0,
number_of_relocations: 0,
number_of_line_numbers: 0,
characteristics: 0x60000020, // CODE | EXECUTE | READ
data: vec![/* 您的机器代码放在这里 */],
};

assembler.sections.push(text_section);

// 生成 PE 文件
use il_assembler::writer;
let config = writer::WriterConfig {
format: writer::IlFormat::Exe,
};
let pe_data = writer::write(assembler, config) ?;
```

## API 参考

### 汇编器接口

`assembler` 模块提供了创建 PE 汇编的核心功能：

- `new_console_app()`: 创建配置为控制台应用程序的新 PE 汇编器
- `easy_exit_code(code: u32)`: 生成以指定代码退出的 PE 文件
- `easy_console_log(text: String)`: 生成向控制台输出文本的 PE 文件

### 写入器接口

`writer` 模块提供了写入 PE 文件的功能：

- `write(assembler: PeAssembler, config: WriterConfig)`: 将 PE 汇编器表示转换为 PE 文件二进制数据

### 读取器接口

`reader` 模块提供了读取 PE 文件的功能（目前仍在开发中）：

- `read(pe_data: Vec<u8>, config: ReadConfig)`: 解析 PE 文件二进制数据为 PE 汇编器表示

## 开发

### 构建

```bash
cargo build
```

### 测试

```bash
cargo test
```

### 文档

```bash
cargo doc --open
```

## WebAssembly 集成

该库设计用于通过 WIT 规范与 WebAssembly 协同工作。`wit/il-assembly.wit` 文件定义了可跨不同语言和平台使用的接口。

## 许可证

本项目采用 Mozilla 公共许可证 2.0。详见 [License.md](../../License.md) 文件。

## 贡献

欢迎贡献！请随时提交拉取请求。

## 项目结构

```
il-rust/
├── src/
│   ├── assembler/    # PE 汇编功能
│   ├── writer/       # PE 文件写入功能
│   ├── reader/       # PE 文件读取功能（开发中）
│   ├── errors/       # 错误处理
│   └── lib.rs        # 库入口点
├── wit/
│   └── il-assembly.wit  # WIT 接口定义
└── tests/
    └── readme.md      # 测试示例
```

## 路线图

- [ ] 完成 PE 文件读取器实现
- [ ] 添加对更多 PE 文件特性的支持（资源、调试信息等）
- [ ] 改进错误处理和报告
- [ ] 添加更全面的测试覆盖
- [ ] 优化 PE 文件生成性能