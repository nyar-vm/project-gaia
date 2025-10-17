# lua-assembler

用于读取/写入 Lua `.luac` 字节码文件的 Rust 实现。

## 架构概览

```mermaid
graph TB
    subgraph "Lua 汇编器架构"
        A[Lua 字节码请求] --> B[Lua 字节码读取器]
        B --> C[Lua 字节码写入器]
        C --> D[.luac 文件输出]
        
        subgraph "核心组件"
            E[assembler 模块]
            F[reader 模块]
            G[writer 模块]
            H[helpers 模块]
        end
        
        A --> E
        E --> F
        F --> G
        E --> H
        F --> H
        
        subgraph "Lua 版本支持"
            I[Lua 5.1]
            J[Lua 5.2]
            K[Lua 5.3]
            L[Lua 5.4]
        end
        
        G --> I
        G --> J
        G --> K
        G --> L
    end
```

### Lua 字节码处理流程

```mermaid
sequenceDiagram
    participant Developer
    participant Assembler
    participant LuaReader
    participant LuaAnalyzer
    participant LuaWriter
    participant LuaRuntime
    
    Developer->>Assembler: 调用 read_and_write("test.luac")
    Assembler->>LuaReader: 读取 Lua 字节码文件
    LuaReader->>LuaAnalyzer: 分析字节码结构
    LuaAnalyzer->>LuaAnalyzer: 验证文件完整性
    LuaAnalyzer->>LuaWriter: 无损回写字节码
    LuaWriter->>LuaRuntime: 生成 test_copy.luac
    LuaRuntime->>Developer: 返回处理结果
```

## 功能

- 读取 `.luac` 文件：解析文件头（魔数、版本信息等）并保留字节码主体
- 写入 `.luac` 文件：按原样写回头与主体，实现无损回写
- 命令行工具：支持 `lua-assembler <in.luac> <out.luac>` 进行回写验证

## 快速开始

### 构建

```bash
cargo build -p lua-assembler
```

### 生成示例 .luac（需要已安装 Lua）

```bash
# 创建测试 Lua 源文件
mkdir -p tests/luac_src
echo 'print("hello from luac")' > tests/luac_src/hello.lua

# 编译为字节码
luac -o tests/luac_src/hello.luac tests/luac_src/hello.lua
```

### 回写并执行验证

```bash,no_run
cargo run -p lua-assembler -- tests/luac_src/hello.luac tests/luac_src/out.luac
lua tests/luac_src/out.luac
# 预期输出：hello from luac
```

## API 示例

```rust,no_run
use std::path::Path;
use lua_assembler::formats::luac::{read_luac_file, write_luac_file};

let luac = read_luac_file(Path::new("tests/luac_src/hello.luac")).unwrap();
write_luac_file(Path::new("tests/luac_src/out.luac"), &luac).unwrap();
# Ok::<(), Box<dyn std::error::Error>>(())
```

## 说明

`.luac` 主体为 Lua 的字节码序列化数据，本库当前不完全解析主体，仅做基本的头部验证与无损读写；这足以用于执行验证与后续扩展。