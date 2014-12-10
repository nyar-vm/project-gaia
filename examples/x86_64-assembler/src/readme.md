# x86_64-assembler Maintenance Documentation

## Project Overview

This project is a strongly-typed x86/x86_64 assembler implementation developed in Rust. The design goal is to provide reliable, efficient, and easy-to-maintain assembly instruction encoding/decoding functionality.

## Architecture Design

### Core Design Principles
- **Strong Typing First**: Leverage Rust's type system to catch errors at compile time.
- **Zero-Dependency Core**: The core library does not depend on external crates, ensuring stability.
- **Modular Design**: Clear module boundaries to reduce maintenance complexity.
- **Performance Oriented**: Zero allocation on critical paths, cache-friendly design.

### Tech Stack
- **Language**: Rust (Edition 2021)
- **Build Tool**: Cargo
- **Testing Framework**: Built-in tests + documentation tests
- **CI/CD**: GitHub Actions (To be configured)

## Module Architecture

### Module Responsibility Division

#### 1. `assembler` Module - Main Entry Point
- **Responsibility**: Provides a unified public API interface.
- **Key Type**: `X86_64Assembler` - Main assembler struct.
- **Design Consideration**: Facade pattern, hiding internal complexity while providing a clean interface.

#### 2. `builder` Module - High-Level API Layer
- **Responsibility**: Type-safe instruction construction interface.
- **Key Type**: `ProgramBuilder` - Program builder.
- **Design Consideration**: Fluent interface design, compile-time type checking, runtime validation.

#### 3. `instruction` Module - Core Data Structures
- **Responsibility**: Defines the semantic model of the assembly language.
- **Key Type**: `Instruction`, `Operand`, `Register` enums.
- **Design Consideration**: Algebraic Data Types (ADTs), precise expression of x86 semantics, zero-cost abstraction.

#### 4. `encoder` Module - Encoding Engine
- **Responsibility**: Conversion from instructions to bytecode.
- **Key Algorithm**: Two-phase encoding - length calculation + actual encoding.
- **Design Consideration**: Zero-allocation strategy, cache-friendly, branch prediction optimization.

#### 5. `decoder` Module - Decoding Engine
- **Responsibility**: Reverse parsing from bytecode to instructions.
- **Key Algorithm**: Three-phase decoding - prefix parsing + opcode identification + operand decoding.
- **Design Consideration**: Lookup table optimization, state machine design, error recovery mechanism.

## Detailed Module Documentation

Each module contains detailed maintenance documentation, covering:

- **Design Decisions**: Trade-off analysis for architectural choices.
- **Algorithm Description**: Implementation details of key algorithms.
- **Performance Considerations**: Optimization strategies and performance characteristics.
- **Extension Guide**: How to add new features.
- **Testing Strategy**: Test coverage and maintenance requirements.

See the `readme.md` file in each module for details.

## Development Environment Configuration

### Required Tools
- Rust 1.70+ (rustup management recommended)
- Cargo (installed with Rust)
- Git (version control)

### Recommended Development Tools
- rust-analyzer (IDE support)
- cargo-watch (automatic rebuild)
- cargo-tree (dependency analysis)

### Common Commands
```bash
# Build the project
cargo build

# Run tests
cargo test

# Run documentation tests
cargo test --doc

# Generate documentation
cargo doc --open

# Benchmarking
cargo bench

# Check code quality
cargo clippy
```

## Release Process

### Version Management
- Follow Semantic Versioning (SemVer).
- Major version: Incompatible API changes.
- Minor version: Downward compatible functional additions.
- Patch version: Downward compatible bug fixes.

### Pre-release Checklist
- [ ] All tests pass (`cargo test`)
- [ ] Documentation tests pass (`cargo test --doc`)
- [ ] No warnings (`cargo build`)
- [ ] Documentation complete (`cargo doc`)
- [ ] Version number updated (`Cargo.toml`)
- [ ] CHANGELOG updated

## Key Design Decisions

### Architecture Abstraction Strategy
- **Unified Interface**: Single assembler type handles dual architectures, avoiding API split.
- **Runtime Validation**: Architecture checks are performed at runtime, not compile-time, providing flexibility.
- **Error Propagation**: Use `Result<T, GaiaError>` throughout all APIs that may fail.

### Performance Optimization Considerations
- **Zero-Allocation Principle**: Avoid heap allocation on core paths; use stack-based data structures.
- **Lookup Table Optimization**: Encoding/decoding extensively use precomputed tables, O(1) complexity.
- **Cache Friendly**: Compact data structures to improve CPU cache hit rates.

### Maintainability Design
- **Single Responsibility**: Each module focuses on specific functionality, reducing cognitive load.
- **Explicit Errors**: No panic paths; all error conditions are explicitly handled.
- **Documentation Driven**: Public APIs must be documented, and complex algorithms must have implementation descriptions.

## Maintenance Guide

### New Instruction Support Process
1. **Instruction Research**: Consult Intel manuals to confirm opcode and operand formats.
2. **Type Definition**: Add new instruction variants in the `instruction` module.
3. **Encoding Implementation**: Add encoding logic in the `encoder` module.
4. **Decoding Implementation**: Add decoding logic in the `decoder` module.
5. **Builder Integration**: Add convenience methods in the `builder` module (if applicable).
6. **Test Coverage**: Add unit and integration tests to ensure round-trip correctness.

### Architecture Extension Considerations
- **New Architecture Support**: Requires modifying the `Architecture` enum and all architecture-related matches.
- **Register Extension**: Affects the `Register` enum, encoding tables, and decoding logic.
- **Prefix Handling**: New prefixes require modifying `PrefixState` and related parsing logic.

### Common Maintenance Pitfalls
1. **ModR/M Byte**: 256 combinations, each case needs to be tested.
2. **REX Prefix**: Unique to x86_64, easy to miss edge cases.
3. **Displacement Sign Extension**: 8-bit displacements need correct sign extension to 32/64 bits.
4. **Immediate Endianness**: x86 is little-endian; pay attention to the order of multi-byte immediates.
5. **Architecture Differences**: The same instruction may behave differently under different architectures.

## Performance Tuning Suggestions

### Benchmarking
- Use `cargo bench` for performance benchmarking.
- Focus on encoding/decoding throughput.
- Monitor memory allocation (using `valgrind` or similar tools).

### Optimization Directions
- **Lookup Table Size**: Balance memory usage and lookup speed.
- **Branch Prediction**: Reduce conditional branches and improve prediction accuracy.
- **Inlining Strategy**: Consider `#[inline]` attributes for critical path functions.

## Error Handling Strategy

### Error Classification
- **Input Error**: Invalid parameters, unsupported architecture, etc.
- **Encoding Error**: Instruction cannot be encoded, operand mismatch, etc.
- **Decoding Error**: Invalid byte sequence, unsupported instruction, etc.
- **Runtime Error**: Out of memory, inconsistent internal state, etc.

### Error Message Design
- **Specificity**: Include specific error context and parameter values.
- **Actionability**: Provide clear repair suggestions.
- **Consistency**: Unified error format for easy automated processing.
