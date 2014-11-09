# Tests

This directory contains tests for the IL Assembly Rust library.

## Running Tests

To run all tests, use the following command:

```bash
cargo test
```

## Test Examples

### Hello World Example

The following assembly code creates a simple "Hello World" executable:

```asm
; hello_world.asm
  BITS 64                 ; change to 64-bit mode
  GLOBAL main
  SECTION .data
    hello db "Hello World!", 10 ; 10 is the ASCII code for newline
  SECTION .text
  main:
    ; write "Hello World!" to stdout
    mov eax, 1            ; system call for write
    mov edi, 1            ; file descriptor for stdout
    mov rsi, hello        ; pointer to string to write
    mov edx, 13           ; length of string to write
    syscall               ; invoke the system call
    ; exit with status code 0
    mov eax, 60      ; system call number for exit
    xor edi, edi     ; exit status code (0)
    syscall          ; invoke the system call
```

### Using the Library API

Instead of writing assembly directly, you can use the library's API:

```rust
use il_assembler::assembler;

// Create a PE file that outputs "Hello, World!" to the console
let text = "Hello, World!";
let pe_data = assembler::easy_console_log(text.to_string());

// Write to a file
std::fs::write("hello_world.exe", pe_data).unwrap();
```

### Exit Code Example

```rust
use il_assembler::assembler;

// Create a PE file that exits with code 42
let exit_code = 42;
let pe_data = assembler::easy_exit_code(exit_code);

// Write to a file
std::fs::write("exit_example.exe", pe_data).unwrap();
```

## Test Structure

The tests are organized as follows:

- `main.rs`: Contains the main test suite
- `readme.md`: This file, containing test examples and documentation

## Adding New Tests

To add new tests:

1. Create a new test function in `main.rs`
2. Add documentation and examples to this file
3. Ensure your test covers both success and error cases

## Test Coverage

The current test suite covers:

- Basic PE file generation
- Console output functionality
- Exit code functionality
- Import table generation

Future tests should cover:

- PE file reading (when implemented)
- Error handling
- Advanced PE features
- Performance benchmarks