# Writer Module

The Writer module provides functionality for writing text and binary data, supporting formatted text output and structured binary data writing.

## Main Components

### TextWriter

A text writer used to generate formatted text output, supporting indentation management and custom indentation text.

#### Features

- **Indentation Management**: Supports nested indentation and automatically handles indentation levels.
- **Custom Indentation**: Configurable indentation text (default is spaces).
- **Formatted Output**: Provides convenient text formatting methods.

#### Functionality

TextWriter provides complete text formatting functionality, including:
- Creating new text writers
- Increasing and decreasing indentation levels
- Writing a line of text (automatic newline)
- Writing text (no newline)
- Getting the current indentation level
- Getting the internal writer

### BinaryWriter

A binary writer used to generate structured binary data, supporting endianness control and type-safe writing operations.

#### Features

- **Endianness Control**: Supports both big-endian and little-endian writing.
- **Type-Safe**: Provides type-safe writing methods.
- **Error Handling**: Comprehensive error handling mechanism.

#### Functionality

BinaryWriter provides complete binary data writing functionality, including:
- Creating new binary writers
- Setting endianness
- Writing various integer types (u8, u16, u32, u64)
- Writing strings (with length prefix)
- Getting the internal writer

## Design Characteristics

- **Type-Safe**: Ensures correct data types at compile time.
- **Zero-Copy**: Minimizes memory allocation and copying.
- **Composable**: Supports method chaining and nested structures.
- **High Performance**: Optimized low-level implementation.
- **Error Handling**: Comprehensive error handling mechanism.

## Usage Scenarios

- Compiler backend code generation
- Data serialization
- Protocol implementation
- File format processing
- Configuration file generation
- Log and report generation
