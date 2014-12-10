# Mach-O Dynamic Library Reader

This module implements a lazy-loading reader for Mach-O dynamic library files, following the design pattern of the JVM Class file reader.

## Features

- **Lazy Loading**: Supports reading file content on demand to improve performance.
- **Caching Mechanism**: Uses `OnceCell` to cache parsing results, avoiding redundant parsing.
- **Error Handling**: Complete error diagnostics and reporting.
- **Memory Safety**: Uses `RefCell` to ensure the safety of borrow checking.

## Usage

```rust
use macho_assembler::{MachoReadConfig, DylibReader};

// Create configuration
let config = MachoReadConfig::default();

// Create reader
let reader = config.as_dylib_reader(file)?;

// Get program info (lazy loading)
let program = reader.get_program()?;

// Get file info (lazy loading)
let info = reader.get_info()?;

// Or directly read the entire file
let diagnostics = reader.read();
```

## Architectural Design

- `DylibReader`: Main reader class, implementing lazy loading.
- `DylibInfo`: Basic information view of the dynamic library file.
- Caching mechanism ensures each part is parsed only once.
- Error collection and diagnostic reporting.
