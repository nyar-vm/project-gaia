# Compilation Target Platform Definition Module

This module defines a triple-combination system for compiler target platforms, similar to LLVM's target triple concept. It is used to accurately describe the characteristics of the target platform for compilation output.

## Triple-Element Architecture

The format is: `build-host-target`

- **build**: Underlying runtime architecture (`Architecture`)
- **host**: Binary format (`AbiCompatible`)
- **target**: API interface (`ApiCompatible`)

## Example Combinations

### Traditional Platforms
- `x86-linux-gnu`: 32-bit x86 architecture, Linux system, GNU toolchain
- `x86_64-windows_pc-msvc`: 64-bit x86 architecture, Windows system, MSVC toolchain
- `x86_64-windows_pc-gnu`: 64-bit x86 architecture, Windows system, GNU toolchain
- `arm64-linux-gnu`: 64-bit ARM architecture, Linux system, GNU toolchain

### Virtual Machine Platforms
- `jvm-unknown-unknown`: JVM architecture, unknown ABI, general API
- `jvm-unknown-jdk8`: JVM architecture, unknown ABI, JDK8 API
- `jvm-jasm-jdk8`: JVM architecture, JASM text format, JDK8 API
- `clr-unknown-net2_0`: CLR architecture, unknown ABI, .NET 2.0 API
- `clr-msil-net4_0`: CLR architecture, MSIL text format, .NET 4.0 API

### WebAssembly Platform
- `wasm32-unknown-unknown`: 32-bit WebAssembly, unknown ABI, general API
- `wasm32-wat-unknown`: 32-bit WebAssembly, WAT text format, general API
- `wasm32-wat-wasi_p2gnu`: 32-bit WebAssembly, WAT text format, WASI P2 GNU interface

## Architecture Types (`Architecture`)

### Physical Architectures
- **X86**: 32-bit x86 architecture
- **X86_64**: 64-bit x86 architecture
- **ARM32**: 32-bit ARM architecture
- **ARM64**: 64-bit ARM/AArch64 architecture
- **RISCV32**: 32-bit RISC-V architecture
- **RISCV64**: 64-bit RISC-V architecture
- **MIPS32**: 32-bit MIPS architecture
- **MIPS64**: 64-bit MIPS architecture
- **WASM32**: 32-bit WebAssembly
- **WASM64**: 64-bit WebAssembly

### Virtual Machine Architectures
- **JVM**: Java Virtual Machine
- **CLR**: .NET Common Language Runtime
- **Other(String)**: Custom architecture name

## ABI Compatibility (`AbiCompatible`)

Represents the binary interface format:

- **Unknown**: Maximum compatibility, virtual machine bytecode or bare-metal machine code
- **ELF**: ELF format (Linux, macOS, etc.)
- **PE**: PE format (Windows)
- **Jasm**: JVM bytecode text format
- **Msil**: CLR bytecode text format
- **WAT**: WebAssembly text format

## API Compatibility (`ApiCompatible`)

Represents the API interface of the target platform:

- **Unknown**: Unknown API, maximum compatibility
- **Msvc**: Microsoft Visual C++ runtime
- **Gnu**: GNU toolchain and glibc
- **JDK(u16)**: Java Development Kit version (e.g., JDK8, JDK11)
- **CLR(u16)**: .NET Common Language Runtime version
- **Unity**: Unity engine API
- **WASI**: WebAssembly System Interface

## Features and Functionality

### Architecture Methods
Creates an architecture from a COFF machine type. Supported COFF machine types:
- `0x014c`: `IMAGE_FILE_MACHINE_I386` → `X86`
- `0x8664`: `IMAGE_FILE_MACHINE_AMD64` → `X86_64`
- `0x01c0`: `IMAGE_FILE_MACHINE_ARM` → `ARM32`
- `0xaa64`: `IMAGE_FILE_MACHINE_ARM64` → `ARM64`
- `0x0166`: `IMAGE_FILE_MACHINE_R4000` → `MIPS32`
- Others: Converted to `Other("machine_XXXX")` format

### Display Implementation
`Architecture` implements the `Display` trait, with the following output formats:
- `X86` → "x86"
- `X86_64` → "x64"
- `ARM32` → "arm"
- `ARM64` → "arm64"
- `RISCV32` → "riscv32"
- `RISCV64` → "riscv64"
- `MIPS32` → "mips"
- `MIPS64` → "mips64"
- `WASM32` → "wasm32"
- `WASM64` → "wasm64"
- `JVM` → "jvm"
- `CLR` → "clr"
- `Other(name)` → name

## Design Characteristics

1. **Triple-Element Separation**: Clearly distinguishes between architecture, ABI, and API layers.
2. **Serialization Support**: Uses `serde` for serialization/deserialization to formats like JSON.
3. **Extensibility**: Supports custom architectures through the `Other` variant.
4. **Version Support**: Parameterized version numbers for JDK and CLR.
5. **Multi-Platform Coverage**: Supports physical architectures, virtual machines, and WebAssembly.
6. **Utility**: Provides utility functions for conversion from COFF machine types.

This design provides a flexible target description mechanism for the compiler, allowing precise specification of target platform characteristics for compilation output.
