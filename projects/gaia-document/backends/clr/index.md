# CLR Backend

Common Language Runtime (CLR) 后端支持，用于生成 .NET 平台的可执行文件和库。

## 概述

CLR 后端通过 Gaia 统一接口为 .NET 平台提供编译支持，生成 Microsoft Intermediate Language (MSIL) 和 PE 格式的可执行文件或动态链接库。

## 支持的功能

- MSIL 代码生成
- PE 文件格式输出
- .NET Framework 兼容性
- 元数据生成
- 程序集清单

## 目标平台

- Windows (.NET Framework)
- 跨平台 (.NET Core/.NET 5+)
- Mono 运行时

## MSIL 指令文档

### 基础指令

- [基础指令](./basic-instructions.md) - 常量加载、局部变量操作、栈操作等基础指令

### 算术指令

- [算术指令](./arithmetic-instructions.md) - 算术运算、位运算、类型转换和比较指令

### 控制流指令

- [控制流指令](./control-flow-instructions.md) - 条件分支、循环、跳转和 switch 语句

### 方法调用指令

- [方法调用指令](./method-instructions.md) - 实例方法、静态方法、虚方法调用和返回指令

### 对象操作指令

- [对象操作指令](./object-instructions.md) - 对象创建、字段访问、数组操作、类型检查和转换

### 异常处理指令

- [异常处理指令](./exception-instructions.md) - 异常抛出、捕获、finally 块和资源管理

## 快速开始

### 基本 MSIL 程序结构

```msil
.assembly extern mscorlib {}
.assembly MyProgram {}

.class public Program extends [mscorlib]System.Object
{
    .method public static void Main() cil managed
    {
        .entrypoint
        .maxstack 1
        
        ldstr "Hello, MSIL World!"
        call void [mscorlib]System.Console::WriteLine(string)
        ret
    }
}
```

### 编译和运行

```bash
# 编译 Gaia 代码到 MSIL
gaia compile --backend clr --target exe input.gaia

# 编译库文件
gaia compile --backend clr --target dll library.gaia

# 指定 .NET 版本
gaia compile --backend clr --framework net48 input.gaia
gaia compile --backend clr --framework net6.0 input.gaia
```

## 使用示例

```bash
gaia compile --backend clr --target exe input.gaia
gaia compile --backend clr --target dll library.gaia
```

## 核心特性

### IL 代码生成

- 生成标准的 .NET IL 指令
- 支持完整的 .NET 类型系统
- 优化的字节码输出
- 调试信息保留

### Unity 集成

- Unity MonoBehaviour 组件支持
- Unity 特定的 API 绑定
- 游戏对象生命周期管理
- 编辑器工具集成

### .NET 互操作

- 与现有 .NET 库无缝集成
- P/Invoke 原生代码调用
- COM 互操作支持
- 反射和动态代码生成

## 快速开始

### 安装依赖

```toml
[dependencies]
clr-assembler = { path = "../clr-assembler" }
clr-msil = { path = "../clr-msil" }
gaia-types = { path = "../gaia-types" }
```

### 基本用法

#### 创建简单的控制台应用

```rust
use clr_assembler::assembler::ClrAssembler;

// 创建新的 CLR 汇编器
let mut assembler = ClrAssembler::new();

// 设置程序集信息
assembler.set_assembly_name("HelloGaia");
assembler.add_extern_assembly("mscorlib");
assembler.add_extern_assembly("System.Console");

// 创建主类
let main_class = assembler.create_class(
"Valkyrie.Translator.HelloGaia.GaiaAssembler",
Some("[mscorlib]System.Object")
);

// 添加 Main 方法
main_class.add_method("Main", | method| {
method.set_static(true);
method.set_entry_point(true);
method.add_parameter("args", "string[]");

// 生成 IL 代码
method.emit_ldstr("Hello Gaia!");
method.emit_call("void [System.Console]System.Console::WriteLine(string)");
method.emit_ret();
});

// 生成程序集
let assembly_data = assembler.build() ?;
```

#### Unity MonoBehaviour 组件

