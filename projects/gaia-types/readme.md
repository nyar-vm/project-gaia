# Gaia Types - 核心类型系统

Gaia Types 是 Gaia 项目中的核心类型定义库，提供了统一的错误处理、架构抽象、编译目标定义等基础功能。作为整个工具链的基础组件，该库采用零依赖设计，确保稳定性和可移植性。

## 特性

- **统一错误模型**: 提供一致的错误处理接口，支持详细的错误诊断信息
- **架构抽象**: 支持多架构的统一抽象，便于跨平台开发
- **类型安全**: 利用 Rust 类型系统防止常见编程错误
- **零依赖核心**: 核心功能不依赖外部 crate，确保最小化依赖树
- **序列化支持**: 可选的 serde 支持，便于数据交换和持久化

## 项目结构

```ignore
gaia-types/
├── src/
│   ├── errors/          # 错误处理系统
│   │   ├── mod.rs       # 错误类型定义和主要接口
│   │   ├── diagnostics.rs # 诊断信息收集器
│   │   ├── display.rs   # 错误显示实现
│   │   └── convert.rs   # 错误转换实现
│   ├── helpers/         # 辅助类型和工具
│   ├── reader/          # 二进制读取器
│   ├── writer/          # 二进制和文本写入器
│   ├── lexer/           # 词法分析器
│   └── lib.rs           # 库入口点
├── tests/               # 测试文件
└── readme.md           # 本文档
```

## 安装

在 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
gaia-types = { path = "../gaia-types" }
```

## 快速开始

### 错误处理

```rust
use gaia_types::{GaiaError, Result};

// 处理文件操作错误
fn read_file(path: &str) -> Result<String> {
    std::fs::read_to_string(path)
        .map_err(|e| GaiaError::io_error(e, path.into()))
}

// 使用结果类型
fn process_data() -> Result<()> {
    let content = read_file("input.txt")?;
    if content.is_empty() {
        return Err(GaiaError::invalid_data("文件内容为空"));
    }
    Ok(())
}
```

### 架构抽象

```rust
use gaia_types::helpers::Architecture;

// 检查架构支持
fn check_architecture(arch: Architecture) -> bool {
    match arch {
        Architecture::X86 | Architecture::X86_64 => true,
        Architecture::ARM32 | Architecture::ARM64 => true,
        _ => false,
    }
}

// 获取架构名称
let arch = Architecture::X86_64;
println!("当前架构: {}", arch); // 输出: x64
```

### 二进制读写

```rust
use gaia_types::reader::BinaryReader;
use std::io::Cursor;

// 读取二进制数据
let data = vec![0x48, 0x65, 0x6C, 0x6C, 0x6F]; // "Hello"
let mut reader = BinaryReader::new(Cursor::new(data));

// 读取字节
let byte = reader.read_u8()?;
println!("读取的字节: 0x{:02X}", byte);
```

## API 参考

### 主要类型

#### GaiaError - 统一错误类型

提供统一的错误处理，包含错误级别和具体的错误信息。支持错误、警告、信息等不同级别。

#### Architecture - 架构枚举

```rust
pub enum Architecture {
    Unknown,    // 未知架构
    X86,        // x86 32位
    X86_64,     // x86-64 64位
    ARM32,      // ARM 32位
    ARM64,      // ARM64 64位
    RISCV32,    // RISC-V 32位
    RISCV64,    // RISC-V 64位
    MIPS32,     // MIPS 32位
    MIPS64,     // MIPS 64位
    WASM32,     // WebAssembly 32位
    WASM64,     // WebAssembly 64位
    JVM,        // Java虚拟机
    CLR,        // .NET运行时
    Other(String), // 其他架构
}
```

#### BinaryReader - 二进制读取器

```rust
pub struct BinaryReader<R: Read> {
    reader: R,
    position: u64,
}
```

主要方法：
- `read_u8()`, `read_u16()`, `read_u32()`, `read_u64()` - 读取整数
- `read_i8()`, `read_i16()`, `read_i32()`, `read_i64()` - 读取有符号整数
- `read_f32()`, `read_f64()` - 读取浮点数
- `read_bytes()` - 读取字节数组

#### BinaryWriter - 二进制写入器

```rust
pub struct BinaryWriter<W: Write> {
    writer: W,
    position: u64,
}
```

主要方法：
- `write_u8()`, `write_u16()`, `write_u32()`, `write_u64()` - 写入整数
- `write_i8()`, `write_i16()`, `write_i32()`, `write_i64()` - 写入有符号整数
- `write_f32()`, `write_f64()` - 写入浮点数
- `write_bytes()` - 写入字节数组

### 辅助功能

#### 文件操作工具

```rust
use gaia_types::helpers::{open_file, create_file};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 打开文件
    let path = Path::new("input.txt");
    let (file, url) = open_file(path)?;
    
    // 创建文件
    let (file, url) = create_file(Path::new("output.txt"))?;
    Ok(())
}
```

#### JSON 序列化

```rust
#[cfg(feature = "serde_json")]
use gaia_types::helpers::save_json;

// 保存 JSON 数据
let data = serde_json::json!({
    "name": "example",
    "version": "1.0.0"
});
save_json(&data, Path::new("config.json"))?;
```

## 开发指南

### 添加新的错误类型

1. 在 `GaiaErrorKind` 枚举中添加新的错误变体
2. 为新的错误类型创建便捷的构造函数
3. 实现错误转换 trait
4. 添加单元测试和文档测试

### 扩展架构支持

1. 在 `Architecture` 枚举中添加新架构
2. 更新 `Display` trait 实现
3. 添加架构兼容性检查
4. 确保序列化支持

### 性能优化建议

1. **零分配设计**: 核心路径避免堆分配
2. **缓存友好**: 使用小的数据结构提高缓存效率
3. **内联优化**: 关键函数使用内联属性
4. **错误处理**: 使用 `Box` 包装减少枚举大小
2. 在 `GaiaError` 结构体中添加对应的构造函数
3. 在 `display.rs` 中实现错误显示逻辑

### 运行测试

```bash
cd gaia-types
cargo test
```

## 许可证

本项目采用 MIT 许可证。详见 LICENSE 文件。

## 贡献

欢迎提交 Issue 和 Pull Request 来改进这个项目。