# Instruction 模块

## 模块定位

`instruction` 模块定义了汇编器的核心数据结构：`Register`、`Operand` 和 `Instruction` 枚举。这些是后续编码器、解码器、构建器等模块的基础类型。

## 核心数据结构

### Register 枚举

**设计要点：**
- 按位宽分组：8/16/32/64 位寄存器有明确的命名规律
- 扩展寄存器：R8-R15 及其子寄存器单独列出
- 历史兼容：保持与传统 x86 寄存器命名的一致性

### Operand 枚举  
```rust
pub enum Operand {
    Reg(Register),                    // 寄存器操作数
    Imm { value: i64, size: u8 },     // 立即数操作数
    Mem { base, index, scale, displacement }, // 内存操作数
    Label(String),                    // 标签操作数
}
```

**设计权衡：**
- `Imm` 使用 `i64` 存储，覆盖 8/16/32/64 位立即数需求
- `Mem` 的 `scale` 限制为 1/2/4/8，符合 x86 寻址模式约束
- `Label` 用 `String` 而非 `&str`，避免生命周期复杂性

### Instruction 枚举
```rust
pub enum Instruction {
    Mov { dst, src },    // 数据传输
    Push { op },         // 栈操作
    Pop { dst },         
    Add { dst, src },    // 算术运算
    Sub { dst, src },
    Call { target },     // 控制流
    Ret,
    Lea { dst, displacement, rip_relative }, // 地址计算
    Nop,                 // 空操作
}
```

**扩展策略：**
- 二元操作（MOV/ADD/SUB）统一为 `{dst, src}` 结构
- 一元操作（PUSH/POP）明确操作数角色
- 特殊指令（LEA）单独处理其独特需求

## 类型关系图

```
Instruction
├── Mov/Add/Sub: 需要两个 Operand
├── Push/Pop: 需要一个 Operand  
├── Call: 需要一个 Operand (目标)
├── Lea: 需要 Register + 立即数 + bool
├── Ret/Nop: 无操作数
└── 未来扩展...

Operand  
├── Reg: 引用 Register
├── Imm: 包含值和大小
├── Mem: 包含寻址组件
└── Label: 字符串引用

Register
├── 传统寄存器 (AL/AH/AX/EAX/RAX)
├── 扩展寄存器 (R8-R15)
└── 子寄存器 (R8B/R8W/R8D)
```

## 设计决策记录

### 1. 立即数符号处理
`Imm.value` 使用 `i64` 而非 `u64`，原因：
- x86 立即数可以是负值（如 `sub eax, -5`）
- 编码时需要符号扩展，`i64` 更自然
- 但编码时需注意无符号立即数的范围检查

### 2. 内存操作数设计  
`Mem` 包含四个组件而非简化，原因：
- 支持完整的 `[base + index*scale + displacement]` 寻址
- `scale` 单独存储，避免运行时计算
- `displacement` 用 `i32`，覆盖 8/32 位位移需求

### 3. LEA 指令特殊性
LEA 单独设计而非复用 MOV 结构，原因：
- LEA 目标必须是寄存器，不能是内存
- 需要 `rip_relative` 标志处理 RIP 相对寻址
- 位移处理方式与普通内存操作数不同

## 扩展指南

### 新增指令类型
1. 评估指令类别（数据传输/算术/控制流/特殊）
2. 确定操作数数量和类型约束
3. 在 `Instruction` 枚举中添加合适结构的变体
4. 更新编码器/解码器的模式匹配

**示例：新增乘法指令**
```rust
Mul { src: Operand },  // 隐含目标 (EAX/RAX)
Imul { dst: Operand, src: Operand }, // 明确目标
```

### 新增操作数类型  
1. 评估是否现有 `Operand` 变体能满足需求
2. 考虑对编码器/解码器的影响范围
3. 更新 `Operand::fmt` 显示实现
4. 添加相应的构造函数和访问方法

### 寄存器扩展
1. 遵循命名规律（R16/R16B/R16W/R16D）
2. 更新 `register_code()` 映射函数
3. 考虑对 REX 前缀逻辑的影响
4. 测试架构兼容性（x86 vs x86_64）

## 维护注意事项

### 1. 类型一致性
- 操作数大小必须与实际数据匹配（`Imm.size` vs 值范围）
- 寄存器类型必须与架构兼容（R8 不能在 x86 使用）
- 内存寻址组件的有效性检查（如 scale 只能是 1/2/4/8）

### 2. 显示格式稳定性  
`Operand::fmt` 的实现影响调试输出和错误信息：
- 保持 Intel 语法格式（`[base + index*scale + disp]`）
- 正确处理位移的符号显示
- 标签引用保持原始名称

### 3. 序列化兼容性
模块使用 `serde` 派生宏：
- 枚举变体的顺序影响序列化格式
- 字段名称变更会破坏兼容性
- 新增字段需提供默认值

### 4. 性能考虑
- `Register` 实现了 `Copy`，大小为 1 byte
- `Operand` 包含 `String`（Label），不是 `Copy`  
- `Instruction` 大小差异大（Nop vs Mem 操作数）
- 大量指令场景考虑使用 `Box<Instruction>` 减少内存占用

## 常见陷阱

1. **立即数符号扩展**：负立即数在编码时的处理
2. **内存操作数默认值**：`Option<Register>` 的 `None` vs 显式处理
3. **标签生命周期**：`String` 避免了 `&str` 的复杂性但增加了分配
4. **寄存器映射**：新增寄存器必须同步更新编码映射表
5. **架构特异性**：某些指令/寄存器在 x86 vs x86_64 下的差异