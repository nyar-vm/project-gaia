# Mach-O 后端

Mach-O (Mach Object) 是 Apple 操作系统（包括 macOS、iOS、watchOS 和 tvOS）使用的可执行文件、目标代码、共享库和核心转储的文件格式。

Gaia 框架的 Mach-O 后端提供了生成原生 Apple 平台二进制文件的能力。

## 主要特性

- **格式支持**: 完全支持 Mach-O 64 位格式。
- **文件类型**:
  - **Executable**: 可直接运行的程序。
  - **Dynamic Library (.dylib)**: 动态链接库。
  - **Object File (.o)**: 可重定位的目标文件。
- **架构支持**: 针对 Apple Silicon (M1/M2/M3) 和 Intel x86_64 进行了优化。
- **符号管理**: 支持导出符号、导入符号和本地符号管理。
- **加载命令**: 自动生成必要的加载命令（Load Commands），如 `LC_SEGMENT_64`、`LC_DYLD_INFO_ONLY` 等。

## 使用入门

要使用 Mach-O 后端，您需要在构建配置中指定目标平台为 `apple-darwin` 或相关平台。

```rust
use macho_assembler::types::MachoProgram;
// 构建您的 Mach-O 程序逻辑
```

## 平台兼容性

- **macOS**: 10.15 及更高版本
- **iOS**: 13.0 及更高版本

## 相关概念

- [Mach-O 文件结构](./file-structure.md)
- [动态链接](./concepts.md#dynamic-linking)
- [代码签名](./concepts.md#code-signing)
