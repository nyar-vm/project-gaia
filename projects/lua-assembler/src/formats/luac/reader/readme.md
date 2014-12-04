# LuacReader 模块

本模块负责读取和解析 `.luac` 文件（Lua 字节码文件）。

## 主要职责

1. **文件头解析**：读取并验证 `.luac` 文件的魔数、版本信息等头部数据
2. **字节码反序列化**：将二进制字节码转换为内部的 `LuaProgram` 结构
3. **懒加载**：使用 `OnceLock` 实现延迟解析，提高性能
4. **错误处理**：提供详细的错误信息和诊断

## 使用示例

```rust,no_run
use lua_assembler::formats::luac::{LuacReadConfig, reader::LuacReader};
use std::fs::File;

// 创建配置
let config = LuacReadConfig::default();

// 打开文件并创建 reader
let file = File::open("example.luac")?;
let reader = config.as_reader(file);

// 获取文件信息
let info = reader.get_info()?;
println!("Lua version: {:?}", info.version);

// 解析完整程序
let result = reader.finish();
# Ok::<(), Box<dyn std::error::Error>>(())
```

## 设计原则

- **延迟加载**：只有在需要时才解析文件内容
- **错误传播**：使用 `Result` 类型确保错误能够正确传播
- **内存效率**：避免重复解析，使用 `OnceLock` 缓存结果
- **类型安全**：使用强类型确保数据的正确性

## 模块结构

- `LuacReader<'config, R>`：主要的读取器结构
- `LuacInfo`：包含文件头信息的轻量级结构
- `read_header()`：解析 `.luac` 文件头
- `read_code_object()`：解析字节码对象（当前为占位符实现）

## Lua 字节码格式

`.luac` 文件的基本结构：
1. 魔数：`\x1bLua` (4 字节)
2. 版本信息 (1 字节)
3. 格式版本 (1 字节)
4. 字节序标识 (1 字节)
5. 各种大小信息 (多个字节)
6. 字节码数据

## 维护说明

- 当前 `read_code_object` 方法返回默认值，需要根据具体的 Lua 版本实现完整的字节码解析
- 支持多个 Lua 版本需要在 `read_header` 中添加版本特定的逻辑
- 错误处理应该提供足够的上下文信息以便调试