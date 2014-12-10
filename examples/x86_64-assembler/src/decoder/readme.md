# InstructionDecoder Module

## Design Positioning

`InstructionDecoder` is the reverse-engineering engine that converts machine code back into instructions. The core design philosophy is: **accurately restore instruction semantics from a byte stream without relying on external information**. It acts as the "reverse gear" of the entire assembler, handling the historical baggage and complexity of the x86 instruction set.

## Architectural Challenges

### Unique Difficulties in x86 Instruction Decoding
1. **Variable-length Prefixes**: 1-4 byte prefixes with complex combination rules.
2. **Operand Size Ambiguity**: The same opcode can have different meanings in different modes.
3. **ModR/M Byte Explosion**: 256 combinations, each with sub-cases.
4. **SIB Byte Nesting**: Ternary addressing consisting of Base + Index × Scale.
5. **Mixed Displacement/Immediates**: Requires precise length calculation; a single byte error can derail the entire process.

### Decoder Design Philosophy
**"Trust no byte"** - Every parsing step must be validated:
- Is the prefix combination legal?
- Is the opcode valid under the current architecture?
- Is the ModR/M mode supported?
- Is the calculated instruction length reasonable?

## Core Algorithm Flow

**Decoding Flow:** The byte stream passes through three stages: prefix parsing, opcode identification, and operand decoding, finally generating an `Instruction` object.

**Intermediate States:** Each stage produces corresponding intermediate structures: `PrefixState` records prefix information, `OpcodeDesc` describes the opcode, and `OperandSpec` defines operand specifications.

### 1. Prefix Parsing Stage
Maintains a `PrefixState` structure and processes prefix bytes according to priority.

**Prefix State Components:**
- `operand_size`: Operand size (16/32/64 bits)
- `address_size`: Address size (16/32/64 bits)
- `segment_override`: Segment prefix override
- `repeat_prefix`: rep/repne repeat prefixes
- `lock`: lock atomic operation prefix

**Key Decision:** Prefix order does not affect semantics but affects encoding length. All prefixes are recorded during decoding and output in standard order during encoding.

### 2. Opcode Identification Stage
Two-level lookup table design:
- **Main Table**: 256 primary opcodes.
- **Extension Table**: Two-byte opcodes starting with 0x0F.
- **Architecture-specific**: Different interpretations for x86 vs. x86_64.

**Ambiguity Handling:** The same opcode maps to different instructions in different modes, requiring a combined judgment of `PrefixState` and the current `Architecture`.

### 3. Operand Decoding Stage
Based on pattern matching of `OpcodeDesc.operand_spec`:

> **Pseudo-code Description** - Operand Specification Enum Concept:
> ```rust
> enum OperandSpec {
>     Reg,        // Implicit register (e.g., AL, AX, EAX)
>     ModRM,      // Determined by ModR/M byte
>     Imm,        // Immediate, size determined by prefix
>     Mem,        // Memory operand, size determined by prefix
>     Rel,        // Relative address, used for jumps
> }
> ```

**ModR/M Operand Specification Parsing**

Operand specifications define the operand representation during the decoding stage.

**Operand Types:**
- **Register Operand**: Uses register encoding directly.
- **Memory Operand**: Complex addressing with Base + Index + Displacement.
- **Immediate Operand**: Constant values embedded within the instruction.
- **Relative Operand**: Jump targets relative to RIP/EIP.

**Key Design:** Retain original encoding information during decoding and reuse original formats as much as possible during encoding.

## Architecture-Specific Handling

### Key Differences between x86 vs. x86_64

| Feature | x86 (32-bit) | x86_64 (64-bit) | Decoding Impact |
|------|------------|---------------|----------|
| Default Operand Size | 32-bit | 32-bit | Requires REX.W to switch to 64-bit |
| Default Address Size | 32-bit | 64-bit | Affects ModR/M displacement calculation |
| Available Registers | 8 | 16 | REX prefix extends register encoding |
| RIP-relative Addressing | Not supported | Supported | New addressing mode added |

