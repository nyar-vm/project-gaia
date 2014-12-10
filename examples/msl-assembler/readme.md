# MSL Assembler

A Metal Shading Language (MSL) code generator for the Gaia project, enabling cross-compilation of Gaia IR to high-performance GPU shaders for Apple's Metal framework.

## 🏛️ Architecture

```mermaid
graph TB
    subgraph "Metal Shader Generation"
        A[Gaia Module] --> B[MSL Backend Generator]
        B --> C[Instruction Mapper]
        C --> D[Source Code Emitter]
        D --> E[Metal Source (.metal)]
        D --> F[Metal Library (.metallib)]
        
        subgraph "Metal Context"
            G[metal_stdlib]
            H[Device/Constant Memory]
            I[Compute Kernels]
        end
        
        B --> G
        C --> H
        C --> I
    end
```

## 🚀 Features

### Core Capabilities
- **Direct Source Generation**: Compiles Gaia IR instructions directly into readable C++14-based Metal Shading Language source code.
- **Kernel Abstraction**: Automatically wraps Gaia functions into `kernel void` entry points with standard Metal buffer arguments.
- **Resource Binding**: Manages the mapping of Gaia memory operands to Metal buffer indexes (e.g., `device float* data [[buffer(0)]]`).

### Advanced Features
- **Metal Library Simulation**: Generates placeholder `.metallib` files to satisfy toolchain requirements for offline shader compilation.
- **Standard Library Integration**: Automatically includes `<metal_stdlib>` and configures the `metal` namespace for all generated modules.
- **Unified Backend Interface**: Implements the `Backend` trait, allowing seamless integration with the `gaia-assembler` orchestration layer.

## 💻 Usage

### Generating Metal Shaders via Gaia Assembler
The following example shows how to use the MSL generator as a backend for the main assembler.

```rust
use gaia_assembler::assembler::GaiaAssembler;
use msl_assembler::MslGenerator;
use gaia_types::helpers::{Architecture, AbiCompatible, ApiCompatible, CompilationTarget};

fn main() {
    let assembler = GaiaAssembler::new();
    // Register MSL backend (usually done automatically)
    // ...

    let target = CompilationTarget {
        build: Architecture::X86_64,
        host: AbiCompatible::MSL,
        target: ApiCompatible::Metal,
    };

    // program: GaiaModule
    let generated = assembler.compile(&program, &target).expect("Metal generation failed");
    
    if let Some(source) = generated.files.get("module.metal") {
        println!("Generated MSL source:\n{}", String::from_utf8_lossy(source));
    }
}
```

## 🛠️ Support Status

| Feature | Support Level | Metal Version |
| :--- | :---: | :--- |
| Arithmetic Ops | ✅ Full | 2.0+ |
| Control Flow | ✅ Full | 2.0+ |
| Atomic Ops | 🚧 In Progress | 2.1+ |
| Texture Sampling | ❌ Not Supported | - |
| Ray Tracing | ❌ Not Supported | - |

*Legend: ✅ Supported, 🚧 In Progress, ❌ Not Supported*

## 🔗 Relations

- **[gaia-assembler](../../projects/gaia-assembler/readme.md)**: Acts as a specialized backend that plugs into the assembler's dispatcher for Metal-specific targets.
- **[gaia-types](../../projects/gaia-types/readme.md)**: Uses the `CompilationTarget` and `Architecture` definitions to ensure ABI compatibility.
