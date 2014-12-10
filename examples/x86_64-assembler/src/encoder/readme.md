# InstructionEncoder Module

## Module Responsibilities

`InstructionEncoder` is responsible for converting the `Instruction` enum into an x86/x86_64 machine code byte sequence. The core logic resides in the `encode()` method, which handles the encoding details of different instruction types through pattern matching.

## Key Data Structures

Architecture information primarily affects:
- Register numbering (differences in register mapping between x86 vs x86_64)
- REX prefix generation logic (64-bit operations require a REX prefix)
- Default operand size (32-bit vs 64-bit)

## Core Algorithm Flow

### encode() Main Flow
1. Pattern match the `Instruction` enum
2. Select the encoding function based on operand types
3. Handle encoding differences for register/immediate/memory operands
4. Generate necessary instruction prefixes (REX, operand size, etc.)
5. Combine opcodes and operand encodings

For example:
```rust
use x86_64_assembler::encoder::InstructionEncoder;
use gaia_types::helpers::Architecture;

let encoder = InstructionEncoder::new(Architecture::X86_64);
// Encoding logic is implemented here
```

### Register Encoding Logic
Key function: `encode_register_operand()`
- Handles register number mapping (`register_code()`)
- Handles REX prefix requirements for 64-bit registers
- Handles matching between register size and opcode

### Memory Operand Encoding
Key function: `encode_memory_operand()`
- SIB byte generation logic (Scale-Index-Base)
- Displacement encoding optimization
- Special addressing mode handling (e.g., `[rip+disp32]`)

### Immediate Encoding
Key function: `encode_immediate_operand()`
- Consistency checks between immediate size and operand size
- Special encoding optimizations for small immediates (e.g., `add eax, imm8`)

## Instruction Prefix Generation

### REX Prefix
Generation conditions:
- 64-bit operations (REX.W = 1)
- Accessing extended registers (REX.B/R/X bits)
- In 64-bit mode, the default operand size is 32-bit; a REX prefix is required to enable 64-bit operations

### Operand Size Prefix (0x66)
Generation conditions:
- 16-bit operands in 32/64-bit mode
- Note: In 64-bit mode, 32-bit operations are default and do not require a prefix

## Architecture-Specific Handling

### x86 (32-bit) vs x86_64 (64-bit)
- Register numbering: x86_64 has extended registers R8-R15
- Addressing modes: x86_64 supports RIP-relative addressing
- Default operand size: x86 defaults to 32-bit, x86_64 defaults to 32-bit (requires REX prefix for 64-bit)

### Common Pitfalls
1. **Missing REX prefix**: 64-bit register operations must check for REX requirements
2. **Immediate size confusion**: Immediate size must match operand size
3. **Memory addressing modes**: Some combinations are invalid on specific architectures
4. **Opcode selection**: The same instruction may have multiple opcode forms

## Performance Considerations

### Encoding Optimization
- Prioritize short opcode forms (e.g., `add eax, imm8` vs `add eax, imm32`)
- SIB byte optimization for memory operands (avoid unnecessary SIB)
- Immediate size optimization (use 8-bit instead of 32-bit when possible)

### Memory Allocation
The current implementation creates a new `Vec<u8>` for each encoding. For batch encoding scenarios, consider:
- Pre-allocating buffers
- Reusing encoder instances
- Providing APIs to encode into existing buffers

## Error Handling Strategy

### Encoding Failure Scenarios
- Operand size mismatch (e.g., `mov eax, imm64`)
- Unsupported addressing modes (e.g., `[rax+rbx*8+disp32]` on x86)
- Registers not supported by the architecture (e.g., R8 on x86)

### Error Message Design
Error types should contain sufficient context information to help locate issues:
- Specific instruction type involved
- Failed operand information
- Expected vs. actual parameter values

## Testing Strategy

### Unit Testing Focus
- Basic encoding for each instruction type
- Boundary conditions (maximum/minimum immediates)
- Architecture differences (behavior of the same instruction in x86 vs x86_64)
- Error conditions (invalid operand combinations)

### Regression Testing
- Encoding results for existing instructions should not change
- New instructions must not break existing functionality
- Performance benchmarking (avoid encoding speed degradation)

## Extension Guide

### Adding New Instruction Types
1. Add a new variant to the `Instruction` enum
2. Add the corresponding pattern match branch in `encode()`
3. Implement the specific encoding logic function
4. Add corresponding test cases

### Adding New Operand Types
1. Add a new variant to the `Operand` enum
2. Add handling logic in `encode_operand()`
3. Consider the impact on existing instructions (whether they need updates)

### Architecture Extension
1. Add the new architecture to the `Architecture` enum
2. Update register encoding mappings
3. Adjust prefix generation logic
4. Consider backward compatibility

## Code Organization

### File Structure
- `mod.rs`: Main module, containing the `InstructionEncoder` definition and core encoding logic
- Internal functions organized by operand type: `encode_register_operand()`, `encode_memory_operand()`, etc.

### Naming Conventions
- Encoding functions: `encode_*_operand()`
- Helper functions: `register_code()`, `needs_rex_prefix()`, etc.
- Constants: `REX_PREFIX`, `OPERAND_SIZE_PREFIX`, etc.

## Maintenance Notes

1. **Intel vs AT&T Syntax**: Internal use of Intel syntax (destination, source)
2. **Opcode Reference**: Primarily refer to the Intel manual, noting differences between versions
3. **Endianness**: Immediates and addresses use little-endian byte order
4. **Alignment Requirements**: Current implementation does not consider instruction alignment optimization
