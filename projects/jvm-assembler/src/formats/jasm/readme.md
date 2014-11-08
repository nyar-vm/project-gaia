# JASM 格式

JASM (Java ASseMbler) 是一种人类可读的 JVM 字节码汇编语言格式，用于表示 Java 类文件的结构和字节码指令。

## 概述

JASM 格式提供了一种直观的方式来编写和阅读 JVM 字节码，它是 Java 类文件的文本表示形式。该格式支持：

- **类定义**: 类名、访问修饰符、超类、接口
- **字段定义**: 字段名、类型、访问修饰符
- **方法定义**: 方法名、签名、字节码指令
- **常量池**: 自动管理字符串、数字、类引用等常量
- **属性**: 源文件、行号表、局部变量表等标准属性

## 语法结构

### 基本类结构

```jasm
.class public MyClass
.super java/lang/Object

.method public "<init>":"()V"
    .limit stack 1
    .limit locals 1
    aload_0
    invokespecial Method java/lang/Object."<init>":"()V"
    return
.end method
```

### 字段定义

```jasm
.field private "count":"I"
.field public static final "MAX_VALUE":"I" = 100
```

### 方法定义

```jasm
.method public "add":"(II)I"
    .limit stack 2
    .limit locals 3
    iload_1
    iload_2
    iadd
    ireturn
.end method
```

## 指令系统

### 常量加载指令

- `aconst_null` - 加载 null 引用
- `iconst_0` 到 `iconst_5` - 加载整数常量 0-5
- `ldc "string"` - 加载字符串常量
- `ldc 123` - 加载数字常量

### 局部变量操作

- `iload_0` 到 `iload_3` - 加载局部变量到操作数栈
- `istore_0` 到 `istore_3` - 从操作数栈存储到局部变量
- `iinc 1 1` - 递增局部变量

### 栈操作指令

- `pop` - 弹出栈顶元素
- `dup` - 复制栈顶元素
- `swap` - 交换栈顶两个元素

### 算术运算

- `iadd` - 整数加法
- `isub` - 整数减法
- `imul` - 整数乘法
- `idiv` - 整数除法

### 方法调用

```jasm
invokevirtual Method java/io/PrintStream.println:"(Ljava/lang/String;)V"
invokestatic Method java/lang/Math.max:"(II)I"
```

### 控制流

```jasm
ifeq label1
goto label2
label1:
    ; 代码块
label2:
    ; 代码块
```

## 示例程序

### Hello World

```jasm
.class public HelloWorld
.super java/lang/Object

.method public static "main":"([Ljava/lang/String;)V"
    .limit stack 2
    .limit locals 1
    getstatic Field java/lang/System.out:"Ljava/io/PrintStream;"
    ldc "Hello, World!"
    invokevirtual Method java/io/PrintStream.println:"(Ljava/lang/String;)V"
    return
.end method
```

### 简单计算器

```jasm
.class public Calculator
.super java/lang/Object

.method public "add":"(II)I"
    .limit stack 2
    .limit locals 3
    iload_1
    iload_2
    iadd
    ireturn
.end method

.method public "multiply":"(II)I"
    .limit stack 2
    .limit locals 3
    iload_1
    iload_2
    imul
    ireturn
.end method
```

## 高级特性

### 异常处理

```jasm
.method public "safeDivide":"(II)I"
    .limit stack 2
    .limit locals 3
    .catch java/lang/ArithmeticException from start to end using handler
start:
    iload_1
    iload_2
    idiv
    ireturn
end:
handler:
    pop  ; 移除异常对象
    iconst_m1
    ireturn
.end method
```

### 泛型支持

JASM 支持 Java 泛型的类型描述符：

```jasm
.method public "processList":"(Ljava/util/List;)V"
    .signature "(Ljava/util/List<Ljava/lang/String;>;)V"
    .limit stack 1
    .limit locals 2
    ; 方法实现
.end method
```

### 注解支持

```jasm
.runtime_visible_annotation @Ljava/lang/Deprecated;()
.method public "oldMethod":"()V"
    ; 方法实现
.end method
```

## 转换过程

JASM 格式通过以下步骤转换为 JVM 类文件：

1. **词法分析**: 将文本分解为标记（tokens）
2. **语法分析**: 构建抽象语法树（AST）
3. **语义分析**: 验证语法和类型正确性
4. **代码生成**: 生成 JVM 字节码和常量池
5. **类文件写入**: 写入标准 Java 类文件格式

## 相关模块

- [`lexer`](lexer/index.html) - JASM 词法分析器
- [`parser`](parser/index.html) - JASM 语法分析器
- [`ast`](ast/index.html) - JASM 抽象语法树定义
- [`converter`](converter/index.html) - AST 到 JVM 程序的转换器
- [`writer`](writer/index.html) - JVM 程序到类文件的写入器

## 错误处理

JASM 解析和转换过程中可能遇到的错误：

- **语法错误**: 无效的语法结构
- **类型错误**: 类型不匹配或无效类型
- **引用错误**: 未定义的类、方法或字段
- **验证错误**: 字节码验证失败

所有错误都通过 [`GaiaError`](../../gaia_types/struct.GaiaError.html) 类型返回，提供详细的错误信息和位置。