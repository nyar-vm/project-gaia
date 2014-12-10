# Python Assembler (Internal Maintenance Document)

This project aims to provide a Rust implementation for deeply understanding and manipulating Python `.pyc` bytecode files. This maintenance document details its design philosophy, module structure, and key maintenance details to help developers better understand and contribute.

## Design Philosophy

- **Modularity and Layering**: The project structure is clear, dividing `.pyc` file reading, parsing, representation, and writing into independent modules for ease of understanding and maintenance.
- **Lossless Operation**: When reading and writing `.pyc` files, we strive for lossless operation—accurately preserving all information from the original file, including headers and `marshal`-serialized `code objects`.
- **Extensibility**: Designed with future extensibility in mind, such as detailed parsing and modification of `code objects` and support for different Python versions' `.pyc` formats.
- **Performance Optimization**: Rust's features enable high performance when processing bytecode while ensuring memory safety.

## Module Structure

- `formats`: Handles different file formats, currently primarily including the `pyc` submodule.
  - `pyc`: Focused on specific processing of Python `.pyc` files, including file structure definitions, readers, writers, and views.
    - `file`: Defines the overall structure of `.pyc` files.
    - `reader`: Responsible for reading and parsing `.pyc` files from byte streams.
    - `writer`: Responsible for writing program representations back to `.pyc` file byte streams.
    - `view`: Provides views of `.pyc` files for convenient access to internal structures.
    - `to_program`: Contains logic for converting `.pyc` views into `PythonProgram`.
- `builder`: Module for constructing `PythonProgram`.
- `helpers`: Provides general auxiliary functions and tools.
- `instructions`: Module for handling Python bytecode instructions.
- `program`: Defines the `PythonProgram` structure, serving as the intermediate representation of `.pyc` files.

## Maintenance Details

- **Dependency Management**: Uses `Cargo.toml` to manage project dependencies, ensuring all dependencies are reviewed and kept up to date.
- **Testing**: The `tests` directory contains comprehensive unit and integration tests to ensure code correctness and stability. Be sure to add corresponding tests when adding new features.
- **Code Style**: Follows Rust's official recommended code style, using `rustfmt` for formatting.
- **Error Handling**: Employs Rust's `Result` type for error handling, ensuring all possible error scenarios are properly addressed.
- **Documentation**: Each module and key structure should have clear documentation comments explaining its functionality, usage, and design considerations.

## Contribution Guide

Community contributions are welcome! If you have any improvement suggestions or find a bug, please feel free to submit a Pull Request or Issue. Before submitting code, ensure your code passes all tests and follows the project's code style guidelines.
