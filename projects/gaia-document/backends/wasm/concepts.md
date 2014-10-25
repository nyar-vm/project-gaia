# WASM 基础概念

本文档介绍 WebAssembly 的核心概念和基础知识，帮助您理解 WASM 的工作原理和设计思想。

## 什么是 WebAssembly？

WebAssembly (WASM) 是一种低级的类汇编语言，具有紧凑的二进制格式，可以在现代 Web 浏览器中以接近原生的性能运行。它被设计为
C/C++、Rust 等高级语言的编译目标。

### 核心特性

- **🚀 高性能**: 接近原生代码的执行速度
- **🔒 安全**: 在沙箱环境中运行，内存安全
- **🌐 跨平台**: 在所有主流浏览器和平台上运行
- **📦 紧凑**: 高效的二进制格式，快速加载
- **🔧 可移植**: 与平台无关的字节码

## WASM 的设计目标

### 1. 性能

WASM 被设计为接近原生性能：

- 直接映射到机器码
- 最小的运行时开销
- 高效的内存访问模式

### 2. 安全性

WASM 在安全的沙箱环境中执行：

- 内存隔离
- 控制流完整性
- 类型安全

### 3. 可移植性

WASM 在不同平台上提供一致的行为：

- 平台无关的指令集
- 确定性执行
- 标准化的 API

## 核心概念

### 1. 模块 (Module)

模块是 WASM 的基本部署单位，包含：

```rust
pub struct WasmModule {
    pub types: Vec<FuncType>,        // 函数类型定义
    pub functions: Vec<Function>,    // 函数实现
    pub tables: Vec<Table>,          // 函数表
    pub memories: Vec<Memory>,       // 线性内存
    pub globals: Vec<Global>,        // 全局变量
    pub exports: Vec<Export>,        // 导出项
    pub imports: Vec<Import>,        // 导入项
    pub start: Option<FuncIdx>,      // 启动函数
}
```

**示例**:

```rust
use gaia_assembler::backends::wasm::*;

let mut assembler = WasmAssembler::new();
assembler.set_module_name("my_module");

// 模块现在包含基本结构
let module_bytes = assembler.build() ?;
```

### 2. 值类型 (Value Types)

WASM 支持四种基本值类型：

```rust
#[derive(Clone, Debug, PartialEq)]
pub enum ValType {
    I32,    // 32位有符号整数
    I64,    // 64位有符号整数
    F32,    // 32位IEEE 754浮点数
    F64,    // 64位IEEE 754浮点数
}
```

**类型特性**:

- **I32**: 用于指针、索引、布尔值
- **I64**: 用于大整数、时间戳
- **F32**: 单精度浮点计算
- **F64**: 双精度浮点计算

**示例**:

```rust
// 定义函数类型：(i32, f64) -> i64
let func_type = assembler.add_function_type(
vec![ValType::I32, ValType::F64],  // 参数类型
vec![ValType::I64]                 // 返回类型
);
```

### 3. 函数 (Functions)

函数是 WASM 中的执行单元：

```rust
pub struct Function {
    pub type_idx: TypeIdx,           // 函数类型索引
    pub locals: Vec<ValType>,        // 局部变量
    pub body: Vec<Instruction>,      // 指令序列
}
```

**函数组成**:

- **参数**: 函数输入值
- **局部变量**: 函数内部变量
- **指令体**: 执行逻辑
- **返回值**: 函数输出值

**示例**:

```rust
// 创建一个计算平方的函数
let square_type = assembler.add_function_type(
vec![ValType::I32],  // 参数: i32
vec![ValType::I32]   // 返回: i32
);

let square_func = assembler.add_function(square_type);
square_func.add_instructions( & [
Instruction::LocalGet(0),    // 获取参数
Instruction::LocalGet(0),    // 再次获取参数
Instruction::I32Mul,         // 相乘
Instruction::End,            // 函数结束
]);
```

### 4. 线性内存 (Linear Memory)

WASM 使用线性内存模型：

```rust
pub struct Memory {
    pub min: u32,          // 最小页数 (64KB/页)
    pub max: Option<u32>,  // 最大页数 (可选)
    pub shared: bool,      // 是否共享 (多线程)
}
```

**内存特性**:

- **线性地址空间**: 从 0 开始的连续地址
- **页式管理**: 以 64KB 为单位分配
- **动态增长**: 可以在运行时扩展
- **边界检查**: 自动防止越界访问

**示例**:

```rust
// 添加 1MB 内存 (16页)
assembler.add_memory(Memory {
min: 16,           // 1MB 初始大小
max: Some(256),    // 16MB 最大大小
shared: false,     // 非共享内存
});

// 内存操作指令
let load_func = assembler.add_function(load_type);
load_func.add_instructions( & [
Instruction::I32Const(0),        // 内存地址 0
Instruction::I32Load(MemArg {    // 加载 32位整数
align: 2,                    // 4字节对齐
offset: 0,                   // 偏移量
}),
Instruction::End,
]);
```

