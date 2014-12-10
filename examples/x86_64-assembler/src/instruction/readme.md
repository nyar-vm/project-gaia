# Instruction Module

## Module Positioning

The `instruction` module defines the core data structures of the assembler: `Register`, `Operand`, and `Instruction` enums. These are the foundational types for subsequent modules like encoder, decoder, and builder.

## Core Data Structures

### Register Enum

**Design Highlights:**
- Grouped by bit width: 8/16/32/64-bit registers have clear naming conventions.
- Extended registers: R8-R15 and their sub-registers are listed separately.
- Historical compatibility: Maintains consistency with traditional x86 register naming.

### Operand Enum
```rust,ignore
pub enum Operand {
    Reg(Register),                    // Register operand
    Imm { value: i64, size: u8 },     // Immediate operand
    Mem { base, index, scale, displacement }, // Memory operand
    Label(String),                    // Label operand
}
```

**Design Trade-offs:**
- `Imm` uses `i64` for storage, covering 8/16/32/64-bit immediate needs.
- `Mem`'s `scale` is restricted to 1/2/4/8, matching x86 addressing mode constraints.
- `Label` uses `String` instead of `&str` to avoid lifetime complexity.

### Instruction Enum
```rust,ignore
pub enum Instruction {
    Mov { dst, src },    // Data transfer
    Push { op },         // Stack operation
    Pop { dst },         
    Add { dst, src },    // Arithmetic operation
    Sub { dst, src },
    Call { target },     // Control flow
    Ret,
    Lea { dst, displacement, rip_relative }, // Address calculation
    Nop,                 // No operation
}
```

**Expansion Strategy:**
- Binary operations (MOV/ADD/SUB) are unified into a `{dst, src}` structure.
- Unary operations (PUSH/POP) clearly define operand roles.
- Special instructions (LEA) handle their unique requirements separately.

## Type Relationship Diagram

```ignore
Instruction
├── Mov/Add/Sub: Requires two Operands
├── Push/Pop: Requires one Operand
├── Call: Requires one Operand (target)
├── Lea: Requires Register + displacement + bool
├── Ret/Nop: No operands
└── Future extensions...

Operand  
├── Reg: References Register
├── Imm: Contains value and size
├── Mem: Contains addressing components
├── Label: String reference

Register
├── Traditional registers (AL/AH/AX/EAX/RAX)
├── Extended registers (R8-R15)
└── Sub-registers (R8B/R8W/R8D)
```

## Design Decision Log

### 1. Immediate Sign Handling
`Imm.value` uses `i64` instead of `u64` because:
- x86 immediates can be negative (e.g., `sub eax, -5`).
- Sign extension is required during encoding; `i64` is more natural.
- However, range checks for unsigned immediates are needed during encoding.

### 2. Memory Operand Design
`Mem` contains four components instead of a simplified version because:
- It supports full `[base + index*scale + displacement]` addressing.
- `scale` is stored separately to avoid runtime calculation.
- `displacement` uses `i32`, covering 8/32-bit displacement needs.

### 3. Special Handling for LEA Instruction
LEA is designed separately instead of reusing the MOV structure because:
- The target of LEA must be a register, not memory.
- It requires a `rip_relative` flag to handle RIP-relative addressing.
- Displacement handling differs from regular memory operands.

## Extension Guide

### Adding New Instruction Types
1. Evaluate the instruction category (Data Transfer/Arithmetic/Control Flow/Special).
2. Determine the number of operands and type constraints.
3. Add a variant with the appropriate structure to the `Instruction` enum.
4. Update the pattern matching in the encoder/decoder.
