# WASI 汇编器 - 格式处理模块

这个模块提供了 WebAssembly 文本格式 (WAT) 和二进制格式 (WASM) 的处理功能。

## 模块结构

```
formats/
├── wat/          # WAT 文本格式处理
│   ├── ast/      # 抽象语法树定义
│   ├── lexer/    # 词法分析器
│   ├── parser/   # 语法分析器
│   ├── compiler/ # AST 到程序编译器
│   └── writer/   # WAT 文本生成器
└── wasm/         # WASM 二进制格式处理
    ├── reader/   # 二进制读取器
    └── writer/   # 二进制写入器
```

## 功能概述

### WAT 处理 (wat)
- **词法分析**: 将 WAT 文本分解为标记流
- **语法分析**: 将标记流转换为抽象语法树 (AST)
- **语义分析**: 验证 AST 的语义正确性
- **编译**: 将 AST 编译为内部程序表示
- **代码生成**: 从内部表示生成 WAT 文本

### WASM 处理 (wasm)
- **二进制读取**: 解析 WASM 二进制格式
- **二进制写入**: 生成 WASM 二进制格式
- **格式验证**: 验证二进制格式的正确性

## 使用示例

### WAT 处理

```rust
use wasi_assembler::formats::wat::{lexer, parser, ast};

// 词法分析
let tokens = lexer::tokenize(wat_source)?;

// 语法分析
let ast = parser::parse(tokens)?;

// 编译为程序
let program = compiler::compile(ast)?;
```

### WASM 处理

```rust
use wasi_assembler::formats::wasm::{reader, writer};

// 读取 WASM 二进制
let program = reader::read_wasm(binary_data)?;

// 写入 WASM 二进制
let binary_data = writer::write_wasm(program)?;
```

## 错误处理

所有模块都使用 `GaiaDiagnostics` 进行错误处理，支持：
- 详细的错误信息和位置跟踪
- 多个错误的批量收集
- 警告和错误的分类处理

## 扩展性

模块设计支持：
- 新的 WebAssembly 特性
- 自定义扩展和插件
- 多种输出格式
- 性能优化和调优