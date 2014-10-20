# WASI 汇编器库

用于 WebAssembly 系统接口（WASI）的 Rust 汇编器库，支持生成 WASI 可执行文件和相关的汇编操作。

## 概述

该库提供了生成和操作 WASI 文件的 Rust 实现，通过中间语言汇编接口实现。它设计用于通过 WIT（Wasm 接口类型）规范与 WebAssembly
协同工作。

## 特性

- **WASI 文件生成**: 程序化创建 WASI 可执行文件
- **汇编接口**: 用于 WASI 汇编操作的高级中间语言
- **WebAssembly 集成**: 基于 WIT 的跨平台接口
- **系统接口支持**: 内置对 WASI 系统调用的支持
- **模块化设计**: 清晰的模块分离，便于扩展和维护

## 使用方法

在您的 `Cargo.toml` 中添加此库：

```toml
[dependencies]
wasi-assembler = "0.1.0"
```

### 基本示例

#### 创建简单的退出程序

```rust
use wasi_assembler::WasiAssembler;

// 创建新的 WASI 汇编器实例
let mut assembler = WasiAssembler::new();

// 配置汇编器
assembler.set_target("wasm32-wasi");

// 创建简单的 WASI 可执行文件
let result = assembler.assemble_from_str(r#"
    (module
        (import "wasi_snapshot_preview1" "proc_exit" (func $proc_exit (param i32)))
        (func $main (export "_start")
            i32.const 0
            call $proc_exit
        )
    )
"#);

match result {
    Ok(wasm_bytes) => {
        println!("WASI 可执行文件生成成功");
        // 将 WASM 字节码保存到文件
        std::fs::write("output.wasm", wasm_bytes).unwrap();
    }
    Err(e) => {
        eprintln!("汇编失败: {}", e);
    }
}
```

#### 创建控制台输出程序

```rust
use wasi_assembler::WasiAssembler;

// 创建输出文本到控制台的 WASI 程序
let mut assembler = WasiAssembler::new();
assembler.set_target("wasm32-wasi");

let result = assembler.assemble_from_str(r#"
    (module
        (import "wasi_snapshot_preview1" "fd_write" (func $fd_write (param i32 i32 i32 i32) (result i32)))
        (import "wasi_snapshot_preview1" "proc_exit" (func $proc_exit (param i32)))
        (memory 1)
        (data (i32.const 0) "Hello, World!\n")
        (func $main (export "_start")
            ;; 写入 stdout
            i32.const 1
            i32.const 0
            i32.const 1
            i32.const 16
            call $fd_write
            drop
            
            ;; 退出码 0
            i32.const 0
            call $proc_exit
        )
    )
"#);

match result {
    Ok(wasm_bytes) => {
        std::fs::write("hello.wasm", wasm_bytes).unwrap();
    }
    Err(e) => {
        eprintln!("汇编失败: {}", e);
    }
}
```

### Advanced Usage

#### Creating a Custom PE Assembler

```rust
use il_assembler::assembler::{PeAssembler, PeSection, ImportTable, ImportEntry};

// Create a new easy_test
let mut assembler = PeAssembler::new_console_app();

// Add import tables
let kernel32_import = ImportTable {
    dll_name: "kernel32.dll".to_string(),
    imports: vec![
        ImportEntry {
            function_name: "ExitProcess".to_string(),
            ordinal: None,
            iat_offset: 0x2000,
        },
        // Add more imports as needed
    ],
    import_lookup_table_rva: 0x2010,
    time_date_stamp: 0,
    forwarder_chain: 0,
    name_rva: 0x2030,
    import_address_table_rva: 0x2000,
};

assembler.import_tables.push(kernel32_import);

// Add sections
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
    data: vec![/* Your machine code here */],
};

assembler.sections.push(text_section);

// Generate PE file
use il_assembler::writer;
let config = writer::WriterConfig {
    format: writer::IlFormat::Exe,
};
let pe_data = writer::write(assembler, config)?;
```

## API 参考

### 汇编器接口

主要的 `WasiAssembler` 结构体提供以下方法：

- `new()`: 创建新的汇编器实例
- `assemble_from_str(source: &str)`: 从字符串汇编 WASI 代码
- `assemble_from_file(path: &str)`: 从文件汇编 WASI 代码
- `set_target(target: &str)`: 设置目标架构
- `with_config(config: WasiConfig)`: 使用自定义配置创建汇编器

### WIT 集成

该库包含用于 WebAssembly 集成的 WIT 定义：

```wit
interface assembler {
    assemble: func(source: string) -> result<list<u8>, string>
    set-target: func(target: string)
    get-supported-targets: func() -> list<string>
}
```

### 模块结构

该库分为几个模块：

- `assembler`: 核心汇编逻辑
- `wit_bindings`: WebAssembly 接口绑定
- `config`: WASI 配置管理
- `module`: WASI 模块定义
- `target`: 目标平台支持

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

### WebAssembly 集成

为 WebAssembly 构建：

```bash
cargo build --target wasm32-wasi
```

### WIT 绑定生成

生成 WIT 绑定：

```bash
wit-bindgen rust --out-dir src/wit_generated wit/
```

## 许可证

本项目采用 Mozilla 公共许可证 2.0 授权。详见 [License.md](../../License.md) 文件。

## 贡献

欢迎贡献！请随时提交拉取请求。

## 项目结构

```
wasi-assembler/
├── src/
│   ├── assembler/    # WASI 汇编功能
│   ├── writer/       # WASM 文件写入功能
│   ├── reader/       # WASM 文件读取功能（进行中）
│   ├── errors/       # 错误处理
│   └── lib.rs        # 库入口点
├── wit/
│   └── wasi-assembler.wit  # WIT 接口定义
└── tests/
    └── readme.md      # 测试示例
```

## 路线图

- [ ] 完成 WASI 文件读取器实现
- [ ] 添加对更多 WASI 特性的支持（文件系统、网络等）
- [ ] 改进错误处理和报告
- [ ] 添加更全面的测试覆盖
- [ ] 优化 WASM 文件生成性能