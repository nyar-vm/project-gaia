# Main Program Module

This module is the main entry point of the JVM assembler, responsible for coordinating various components (such as the lexer, parser, and writer) to complete the conversion from JASM assembly code to JVM bytecode.

## Main Features

- **Command Line Interface (CLI)**: Handles command-line arguments and configures program behavior.
- **File I/O**: Reads JASM assembly files and writes generated `.class` files.
- **Flow Control**: Orchestrates the entire flow of lexical analysis, parsing, and writing.
- **Error Reporting**: Collects and reports errors and warnings that occur throughout the compilation process.

## Structure

- `run_app`: The program's main function, containing command-line parsing and main logic.
- `config`: Program configuration, such as input/output file paths, compilation options, etc.

## Usage Example