```rust
use clr_assembler::assembler::ClrAssembler;

let mut assembler = ClrAssembler::new();

// 设置 Unity 程序集
assembler.set_assembly_name("GaiaAssembler");
assembler.add_extern_assembly("UnityEngine");

// 创建 MonoBehaviour 组件
let component_class = assembler.create_class(
"GaiaAssembler",
Some("[UnityEngine]UnityEngine.MonoBehaviour")
);

// 添加 Start 方法
component_class.add_method("Start", | method| {
method.set_virtual(true);
method.emit_ldstr("Hello Unity!");
method.emit_call("void [UnityEngine]UnityEngine.Debug::Log(object)");
method.emit_ret();
});

// 添加构造函数
component_class.add_constructor( | ctor| {
ctor.emit_ldarg_0();
ctor.emit_call("instance void [UnityEngine]UnityEngine.MonoBehaviour::.ctor()");
ctor.emit_ret();
});

let dll_data = assembler.build_dll() ?;
```

## IL 指令支持

### 基本指令

- `ldstr` - 加载字符串常量
- `ldarg` - 加载参数
- `ldloc` - 加载局部变量
- `stloc` - 存储局部变量
- `call` - 方法调用
- `ret` - 返回

### 控制流

- `br` - 无条件跳转
- `brtrue` - 条件跳转（真）
- `brfalse` - 条件跳转（假）
- `beq` - 相等比较跳转
- `bne` - 不等比较跳转

### 对象操作

- `newobj` - 创建对象
- `ldobj` - 加载对象
- `stobj` - 存储对象
- `castclass` - 类型转换
- `isinst` - 类型检查

## 类型系统映射

| Gaia 类型    | .NET 类型          | IL 表示     |
|------------|------------------|-----------|
| `i32`      | `System.Int32`   | `int32`   |
| `i64`      | `System.Int64`   | `int64`   |
| `f32`      | `System.Single`  | `float32` |
| `f64`      | `System.Double`  | `float64` |
| `bool`     | `System.Boolean` | `bool`    |
| `string`   | `System.String`  | `string`  |
| `array<T>` | `T[]`            | `T[]`     |

## 错误处理

### 异常处理

```rust
method.begin_try_block();
// 可能抛出异常的代码
method.emit_call("void SomeMethod()");

method.begin_catch_block("System.Exception");
// 异常处理代码
method.emit_ldstr("Error occurred");
method.emit_call("void [System.Console]System.Console::WriteLine(string)");

method.end_exception_block();
```

### 错误诊断

- 编译时类型检查
- IL 验证
- 运行时异常信息
- 调试符号生成

## 性能优化

### 代码优化

- 内联小方法
- 常量折叠
- 死代码消除
- 尾调用优化

### 内存管理

- 垃圾回收器集成
- 对象池模式
- 值类型优化
- 字符串内存化

## 调试支持

### 调试信息

- PDB 文件生成
- 源码行号映射
- 局部变量信息
- 调用栈跟踪

### 开发工具

- Visual Studio 集成
- JetBrains Rider 支持
- VS Code 调试器
- Unity 调试器

## 部署选项

### 独立部署

```bash
# 生成自包含应用
dotnet publish -c Release --self-contained true -r win-x64
```

### 框架依赖部署

```bash
# 依赖已安装的 .NET 运行时
dotnet publish -c Release --no-self-contained
```

### Unity 部署

- 编译为 Unity 程序集
- 热重载支持
- 平台特定优化
- IL2CPP 兼容性

## 最佳实践

### 代码组织

- 使用命名空间组织代码
- 遵循 .NET 命名约定
- 合理使用访问修饰符
- 文档注释完整

### 性能考虑

- 避免频繁的装箱/拆箱
- 使用值类型减少 GC 压力
- 合理使用异步模式
- 缓存重复计算结果

### 安全性

- 输入验证
- 代码访问安全
- 强名称程序集
- 混淆保护

## 故障排除

### 常见问题

1. **程序集加载失败**: 检查依赖项和版本兼容性
2. **IL 验证错误**: 确保类型安全和指令正确性
3. **Unity 集成问题**: 验证 Unity 版本和 API 兼容性
4. **性能问题**: 使用性能分析器定位瓶颈

### 调试技巧

- 使用 `ildasm` 查看生成的 IL 代码
- 启用 JIT 调试
- 检查异常详细信息
- 使用性能计数器

## 示例项目

查看 `tests/` 目录中的示例：

- `HelloGaia/` - 基本控制台应用
- `HelloUnity/` - Unity MonoBehaviour 组件
- `hello-multi.il` - 多组件 Unity 项目

## 相关资源

- [.NET IL 指令参考](https://docs.microsoft.com/en-us/dotnet/api/system.reflection.emit.opcodes)
- [Unity 脚本 API](https://docs.unity3d.com/ScriptReference/)
- [CLR 规范](https://www.ecma-international.org/publications-and-standards/standards/ecma-335/)