# WASM 模块结构

本文档详细介绍 WebAssembly 模块的内部结构、二进制格式和各个段的组成，帮助您深入理解 WASM 模块的工作原理。

## 模块概览

WASM 模块是一个自包含的代码单元，包含类型定义、函数、内存、表等组件。模块采用二进制格式存储，具有高效的加载和执行特性。

### 模块结构图

```
┌─────────────────────────────────────┐
│            WASM 模块                │
├─────────────────────────────────────┤
│  Magic Number (0x6d736100)         │  ← \0asm
│  Version (0x00000001)              │  ← 版本 1
├─────────────────────────────────────┤
│  Type Section                      │  ← 函数类型定义
│  Import Section                    │  ← 导入声明
│  Function Section                  │  ← 函数声明
│  Table Section                     │  ← 表定义
│  Memory Section                    │  ← 内存定义
│  Global Section                    │  ← 全局变量定义
│  Export Section                    │  ← 导出声明
│  Start Section                     │  ← 启动函数
│  Element Section                   │  ← 表初始化
│  Code Section                      │  ← 函数实现
│  Data Section                      │  ← 内存初始化
│  Custom Sections                   │  ← 自定义段
└─────────────────────────────────────┘
```

## 二进制格式

### 1. 模块头部

每个 WASM 模块都以固定的头部开始：

```rust
pub struct ModuleHeader {
    pub magic: [u8; 4],    // 0x00, 0x61, 0x73, 0x6d ("\0asm")
    pub version: [u8; 4],  // 0x01, 0x00, 0x00, 0x00 (版本 1)
}
```

**示例**:

```rust
use gaia_assembler::backends::wasm::*;

let mut assembler = WasmAssembler::new();
// 头部会自动添加
let module_bytes = assembler.build() ?;

// 验证头部
assert_eq!(&module_bytes[0..4], b"\0asm");
assert_eq!(&module_bytes[4..8], &[1, 0, 0, 0]);
```

### 2. 段结构

每个段都有统一的结构：

```rust
pub struct Section {
    pub id: u8,           // 段 ID
    pub size: u32,        // 段大小 (LEB128 编码)
    pub content: Vec<u8>, // 段内容
}
```

**段 ID 定义**:

```rust
pub enum SectionId {
    Custom = 0,    // 自定义段
    Type = 1,      // 类型段
    Import = 2,    // 导入段
    Function = 3,  // 函数段
    Table = 4,     // 表段
    Memory = 5,    // 内存段
    Global = 6,    // 全局段
    Export = 7,    // 导出段
    Start = 8,     // 启动段
    Element = 9,   // 元素段
    Code = 10,     // 代码段
    Data = 11,     // 数据段
}
```

## 段详解

### 1. 类型段 (Type Section)

定义模块中使用的所有函数类型：

```rust
pub struct TypeSection {
    pub types: Vec<FuncType>,
}

pub struct FuncType {
    pub params: Vec<ValType>,   // 参数类型
    pub results: Vec<ValType>,  // 返回类型
}
```

**示例**:

```rust
// 定义多种函数类型
let mut assembler = WasmAssembler::new();

// (i32, i32) -> i32
let binary_op_type = assembler.add_function_type(
vec![ValType::I32, ValType::I32],
vec![ValType::I32]
);

// () -> f64
let getter_type = assembler.add_function_type(
vec![],
vec![ValType::F64]
);

// (f32, f32, f32) -> ()
let setter_type = assembler.add_function_type(
vec![ValType::F32, ValType::F32, ValType::F32],
vec![]
);
```

### 2. 导入段 (Import Section)

声明从外部环境导入的功能：

```rust
pub struct ImportSection {
    pub imports: Vec<Import>,
}

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

**示例**:

```rust
// 导入 JavaScript 函数
assembler.add_import(Import {
module: "env".to_string(),
name: "print_i32".to_string(),
desc: ImportDesc::Function(print_type),
});

// 导入共享内存
assembler.add_import(Import {
module: "js".to_string(),
name: "shared_memory".to_string(),
desc: ImportDesc::Memory(MemoryType {
limits: Limits { min: 1, max: Some(10) },
shared: true,
}),
});

// 导入全局计数器
assembler.add_import(Import {
module: "env".to_string(),
name: "global_counter".to_string(),
desc: ImportDesc::Global(GlobalType {
val_type: ValType::I32,
mutable: true,
}),
});
```

### 3. 函数段 (Function Section)

声明模块内定义的函数及其类型：

```rust
pub struct FunctionSection {
    pub type_indices: Vec<TypeIdx>,  // 函数类型索引
}
```

**示例**:

```rust
// 声明函数（只是类型，实现在代码段）
let add_func = assembler.add_function(binary_op_type);
let mul_func = assembler.add_function(binary_op_type);
let get_pi_func = assembler.add_function(getter_type);