### 5. 表 (Tables)

表存储函数引用，支持间接调用：

```rust
pub struct Table {
    pub element_type: RefType,  // 元素类型
    pub limits: Limits,         // 大小限制
}

pub enum RefType {
    FuncRef,    // 函数引用
    ExternRef,  // 外部引用
}
```

**表的用途**:

- **间接函数调用**: 动态调用函数
- **函数指针**: 实现回调机制
- **虚函数表**: 支持面向对象编程

**示例**:

```rust
// 创建函数表
let table = assembler.add_table(Table {
element_type: RefType::FuncRef,
limits: Limits {
min: 10,           // 最少10个元素
max: Some(100),    // 最多100个元素
},
});

// 间接调用
let call_indirect_func = assembler.add_function(call_type);
call_indirect_func.add_instructions( & [
Instruction::I32Const(5),        // 参数
Instruction::I32Const(0),        // 表索引
Instruction::CallIndirect(       // 间接调用
func_type_idx,               // 函数类型
table.index()                // 表索引
),
Instruction::End,
]);
```

### 6. 全局变量 (Globals)

全局变量存储模块级别的状态：

```rust
pub struct Global {
    pub global_type: GlobalType,  // 全局变量类型
    pub init: ConstExpr,          // 初始化表达式
}

pub struct GlobalType {
    pub val_type: ValType,  // 值类型
    pub mutable: bool,      // 是否可变
}
```

**全局变量特性**:

- **模块级作用域**: 在整个模块中可见
- **可变性控制**: 可以是常量或变量
- **初始化**: 必须在编译时确定初值

**示例**:

```rust
// 创建可变全局变量
let counter_global = assembler.add_global(Global {
global_type: GlobalType {
val_type: ValType::I32,
mutable: true,
},
init: ConstExpr::I32Const(0),  // 初始值为 0
});

// 访问全局变量
let increment_func = assembler.add_function(increment_type);
increment_func.add_instructions( & [
Instruction::GlobalGet(counter_global.index()),  // 获取当前值
Instruction::I32Const(1),                        // 常量 1
Instruction::I32Add,                             // 加 1
Instruction::GlobalSet(counter_global.index()),  // 设置新值
Instruction::End,
]);
```

## 执行模型

### 1. 栈机模型

WASM 使用栈机执行模型：

```
栈顶 -> [值3] [值2] [值1] <- 栈底

指令执行过程：
1. 从栈中弹出操作数
2. 执行操作
3. 将结果压入栈
```

**示例执行过程**:

```rust
// 计算 (10 + 20) * 3
Instruction::I32Const(10),   // 栈: [10]
Instruction::I32Const(20),   // 栈: [10, 20]
Instruction::I32Add,         // 栈: [30]
Instruction::I32Const(3),    // 栈: [30, 3]
Instruction::I32Mul,         // 栈: [90]
```

### 2. 控制流

WASM 支持结构化控制流：

```rust
pub enum ControlInstruction {
    Block(BlockType),     // 代码块
    Loop(BlockType),      // 循环
    If(BlockType),        // 条件分支
    Else,                 // 否则分支
    End,                  // 结束标记
    Br(LabelIdx),         // 跳转
    BrIf(LabelIdx),       // 条件跳转
    Return,               // 返回
}
```

**控制流示例**:

```rust
// if-else 结构
let condition_func = assembler.add_function(condition_type);
condition_func.add_instructions( & [
Instruction::LocalGet(0),        // 获取条件
Instruction::If(BlockType::Empty), // 如果条件为真
Instruction::I32Const(1),    // 返回 1
Instruction::Else,               // 否则
Instruction::I32Const(0),    // 返回 0
Instruction::End,                // 结束 if
Instruction::End,                // 结束函数
]);

// 循环结构
let loop_func = assembler.add_function(loop_type);
loop_func.add_instructions( & [
Instruction::I32Const(0),        // 初始计数器
Instruction::Loop(BlockType::Empty), // 开始循环
Instruction::I32Const(1),    // 常量 1
Instruction::I32Add,         // 计数器 + 1
Instruction::LocalTee(0),    // 保存到局部变量
Instruction::I32Const(10),   // 循环条件
Instruction::I32LtS,         // 小于比较
Instruction::BrIf(0),        // 条件跳转到循环开始
Instruction::End,                // 结束循环
Instruction::End,                // 结束函数
]);
```

### 3. 函数调用

WASM 支持直接和间接函数调用：

```rust
// 直接调用
Instruction::Call(func_idx),

// 间接调用
Instruction::CallIndirect(type_idx, table_idx),
```

**调用约定**:

1. 参数从栈中弹出（逆序）
2. 执行函数体
3. 返回值压入栈

## 模块系统

### 1. 导入 (Imports)

模块可以导入外部功能：

