# WebAssembly Binary Format (WASM) Processing Module

This module provides processing functionality for the WebAssembly binary format, including:
- **Reader**: Parses WASM structures from binary files.
- **Writer**: Serializes WASM structures into binary format.

## Binary Format Overview

The WebAssembly binary format is a compact binary format designed for efficient decoding and execution.
Key features include:
- **Compactness**: Uses LEB128 encoding to reduce file size.
- **Fast Decoding**: Designed for rapid parsing and execution.
- **Streaming**: Supports streaming parsing and validation.
- **Security**: Built-in validation and security checks.

## Module Components

### `reader` Module

WASM binary file reader, providing:
- **Binary Parsing**: Parses WASM structures from byte streams.
- **Validation**: Syntax and structural validation.
- **Error Handling**: Detailed error reporting.
- **Performance Optimization**: Efficient parsing algorithms.

### `writer` Module

WASM binary file writer, providing:
- **Binary Generation**: Serializes WASM structures into binary format.
- **Optimization**: Code size and performance optimization.
- **Compatibility**: Ensures generated binary files comply with specifications.
- **Debug Support**: Generates debug information and mappings.

## Usage Example

### Reading a WASM File

```rust,no_run
use wasi_assembler::formats::wasm::reader::WasmReader;
use wasi_assembler::formats::wasm::WasmReadConfig;
use std::fs;
use std::io::Cursor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read WASM file
    let wasm_bytes = fs::read("example.wasm")?;
    let cursor = Cursor::new(wasm_bytes);
    
    // Parse WASM structure
    let config = WasmReadConfig { check_magic_head: true };
    let reader = WasmReader::new(cursor, &config);
    let module = reader.get_program()?;
    
    // Access module information
    println!("Number of functions: {}", module.functions.len());
    println!("Number of imports: {}", module.imports.len());
    println!("Number of exports: {}", module.exports.len());
    Ok(())
}
```

### Generating a WASM File

```rust,no_run
use wasi_assembler::formats::wasm::writer::WasmWriter;
use wasi_assembler::program::{WasiProgram, WasiProgramType};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create program
    let program = WasiProgram::new(WasiProgramType::CoreModule);
    
    // Generate WASM bytecode
    let mut writer = WasmWriter::new(Vec::new());
    let wasm_bytes = writer.write(program).result?;
    
    // Save to file
    std::fs::write("output.wasm", wasm_bytes)?;
    Ok(())
}
```

### Error Handling

```rust,no_run
use wasi_assembler::formats::wasm::reader::WasmReader;
use wasi_assembler::formats::wasm::WasmReadConfig;
use std::io::Cursor;
use gaia_types::GaiaError;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let wasm_bytes = vec![0x00, 0x61, 0x73, 0x6d];
    let cursor = Cursor::new(wasm_bytes);
    let config = WasmReadConfig { check_magic_head: true };
    let reader = WasmReader::new(cursor, &config);
    
    match reader.get_program() {
        Ok(module) => {
            // Parse successful
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
    Ok(())
}
```

## Binary Format Specification

This implementation follows the WebAssembly binary format specification:
- **Magic Number**: `\0asm` (0x0061736d)
- **Version**: 0x01 (Version 1)
- **Section Types**: Custom, Type, Import, Function, Table, Memory, Global, Export, Start, Element, Code, Data
- **Encoding**: LEB128 variable-length integer encoding.
- **Validation**: Comprehensive type and structural validation.
