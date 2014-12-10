# LuacReader Module

This module is responsible for reading and parsing `.luac` files (Lua bytecode files).

## Main Responsibilities

1. **Header Parsing**: Read and verify header data such as magic numbers and version information of `.luac` files.
2. **Bytecode Deserialization**: Convert binary bytecode into the internal `LuaProgram` structure.
3. **Lazy Loading**: Use `OnceLock` to implement delayed parsing for improved performance.
4. **Error Handling**: Provide detailed error information and diagnostics.

## Usage Example

```rust,no_run
use lua_assembler::formats::luac::{LuacReadConfig, reader::LuacReader};
use std::fs::File;

// Create configuration
let config = LuacReadConfig::default();

// Open file and create reader
let file = File::open("example.luac")?;
let reader = config.as_reader(file);

// Get file info
let info = reader.get_info()?;
println!("Lua version: {:?}", info.version);

// Parse full program
let result = reader.finish();
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Design Principles

- **Lazy Loading**: Parse file content only when needed.
- **Error Propagation**: Use `Result` types to ensure errors are correctly propagated.
- **Memory Efficiency**: Avoid repeated parsing by using `OnceLock` to cache results.
- **Type Safety**: Use strong typing to ensure data correctness.

## Module Structure

- `LuacReader<'config, R>`: Main reader structure.
- `LuacInfo`: Lightweight structure containing file header info.
- `read_header()`: Parses the `.luac` file header.
- `read_code_object()`: Parses bytecode objects (currently a placeholder implementation).

## Lua Bytecode Format

Basic structure of a `.luac` file:
1. Magic number: `\x1bLua` (4 bytes)
2. Version info (1 byte)
3. Format version (1 byte)
4. Endianness flag (1 byte)
5. Various size information (multiple bytes)
6. Bytecode data

## Maintenance Notes

- The current `read_code_object` method returns default values and needs a full bytecode parsing implementation for specific Lua versions.
- Supporting multiple Lua versions requires adding version-specific logic in `read_header`.
- Error handling should provide enough context information for debugging.
