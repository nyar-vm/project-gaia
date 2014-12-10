# WASI Assembler - Format Processing Module

This module provides processing capabilities for WebAssembly Text format (WAT) and binary format (WASM).

## Module Structure

```text
formats/
├── wat/          # WAT text format processing (reuses oak-wat)
└── wasm/         # WASM binary format processing
    ├── reader/   # Binary reader
    └── writer/   # Binary writer
```

## Feature Overview

### WAT Processing (wat)
- **Syntax Analysis**: Parses WAT text into an Abstract Syntax Tree (AST).
- **Code Generation**: Converts AST back into WAT text.

### WASM Processing (wasm)
- **Binary Reading**: Parses WASM binary format.
- **Binary Writing**: Generates WASM binary format.
- **Format Validation**: Validates the correctness of the binary format.

## Usage Examples

### WAT Processing

```rust
#[cfg(feature = "wat")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use wasi_assembler::formats::wat::{WatParser, WatLanguage};
    use oak_core::parser::{Parser, ParseSession};
    use oak_core::source::SourceText;

    let wat_source = "(module)";
    let source = SourceText::new(wat_source.to_string());
    
    // Syntax analysis
    let language = WatLanguage::default();
    let parser = WatParser::new(&language);
    let mut session = ParseSession::default();
    let output = parser.parse(&source, &[], &mut session);
    
    let ast = output.result?;
    println!("Successfully parsed module");
    
    Ok(())
}

#[cfg(not(feature = "wat"))]
fn main() {
    println!("WAT feature disabled");
}
```

### WASM Processing

```rust
use wasi_assembler::formats::wasm::reader::WasmReader;
use wasi_assembler::formats::wasm::writer::WasmWriter;
use wasi_assembler::formats::wasm::WasmReadConfig;
use std::io::Cursor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let binary_data = vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];
    
    // Read WASM binary
    let config = WasmReadConfig { check_magic_head: true };
    let reader = WasmReader::new(Cursor::new(binary_data), &config);
    let program = reader.get_program()?.clone();
    
    // Write WASM binary
    let mut writer = WasmWriter::new(Vec::new());
    let binary_data = writer.write(program).result?;
    Ok(())
}
```

## Error Handling

All modules use `GaiaDiagnostics` for error handling, supporting:
- Detailed error messages and location tracking.
- Batch collection of multiple errors.
- Categorized handling of warnings and errors.

## Extensibility

The module design supports:
- New WebAssembly features.
- Custom extensions and plugins.
- Multiple output formats.
- Performance optimization and tuning.
