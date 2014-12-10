# Gaia Types - Lexer Module

This module provides lexical analysis functionality for the Gaia project, responsible for converting source code text into a sequence of tokens.

## Features Overview

### Lexical Analysis
- **Token Identification**: Recognizes keywords, identifiers, literals, operators, etc.
- **Source Code Location**: Accurate source code position information.
- **Error Handling**: User-friendly lexical error reporting.
- **Performance Optimization**: Efficient tokenization algorithms.

### Supported Token Types
- **Keywords**: Keywords for WebAssembly and Gaia extensions.
- **Identifiers**: Variable names, function names, type names, etc.
- **Literals**: Integer, floating-point, and string literals.
- **Operators**: Arithmetic, logical, and comparison operators.
- **Delimiters**: Brackets, commas, semicolons, etc.
- **Comments**: Support for single-line and multi-line comments.

### Source Code Location
- **Line and Column Info**: Accurate line and column numbers.
- **Span Information**: Start and end positions of tokens.
- **File Name**: Source file name information.
- **Context**: Context information when errors occur.

## Design Characteristics

### Performance Optimization
- **Streaming Processing**: Supports streaming input without loading the entire source code.
- **Zero-Copy**: Minimizes string copying operations.
- **Cache-Friendly**: Optimized data structures to improve cache hit rates.
- **Incremental Analysis**: Supports incremental lexical analysis.

### Error Recovery
- **Robustness**: Continues analysis when encountering errors to collect multiple errors.
- **Contextual Information**: Provides detailed error context.
- **Suggested Fixes**: Provides possible fix suggestions.

### Extensibility
- **Custom Keywords**: Supports adding new keywords.
- **Extended Tokens**: Supports custom token types.
- **Plugin Mechanism**: Supports lexical analysis plugins.

## Token Types Details

### Literal Tokens
- **Integers**: Supports decimal, hexadecimal, octal, and binary.
- **Floating-point**: Supports standard and decimal forms.
- **Strings**: Supports escape sequences and Unicode.
- **Identifiers**: Unicode identifier support.

### Operator Tokens
- **Arithmetic**: +, -, *, /, %, etc.
- **Logical**: &&, ||, !, etc.
- **Comparison**: ==, !=, <, >, <=, >=, etc.
- **Bitwise**: &, |, ^, ~, <<, >>, etc.

## Memory Efficiency

The lexer is designed to be memory-efficient, using references and slices to avoid unnecessary memory allocations while maintaining high-performance token identification speed.
