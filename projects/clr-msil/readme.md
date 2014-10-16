# CLR MSIL 解析器

用于解析 Microsoft 中间语言（MSIL）代码的 Rust 库，为 .NET 程序集提供全面的分析和转换功能。

## 概述

该库提供完整的 MSIL 解析解决方案，可以：

- 解析 MSIL 字节码和文本表示
- 从 MSIL 代码生成抽象语法树（AST）
- 分析和转换 MSIL 指令
- 支持各种 .NET 元数据格式
- 与其他 CLR 工具和库集成

## 特性

- **MSIL 解析**: 将 MSIL 汇编代码解析为结构化 AST
- **AST 生成**: 生成全面的抽象语法树
- **指令分析**: 分析和转换 MSIL 指令
- **元数据支持**: 支持 .NET 元数据和程序集引用
- **错误处理**: 全面的错误报告和恢复
- **性能**: 快速解析，内存开销最小
- **可扩展性**: 模块化设计，易于扩展和自定义

## 项目结构

```
clr-msil/
├── src/
│   ├── ast/           # 抽象语法树定义
│   │   └── mod.rs     # AST节点结构定义
│   ├── lexer/         # 词法分析器
│   │   ├── mod.rs     # 词法分析器实现
│   │   └── token_type.rs # 标记类型定义
│   ├── parser/        # 语法分析器
│   │   └── mod.rs     # 语法分析器实现
│   ├── writer/        # AST到MSIL代码的转换器
│   │   └── mod.rs     # 代码生成器实现
│   ├── easy_test/     # 测试工具
│   │   └── mod.rs     # 测试辅助功能
│   └── lib.rs         # 库入口点
├── Cargo.toml         # 项目配置
└── readme.md          # 项目文档
```

## 安装

将以下内容添加到您的`Cargo.toml`文件中：

```toml
[dependencies]
clr-msil = { path = "../clr-msil" }
```

## 使用方法

### 基本解析示例

```rust
use clr_msil::parser;

// 解析 MSIL 代码
let msil_code = r#"
    .assembly HelloWorld {}
    .module HelloWorld.exe
    
    .class public auto ansi beforefieldinit Program
        extends [mscorlib]System.Object
    {
        .method public static void Main() cil managed
        {
            .entrypoint
            ldstr "Hello, World!"
            call void [mscorlib]System.Console::WriteLine(string)
            ret
        }
    }
"#;

// 解析 MSIL 代码
let ast = parser::parse_msil(msil_code) ?;

// 访问解析后的 AST
println!("解析了 {} 个程序集", ast.assemblies.len());
for assembly in & ast.assemblies {
println!("程序集: {}", assembly.name);
}
```

### 使用配置的高级解析

```rust
use clr_msil::{parser, ParserConfig};

// 配置解析器选项
let config = ParserConfig::builder()
.with_strict_mode(true)
.with_metadata_validation(true)
.with_source_tracking(true)
.build();

// 使用配置进行解析
let ast = parser::parse_msil_with_config(msil_code, config) ?;
```

### 解析MSIL文件

```rust
use std::fs;
use clr_msil::parser::MsilParser;

// 读取MSIL文件
let msil_content = fs::read_to_string("example.msil") ?;

// 直接解析文本
let parser = MsilParser::new();
let result = parser.parse_text( & msil_content) ?;

match result {
Ok(ast) => {
println ! ("成功解析MSIL文件");
// 处理AST
},
Err(diagnostics) => {
eprintln ! ("解析错误: {:?}", diagnostics);
}
}
```

## API参考

### 主要类型

#### MsilParser

MSIL语法分析器的主要结构体：

```rust
pub struct MsilParser {
    config: ParserConfig,
}
```

**方法：**

- `new()` - 创建新的解析器实例
- `parse_text(text: &str)` - 直接解析MSIL文本
- `parse(tokens: Vec<MsilToken>)` - 解析标记流

#### MsilLexer

MSIL词法分析器：

```rust
pub struct MsilLexer;
```

**方法：**

- `new()` - 创建新的词法分析器实例
- `tokenize(text: &str)` - 将文本分解为标记流

#### AST结构

主要的AST节点类型：

