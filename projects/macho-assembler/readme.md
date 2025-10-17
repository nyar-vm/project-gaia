# Mach-O Assembler

支持 x64 和 ARM64 指令集的现代 Mach-O 汇编器 - 强类型、面向对象、零依赖核心

## 架构概览

```mermaid
graph TB
    subgraph "Mach-O 汇编器架构"
        A[Mach-O 生成请求] --> B[Mach-O 构建器]
        B --> C[Mach-O 文件生成器]
        C --> D[macOS 可执行文件]
        
        subgraph "核心组件"
            E[assembler 模块]
            F[writer 模块]
            G[types 模块]
            H[helpers 模块]
        end
        
        A --> E
        E --> F
        F --> G
        E --> H
        F --> H
        
        subgraph "支持的架构"
            I[x86_64 架构]
            J[ARM64 架构]
            K[未来扩展支持]
        end
        
        G --> I
        G --> J
        G --> K
    end
```

### Mach-O 生成流程

```mermaid
sequenceDiagram
    participant Developer
    participant Assembler
    participant MachoBuilder
    participant MachoWriter
    participant MacOSSystem
    
    Developer->>Assembler: 调用 easy_hello_world(X86_64)
    Assembler->>MachoBuilder: 创建 Mach-O 构建器
    MachoBuilder->>MachoBuilder: 添加代码段
    MachoBuilder->>MachoBuilder: 设置加载命令
    MachoBuilder->>MachoWriter: 构建 Mach-O 文件
    MachoWriter->>MacOSSystem: 生成可执行文件
    MacOSSystem->>Developer: 返回 hello_world 可执行文件
```

## 特性

- 🚀 **高性能**: 零依赖核心，优化的二进制读写
- 🔧 **强类型**: 完整的 Rust 类型系统支持
- 📱 **多架构**: 支持 x86_64 和 ARM64 架构
- 🔍 **延迟加载**: 支持按需读取 Mach-O 文件内容
- 📊 **结构化**: 面向对象的 API 设计
- 🛡️ **安全**: 内存安全的 Rust 实现

## 支持的格式

- Mach-O 可执行文件
- Mach-O 动态库 (.dylib)
- Mach-O 静态库 (.a)
- Mach-O 目标文件 (.o)

## 快速开始

```rust
use macho_assembler::*;

// 读取 Mach-O 文件
let config = MachoReadConfig::default();
let reader = config.as_reader(file)?;
let program = reader.read()?;

// 写入 Mach-O 文件
macho_write_path(&program, "output.dylib")?;
```

## 架构支持

- **x86_64**: Intel/AMD 64位处理器
- **ARM64**: Apple Silicon (M1/M2/M3) 处理器