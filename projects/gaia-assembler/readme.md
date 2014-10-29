# Gaia 汇编器

支持多个目标平台（包括 IL、JVM、PE 和 WASI）的统一汇编器框架。

## 支持的目标平台

- **IL** - .NET Intermediate Language
- **JVM** - Java Virtual Machine 字节码
- **PE** - Portable Executable (Windows 可执行文件)
- **WASI** - WebAssembly System Interface

## 特性

- **多平台支持**: 目标平台包括 IL、JVM、PE 和 WASI
- **统一指令集**: 所有平台通用的指令集
- **模块化架构**: 清晰的关注点分离
- **类型安全**: 与 gaia-types 集成，确保类型安全
- **可扩展性**: 易于添加新的目标平台
- **性能**: 针对快速汇编和代码生成进行优化

## 设计理念

- **统一指令集**：以 .NET IL 为骨架设计的统一指令集
- **无优化**：Gaia 完全不做优化，保持指令的直接映射
- **对象传递**：所有后端都使用对象传递，避免字符串拼接
- **类型安全**：使用 Rust 的类型系统确保编译安全性

## 架构

```
gaia-assembler
├── instruction.rs    # 核心指令集定义
├── backends/         # 各平台后端实现
│   ├── il.rs        # .NET IL 后端
│   ├── jvm.rs       # JVM 字节码后端
│   ├── pe.rs        # PE 可执行文件后端
│   └── wasi.rs      # WASI 后端
└── lib.rs           # 主要 API
```

## 使用方法

### 基本汇编

```rust
use gaia_assembler::{Assembler, TargetPlatform, AssemblyOptions};

// 创建汇编器实例
let mut assembler = Assembler::new(TargetPlatform::JVM);

// 配置汇编选项
let options = AssemblyOptions {
optimize: true,
debug_info: true,
output_format: OutputFormat::Binary,
};

// 汇编源代码
let result = assembler.assemble("source.gaia", & options) ?;
```

### 多平台汇编

```rust
use gaia_assembler::{MultiPlatformAssembler, TargetPlatform};

// 创建多平台汇编器
let mut assembler = MultiPlatformAssembler::new();

// 添加多个目标平台
assembler.add_target(TargetPlatform::JVM);
assembler.add_target(TargetPlatform::NET_IL);
assembler.add_target(TargetPlatform::PE);
assembler.add_target(TargetPlatform::WASI);

// 为所有平台汇编
let results = assembler.assemble_all("source.gaia") ?;
```

## 依赖项

- `gaia-types` - 共享类型定义
- `il-assembler` - .NET IL 汇编器
- `jvm-assembler` - JVM 字节码汇编器
- `pe-assembler` - PE 文件汇编器
- `wasi-assembler` - WASI 汇编器

## API 参考

### 汇编器模块

提供不同目标平台汇编功能的主要汇编器接口。

#### 核心类型

- `Assembler` - 单平台主汇编器类型
- `MultiPlatformAssembler` - 同时支持多平台的汇编器
- `TargetPlatform` - 支持的目标平台枚举（IL、JVM、PE、WASI）
- `AssemblyOptions` - 汇编过程配置

#### 关键方法

- `new(platform: TargetPlatform)` - 为特定平台创建新汇编器
- `assemble(source: &str, options: &AssemblyOptions)` - 汇编源代码
- `add_target(platform: TargetPlatform)` - 向多平台汇编器添加目标平台
- `assemble_all(source: &str)` - 为所有配置的平台汇编

### 平台特定模块

#### IL 模块

- IL 特定汇编逻辑
- .NET 元数据生成
- IL 指令映射

#### JVM 模块

- JVM 字节码生成
- 类文件格式支持
- JVM 指令映射

#### PE 模块

- Windows PE 格式支持
- x86/x64 指令编码
- PE 头生成

#### WASI 模块

- WebAssembly 文本格式支持
- WASI 系统接口集成
- WASM 指令映射

## License

## 许可证

本项目采用 MPL-2.0 许可证授权 - 详见 LICENSE 文件。

## 贡献

欢迎提交贡献！请随时提交拉取请求。