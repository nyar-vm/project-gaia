# 用户指南

本指南面向使用 Gaia Assembler 进行开发的用户，提供详细的接口使用说明、集成方法和最佳实践。

## 目录

### 基础知识

- [接口概览](#接口概览) - Gaia 统一接口的设计理念
- [快速集成](#快速集成) - 如何在项目中集成 Gaia
- [配置选项](#配置选项) - 支持的配置参数和选项

### 高级主题

- [调试指南](#调试指南) - 调试技巧和工具使用
- [性能优化](#性能优化) - 优化编译性能
- [最佳实践](#最佳实践) - 使用规范和建议

### 平台特定

- [CLR/.NET 集成](#clr-net-集成) - .NET 平台集成指南
- [JVM/Java 集成](#jvm-java-集成) - Java 平台集成指南
- [WebAssembly 集成](#webassembly-集成) - WASM 平台集成指南

## 快速参考

### 常用接口

| 接口              | 描述     | 示例                                  |
|-----------------|--------|-------------------------------------|
| `compile()`     | 编译源代码  | `assembler.compile(source)`         |
| `set_target()`  | 设置目标平台 | `assembler.set_target(Target::CLR)` |
| `add_backend()` | 添加后端支持 | `assembler.add_backend(backend)`    |
| `optimize()`    | 启用优化   | `assembler.optimize(true)`          |
| `generate()`    | 生成目标代码 | `assembler.generate()`              |

### 使用要点

- 先设置目标平台，再进行编译
- 支持链式调用：`assembler.set_target().compile().generate()`
- 使用 Result 类型处理错误
- 支持异步编译操作

## 示例代码

### 基本使用

```rust
use gaia_assembler::{Assembler, Target, Result};

fn main() -> Result<()> {
    let mut assembler = Assembler::new();
    
    // 设置目标平台
    assembler.set_target(Target::CLR)?;
    
    // 编译源代码
    let source = include_str!("example.cs");
    let result = assembler.compile(source)?;
    
    // 生成目标代码
    let output = assembler.generate()?;
    
    println!("编译成功，生成 {} 字节", output.len());
    Ok(())
}
```

### 多平台编译

```rust
use gaia_assembler::{Assembler, Target};

fn compile_for_all_platforms(source: &str) -> Result<(), Box<dyn std::error::Error>> {
    let targets = vec![Target::CLR, Target::JVM, Target::WASM];
    
    for target in targets {
        let mut assembler = Assembler::new();
        assembler.set_target(target)?;
        
        let result = assembler.compile(source)?;
        let output = assembler.generate()?;
        
        println!("为 {:?} 平台生成了 {} 字节", target, output.len());
    }
    
    Ok(())
}
```

### 自定义后端

```rust
use gaia_assembler::{Assembler, Backend, CustomBackend};

struct MyCustomBackend;

impl Backend for MyCustomBackend {
    fn compile(&mut self, source: &str) -> Result<Vec<u8>> {
        // 自定义编译逻辑
        Ok(vec![])
    }
}

fn main() -> Result<()> {
    let mut assembler = Assembler::new();
    let custom_backend = MyCustomBackend;
    
    assembler.add_backend(Box::new(custom_backend))?;
    
    Ok(())
}
```

## 工具集成

### 编辑器支持

- **VS Code**: Gaia Assembly 语法高亮插件
- **Vim**: 语法文件和代码片段
- **Emacs**: Major mode 支持

### 构建工具

- **Cargo**: Rust 项目集成
- **Make**: 传统构建系统
- **CMake**: 跨平台构建

### 调试器

- **GDB**: Linux/Unix 平台调试
- **LLDB**: macOS 和现代 Linux
- **Visual Studio**: Windows 平台调试

## 常见问题

### 编译错误

**Q: 出现 "未定义的标签" 错误**
A: 检查标签拼写和作用域，确保标签在使用前已定义。

**Q: 栈溢出错误**
A: 检查递归调用深度，确保有正确的终止条件。

### 性能问题

**Q: 程序运行缓慢**
A: 使用性能分析工具，检查热点代码路径。

**Q: 内存使用过多**
A: 检查内存分配和释放，避免内存泄漏。

## 汇编语法

### 基本语法规则

Gaia 汇编语言采用简洁直观的语法设计：

```gaia
// 注释以 // 开头
/* 多行注释 */

// 标签定义
main:
    load 42        // 加载常量
    store result   // 存储到变量
    ret           // 返回

// 数据段
.data
    message: .string "Hello, World!"
    counter: .int 0
```

### 语法要素

- **指令**: 小写字母，如 `load`, `store`, `add`
- **标签**: 标识符后跟冒号，如 `main:`
- **常量**: 数字、字符串、布尔值
- **变量**: 标识符引用
- **注释**: `//` 单行，`/* */` 多行

## 接口概览

### 核心接口

| 方法             | 参数       | 返回值                     | 描述        |
|----------------|----------|-------------------------|-----------|
| `new()`        | -        | `Assembler`             | 创建新的汇编器实例 |
| `set_target()` | `Target` | `Result<()>`            | 设置目标平台    |
| `compile()`    | `&str`   | `Result<CompileResult>` | 编译源代码     |
| `generate()`   | -        | `Result<Vec<u8>>`       | 生成目标代码    |

### 配置接口

| 方法                    | 参数                 | 描述        |
|-----------------------|--------------------|-----------|
| `optimize()`          | `bool`             | 启用/禁用优化   |
| `set_debug()`         | `bool`             | 启用/禁用调试信息 |
| `add_backend()`       | `Box<dyn Backend>` | 添加自定义后端   |
| `set_output_format()` | `OutputFormat`     | 设置输出格式    |

### 目标平台

| 平台             | 描述               |
|----------------|------------------|
| `Target::CLR`  | .NET 中间语言 (MSIL) |
| `Target::JVM`  | Java 字节码         |
| `Target::PE`   | Windows 可执行文件    |
| `Target::ELF`  | Linux/Unix 可执行文件 |
| `Target::WASM` | WebAssembly      |

## 数据类型

### 基本类型

- **整数**: `int8`, `int16`, `int32`, `int64`
- **浮点数**: `float32`, `float64`
- **布尔值**: `bool`
- **字符**: `char`

### 复合类型

- **字符串**: `string`
- **数组**: `array<T>`
- **结构体**: `struct`
- **指针**: `ptr<T>`

### 类型转换

```gaia
load 42          // int32
i32_to_f64       // 转换为 float64
f64_to_string    // 转换为字符串
```

## 调试指南

### 调试信息

使用 `--debug` 标志生成调试信息：

```bash
gaia-assembler --debug program.gasm
```

### 断点设置

```gaia
.debug_info
    .line 10
    .breakpoint main
```

### 变量监视

```gaia
.watch variable_name
```

## 性能优化

### 编译优化

```bash
# 启用优化
gaia-assembler --optimize program.gasm

# 指定优化级别
gaia-assembler --opt-level 3 program.gasm
```

### 代码优化技巧

1. **减少栈操作**: 合并连续的 load/store
2. **循环优化**: 使用寄存器缓存频繁访问的值
3. **内联函数**: 对小函数使用内联

## 最佳实践

### 代码组织

```gaia
// 1. 数据段在前
.data
    constants: .int 42

// 2. 函数定义
.function helper()
    // 实现
.end

// 3. 主程序
main:
    // 主逻辑
```

### 命名规范

- 函数名：`snake_case`
- 常量：`UPPER_CASE`
- 变量：`camelCase`
- 标签：`lowercase`

### 错误处理

```gaia
.function safe_divide(a, b)
    load b
    jz division_by_zero
    
    load a
    load b
    div
    ret
    
division_by_zero:
    load 0
    ret
.end
```

## CLR/.NET 开发

### .NET 集成

```gaia
.assembly "MyAssembly"
.class public MyClass
    .method public static void Main()
        ldstr "Hello, .NET!"
        call void [mscorlib]System.Console::WriteLine(string)
        ret
    .end method
.end class
```

### 互操作性

- 调用 .NET 方法
- 使用 .NET 类型
- 异常处理

## JVM/Java 开发

### Java 字节码生成

```gaia
.class public HelloJava
.super java/lang/Object

.method public static main([Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Hello, Java!"
    invokevirtual java/io/PrintStream/println(Ljava/lang/String;)V
    return
.end method
```

### JVM 特性

- 垃圾回收集成
- 异常处理
- 反射支持

## WebAssembly 开发

### WASM 模块

```gaia
.module
.export "add" (func $add)

.func $add (param $a i32) (param $b i32) (result i32)
    local.get $a
    local.get $b
    i32.add
.end
```

### Web 集成

- JavaScript 互操作
- 内存管理
- 性能优化

## 更多资源

- [后端文档](/backends/) - 特定平台的详细信息
- [开发者指南](/developer-guide/) - 扩展和定制
- [API 参考](/api-reference/) - 编程接口文档
- [内部实现](/internals/) - 深入了解实现细节