// 函数段现在包含三个类型索引
```

### 4. 表段 (Table Section)

定义函数引用表：

```rust
pub struct TableSection {
    pub tables: Vec<TableType>,
}

pub struct TableType {
    pub element_type: RefType,  // 元素类型
    pub limits: Limits,         // 大小限制
}

pub enum RefType {
    FuncRef,    // 函数引用
    ExternRef,  // 外部引用
}
```

**示例**:

```rust
// 创建函数表用于间接调用
let function_table = assembler.add_table(TableType {
element_type: RefType::FuncRef,
limits: Limits {
min: 10,           // 最少 10 个槽位
max: Some(100),    // 最多 100 个槽位
},
});

// 创建外部引用表
let extern_table = assembler.add_table(TableType {
element_type: RefType::ExternRef,
limits: Limits {
min: 0,
max: Some(50),
},
});
```

### 5. 内存段 (Memory Section)

定义线性内存：

```rust
pub struct MemorySection {
    pub memories: Vec<MemoryType>,
}

pub struct MemoryType {
    pub limits: Limits,  // 大小限制（以页为单位）
    pub shared: bool,    // 是否共享
}

pub struct Limits {
    pub min: u32,           // 最小大小
    pub max: Option<u32>,   // 最大大小（可选）
}
```

**示例**:

```rust
// 创建 1MB 初始内存，最大 16MB
let main_memory = assembler.add_memory(MemoryType {
limits: Limits {
min: 16,           // 16 页 = 1MB
max: Some(256),    // 256 页 = 16MB
},
shared: false,
});

// 创建共享内存用于多线程
let shared_memory = assembler.add_memory(MemoryType {
limits: Limits {
min: 1,
max: Some(10),
},
shared: true,
});
```

### 6. 全局段 (Global Section)

定义全局变量：

```rust
pub struct GlobalSection {
    pub globals: Vec<Global>,
}

pub struct Global {
    pub global_type: GlobalType,  // 全局变量类型
    pub init: ConstExpr,          // 初始化表达式
}

pub struct GlobalType {
    pub val_type: ValType,  // 值类型
    pub mutable: bool,      // 是否可变
}
```

**示例**:

```rust
// 创建常量全局变量
let pi_global = assembler.add_global(Global {
global_type: GlobalType {
val_type: ValType::F64,
mutable: false,  // 常量
},
init: ConstExpr::F64Const(3.141592653589793),
});

// 创建可变全局变量
let counter_global = assembler.add_global(Global {
global_type: GlobalType {
val_type: ValType::I32,
mutable: true,   // 可变
},
init: ConstExpr::I32Const(0),
});
```

### 7. 导出段 (Export Section)

声明对外导出的功能：

```rust
pub struct ExportSection {
    pub exports: Vec<Export>,
}

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

**示例**:

```rust
// 导出函数
assembler.add_export(Export {
name: "add".to_string(),
desc: ExportDesc::Function(add_func.index()),
});

// 导出内存
assembler.add_export(Export {
name: "memory".to_string(),
desc: ExportDesc::Memory(main_memory.index()),
});

// 导出全局变量
assembler.add_export(Export {
name: "counter".to_string(),
desc: ExportDesc::Global(counter_global.index()),
});
```

### 8. 启动段 (Start Section)

指定模块加载时自动执行的函数：

```rust
pub struct StartSection {
    pub func_idx: FuncIdx,  // 启动函数索引
}
```

**示例**:

```rust
// 创建初始化函数
let init_func = assembler.add_function(init_type);
init_func.add_instructions( & [
// 初始化全局状态
Instruction::I32Const(42),
Instruction::GlobalSet(counter_global.index()),
Instruction::End,
]);

// 设置为启动函数
assembler.set_start_function(init_func.index());
```

### 9. 元素段 (Element Section)

初始化表的内容：

```rust
pub struct ElementSection {
    pub elements: Vec<Element>,
}

pub struct Element {
    pub table_idx: TableIdx,     // 表索引
    pub offset: ConstExpr,       // 偏移量
    pub init: Vec<FuncIdx>,      // 初始化函数列表
}
```

**示例**:

```rust
// 初始化函数表
assembler.add_element(Element {
table_idx: function_table.index(),
offset: ConstExpr::I32Const(0),  // 从索引 0 开始
init: vec![
    add_func.index(),
    mul_func.index(),
    get_pi_func.index(),
],
});
```

### 10. 代码段 (Code Section)

包含函数的实际实现：

