# WASM Reader Module

WebAssembly binary format reader, providing efficient binary parsing functionality.

## Features

- **Binary Parsing**: Parses WASM structures from byte streams.
- **Validation**: Syntax and structural validation.
- **Error Handling**: Detailed error reporting.
- **Performance Optimization**: Efficient parsing algorithms.

## Usage Example

```rust
use wasi_assembler::formats::wasm::reader::WasmReader;
use wasi_assembler::formats::wasm::WasmReadConfig;
use std::fs;
use std::io::Cursor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read WASM file
    let wasm_bytes = vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];
    let cursor = Cursor::new(wasm_bytes);
    
    // Parse WASM structure
    let config = WasmReadConfig { check_magic_head: true };
    let reader = WasmReader::new(cursor, &config);
    let module = reader.get_program()?;
    
    // Access module information
    println!("Number of functions: {}", module.functions.len());
    println!("Number of imports: {}", module.imports.len());
    Ok(())
}
```

## Supported Formats

This reader supports the WebAssembly binary format specification:
- Magic Number: `\0asm` (0x0061736d)
- Version: 0x01 (Version 1)
- All standard section types.

## Error Handling

The reader provides detailed error information:
- Invalid magic number.
- Unsupported version.
- Parsing errors.
- Structural validation failures.
