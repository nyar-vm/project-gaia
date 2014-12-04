# Mach-O 动态库读取器

本模块实现了 Mach-O 动态库文件的延迟加载读取器，参考了 JVM Class 文件读取器的设计模式。

## 特性

- **延迟加载**: 支持按需读取文件内容，提高性能
- **缓存机制**: 使用 OnceCell 缓存解析结果，避免重复解析
- **错误处理**: 完整的错误诊断和报告
- **内存安全**: 使用 RefCell 确保借用检查的安全性

## 使用方式

```rust
use macho_assembler::{MachoReadConfig, DylibReader};

// 创建配置
let config = MachoReadConfig::default();

// 创建读取器
let reader = config.as_dylib_reader(file)?;

// 获取程序信息（延迟加载）
let program = reader.get_program()?;

// 获取文件信息（延迟加载）
let info = reader.get_info()?;

// 或者直接读取整个文件
let diagnostics = reader.read();
```

## 架构设计

- `DylibReader`: 主要的读取器类，实现延迟加载
- `DylibInfo`: 动态库文件的基本信息视图
- 缓存机制确保每个部分只解析一次
- 错误收集和诊断报告