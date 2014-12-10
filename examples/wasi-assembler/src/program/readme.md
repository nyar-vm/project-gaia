# WASM Program High-Level Abstraction

This module defines a high-level representation of a WASM program, used to provide an intermediate abstraction layer between WAT AST and binary WASM.
Supports WebAssembly Component Model and traditional core modules.

## Core Concepts

### `WasiProgram`

The main program structure, which can represent two types of WebAssembly programs:
- **Core Module** (`WasiProgramType::CoreModule`): Traditional WebAssembly module.
- **Component** (`WasiProgramType::Component`): WebAssembly Component Model component.

### Program Components

The program contains the following main components:
- **Function Types**: Defines the signature of functions (parameters and return types).
- **Functions**: Actual function implementations.
- **Imports**: Functions, memory, etc., imported from external modules.
- **Exports**: Functions, memory, etc., exposed to the outside.
- **Memory**: Linear memory definitions.
- **Tables**: Function table definitions.
- **Global Variables**: Global variable definitions.
- **Custom Sections**: Custom data sections.

### Component Model Support

For component types, it also supports:
- **Component Items**: Type definitions, aliases, instances, etc.
- **Core Modules**: Nested core modules.
- **Instances**: Component or module instances.
- **Aliases**: Name alias definitions.

## Usage Example

### Creating a Simple Core Module

```rust,no_run
use wasi_assembler::program::{
    WasiProgram, WasiProgramType, WasiFunctionType, WasiFunction,
    WasiExport, WasmExportType, WasiInstruction, WasmValueType
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create core module program
    let mut program = WasiProgram::new(WasiProgramType::CoreModule);
    
    // Add function type (i32, i32) -> i32
    program.function_types.push(WasiFunctionType {
        params: vec![WasmValueType::I32, WasmValueType::I32],
        results: vec![WasmValueType::I32],
    });
    
    // Add function implementation
    program.functions.push(WasiFunction {
        type_index: 0, // Use the first function type
        locals: vec![],   // No local variables
        body: vec![
            WasiInstruction::LocalGet { local_index: 0 },  // Get the first parameter
            WasiInstruction::LocalGet { local_index: 1 },  // Get the second parameter
            WasiInstruction::I32Add,       // i32 addition
        ],
    });
    
    // Export function
    program.exports.push(WasiExport {
        name: "add".to_string(),
        export_type: WasmExportType::Function { function_index: 0 },
    });
    
    // Generate WASM bytecode
    let wasm_bytes = program.to_wasm()?;
    Ok(())
}
```

### Creating a Component

```rust,no_run
use wasi_assembler::program::{
    WasiProgram, WasiProgramType, WasiComponentItem,
    WasiTypeDefinition, WasiType, WasiInstance, WasiInstanceType
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create component program
    let mut program = WasiProgram::new(WasiProgramType::Component);
    
    // Add interface type
    program.component_items.push(WasiComponentItem::Type(
        WasiTypeDefinition {
            name: Some("calculator-interface".to_string()),
            index: 0,
            type_content: WasiType::Interface("calculator".to_string()),
        }
    ));
    
    // Add instance
    program.instances.push(WasiInstance {
        name: Some("calc".to_string()),
        index: 0,
        instantiate_target: "calculator-interface".to_string(),
        args: vec![],
        instance_type: WasiInstanceType::Component,
    });
    
    // Generate component bytecode
    let component_bytes = program.to_wasm()?;
    Ok(())
}
```

## Symbol Table Management

The program contains a symbol table used to manage the mapping from names to indices:

```rust,no_run
use wasi_assembler::program::{WasiProgram, WasiProgramType, WasiSymbol, WasiSymbolType};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut program = WasiProgram::new(WasiProgramType::CoreModule);
    
    // The symbol table automatically maintains the mapping from name to index
    program.symbol_table.insert("my_function".to_string(), WasiSymbol {
        name: "my_function".to_string(),
        symbol_type: WasiSymbolType::Function,
        index: 0,
    });
    program.symbol_table.insert("my_memory".to_string(), WasiSymbol {
        name: "my_memory".to_string(),
        symbol_type: WasiSymbolType::Memory,
        index: 0,
    });
    
    // Look up symbol by name
    if let Some(symbol) = program.symbol_table.get("my_function") {
        println!("The index of function my_function is: {}", symbol.index);
    }
    Ok(())
}
```
