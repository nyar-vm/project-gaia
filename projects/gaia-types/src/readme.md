# gaia-types Maintenance Documentation

## Project Overview

gaia-types is the core type system library for the Gaia project, providing foundational functionalities such as unified error handling, architecture abstractions, and compilation target definitions. As a basic component of the entire toolchain, this library uses a zero-dependency design to ensure stability and portability.

## Architecture Design

### Core Design Principles

- **Zero-Dependency Core**: Core functions do not depend on external crates, ensuring a minimal dependency tree.
- **Unified Error Model**: Provides a consistent error handling interface with support for detailed error diagnostics.
- **Architecture Abstraction**: Supports unified abstraction for multiple architectures, facilitating cross-platform development.
- **Type-Safe**: Leverages the Rust type system to prevent common programming errors.
- **Performance-First**: Zero allocations on critical paths, prioritizing stack-based data structures.

### Tech Stack

- **Language**: Rust (Edition 2024)
- **Features**: Uses `#![feature(try_trait_v2)]` for advanced error handling.
- **Serialization**: Optional serde support (controlled via features).
- **Documentation**: Complete documentation test coverage.

## Module Architecture

### Module Responsibility Division

#### 1. `errors` Module - Unified Error Handling System
- **Responsibility**: Provides unified error types and diagnostic information.
- **Key Types**: `GaiaError`, `GaiaErrorKind`, `GaiaDiagnostics`
- **Design Considerations**: 
  - Uses `Box` wrapping to reduce enum size.
  - Supports structured error information.
  - Integrates with the tracing logging system.
  - Provides detailed source code location information.

#### 2. `helpers` Module - Architecture and Target Abstraction
- **Responsibility**: Defines supported architectures and compilation targets.
- **Key Types**: `Architecture`, `CompilationTarget`, `AbiCompatible`, `ApiCompatible`
- **Design Considerations**:
  - Supports physical architectures (x86, ARM, RISC-V, etc.).
  - Supports virtual machine architectures (JVM, CLR).
  - Provides architecture compatibility checks.
  - Serialization support for easy configuration storage.

#### 3. `reader` Module - Binary Reading Abstraction
- **Responsibility**: Provides a unified interface for reading binary data.
- **Key Types**: `BinaryReader`, `SourceLocation`, `SourcePosition`
- **Design Considerations**:
  - Supports both big-endian and little-endian byte orders.
  - Provides detailed error location information.
  - Zero-copy design avoids unnecessary memory allocations.

#### 4. `writer` Module - Text Writing Abstraction
- **Responsibility**: Provides a unified interface for text output.
- **Key Types**: `TextWriter`
- **Design Considerations**:
  - Supports multiple output formats.
  - Provides formatting options.
  - Error handling integration.

#### 5. `lexer` Module - Lexical Analysis Foundation
- **Responsibility**: Provides general lexical analysis functionality.
- **Design Considerations**:
  - Supports multiple token types.
  - Provides detailed lexical error information.

#### 6. `parser` Module - Syntax Analysis Foundation
- **Responsibility**: Provides general syntax analysis functionality.
- **Design Considerations**:
  - Recursive descent parser.
  - Error recovery mechanism.
  - Detailed syntax error information.

#### 7. `assembler` Module - Assembler Foundation
- **Responsibility**: Provides basic functionality related to assembly.
- **Key Types**: `BinaryWriter`
- **Design Considerations**:
  - Supports multiple output formats.
  - Provides assembly error handling.

## Core Types Details

### Error Handling System

#### GaiaError Structure
Provides a unified error handling interface, including error levels and specific error types.

#### GaiaErrorKind Enum
Includes all possible error types: syntax errors, IO errors, architecture errors, compilation errors, configuration errors, etc.

### Architecture Abstraction System

#### Architecture Enum
Supports various physical architectures (x86, ARM, RISC-V, MIPS, WebAssembly) and virtual machine architectures (JVM, CLR).

#### CompilationTarget Structure
Defines the compilation target, including information such as architecture, ABI, and API versions.

## Development Environment Configuration

### Required Tools
- Rust 1.70+ (recommended to manage via rustup)
- Cargo (installed with Rust)

### Common Commands
- `cargo build` - Build the project
- `cargo test` - Run tests
- `cargo test --doc` - Run documentation tests
- `cargo doc --open` - Generate documentation
- `cargo clippy` - Check code quality

## Key Design Decisions

### Error Handling Strategy
1. **Unified Error Type**: All errors are represented by `GaiaError`.
2. **Structured Error Information**: Provides machine-readable error details.
3. **Source Code Location**: Every error includes detailed source code location information.
4. **Error Levels**: Supports different levels such as error, warning, and information.

### Architecture Abstraction Strategy
1. **Enum Representation**: Uses enums to ensure the completeness of architecture types.
2. **Display Implementation**: Provides human-readable architecture names.
3. **Serialization Support**: Facilitates configuration file storage and transmission.
4. **Extensibility**: Supports custom architecture types.

### Performance Optimization Considerations
1. **Zero-Allocation Design**: Avoids heap allocations on core paths.
2. **Box Wrapping**: Uses Box to reduce enum size and improve cache efficiency.
3. **Inline Optimization**: Critical functions use inline attributes.

## Maintenance Guide

### Process for Adding New Error Types
1. **Error Research**: Analyze the scenarios and scope of the error.
2. **Type Definition**: Add new error variants to `GaiaErrorKind`.
3. **Constructor Functions**: Create convenient constructors for new error types.
4. **Error Conversion**: Implement conversion from other error types.
5. **Display Implementation**: Provide human-readable error messages.
6. **Test Coverage**: Add unit tests and documentation tests.

### Process for Adding New Architecture Support
1. **Architecture Research**: Confirm technical specifications and ABI of the new architecture.
2. **Enum Extension**: Add the new architecture to the `Architecture` enum.
3. **Display Implementation**: Update the `Display` trait implementation.
4. **Compatibility Check**: Add architecture compatibility verification logic.
5. **Serialization Support**: Ensure the new architecture can be correctly serialized.
6. **Documentation Update**: Update relevant documentation and examples.

### Common Maintenance Pitfalls
1. **Error Size**: Be careful to control the size of error types to avoid excessive stack allocation.
2. **Architecture Consistency**: Ensure all architecture-related code correctly handles new architectures.
3. **Backward Compatibility**: Consider serialization compatibility when modifying enums.
4. **Documentation Sync**: Update documentation tests promptly after code changes.

## Detailed Module Documentation

Each sub-module contains specific maintenance documentation:

- `errors/readme.md` - Detailed design of the error handling system.
- `helpers/readme.md` - Detailed explanation of architecture abstraction.
- `reader/readme.md` - Implementation details of binary reading.
- `writer/readme.md` - Implementation details of text writing.

## Release Process

### Version Management
- Follow Semantic Versioning (SemVer).
- Major version: Incompatible API changes.
- Minor version: Downward-compatible new functionality.
- Patch version: Downward-compatible bug fixes.

### Pre-release Checklist
- [ ] All tests passed (`cargo test`)
- [ ] Documentation tests passed (`cargo test --doc`)
- [ ] No warnings (`cargo build`)
- [ ] Documentation complete (`cargo doc`)
- [ ] Version number updated (`Cargo.toml`)
- [ ] CHANGELOG updated