```rust
pub struct CodeSection {
    pub codes: Vec<Code>,
}

pub struct Code {
    pub locals: Vec<LocalDecl>,      // 局部变量声明
    pub body: Vec<Instruction>,      // 指令序列
}

pub struct LocalDecl {
    pub count: u32,       // 变量数量
    pub val_type: ValType, // 变量类型
}
```

**示例**:

```rust
// 实现加法函数
add_func.add_instructions( & [
Instruction::LocalGet(0),    // 获取第一个参数
Instruction::LocalGet(1),    // 获取第二个参数
Instruction::I32Add,         // 执行加法
Instruction::End,            // 函数结束
]);

// 实现带局部变量的函数
let complex_func = assembler.add_function(complex_type);
complex_func.add_local(ValType::I32);  // 添加局部变量
complex_func.add_local(ValType::F64);  // 添加局部变量
complex_func.add_instructions( & [
// 使用局部变量的复杂逻辑
Instruction::LocalGet(0),           // 参数 0
Instruction::LocalSet(2),           // 存储到局部变量 2
Instruction::F64Const(2.5),
Instruction::LocalSet(3),           // 存储到局部变量 3
// ... 更多指令
Instruction::End,
]);
```

### 11. 数据段 (Data Section)

初始化内存的内容：

```rust
pub struct DataSection {
    pub data: Vec<Data>,
}

pub struct Data {
    pub memory_idx: MemoryIdx,  // 内存索引
    pub offset: ConstExpr,      // 偏移量
    pub init: Vec<u8>,          // 初始化数据
}
```

**示例**:

```rust
// 在内存中存储字符串
let hello_str = b"Hello, WASM!";
assembler.add_data(Data {
memory_idx: main_memory.index(),
offset: ConstExpr::I32Const(0),  // 从地址 0 开始
init: hello_str.to_vec(),
});

// 存储数值数组
let numbers = vec![1u8, 2, 3, 4, 5, 6, 7, 8];  // 8 字节
assembler.add_data(Data {
memory_idx: main_memory.index(),
offset: ConstExpr::I32Const(1024),  // 从地址 1024 开始
init: numbers,
});
```

### 12. 自定义段 (Custom Sections)

存储调试信息、元数据等：

```rust
pub struct CustomSection {
    pub name: String,      // 段名称
    pub data: Vec<u8>,     // 段数据
}
```

**示例**:

```rust
// 添加调试信息
assembler.add_custom_section(CustomSection {
name: "name".to_string(),
data: create_name_section_data( & function_names),
});

// 添加源码映射
assembler.add_custom_section(CustomSection {
name: "sourceMappingURL".to_string(),
data: b"module.wasm.map".to_vec(),
});

// 添加版本信息
assembler.add_custom_section(CustomSection {
name: "version".to_string(),
data: b"1.0.0".to_vec(),
});
```

## 模块验证

### 1. 结构验证

WASM 模块必须满足结构约束：

```rust
pub fn validate_module(module: &WasmModule) -> Result<(), ValidationError> {
    // 验证段顺序
    validate_section_order(&module.sections)?;

    // 验证索引引用
    validate_indices(&module)?;

    // 验证类型一致性
    validate_types(&module)?;

    Ok(())
}
```

**验证规则**:

- 段必须按正确顺序出现
- 所有索引引用必须有效
- 函数签名必须匹配
- 内存访问必须在边界内

### 2. 类型验证

验证指令序列的类型正确性：

```rust
pub fn validate_function(func: &Function, types: &[FuncType]) -> Result<(), ValidationError> {
    let func_type = &types[func.type_idx as usize];
    let mut stack = TypeStack::new();

    // 初始化参数
    for param_type in &func_type.params {
        stack.push(*param_type);
    }

    // 验证指令序列
    for instruction in &func.body {
        validate_instruction(instruction, &mut stack)?;
    }

    // 验证返回类型
    validate_return_types(&stack, &func_type.results)?;

    Ok(())
}
```

## 模块优化

### 1. 段重排序

优化段的顺序以提高加载性能：

```rust
pub fn optimize_section_order(module: &mut WasmModule) {
    // 将经常访问的段放在前面
    let optimal_order = vec![
        SectionId::Type,
        SectionId::Import,
        SectionId::Function,
        SectionId::Export,
        SectionId::Code,
        SectionId::Memory,
        SectionId::Data,
        // ... 其他段
    ];

    module.sections.sort_by_key(|section| {
        optimal_order.iter().position(|&id| id == section.id)
            .unwrap_or(usize::MAX)
    });
}
```

### 2. 代码压缩

移除未使用的函数和数据：

