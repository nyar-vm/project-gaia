# PE 分析器 - DLL 分析工具

一个用 Rust 编写的综合 PE（可移植可执行文件）分析器，可以分析 Windows DLL 文件和可执行文件。
该工具提供有关 PE 文件结构、节区、导入、导出和调试信息的详细信息。

## 特性

- **PE 文件验证**: 验证 PE 文件结构，包括 DOS 头、NT 头和 COFF 头
- **架构检测**: 识别目标架构（x86、x64）
- **子系统分析**: 确定 Windows 子系统类型（GUI、控制台等）
- **节区分析**: 解析并显示所有 PE 节区的详细信息
- **导入表分析**: 列出所有导入的 DLL 和函数
- **导出表分析**: 显示 DLL 中的导出函数
- **调试信息**: 提取调试目录信息
- **可配置解析**: 控制解析 PE 文件的哪些部分

## 使用方法

在您的 `Cargo.toml` 中添加此库：

```toml
[dependencies]
pe-rust = { path = "../pe-rust" }
```

### 基本示例

```rust
use pe_coff::reader::{read_coff_from_file, CoffReader};
use std::path::Path;

fn analyze_coff_file(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // 读取 COFF 对象文件
    let coff_object = read_coff_from_file(path)?;
    
    println!("机器类型: 0x{:04x}", coff_object.header.machine);
    println!("节数量: {}", coff_object.header.number_of_sections);
    println!("符号数量: {}", coff_object.header.number_of_symbols);
    println!("时间戳: {}", coff_object.header.time_date_stamp);
    
    // 显示节信息
    for (i, section) in coff_object.sections.iter().enumerate() {
        let name_raw = String::from_utf8_lossy(&section.header.name);
        let name = name_raw.trim_end_matches('\0');
        println!("节 {}: {} (大小: {} 字节)", i + 1, name, section.header.size_of_raw_data);
    }
    
    Ok(())
}
```

### 读取 COFF 头

```rust
use pe_coff::reader::CoffReader;
use std::fs::File;

fn read_coff_header() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("example.obj")?;
    let coff_object = CoffReader::new().read(file)?;

    // 访问 COFF 头
    println!("COFF 头: {:?}", coff_object.header);

    // 访问节区头
    for section in &coff_object.sections {
        let name_raw = String::from_utf8_lossy(&section.header.name);
        let name = name_raw.trim_end_matches('\0');
        println!("节区: {}", name);
    }
    
    Ok(())
}
```

### 写入简单的 COFF 文件

```rust,no_run
use pe_coff::types::{CoffObject, CoffHeader};

// 创建最小的 COFF 结构
let coff_object = CoffObject {
    header: CoffHeader {
        machine: 0x014c, // i386
        number_of_sections: 0,
        time_date_stamp: 0,
        pointer_to_symbol_table: 0,
        number_of_symbols: 0,
        size_of_optional_header: 0,
        characteristics: 0,
    },
    sections: Vec::new(),
    symbols: Vec::new(),
    string_table: Vec::new(),
};

// 注意：写入功能需要在 writer 模块中实现
println!("COFF 对象创建成功");
```

## 功能特性

- **安全的 Rust API**: 内存安全的操作和适当的错误处理
- **PE 格式兼容**: 完全支持 PE/COFF 文件格式
- **可扩展**: 模块化设计，便于扩展
- **跨平台**: 适用于所有 Rust 支持的平台
- **零拷贝解析**: 读取 PE 文件时高效的内存使用
- **错误处理**: 全面的错误类型和消息

## API 参考

### 读取器模块

`reader` 模块提供了解析 PE 文件的函数：

- `read(data: &[u8]) -> Result<PeInfo, PeError>`: 解析 PE 文件数据
- `read_file(path: &str) -> Result<PeInfo, PeError>`: 读取并解析 PE 文件

### 写入器模块

`writer` 模块提供了生成 PE 文件的函数：

- `write(pe_info: &PeInfo) -> Result<Vec<u8>, PeError>`: 生成 PE 文件数据
- `write_file(path: &str, pe_info: &PeInfo) -> Result<(), PeError>`: 将 PE 文件写入磁盘

### 数据结构

主要数据结构包括：

- `PeInfo`: 完整的 PE 文件表示
- `DosHeader`: DOS 头结构
- `NtHeaders`: NT 头结构
- `CoffHeader`: COFF 头结构
- `OptionalHeader`: 可选头结构
- `SectionHeader`: 节区头结构
- `PeError`: PE 操作错误类型

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

### 代码格式化

```bash
cargo fmt
```

### 代码检查

```bash
cargo clippy
```

## 集成

该库设计用于：

- **pe-wasm32**: 用于浏览器使用的 WebAssembly 绑定
- **il-rust**: 中间语言汇编库
- **CLI 工具**: 命令行 PE 汇编实用程序
- **其他应用程序**: 任何需要 PE 文件操作的 Rust 项目

## 示例

查看 `examples` 目录获取更详细的使用示例：

- `read_pe.rs`: 演示读取和显示 PE 文件信息
- `write_pe.rs`: 显示如何创建简单的 PE 文件
- `modify_pe.rs`: 修改现有 PE 文件的示例

## 许可证

详见根目录的 [License.md](../../License.md)。

## 贡献

欢迎贡献！请随时提交拉取请求。

## 路线图

- [ ] 添加对更多 PE 文件特性的支持（资源、调试信息等）
- [ ] 改进错误处理和报告
- [ ] 添加更全面的测试覆盖
- [ ] 开发性能基准测试
- [ ] 创建更详细的示例和教程