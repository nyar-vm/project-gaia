# Formats Module

Provides support for parsing, generating, and manipulating various file formats (such as Java Class files, JASM assembly files, etc.).

## Overview

This module is a core component of the JVM assembler, responsible for handling input and output of different formats. It provides a unified set of interfaces for:

- **Parsing**: Parsing files in specific formats (such as `.class` or `.jasm` files) into internal representations.
- **Generation**: Converting internal representations back into specific format files.
- **Manipulation**: Providing structured access and modification capabilities for these formats.

## Core Features

### Java Class File Format (`class` sub-module)

- **Parse `.class` files**: Parse binary `.class` files into a `JvmClass` structure.
- **Generate `.class` files**: Serialize `JvmClass` structures into binary `.class` files.
- **Structured Access**: Provide access interfaces to components such as the constant pool, fields, methods, attributes, etc.

### JASM Assembly File Format (`jasm` sub-module)

- **Parse `.jasm` files**: Parse text-based `.jasm` assembly files into a `JvmProgram` structure.
- **Generate `.jasm` files**: Disassemble `JvmProgram` structures into text-based `.jasm` files.
- **Syntax Checking**: Provide validation and error reporting for JASM syntax.

## Usage Example

### Parsing a Class File

### Generating a JASM File

## Error Handling

Parsing and generation functions in the module typically return `Result<T, E>`, where `E` is an error type containing detailed error information, such as `ClassFileError` or `JasmError`.

## Related Modules

- [`class`](./class/index.html) - Specific implementation for Java Class file format.
- [`jasm`](./jasm/index.html) - Specific implementation for JASM assembly file format.
- [`program`](../program/index.html) - Internal JVM program representation.
