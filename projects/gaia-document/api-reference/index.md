# API 参考

本节提供 Gaia Assembler 各个组件的详细 API 文档。

## 核心库

### [gaia-assembler](./gaia-assembler/)

核心汇编器库，提供主要的编译功能。

- **Compiler**: 主编译器接口
- **Backend**: 后端抽象接口
- **Instruction**: 指令定义
- **Module**: 模块管理

### [gaia-types](./gaia-types/)

共享类型定义库，包含所有核心数据结构。

- **Value**: 值类型定义
- **Type**: 类型系统
- **Symbol**: 符号表
- **Error**: 错误类型

### [gaia-frontend](./gaia-frontend/)

前端解析器库，负责源码解析和 AST 生成。

- **Lexer**: 词法分析器
- **Parser**: 语法分析器
- **AST**: 抽象语法树
- **SourceMap**: 源码映射

## 后端库

### [pe-assembler](./backends/pe-assembler/)

Windows PE 格式后端。

- **PEBackend**: PE 后端实现
- **PEHeader**: PE 文件头
- **Section**: 节区管理
- **Import**: 导入表

### [clr-assembler](./backends/clr-assembler/)

.NET CLR 后端。

- **CLRBackend**: CLR 后端实现
- **Assembly**: 程序集
- **Method**: 方法定义
- **Metadata**: 元数据

### [jvm-assembler](./backends/jvm-assembler/)

Java JVM 后端。

- **JVMBackend**: JVM 后端实现
- **ClassFile**: 类文件
- **ConstantPool**: 常量池
- **Bytecode**: 字节码

### [wasi-assembler](./backends/wasi-assembler/)

WebAssembly WASI 后端。

- **WASIBackend**: WASI 后端实现
- **Module**: WASM 模块
- **Function**: 函数定义
- **Memory**: 内存管理

## 工具库

### [gaia-lsp](./gaia-lsp/)

语言服务器协议实现。

- **LanguageServer**: LSP 服务器
- **Diagnostics**: 诊断信息
- **Completion**: 代码补全
- **Hover**: 悬停信息

### [gaia-cli](./gaia-cli/)

命令行工具。

- **CLI**: 命令行接口
- **Config**: 配置管理
- **Commands**: 子命令
- **Output**: 输出格式

## 快速参考

### 基本使用

```rust
use gaia_assembler::{Compiler, Backend};
use pe_assembler::PEBackend;

// 创建编译器
let mut compiler = Compiler::new();

// 添加后端
let backend = PEBackend::new();
compiler.add_backend("pe", Box::new(backend));

// 编译源码
let result = compiler.compile("source.gasm", "pe")?;
```

### 自定义后端

```rust
use gaia_assembler::{Backend, Instruction, Result};

struct MyBackend;

impl Backend for MyBackend {
    fn emit_instruction(&mut self, instr: &Instruction) -> Result<()> {
        // 实现指令生成
        Ok(())
    }
    
    fn finalize(&mut self) -> Result<Vec<u8>> {
        // 生成最终输出
        Ok(vec![])
    }
}
```

### 错误处理

```rust
use gaia_types::{Error, Result};
use miette::{Diagnostic, SourceSpan};

#[derive(Debug, Diagnostic)]
#[error("Compilation failed")]
struct CompileError {
    #[source_code]
    src: String,
    
    #[label("Error occurred here")]
    span: SourceSpan,
}
```

## 版本兼容性

| 版本    | Rust 版本 | 状态  |
|-------|---------|-----|
| 0.3.x | 1.70+   | 开发中 |
| 0.2.x | 1.65+   | 维护  |
| 0.1.x | 1.60+   | 已停止 |

## 特性标志

### gaia-assembler

- `default`: 默认特性
- `pe-backend`: PE 后端支持
- `clr-backend`: CLR 后端支持
- `jvm-backend`: JVM 后端支持
- `wasm-backend`: WASM 后端支持
- `lsp`: 语言服务器支持
- `cli`: 命令行工具

### 示例配置

```toml
[dependencies]
gaia-assembler = { version = "0.3", features = ["pe-backend", "clr-backend"] }
gaia-types = "0.3"
```

## 更多资源

- [用户指南](/user-guide/) - 使用说明和示例
- [开发者指南](/developer-guide/) - 扩展和定制
- [内部实现](/internals/) - 实现细节
- [GitHub 仓库](https://github.com/nyar-vm/project-gaia)