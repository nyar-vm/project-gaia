# WASM 读取器模块

WebAssembly 二进制格式读取器，提供高效的二进制解析功能。

## 功能特性

- **二进制解析**: 从字节流解析 WASM 结构
- **验证**: 语法和结构验证
- **错误处理**: 详细的错误报告
- **性能优化**: 高效的解析算法

## 使用示例

```rust
use wasi_assembler::formats::wasm::reader::WasmReader;
use std::fs;

// 读取 WASM 文件
let wasm_bytes = fs::read("example.wasm")?;

// 解析 WASM 结构
let mut reader = WasmReader::new();
let module = reader.parse(&wasm_bytes)?;

// 访问模块信息
println!("函数数量: {}", module.functions.len());
println!("导入数量: {}", module.imports.len());
```

## 支持的格式

本读取器支持 WebAssembly 二进制格式规范：
- 魔数: `\0asm` (0x0061736d)
- 版本: 0x01 (版本 1)
- 所有标准段类型

## 错误处理

读取器提供详细的错误信息：
- 无效的魔数
- 不支持的版本
- 解析错误
- 结构验证失败