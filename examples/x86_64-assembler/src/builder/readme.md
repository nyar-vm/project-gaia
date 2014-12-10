# ProgramBuilder Module

## Design Positioning

`ProgramBuilder` is the high-level API layer of the assembler. The core design philosophy is: **to provide type-safe instruction building interfaces that hide the complexity of underlying instruction creation**. It is not just a simple instruction collector but a semantic-aware program construction coordinator.

## Architectural Decisions

### 1. Fluent Interface vs. Imperative Interface
Method chaining (`builder.mov().push().call()`) was chosen over separate `add_instruction()` calls for the following reasons:
- More intuitive for writing assembly code.
- Most type errors can be caught at compile time.
- Reduces boilerplate code on the user side.

### 2. Specialized Methods vs. Generic Methods
Providing specialized methods (e.g., `mov_reg_imm()`) for common instruction patterns instead of a generic `add_instruction(Mov{...})` because:
- Enforces parameter type checking, reducing runtime errors.
- Method signatures serve as documentation, lowering the learning curve.
- Future optimization logic can be added within specialized methods.

### 3. Immediate Processing vs. Deferred Processing
The current implementation uses an immediate validation strategy (checking at method call time), with potential future support for deferred validation:
- Immediate Validation: Early error detection, simpler debugging.
- Deferred Validation: Supports more complex cross-instruction optimizations.
- The current choice is immediate validation, which can be adjusted based on future requirements.

## Core Data Structures

`ProgramBuilder` is the high-level API layer of the assembler, focusing on providing type-safe instruction building interfaces while hiding underlying complexities.

**Core Components:**
- `architecture`: Source of architectural constraints, used throughout the building process.
- `instructions`: Instruction sequence, maintaining insertion order.
- `data_sections`: Data section management with independent address spaces.

**Design Considerations:**
- `Architecture` serves as an immutable constraint throughout the construction process.
- `Vec<Instruction>` maintains insertion order and supports forward references for labels.
- `DataSection` is managed independently to avoid coupling between instructions and data.

## Method Classification Strategy

### 1. Register Operation Instructions
Unified naming pattern: `<op>_<dst-type>_<src-type>`
- `mov_reg_imm()` - Register ← Immediate
- `mov_reg_reg()` - Register ← Register
- `mov_reg_label()` - Register ← Label address

**Consistency Rules:**
- The destination is always the first parameter (matching x86 operand order).
- Immediate parameters use primitive types (`i64`/`u64`) instead of `Operand`.
- Register parameters use the `Register` enum directly.

### 2. Stack Operation Instructions
Distinction between immediate and register versions:
- `push_imm()` / `push_reg()` / `push_label()`
- `pop_reg()` (Stack operations only have register destinations)

**Architecture Awareness:** Automatically handles differences in stack pointer size between x86 vs. x86_64.

### 3. Control Flow Instructions
Simplified call interfaces:
- `call(target: &str)` - Direct call using a string label.
- `ret()` - Parameterless return.
- Future extensions: `call_reg()` / `call_imm()`.

## Error Handling Strategy

### Validation Levels
1. **Syntactic Layer**: Parameter types and counts (compile-time).
2. **Semantic Layer**: Register architecture compatibility (runtime).
3. **Architectural Layer**: Validity of the instruction on the target architecture (runtime).

### Error Type Design
Provides detailed error messages containing specific context and debugging information.

**Error Characteristics:**
- Includes specific register and architecture information.
- Provides clear descriptions of error contexts.
- Supports layered validation strategies (syntactic, semantic, architectural).
- Error messages contain all information required for debugging.

**Layered Validation Strategy:**
- **Syntactic Layer**: Immediate range checks, register existence validation.
- **Semantic Layer**: Architecture compatibility checks, operand type matching.
- **Architectural Layer**: Instruction set support, privilege level requirements.

## Data Section Management

### Design Philosophy
Data sections have **independent address spaces** and are decoupled from the instruction stream:
- Support for multiple data sections, distinguished by name.
- Alignment options provided for SIMD/cache requirements.
- Data section references are not handled automatically; they are left for the linking phase.

### Alignment Strategy
Data section alignment is provided through specialized APIs, supporting power-of-two alignment requirements.

**Implementation Details:**
- Alignment must be a power of two.
- Alignment requirements are recorded, with actual addresses determined during linking.
- Supports querying for platform-specific maximum alignment values.

## Extension Design

### Path to Adding New Instruction Methods
1. **Analyze Usage Frequency**: Only frequently used instructions deserve specialized methods.
2. **Parameter Complexity**: Priority given to simple parameter combinations.
3. **Architecture Universality**: Added only if supported by both x86 and x86_64.

**Counter-example:** `mov_mem_reg()` is not added because building memory operands is complex.
**Positive-example:** `mov_reg_imm()` is added because it is frequently used and has simple parameters.

## Performance Considerations

### Memory Allocation Strategy
- `instructions: Vec<Instruction>` pre-allocated capacity (common program sizes).
- `data_sections` are typically few, so not pre-optimized.
- Consider providing a `with_capacity(instructions: usize)` constructor.

### Build-time vs. Runtime Trade-off
The current design favors **build-time validation**, sacrificing some runtime performance for:
- Better developer experience (early error discovery).
- Clearer error messages.
- Simpler implementation (no complex optimization passes required).

## Interaction with Other Modules

```text
User Code → ProgramBuilder → Vec<Instruction> → InstructionEncoder → Vec<u8>
                ↓
            Vec<DataSection> → Linker → Final Binary
```

**Responsibility Boundaries:**
- **ProgramBuilder**: Semantic validation + Instruction collection.
- **InstructionEncoder**: Pure encoding, no validation.
- **Linker** (Future): Address resolution + Relocation.

## Common Pitfalls

1. **Architecture Assumptions**: Do not hardcode architectural features (like register numbers) inside methods.
2. **Immediate Ranges**: An `i64` parameter does not mean all values are legal; architecture-aware validation is required.
3. **Label Lifecycles**: String labels are not resolved during the build phase to avoid premature validation.
4. **Method Order**: Maintain alphabetical order for easier lookup and maintenance.
5. **Error Messages**: Include specific parameter values to assist user debugging.
