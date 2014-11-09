# WebAssembly 二进制格式 (WASM) 处理模块

这个模块提供了 WebAssembly 二进制格式的处理功能，包括：
- **读取器**: 从二进制文件解析 WASM 结构
- **写入器**: 将 WASM 结构序列化为二进制格式

## 二进制格式概述

WebAssembly 二进制格式是一种紧凑的二进制格式，设计用于高效解码和执行。
主要特点包括：
- **紧凑性**: 使用 LEB128 编码减少文件大小
- **快速解码**: 设计用于快速解析和执行
- **流式处理**: 支持流式解析和验证
- **安全性**: 内置验证和安全检查

## 模块组件

### `reader` 模块

WASM 二进制文件读取器，提供：
- **二进制解析**: 从字节流解析 WASM 结构
- **验证**: 语法和结构验证
- **错误处理**: 详细的错误报告
- **性能优化**: 高效的解析算法

### `writer` 模块

WASM 二进制文件写入器，提供：
- **二进制生成**: 将 WASM 结构序列化为二进制格式
- **优化**: 代码大小和性能优化
- **兼容性**: 确保生成的二进制文件符合规范
- **调试支持**: 生成调试信息和映射

## 使用示例

### 读取 WASM 文件

```rust,no_run
use wasi_assembler::formats::wasm::WasmReader;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 读取 WASM 文件
    let wasm_bytes = fs::read("example.wasm")?;
    
    // 解析 WASM 结构
    let mut reader = WasmReader::new();
    let module = reader.parse(&wasm_bytes)?;
    
    // 访问模块信息
    println!("函数数量: {}", module.functions.len());
    println!("导入数量: {}", module.imports.len());
    println!("导出数量: {}", module.exports.len());
    Ok(())
}
```

### 生成 WASM 文件

```rust,no_run
use wasi_assembler::formats::wasm::WasmWriter;
use wasi_assembler::program::{WasiProgram, WasiProgramType};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建程序
    let program = WasiProgram::new(WasiProgramType::CoreModule);
    
    // 生成 WASM 字节码
    let mut writer = WasmWriter::new();
    let wasm_bytes = writer.write(&program)?;
    
    // 保存到文件
    std::fs::write("output.wasm", wasm_bytes)?;
    Ok(())
}
```

### 错误处理

```rust,no_run
use wasi_assembler::formats::wasm::{WasmReader, WasmError};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = WasmReader::new();
    match reader.parse(&[0x00, 0x61, 0x73, 0x6d]) { // 无效的魔数
        Ok(module) => {
            // 解析成功
        }
        Err(WasmError::InvalidMagic) => {
            eprintln!("无效的 WASM 魔数");
        }
        Err(WasmError::InvalidVersion) => {
            eprintln!("不支持的 WASM 版本");
        }
        Err(WasmError::ParseError(message)) => {
            eprintln!("解析错误: {}", message);
        }
        Err(e) => {
            eprintln!("其他错误: {}", e);
        }
    }
    Ok(())
}
```

## 二进制格式规范

本实现遵循 WebAssembly 二进制格式规范：
- **魔数**: `\0asm` (0x0061736d)
- **版本**: 0x01 (版本 1)
- **段类型**: 自定义、类型、导入、函数、表、内存、全局、导出、起始、元素、代码、数据
- **编码**: LEB128 变长整数编码
- **验证**: 完整的类型和结构验证