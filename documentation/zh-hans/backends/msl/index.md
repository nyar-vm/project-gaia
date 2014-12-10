# MSL (Metal) 后端

MSL (Metal Shading Language) 是 Apple 为 Metal 图形和计算框架设计的编程语言。

Gaia 的 MSL 后端可以将逻辑编译为可在 Apple 芯片（M1, M2, M3, A-series）上高效运行的着色器。

## 主要特性

- **现代语法**: 基于 C++14 标准，易于编写和维护。
- **统一内存支持**: 充分利用 Apple 芯片的统一内存架构。
- **图形管线集成**: 完美支持 Metal 渲染和计算管线。
- **自动绑定**: 自动处理缓冲区、纹理和采样器的绑定。

## 支持平台

- **macOS**
- **iOS / iPadOS**
- **tvOS**

## 示例

```rust
use msl_assembler::MslGenerator;
```

## 开发者指南

- [Apple Metal 开发者文档](https://developer.apple.com/metal/)
- [MSL 语言规范](https://developer.apple.com/metal/Metal-Shading-Language-Specification.pdf)
