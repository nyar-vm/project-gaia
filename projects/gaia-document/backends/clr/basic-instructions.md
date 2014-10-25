# MSIL 基础指令

Microsoft Intermediate Language (MSIL) 基础指令，包括常量加载、局部变量操作、栈操作和基本数据操作。

## 常量加载指令

### 整数常量

```msil
ldc.i4.0                    ; 加载整数常量 0
ldc.i4.1                    ; 加载整数常量 1
ldc.i4.2                    ; 加载整数常量 2
ldc.i4.3                    ; 加载整数常量 3
ldc.i4.4                    ; 加载整数常量 4
ldc.i4.5                    ; 加载整数常量 5
ldc.i4.6                    ; 加载整数常量 6
ldc.i4.7                    ; 加载整数常量 7
ldc.i4.8                    ; 加载整数常量 8
ldc.i4.m1                   ; 加载整数常量 -1

ldc.i4.s 100                ; 加载 8 位有符号整数常量（-128 到 127）
ldc.i4 1000000              ; 加载 32 位整数常量
ldc.i8 9223372036854775807  ; 加载 64 位整数常量
```

### 浮点常量

```msil
ldc.r4 3.14159              ; 加载 32 位浮点常量
ldc.r8 2.718281828459045    ; 加载 64 位浮点常量
```

### 字符串常量

```msil
ldstr "Hello, World!"       ; 加载字符串常量
ldstr ""                    ; 加载空字符串
ldstr "包含\n换行符的字符串"  ; 加载包含转义字符的字符串
```

### 空值常量

```msil
ldnull                      ; 加载 null 引用
```

## 局部变量操作

### 局部变量加载

```msil
ldloc.0                     ; 加载局部变量 0
ldloc.1                     ; 加载局部变量 1
ldloc.2                     ; 加载局部变量 2
ldloc.3                     ; 加载局部变量 3

ldloc.s 10                  ; 加载局部变量（索引 0-255）
ldloc 256                   ; 加载局部变量（任意索引）

ldloca.s 5                  ; 加载局部变量地址（索引 0-255）
ldloca 300                  ; 加载局部变量地址（任意索引）
```

### 局部变量存储

```msil
stloc.0                     ; 存储到局部变量 0
stloc.1                     ; 存储到局部变量 1
stloc.2                     ; 存储到局部变量 2
stloc.3                     ; 存储到局部变量 3

stloc.s 10                  ; 存储到局部变量（索引 0-255）
stloc 256                   ; 存储到局部变量（任意索引）
```

## 参数操作

### 参数加载

```msil
ldarg.0                     ; 加载参数 0（实例方法中为 this）
ldarg.1                     ; 加载参数 1
ldarg.2                     ; 加载参数 2
ldarg.3                     ; 加载参数 3

ldarg.s 10                  ; 加载参数（索引 0-255）
ldarg 256                   ; 加载参数（任意索引）

ldarga.s 5                  ; 加载参数地址（索引 0-255）
ldarga 300                  ; 加载参数地址（任意索引）
```

### 参数存储

```msil
starg.s 1                   ; 存储到参数（索引 0-255）
starg 256                   ; 存储到参数（任意索引）
```

## 栈操作指令

### 基本栈操作

```msil
nop                         ; 无操作
pop                         ; 弹出栈顶值
dup                         ; 复制栈顶值
```

### 栈值交换

```msil
; 没有直接的 swap 指令，需要使用局部变量
; 交换栈顶两个值的模式：
; 栈：[..., value1, value2] → [..., value2, value1]

; 使用局部变量交换
stloc.0                     ; 存储 value2 到局部变量 0
stloc.1                     ; 存储 value1 到局部变量 1
ldloc.0                     ; 加载 value2
ldloc.1                     ; 加载 value1
; 栈：[..., value2, value1]
```

## 基础指令示例

### Hello World 程序

```msil
.assembly HelloWorld {}

.class public HelloWorld
{
    .method public static void Main() cil managed
    {
        .entrypoint
        .maxstack 8
        
        ldstr "Hello, World!"           ; 加载字符串常量
        call void [mscorlib]System.Console::WriteLine(string)
        ret                             ; 返回
    }
}
```

### 局部变量使用示例

