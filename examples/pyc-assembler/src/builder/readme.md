# Builder Module

The `builder` module is responsible for constructing `PythonProgram` objects. It assembles various components (such as instructions, constants, etc.) into a complete program structure that represents Python bytecode.

## Structure

- `PythonBuilder`: A struct used to progressively build a `PythonProgram`.

## Example

```rust
use python_assembler::{
    builder::PythonBuilder,
    program::{PycHeader, PythonObject, PythonProgram},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let builder = PythonBuilder::new();
    let header = PycHeader::default();
    let program = builder.build(header);

    assert_eq!(program.code_object.source_name, "<string>".to_string());
    Ok(())
}
```