### Special Handling for REX Prefix
The REX prefix is the "decoding key" for x86_64:
- **REX.W**: Switches to 64-bit operand size.
- **REX.R**: Extends the `reg` field of ModR/M.
- **REX.X**: Extends the `index` field of SIB.
- **REX.B**: Extends the `r/m` field of ModR/M or register numbering.

**Decoding Strategy:** Set `rex_state` immediately upon encountering a REX prefix; all subsequent parsing refers to this state.

## Error Handling Strategy

### Levels of Decoding Errors
1. **Format Error**: Byte sequence does not conform to any legal instruction pattern.
2. **Architecture Error**: Instruction exists but is not supported in the current architecture (e.g., using 32-bit segment registers in 64-bit mode).
3. **Incompleteness Error**: Byte stream is truncated in the middle of an instruction.
4. **Ambiguity Error**: Theoretically legal but undefined behavior in practice.

### Error Recovery Mechanism
**Current Strategy:** Return immediately upon encountering an error; no recovery attempt.
**Future Considerations:** Provide a "best guess" mode, returning a `PartialInstruction` and skipping problematic bytes to continue.

## Performance Optimization Ideas

### Lookup Table Optimization
- **Main Opcode Table**: 256-entry array, O(1) lookup.
- **Extension Opcode Table**: Hash table, balancing memory and speed.
- **ModR/M Mode Table**: Pre-calculated parsing results for 256 cases.

### Zero-Allocation Strategy
The current design avoids heap allocation:
- Returned `Instruction` is on the stack.
- No dynamic string storage used.
- Displacements/Immediates parsed directly into the struct.

### Cache Friendliness
Consider instruction boundary prediction:
- Record historical instruction length patterns.
- Cache jump targets.
- Prefetch possible next instructions.

## Testing Strategy

### Testing Pyramid
1. **Unit Tests**: Independent testing for each parsing function.
2. **Integration Tests**: Full instruction round-trip (encode → decode → encode).
3. **Boundary Tests**: Architecture switching, prefix combinations, extreme lengths.
4. **Regression Tests**: Add tests for discovered bugs to prevent recurrence.

### Test Data Generation
**Manual Construction**: Covers special cases and boundary conditions.
**Random Generation**: Large-scale random byte sequences to ensure no panics.
**Known Samples**: Standard test cases generated using NASM/YASM.

**Key Test:** Ensure `encode(decode(bytes)) == bytes` holds for all legal instructions.

## Extension Guide

### Adding New Instruction Support
1. **Update Opcode Table**: Add entries to the main or extension tables.
2. **Define Operand Pattern**: Specify the `OperandSpec` sequence.
3. **Architecture Tags**: Note support for x86/x86_64/both.
4. **Add Test Cases**: Especially for architecture-specific differences.

### New Architecture Support
1. **Architecture Detection**: Add new members to the `Architecture` enum.
2. **Opcode Remapping**: Handle different semantics for the same opcode.
3. **Prefix Interpretation**: New architectures may have different prefix meanings.
4. **Register Extension**: Update register encoding parsing logic.

## Maintenance Notes

### Code Organization Principles
- **Group by Functionality**: Separate prefix parsing, opcode identification, and operand decoding.
- **Minimize State**: Pass structures instead of multiple parameters.
- **Early Validation**: Check input validity at each parsing step.

### Common Traps
1. **Displacement Sign Extension**: 8-bit displacements must be correctly sign-extended during decoding.
2. **Immediate Endianness**: x86 is little-endian; pay attention to multi-byte immediates.
3. **RIP-relative Addressing**: Calculation is based on the address of the next instruction.
4. **Segment Override Prefixes**: Affect segment selection for memory operands but do not change instruction semantics.
5. **Lock Prefix**: Only allowed for specific atomic operations.

### Debugging Tips
- **Hexdump Comparison**: Print original bytes versus decoding results for comparison.
- **Stage-by-stage Verification**: Prefix → Opcode → Operands, confirm step-by-step.
- **Reference Implementation**: Compare against the Intel manual and existing assembler behavior.
- **Boundary Testing**: Cover single-byte instructions and maximum-length instructions.

The decoder is the most complex part of the assembler because it must handle decades of accumulated instruction set complexity. Every modification must be made with extreme caution, as a single-bit error can lead to a completely different instruction interpretation.