```rust
pub struct Import {
    pub module: String,    // 模块名
    pub name: String,      // 导入名
    pub desc: ImportDesc,  // 导入描述
}

pub enum ImportDesc {
    Function(TypeIdx),     // 导入函数
    Table(TableType),      // 导入表
    Memory(MemoryType),    // 导入内存
    Global(GlobalType),    // 导入全局变量
}
```

**导入示例**:

```rust
// 导入 JavaScript 函数
assembler.add_import(Import {
module: "env".to_string(),
name: "console_log".to_string(),
desc: ImportDesc::Function(console_log_type),
});

// 导入浏览器内存
assembler.add_import(Import {
module: "js".to_string(),
name: "memory".to_string(),
desc: ImportDesc::Memory(MemoryType {
limits: Limits { min: 1, max: None },
shared: false,
}),
});
```

### 2. 导出 (Exports)

模块可以导出功能供外部使用：

```rust
pub struct Export {
    pub name: String,      // 导出名
    pub desc: ExportDesc,  // 导出描述
}

pub enum ExportDesc {
    Function(FuncIdx),     // 导出函数
    Table(TableIdx),       // 导出表
    Memory(MemoryIdx),     // 导出内存
    Global(GlobalIdx),     // 导出全局变量
}
```

**导出示例**:

```rust
// 导出函数
assembler.add_export(Export {
name: "calculate".to_string(),
desc: ExportDesc::Function(calc_func.index()),
});

// 导出内存
assembler.add_export(Export {
name: "memory".to_string(),
desc: ExportDesc::Memory(memory.index()),
});
```

## 类型系统

### 1. 值类型验证

WASM 在编译时进行严格的类型检查：

```rust
// 类型匹配的操作
Instruction::I32Const(42),    // 栈: [i32]
Instruction::I32Const(8),     // 栈: [i32, i32]
Instruction::I32Add,          // 栈: [i32] ✓ 正确

// 类型不匹配的操作
Instruction::I32Const(42),    // 栈: [i32]
Instruction::F32Const(3.14),  // 栈: [i32, f32]
Instruction::I32Add,          // ❌ 错误：类型不匹配
```

### 2. 函数签名验证

函数调用必须匹配签名：

```rust
// 函数类型：(i32, i32) -> i32
let add_type = assembler.add_function_type(
vec![ValType::I32, ValType::I32],
vec![ValType::I32]
);

// 正确的调用
Instruction::I32Const(10),    // 参数 1
Instruction::I32Const(20),    // 参数 2
Instruction::Call(add_func),  // 调用函数 ✓

// 错误的调用
Instruction::I32Const(10),    // 只有一个参数
Instruction::Call(add_func),  // ❌ 参数数量不匹配
```

## 安全模型

### 1. 内存安全

WASM 提供内存安全保证：

- **边界检查**: 自动检查内存访问边界
- **类型安全**: 防止类型混淆攻击
- **沙箱隔离**: 无法访问宿主系统内存

### 2. 控制流完整性

WASM 确保控制流的完整性：

- **结构化控制流**: 只能使用 block、loop、if 等结构
- **标签验证**: 跳转目标必须有效
- **栈平衡**: 确保栈状态一致

### 3. 资源限制

WASM 运行时可以限制资源使用：

```rust
// 内存限制
let memory = Memory {
min: 1,
max: Some(100),  // 最大 6.4MB
shared: false,
};

// 表大小限制
let table = Table {
element_type: RefType::FuncRef,
limits: Limits {
min: 0,
max: Some(1000),  // 最多 1000 个函数引用
},
};
```

## 性能特性

### 1. 编译时优化

WASM 支持多种编译时优化：

- **常量折叠**: 编译时计算常量表达式
- **死代码消除**: 移除未使用的代码
- **内联优化**: 内联小函数调用

### 2. 运行时优化

现代 WASM 引擎提供运行时优化：

- **JIT 编译**: 即时编译为机器码
- **分层编译**: 先解释执行，后编译优化
- **SIMD 支持**: 向量指令加速

### 3. 内存效率

WASM 设计为内存高效：

- **紧凑格式**: 高效的二进制编码
- **流式编译**: 边下载边编译
- **增量加载**: 支持模块分割加载

## 下一步

现在您已经了解了 WASM 的基础概念，可以继续学习：

1. 🏗️ [**模块结构**](./module-structure.md) - 深入了解 WASM 模块格式
2. 🔧 [**指令集**](./instruction-set.md) - 掌握完整的指令参考
3. 💾 [**内存管理**](./memory-management.md) - 学习内存操作和管理
4. 🌐 [**JavaScript 互操作**](./js-interop.md) - 实现与 JavaScript 的交互

## 参考资料

- [WebAssembly 规范](https://webassembly.github.io/spec/)
- [MDN WebAssembly 文档](https://developer.mozilla.org/en-US/docs/WebAssembly)
- [WebAssembly 设计文档](https://github.com/WebAssembly/design)

---

*理解这些概念是掌握 WASM 开发的基础。如有疑问，请参考 [调试指南](./debugging.md) 或提交 Issue。*