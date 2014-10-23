# JVM JASM Assembler/Disassembler

JVM assembler and disassembler library for bytecode manipulation, supporting both jasm and jcod formats.

## Overview

This library provides comprehensive JVM bytecode manipulation capabilities, allowing you to:

- Assemble JVM bytecode from text format (jasm/jcod)
- Disassemble JVM bytecode to text format
- Convert between different JVM bytecode representations
- Validate bytecode structure and semantics

## Usage

## 特性

- **字节码汇编**: 将基于文本的 JVM 汇编代码转换为字节码
- **字节码反汇编**: 将字节码转换为人类可读的文本格式
- **格式支持**: 支持 jasm（Java 汇编）和 jcod（Java 代码）格式
- **验证**: 验证字节码结构和语义
- **跨平台**: 在任何支持 Rust 的平台上运行
- **集成**: 使用 gaia-types 提供一致的类型定义

## 使用方法

### 基本汇编示例

```rust
use jvm_jasm::assembler;

// 将 jasm 代码汇编为字节码
let jasm_code = r#"
    public class HelloWorld {
        public static void main(String[] args) {
            System.out.println("Hello, World!");
        }
    }
"#;

let bytecode = assembler::assemble_jasm(jasm_code) ?;
println!("字节码已生成: {} 字节", bytecode.len());
```

### 基本反汇编示例

```rust
use jvm_jasm::disassembler;

// 将字节码反汇编为 jasm 格式
let bytecode = vec![/* 字节码数据 */];
let jasm_code = disassembler::disassemble_to_jasm( & bytecode) ?;
println!("JASM 代码:\n{}", jasm_code);
```

## Java 到 Class 到 JASM 工作流程

### 1. 将 Java 源代码编译为 Class 文件

```bash
# 将 HelloJava.java 编译为 HelloJava.class
javac HelloJava.java
```

这将生成包含 JVM 字节码的 `HelloJava.class`。

### 2. 生成 JASM 格式（Java 汇编）

使用 asmtools.jar 将 class 文件反汇编为可读的 JVM 汇编：

```bash
# 生成 jasm 格式（Java 汇编）
java -jar asmtools.jar jdis HelloJava.class > HelloJava.jasm
```

`.jasm` 文件包含人类可读的 JVM 字节码汇编，显示：

- 类结构和方法
- 字节码指令
- 栈和局部变量信息
- 常量池引用

### 3. 生成 JCOD 格式（Java 代码）

使用 asmtools.jar 生成详细的字节码表示：

```bash
# 生成 jcod 格式（详细字节码）
java -jar asmtools.jar jdec HelloJava.class > HelloJava.jcod
```

`.jcod` 文件包含：

- 完整的常量池定义
- 详细的函数字节码
- 访问标志和属性
- 行号表

### 4. 使用 Rust API 进行反汇编与汇编

除了命令行工具，你也可以使用本库提供的 Rust API 来完成相同的流程：

```rust
use jvm_jasm::{disassembler, assembler};
use std::fs;

// 读取编译后的 class 文件
let class_bytes = fs::read("HelloJava.class") ?;

// 反汇编为 JASM 格式
let jasm_code = disassembler::disassemble_to_jasm( & class_bytes) ?;
fs::write("HelloJava_api.jasm", jasm_code) ?;

// 修改 JASM 代码后重新汇编
let modified_jasm = fs::read_to_string("HelloJava_api.jasm") ?
.replace("HelloJava", "HelloJavaModified");
let new_class = assembler::assemble_jasm( & modified_jasm) ?;
fs::write("HelloJavaModified.class", new_class) ?;
```

### 示例文件

完成上述步骤后，你将得到：

- `HelloJava.java` - 原始 Java 源代码
- `HelloJava.class` - 编译后的 JVM 字节码
- `HelloJava.jasm` - 人类可读的汇编格式（asmtools 生成）
- `HelloJava_api.jasm` - 人类可读的汇编格式（Rust API 生成）
- `HelloJava.jcod` - 详细的字节码表示

### ASM 工具使用

`asmtools.jar` 提供的工具：

- `jdis` - Java 反汇编器（生成 jasm）
- `jdec` - Java 解码器（生成 jcod）
- `jasm` - Java 汇编器
- `jcoder` - Java 代码生成器

获取帮助：

```bash
java -jar asmtools.jar --help
```

## API 参考

### 汇编器模块

`assembler` 模块提供汇编 JASM 代码的功能：

- `assemble_jasm(source: &str) -> Result<Vec<u8>, AssemblyError>`: 将 JASM 代码汇编为字节码
- `assemble_jcod(source: &str) -> Result<Vec<u8>, AssemblyError>`: 将 JCOD 代码汇编为字节码
- `validate_bytecode(bytecode: &[u8]) -> Result<(), ValidationError>`: 验证字节码结构

### 反汇编器模块

`disassembler` 模块提供反汇编字节码的功能：

- `disassemble_to_jasm(bytecode: &[u8]) -> Result<String, DisassemblyError>`: 将字节码反汇编为 JASM 格式
- `disassemble_to_jcod(bytecode: &[u8]) -> Result<String, DisassemblyError>`: 将字节码反汇编为 JCOD 格式
- `get_class_info(bytecode: &[u8]) -> Result<ClassInfo, DisassemblyError>`: 从字节码中提取类信息

### 错误处理

该库提供全面的错误处理：

- `AssemblyError`: 汇编过程中发生的错误
- `DisassemblyError`: 反汇编过程中发生的错误
- `ValidationError`: 字节码验证过程中发生的错误

## License

## 许可证

本项目采用 Mozilla 公共许可证 2.0 授权 - 详见 [LICENSE](LICENSE) 文件。

## 贡献

欢迎贡献！请随时提交拉取请求。

## 项目结构

```
jvm-jasm/
├── src/
│   ├── assembler.rs    # 字节码汇编功能
│   ├── disassembler.rs # 字节码反汇编功能
│   ├── errors.rs       # 错误处理
│   └── lib.rs          # 库入口点
├── tests/              # 集成测试
├── examples/           # 使用示例
└── Cargo.toml          # Rust 包配置
```

## 路线图

- [ ] 完成汇编器实现
- [ ] 完成反汇编器实现
- [ ] 添加全面的测试覆盖
- [ ] 添加性能基准测试
- [ ] 创建详细的文档和示例