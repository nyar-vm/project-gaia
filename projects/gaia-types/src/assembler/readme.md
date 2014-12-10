# Assembler Module

This module provides functionality for binary data assembly and text writing, primarily used for converting high-level data structures into binary or text formats.

## Main Components

### BinaryAssembler
BinaryAssembler provides efficient binary data writing functionality, supporting different endianness and various data types. It features type safety, performance optimization, and cross-platform compatibility.

### TextWriter
TextWriter provides flexible text writing functionality, supporting indentation control, line writing, and formatted output. It is suitable for generating structured text data such as code and configuration files.

## Design Characteristics

- **Type-Safe**: Ensures correct data types at compile time.
- **Zero-Copy**: Minimizes memory allocation and copying.
- **Composable**: Supports method chaining and nested structures.
- **High Performance**: Optimized low-level implementation.

## Usage Scenarios

- Compiler backend code generation
- Data serialization
- Protocol implementation
- File format processing
