# 格式模块

提供对各种文件格式（如 Java Class 文件、JASM 汇编文件等）的解析、生成和操作支持。

## 概述

该模块是 JVM 汇编器的核心组成部分，负责处理不同格式的输入和输出。它提供了一套统一的接口，用于：

- **解析**: 将特定格式的文件（如 `.class` 文件或 `.jasm` 文件）解析成内部表示。
- **生成**: 将内部表示转换回特定格式的文件。
- **操作**: 提供对这些格式的结构化访问和修改能力。

## 核心功能

### Java Class 文件格式 (`class` 子模块)

- **解析 `.class` 文件**: 将二进制 `.class` 文件解析为 `JvmClass` 结构。
- **生成 `.class` 文件**: 将 `JvmClass` 结构序列化为二进制 `.class` 文件。
- **结构化访问**: 提供对常量池、字段、方法、属性等组件的访问接口。

### JASM 汇编文件格式 (`jasm` 子模块)

- **解析 `.jasm` 文件**: 将文本 `.jasm` 汇编文件解析为 `JvmProgram` 结构。
- **生成 `.jasm` 文件**: 将 `JvmProgram` 结构反汇编为文本 `.jasm` 文件。
- **语法检查**: 提供对 JASM 语法的验证和错误报告。

## 使用示例

### 解析 Class 文件



### 生成 JASM 文件



## 错误处理

模块中的解析和生成函数通常返回 `Result<T, E>`，其中 `E` 是一个包含详细错误信息的错误类型，例如 `ClassFileError` 或 `JasmError`。

## 相关模块

- [`class`](./class/index.html) - Java Class 文件格式的具体实现。
- [`jasm`](./jasm/index.html) - JASM 汇编文件格式的具体实现。
- [`program`](../program/index.html) - 内部 JVM 程序表示。