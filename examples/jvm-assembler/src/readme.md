# JVM Assembler Library

`jvm-assembler` is a Rust library for processing Java Virtual Machine (JVM) bytecode and JASM (Java ASseMbler) assembly language. It provides a set of tools for parsing, generating, and manipulating JVM Class files as well as JASM assembly code.

## Main Features

- **JASM Assembler**: Converts JASM assembly code into JVM bytecode.
- **Class File Parser**: Reads and parses JVM Class files, converting them into a structured program representation.
- **Class File Writer**: Serializes program representations into JVM Class files.
- **Helper Tools**: Provides practical utilities for byte manipulation, string processing, numerical conversion, etc.

## Module Structure

- [`formats`](./formats/index.html): Contains parsers and writers for various file formats (such as Class files and JASM).
  - [`class`](./formats/class/index.html): Handles reading, writing, and views for JVM Class files.
  - [`jasm`](./formats/jasm/index.html): Handles lexical analysis, parsing, and writing for the JASM assembly language.
- [`helpers`](./helpers/index.html): Provides general helper functions and utilities.
- [`program`](./program/index.html): Defines the abstract representation of JVM programs, used for conversion between different formats.

## Usage Example

### Assembling JASM into a Class File

### Reading a Class File

## Error Handling

All operations in the library use `gaia_types::Result` and `gaia_types::GaiaError` for error handling, providing detailed error information and context.

## Contribution

Contributions are welcome! Please refer to the project's `CONTRIBUTING.md` file for more information.