```rust
pub fn eliminate_dead_code(module: &mut WasmModule) {
    let mut used_functions = HashSet::new();

    // 标记导出的函数
    for export in &module.exports {
        if let ExportDesc::Function(func_idx) = export.desc {
            mark_function_used(func_idx, &mut used_functions, module);
        }
    }

    // 移除未使用的函数
    module.functions.retain(|func| used_functions.contains(&func.index()));
}
```

## 实际应用示例

### 完整模块示例

```rust
use gaia_assembler::backends::wasm::*;

fn create_math_module() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut assembler = WasmAssembler::new();
    assembler.set_module_name("math_utils");

    // 1. 定义类型
    let binary_op_type = assembler.add_function_type(
        vec![ValType::F64, ValType::F64],
        vec![ValType::F64]
    );

    let unary_op_type = assembler.add_function_type(
        vec![ValType::F64],
        vec![ValType::F64]
    );

    // 2. 导入 JavaScript 函数
    assembler.add_import(Import {
        module: "Math".to_string(),
        name: "sin".to_string(),
        desc: ImportDesc::Function(unary_op_type),
    });

    // 3. 定义内存
    let memory = assembler.add_memory(MemoryType {
        limits: Limits { min: 1, max: Some(10) },
        shared: false,
    });

    // 4. 定义全局常量
    let pi_global = assembler.add_global(Global {
        global_type: GlobalType {
            val_type: ValType::F64,
            mutable: false,
        },
        init: ConstExpr::F64Const(std::f64::consts::PI),
    });

    // 5. 实现函数
    let add_func = assembler.add_function(binary_op_type);
    add_func.add_instructions(&[
        Instruction::LocalGet(0),
        Instruction::LocalGet(1),
        Instruction::F64Add,
        Instruction::End,
    ]);

    let circle_area_func = assembler.add_function(unary_op_type);
    circle_area_func.add_instructions(&[
        Instruction::LocalGet(0),      // 半径
        Instruction::LocalGet(0),      // 半径
        Instruction::F64Mul,           // 半径²
        Instruction::GlobalGet(pi_global.index()), // π
        Instruction::F64Mul,           // π × 半径²
        Instruction::End,
    ]);

    // 6. 导出功能
    assembler.add_export(Export {
        name: "add".to_string(),
        desc: ExportDesc::Function(add_func.index()),
    });

    assembler.add_export(Export {
        name: "circle_area".to_string(),
        desc: ExportDesc::Function(circle_area_func.index()),
    });

    assembler.add_export(Export {
        name: "memory".to_string(),
        desc: ExportDesc::Memory(memory.index()),
    });

    // 7. 生成模块
    let module_bytes = assembler.build()?;
    Ok(module_bytes)
}
```

## 调试和分析

### 1. 模块分析工具

```rust
pub fn analyze_module(module_bytes: &[u8]) -> ModuleAnalysis {
    let module = parse_wasm_module(module_bytes).unwrap();

    ModuleAnalysis {
        total_size: module_bytes.len(),
        section_sizes: calculate_section_sizes(&module),
        function_count: module.functions.len(),
        import_count: module.imports.len(),
        export_count: module.exports.len(),
        memory_pages: module.memories.iter().map(|m| m.limits.min).sum(),
    }
}
```

### 2. 性能分析

```rust
pub fn profile_module_loading(module_bytes: &[u8]) -> LoadingProfile {
    let start = std::time::Instant::now();

    // 解析模块
    let parse_start = std::time::Instant::now();
    let module = parse_wasm_module(module_bytes).unwrap();
    let parse_time = parse_start.elapsed();

    // 验证模块
    let validate_start = std::time::Instant::now();
    validate_module(&module).unwrap();
    let validate_time = validate_start.elapsed();

    // 编译模块
    let compile_start = std::time::Instant::now();
    let compiled = compile_module(&module).unwrap();
    let compile_time = compile_start.elapsed();

    let total_time = start.elapsed();

    LoadingProfile {
        parse_time,
        validate_time,
        compile_time,
        total_time,
    }
}
```

## 下一步

现在您已经深入了解了 WASM 模块结构，可以继续学习：

1. 📖 [**基础概念**](./concepts.md) - 掌握 WASM 核心概念
2. 🚀 [**入门指南**](./getting-started.md) - 深入理解实际应用
3. 📚 [**用户指南**](../../user-guide/index.md) - 学习框架功能技巧
4. 🔧 [**维护指南**](../../maintenance/index.md) - 优化模块性能

## 参考资料

- [WASM 二进制格式规范](https://webassembly.github.io/spec/core/binary/index.html)
- [WASM 文本格式规范](https://webassembly.github.io/spec/core/text/index.html)
- [WABT 工具包](https://github.com/WebAssembly/wabt)

---

*理解模块结构是优化 WASM 应用的关键。如需更多帮助，请查看 [维护指南](../../maintenance/troubleshooting.md)。*