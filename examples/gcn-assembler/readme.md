# GCN Assembler

A specialized assembler library for AMD Graphics Core Next (GCN) architecture, enabling low-level GPU kernel generation.

## 🏛️ Architecture

```mermaid
graph TB
    subgraph "GCN Compilation Flow"
        A[GCN Instructions] --> B[GCN Program Builder]
        B --> C[Instruction Formatter]
        C --> D[ELF Wrapper]
        D --> E[GPU Binary (.bin / .elf)]
        
        subgraph "Instruction Types"
            F[SOP - Scalar Ops]
            G[VOP - Vector Ops]
            H[SMEM - Scalar Memory]
            I[VMEM - Vector Memory]
        end
        
        B --> F
        B --> G
        B --> H
        B --> I
    end
```

## 🚀 Features

### Core Capabilities
- **Architecture Support**: Comprehensive support for GCN ISA (Instruction Set Architecture) covering scalar and vector operations.
- **Kernel Orchestration**: Simplifies the construction of GPU kernels with automatic text section and global symbol generation.
- **Memory Mapping**: Handles complex GCN memory operand formats, including buffer descriptors and displacement offsets.

### Advanced Features
- **ELF Encapsulation**: Automatically wraps generated GCN machine code into an ELF container compatible with AMD's GPU loaders.
- **Instruction Formatting**: Provides a human-readable formatter for debugging and verifying instruction sequences before binary emission.
- **Error Tracking**: Integrated with Gaia's diagnostic system to catch invalid operand combinations or illegal instruction pairings.

## 💻 Usage

### Generating a GCN Kernel
The following example shows how to create a simple GCN program and write it to a binary file.

```rust
use gcn_assembler::{GcnProgram, GcnKernel, GcnInstruction, GcnWriter};
use std::fs;

fn main() {
    // 1. Define instructions (e.g., s_endpgm)
    let instructions = vec![
        GcnInstruction::new("s_endpgm"),
    ];

    // 2. Build kernel and program
    let kernel = GcnKernel {
        name: "my_kernel".to_string(),
        instructions,
    };
    let program = GcnProgram { kernels: vec![kernel] };

    // 3. Write to binary
    let writer = GcnWriter::new();
    let binary = writer.write(&program).expect("Failed to generate GCN binary");
    
    fs::write("kernel.bin", binary).unwrap();
}
```

## 🛠️ Support Status

| Category | Status | Instruction Groups |
| :--- | :---: | :--- |
| Scalar Ops (SOP) | ✅ | SOP1, SOP2, SOPK, SOPC, SOPP |
| Vector Ops (VOP) | ✅ | VOP1, VOP2, VOP3, VOPC |
| Memory Ops | 🚧 | SMEM, VMEM, DS, FLAT |
| ELF Wrapping | ✅ | AMDGPU OSABI |

*Legend: ✅ Supported, 🚧 In Progress, ❌ Not Supported*

## 🔗 Relations

- **[gaia-types](../../projects/gaia-types/readme.md)**: Utilizes basic error handling and binary types for instruction encoding.
- **[gaia-assembler](../../projects/gaia-assembler/readme.md)**: Serves as the native AMD GPU backend for the Gaia project, allowing high-performance offloading of compute tasks.
