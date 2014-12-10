# Mach-O Type Definitions

This module contains all core type definitions for the Mach-O file format.

## Main Types

### MachoHeader
Mach-O file header, containing basic information about the file:
- Magic number
- CPU type and subtype
- File type
- Load command information

### LoadCommand
Base load command structure, used to describe how to load and link various parts of the file.

### SegmentCommand64
64-bit segment command, defining the layout and attributes of memory segments.

### Section64
64-bit section structure, defining detailed information for each section within a segment.

### MachoProgram
Complete Mach-O program structure, containing all necessary components.

## Supported Architectures

- x86_64: Intel/AMD 64-bit processors
- ARM64: Apple Silicon processors
- ARM64e: ARM64 with pointer authentication
- i386: 32-bit Intel processors
- ARM: 32-bit ARM processors

## Supported File Types

- Object: Object files (.o)
- Execute: Executable files
- Dylib: Dynamic libraries (.dylib)
- Bundle: Dynamically loaded bundles
- Various other Mach-O file types