```msil
.method public static void LocalVariableExample() cil managed
{
    .maxstack 2
    .locals init (
        [0] int32 number,
        [1] string text,
        [2] bool flag
    )
    
    ; 设置局部变量
    ldc.i4 42                   ; 加载整数 42
    stloc.0                     ; 存储到 number
    
    ldstr "Hello"               ; 加载字符串
    stloc.1                     ; 存储到 text
    
    ldc.i4.1                    ; 加载 true (1)
    stloc.2                     ; 存储到 flag
    
    ; 使用局部变量
    ldloc.0                     ; 加载 number
    ldloc.1                     ; 加载 text
    ldloc.2                     ; 加载 flag
    
    ; 清理栈
    pop
    pop
    pop
    
    ret
}
```

### 参数处理示例

```msil
.method public static int32 AddNumbers(int32 a, int32 b) cil managed
{
    .maxstack 2
    
    ldarg.0                     ; 加载参数 a
    ldarg.1                     ; 加载参数 b
    add                         ; 相加
    ret                         ; 返回结果
}
```

### 实例方法示例

```msil
.class public MyClass
{
    .field private int32 value
    
    .method public void SetValue(int32 newValue) cil managed
    {
        .maxstack 2
        
        ldarg.0                 ; 加载 this
        ldarg.1                 ; 加载 newValue
        stfld int32 MyClass::value  ; 设置字段值
        ret
    }
    
    .method public int32 GetValue() cil managed
    {
        .maxstack 1
        
        ldarg.0                 ; 加载 this
        ldfld int32 MyClass::value  ; 获取字段值
        ret                     ; 返回字段值
    }
}
```

## 常量优化

### 整数常量优化

```msil
; 优化前：使用通用指令
ldc.i4 0
ldc.i4 1
ldc.i4 -1

; 优化后：使用专用指令
ldc.i4.0                    ; 更短的字节码
ldc.i4.1                    ; 更短的字节码
ldc.i4.m1                   ; 更短的字节码

; 小整数范围优化
ldc.i4.s 100                ; 对于 -128 到 127 的值
ldc.i4 1000                 ; 对于更大的值
```

### 局部变量索引优化

```msil
; 优化前：使用通用指令
ldloc 0
ldloc 1
stloc 0

; 优化后：使用专用指令
ldloc.0                     ; 更短的字节码
ldloc.1                     ; 更短的字节码
stloc.0                     ; 更短的字节码

; 对于索引 0-3 使用专用指令
; 对于索引 4-255 使用 .s 变体
; 对于更大索引使用完整形式
```

## 类型系统基础

### 基本类型

| MSIL 类型 | .NET 类型 | 描述         |
|---------|---------|------------|
| int8    | sbyte   | 8 位有符号整数   |
| uint8   | byte    | 8 位无符号整数   |
| int16   | short   | 16 位有符号整数  |
| uint16  | ushort  | 16 位无符号整数  |
| int32   | int     | 32 位有符号整数  |
| uint32  | uint    | 32 位无符号整数  |
| int64   | long    | 64 位有符号整数  |
| uint64  | ulong   | 64 位无符号整数  |
| float32 | float   | 32 位浮点数    |
| float64 | double  | 64 位浮点数    |
| bool    | bool    | 布尔值        |
| char    | char    | Unicode 字符 |
| string  | string  | 字符串        |
| object  | object  | 对象引用       |

### 局部变量声明

```msil
.locals init (
    [0] int32,                  ; 未命名的 int32 变量
    [1] string text,            ; 命名的 string 变量
    [2] bool flag,              ; 命名的 bool 变量
    [3] class MyClass obj       ; 自定义类型变量
)
```

## 性能考虑

### 指令选择优化

```msil
; 使用最短的指令形式
ldc.i4.0                    ; 而不是 ldc.i4 0
ldloc.0                     ; 而不是 ldloc 0
stloc.1                     ; 而不是 stloc 1

; 对于常用值使用专用指令
ldc.i4.m1                   ; -1
ldc.i4.0                    ; 0
ldc.i4.1                    ; 1
; ... 到 ldc.i4.8           ; 8
```

### 栈管理

```msil
; 避免不必要的栈操作
; 优化前：
ldloc.0
dup
stloc.1
pop

; 优化后：
ldloc.0
stloc.1
```

## 调试支持

### 调试信息

```msil
.method public static void DebugExample() cil managed
{
    .maxstack 1
    .locals init ([0] int32 x)
    
    .line 10,10 : 5,15 'source.cs'  ; 源码行信息
    ldc.i4 42
    stloc.0
    
    .line 11,11 : 5,20 'source.cs'
    ldloc.0
    pop
    
    ret
}
```

## 相关文档

- [算术指令](./arithmetic-instructions.md)
- [控制流指令](./control-flow-instructions.md)
- [方法调用指令](./method-instructions.md)
- [对象操作指令](./object-instructions.md)