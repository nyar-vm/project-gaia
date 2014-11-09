# WASM 写入器模块

WebAssembly 二进制格式写入器，将 WASM 结构序列化为二进制格式。

## 功能特性

- **二进制生成**: 将 WASM 结构序列化为二进制格式
- **优化**: 代码大小和性能优化
- **兼容性**: 确保生成的二进制文件符合规范
- **调试支持**: 生成调试信息和映射

## 使用示例

```rust
use wasi_assembler::formats::wasm::writer::WasmWriter;
use wasi_assembler::program::{WasiProgram, WasiProgramType};

// 创建程序
let program = WasiProgram::new(WasiProgramType::CoreModule);

// 生成 WASM 字节码
let mut writer = WasmWriter::new();
let wasm_bytes = writer.write(&program)?;

// 保存到文件
std::fs::write("output.wasm", wasm_bytes)?;
```

## 输出格式

生成的二进制文件严格遵循 WebAssembly 规范：
- 标准魔数和版本号
- 优化的段布局
- LEB128 编码
- 完整的验证信息

## 性能优化

写入器包含多项优化：
- 最小化文件大小
- 高效的编码算法
- 内存使用优化