# Lua Assembler

A versatile toolchain for the Lua programming language, supporting the assembly and disassembly of Lua bytecode across different versions (5.1 - 5.4).

## 🏛️ Architecture

```mermaid
graph TB
    subgraph "Lua Bytecode Toolchain"
        A[Lua Source / Bytecode] --> B[Lua Program Model]
        B --> C[Bytecode Reader/Writer]
        C --> D[Luac View / AST]
        D --> E[Lua Binary (.luac)]
        
        subgraph "Core Engines"
            F[Instruction Builder]
            G[Constant Pool Manager]
            H[Upvalue Resolver]
        end
        
        C --> F
        C --> G
        C --> H
    end
```

## 🚀 Features

### Core Capabilities
- **Version Compatibility**: Handles bytecode format variations between Lua 5.1, 5.2, 5.3, and 5.4.
- **Bi-directional Conversion**: Supports both compiling Lua IR to binary and decompiling `.luac` files back into a structured program model.
- **Instruction Encoding**: Precise encoding of Lua opcodes (iABC, iABx, iAsBx, iAx) including register allocation metadata.

### Advanced Features
- **Header Customization**: Flexible configuration of Lua bytecode headers (endianness, integer size, float size, etc.) for cross-platform compatibility.
- **Constant Management**: Efficient handling of Lua's multi-type constant pools (nil, boolean, number, string).
- **Prototyping**: Support for nested function prototypes and complex upvalue mapping.

## 💻 Usage

### Reading and Writing Lua Bytecode
The following example shows how to read a precompiled Lua file and write it back with a different configuration.

```rust
use lua_assembler::formats::luac::{LuacReadConfig, LuacWriteConfig};
use std::fs::File;

fn main() {
    // 1. Read .luac file
    let input_file = File::open("hello.luac").expect("Failed to open input");
    let read_config = LuacReadConfig::default();
    let reader = read_config.as_reader(input_file);
    let program = reader.read_program().expect("Failed to read program");

    // 2. Modify or Analyze program (optional)
    println!("Main function has {} instructions", program.main_function.instructions.len());

    // 3. Write back to .luac with custom config
    let output_file = File::create("hello_copy.luac").expect("Failed to create output");
    let write_config = LuacWriteConfig::default();
    let writer = write_config.as_writer(output_file);
    writer.write_program(&program).expect("Failed to write program");
}
```

## 🛠️ Support Status

| Lua Version | Bytecode Reading | Bytecode Writing | Instruction Set |
| :--- | :---: | :---: | :--- |
| Lua 5.1 | ✅ | ✅ | Full |
| Lua 5.2 | ✅ | ✅ | Full |
| Lua 5.3 | ✅ | ✅ | Full |
| Lua 5.4 | ✅ | ✅ | Full |
| Luajit | 🚧 | 🚧 | Partial |

*Legend: ✅ Supported, 🚧 In Progress, ❌ Not Supported*

## 🔗 Relations

- **[gaia-types](../../projects/gaia-types/readme.md)**: Uses the binary I/O primitives to handle the platform-dependent header sizes in Lua bytecode.
- **[gaia-assembler](../../projects/gaia-assembler/readme.md)**: Serves as a target for Gaia IR when compiling for Lua-based environments (e.g., game engines, embedded scripting).