- `MsilRoot` - AST根节点，包含所有语句
- `MsilStatement` - 语句枚举（程序集、模块、类等）
- `MsilClass` - 类定义
- `MsilMethod` - 方法定义
- `MsilInstruction` - MSIL指令

### 解析器模块

`parser` 模块提供主要的解析功能：

- `parse_msil(source: &str) -> Result<Ast, ParseError>`: 解析 MSIL 源代码
- `parse_msil_with_config(source: &str, config: ParserConfig) -> Result<Ast, ParseError>`: 使用配置选项进行解析
- `validate_msil(source: &str) -> Result<(), ParseError>`: 验证 MSIL 语法而不进行完整解析

### AST 结构

生成的 AST 包括：

- `Assembly`: 顶级程序集定义
- `Module`: 模块声明和引用
- `Class`: 包含方法和字段的类定义
- `Method`: 带有 IL 指令的方法实现
- `Instruction`: 单个 IL 指令和操作数

### 配置选项

解析器配置包括：

- `strict_mode`: 启用严格的语法验证
- `metadata_validation`: 验证程序集引用和元数据
- `source_tracking`: 跟踪源位置以进行错误报告
- `preserve_comments`: 在 AST 中包含注释

### 解析方法

#### parse_text方法

解析MSIL源代码文本，返回解析结果或诊断信息。

**参数：**

- `text: &str` - MSIL源代码

**返回值：**

- `GaiaDiagnostics<MsilRoot>` - 解析结果或错误信息

#### parse方法

解析标记流，构建抽象语法树。

**解析过程：**

1. 处理程序集外部引用（`.assembly extern`）
2. 处理程序集声明（`.assembly`）
3. 处理模块声明（`.module`）
4. 处理类声明（`.class`）
5. 处理方法定义（`.method`）

#### parse_method方法

解析方法声明，包括修饰符、返回类型、参数列表和方法体。

**解析步骤：**

1. 处理方法修饰符（public、static、specialname等）
2. 解析返回类型
3. 获取方法名
4. 解析参数列表
5. 处理调用约定
6. 解析方法体
7. 构建方法AST节点

#### parse_method_body方法

解析方法体内容，包括堆栈大小、入口点和IL指令。

**解析步骤：**

1. 处理`.maxstack`指令
2. 处理`.entrypoint`指令
3. 解析局部变量声明
4. 解析IL指令序列
5. 构建方法体AST节点

## 示例

### 解析简单的Hello World程序

```rust
use clr_msil::parser::MsilParser;

let msil = r#
.assembly HelloWorld {}

.class public HelloWorld
{
.method public static void Main() cil managed
{
.entrypoint
.maxstack 8
ldstr "Hello, World!"
call void [mscorlib]System.Console::WriteLine(string)
ret
}
}
"#;

let parser = MsilParser::new();
let result = parser.parse_text(msil)?;

if let Ok(ast) = result {
    for statement in &ast.statements {
        println!("语句: {: ?}", statement);
    }
}
```

### 处理解析错误

```rust
use clr_msil::parser::MsilParser;

let invalid_msil = ".invalid instruction";
let parser = MsilParser::new();
let result = parser.parse_text(invalid_msil) ?;

match result {
Ok(ast) => {
// 成功解析
process_ast(ast);
},
Err(diagnostics) => {
// 处理错误
for diagnostic in & diagnostics.errors {
eprintln ! ("错误: {} at {:?}", diagnostic.message, diagnostic.location);
}
}
}
```

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

## 许可证

本项目采用 Mozilla 公共许可证 2.0 授权 - 详见 [License.md](../../License.md) 文件。

## 贡献

欢迎贡献！请随时提交拉取请求。

## 项目结构

```
clr-msil/
├── src/
│   ├── lexer.rs      # 标记化逻辑
│   ├── parser.rs     # 解析和 AST 生成
│   ├── ast.rs        # AST 节点定义
│   ├── errors.rs     # 错误处理
│   └── lib.rs        # 库入口点
├── tests/            # 集成测试
└── Cargo.toml        # Rust 包配置
```

## 路线图

- [ ] 完成 MSIL 指令集支持
- [ ] 添加语义分析功能
- [ ] 改进错误消息和诊断
- [ ] 添加更全面的测试覆盖
- [ ] 创建详细的文档和示例