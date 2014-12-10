---
layout: home

hero:
  name: "Gaia Assembler"
  text: "多语言统一接口框架"
  tagline: 为各种编程语言提供统一的多平台编译接口
  actions:
    - theme: brand
      text: 快速开始
      link: /getting-started/
    - theme: alt
      text: 查看后端支持
      link: /backends/

features:
  - icon: 🎯
    title: 多后端支持
    details: 支持 CLR (.NET)、PE (Windows)、ELF (Linux/Unix)、JVM (Java)、WASM (WebAssembly) 等多种目标平台。
  - icon: 🔧
    title: 现代化工具链
    details: 基于 Rust 构建的高性能编译框架，提供完整的开发工具链和调试支持。
  - icon: 🌐
    title: 统一接口设计
    details: 为不同编程语言提供统一的编译接口，简化多平台开发流程。
  - icon: ⚡️
    title: 高性能编译
    details: 优化的编译流程，支持增量编译和并行处理，提供卓越的编译性能。
  - icon: 🛠️
    title: 易于扩展
    details: 模块化架构设计，支持自定义后端和语言前端扩展，满足特殊需求。
  - icon: 🔗
    title: 语言互操作
    details: 支持多种编程语言的互操作，实现跨语言的无缝集成。
---

## 什么是 Gaia Assembler？

Gaia Assembler 是一个现代化的多语言统一接口框架，专为不同编程语言提供统一的多平台编译能力而设计。它抽象了各种目标平台的差异，为开发者提供一致的编译接口。

### 优化边界与职责

Gaia 遵循明确的职责划分，以确保编译流程的高效与清晰：

- **指令透传**: Gaia 默认**不对**前端传入的指令序列进行逻辑优化（如指令重排、强度削减等）。这些“脑力劳动”应当由编译器中端（如 Project Chomsky）完成。
- **物理优化**: 在保持指令语义不变的前提下，Gaia 会进行深度的“物理劳动”，包括：
    - **符号修剪**: 在链接阶段精准剔除未引用的死代码。
    - **内存布局优化**: 自动重排字段以消除对齐空隙（Padding）。
    - **Section 重排**: 根据热点分析优化代码段的物理布局，提升缓存命中率。
    - **重定向压缩**: 在生成二进制文件时选择最紧凑的跳转指令编码。

### 核心特性

- **多后端支持**: 支持 CLR、PE、ELF、JVM、WASM 等多种目标平台
- **高性能编译**: 基于 Rust 的高性能编译器实现
- **统一接口**: 为不同语言提供一致的编译接口
- **模块化设计**: 易于扩展和定制的架构
- **语言无关**: 支持多种编程语言的前端集成

### 支持的平台

| 后端   | 描述               | 状态     |
|------|------------------|--------|
| CLR  | .NET 中间语言 (MSIL) | ✅ 支持   |
| JVM  | Java 字节码         | ✅ 支持   |
| PE   | Windows 可执行文件    | 🚧 开发中 |
| ELF  | Linux/Unix 可执行文件 | 🚧 开发中 |
| WASM | WebAssembly      | 🚧 开发中 |

## 快速开始

```bash
# 克隆项目
git clone https://github.com/nyar-vm/project-gaia.git
cd project-gaia

# 构建项目
cargo build --release

# 运行示例
cargo run --example hello_world
```

## 文档导航

### 用户文档

- [快速开始](/getting-started/) - 安装和基本使用
- [用户指南](/user-guide/) - 详细的使用说明
- [后端文档](/backends/) - 各平台后端支持

### 开发者文档

- [开发者指南](/developer-guide/) - 扩展和定制指南
- [API 参考](/api-reference/) - 详细的 API 文档
- [内部实现](/api-reference/) - 深入的实现细节

### 维护文档

- [维护指南](/maintenance/) - 项目维护信息

## 项目结构

- `gaia-assembler/` - 核心汇编器框架
- `clr-assembler/` - CLR/.NET 后端
- `jvm-assembler/` - JVM/Java 后端
- `pe-assembler/` - PE/Windows 后端
- `wasi-assembler/` - WASM 后端
- `gaia-types/` - 共享类型定义