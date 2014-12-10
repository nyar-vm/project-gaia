# MSIL 方法调用指令

Microsoft Intermediate Language (MSIL) 方法调用指令，包括实例方法、静态方法、虚方法调用和返回指令。

## 方法调用指令

### 实例方法调用

```msil
call method                 ; 调用静态方法或实例方法（非虚拟）
callvirt method             ; 调用虚拟方法或接口方法
calli signature             ; 通过函数指针调用方法

; 示例：调用实例方法
.method public static void CallInstanceMethod() cil managed
{
    .maxstack 2
    .locals init ([0] class [mscorlib]System.String str)
    
    ldstr "Hello"
    stloc.0                     ; str = "Hello"
    
    ldloc.0                     ; 加载 this 指针
    ldstr "World"               ; 加载参数
    callvirt instance string [mscorlib]System.String::Concat(string)
    
    ; 调用 Console.WriteLine
    call void [mscorlib]System.Console::WriteLine(string)
    ret
}
```

### 静态方法调用

```msil
; 调用静态方法
call return_type class_name::method_name(parameter_types)

; 示例：调用静态方法
.method public static void CallStaticMethod() cil managed
{
    .maxstack 1
    
    ldstr "Hello, World!"
    call void [mscorlib]System.Console::WriteLine(string)
    ret
}
```

### 构造函数调用

```msil
newobj constructor          ; 创建对象并调用构造函数

; 示例：创建对象
.method public static void CreateObject() cil managed
{
    .maxstack 2
    .locals init ([0] class [mscorlib]System.Object obj)
    
    ; 调用默认构造函数
    newobj instance void [mscorlib]System.Object::.ctor()
    stloc.0                     ; obj = new Object()
    
    ; 调用带参数的构造函数
    ldstr "Hello"
    newobj instance void [mscorlib]System.String::.ctor(string)
    
    call void [mscorlib]System.Console::WriteLine(string)
    ret
}
```

## 虚方法调用

### 虚方法和重写

```msil
; 基类方法定义
.class public BaseClass extends [mscorlib]System.Object
{
    .method public virtual void VirtualMethod() cil managed
    {
        .maxstack 1
        ldstr "Base implementation"
        call void [mscorlib]System.Console::WriteLine(string)
        ret
    }
}

; 派生类重写
.class public DerivedClass extends BaseClass
{
    .method public virtual void VirtualMethod() cil managed
    {
        .maxstack 1
        ldstr "Derived implementation"
        call void [mscorlib]System.Console::WriteLine(string)
        ret
    }
}

; 调用虚方法
.method public static void CallVirtualMethod() cil managed
{
    .maxstack 1
    .locals init ([0] class BaseClass obj)
    
    ; 创建派生类实例
    newobj instance void DerivedClass::.ctor()
    stloc.0
    
    ; 调用虚方法（运行时绑定）
    ldloc.0
    callvirt instance void BaseClass::VirtualMethod()
    ret
}
```

### 接口方法调用

```msil
; 接口定义
.class interface public abstract IExample
{
    .method public abstract virtual void InterfaceMethod() cil managed
    {
    }
}

; 实现接口的类
.class public ExampleClass extends [mscorlib]System.Object implements IExample
{
    .method public virtual void InterfaceMethod() cil managed
    {
        .maxstack 1
        ldstr "Interface implementation"
        call void [mscorlib]System.Console::WriteLine(string)
        ret
    }
}

; 调用接口方法
.method public static void CallInterfaceMethod() cil managed
{
    .maxstack 1
    .locals init ([0] class IExample obj)
    
    newobj instance void ExampleClass::.ctor()
    stloc.0
    
    ; 通过接口调用方法
    ldloc.0
    callvirt instance void IExample::InterfaceMethod()
    ret
}
```

## 函数指针调用

### 委托调用

```msil
; 委托类型定义
.class public sealed MyDelegate extends [mscorlib]System.MulticastDelegate
{
    .method public hidebysig specialname rtspecialname 
            instance void .ctor(object 'object', native int 'method') runtime managed
    {
    }
    
    .method public hidebysig virtual instance void Invoke(string message) runtime managed
    {
    }
}

; 使用委托
.method public static void UseDelegateExample() cil managed
{
    .maxstack 3
    .locals init ([0] class MyDelegate del)
    
    ; 创建委托
    ldnull
    ldftn void [mscorlib]System.Console::WriteLine(string)
    newobj instance void MyDelegate::.ctor(object, native int)
    stloc.0
    
    ; 调用委托
    ldloc.0
    ldstr "Hello from delegate"
    callvirt instance void MyDelegate::Invoke(string)
    ret
}
```

### 函数指针直接调用

```msil
; 通过函数指针调用
.method public static void CallFunctionPointer() cil managed
{
    .maxstack 2
    
    ; 获取函数指针
    ldftn void [mscorlib]System.Console::WriteLine(string)
    
    ; 准备参数
    ldstr "Hello from function pointer"
    
    ; 通过函数指针调用
    calli void(string)
    ret
}
```

## 返回指令

### 基本返回

```msil
ret                         ; 从方法返回

; 返回值的方法
.method public static int32 ReturnValue() cil managed
{
    .maxstack 1
    
    ldc.i4 42                   ; 加载返回值
    ret                         ; 返回值
}

; 无返回值的方法
.method public static void ReturnVoid() cil managed
{
    .maxstack 1
    
    ldstr "Method executed"
    call void [mscorlib]System.Console::WriteLine(string)
    ret                         ; 返回（无值）
}
```

### 提前返回

```msil
; 条件提前返回
.method public static int32 EarlyReturn(int32 value) cil managed
{
    .maxstack 2
    
    ldarg.0                     ; 加载参数
    ldc.i4.0
    ble.s negative_case         ; 如果 value <= 0
    
    ; 正数情况
    ldarg.0
    ldc.i4.2
    mul                         ; value * 2
    ret                         ; 提前返回
    
negative_case:
    ldc.i4.0                    ; 返回 0
    ret
}
```

## 方法签名和描述符

### 方法签名格式

```msil
; 基本格式：
; [calling_convention] return_type [class_name::]method_name(parameter_types)

; 静态方法
call int32 MyClass::StaticMethod(string, int32)

; 实例方法
callvirt instance void MyClass::InstanceMethod(string)

; 泛型方法
call !!0 MyClass::GenericMethod<int32>(!!0)

; 数组方法
call int32 int32[]::get_Length()
```

### 调用约定

```msil
; 默认调用约定（managed）
call void MyClass::ManagedMethod()

; 非托管调用约定
call unmanaged stdcall int32 NativeMethod(int32)
call unmanaged cdecl void NativeMethod2()
call unmanaged fastcall int32 NativeMethod3(int32, int32)
```

## 参数传递

### 值类型参数

```msil
.method public static int32 AddNumbers(int32 a, int32 b) cil managed
{
    .maxstack 2
    
    ldarg.0                     ; 加载第一个参数
    ldarg.1                     ; 加载第二个参数
    add                         ; 相加
    ret                         ; 返回结果
}

; 调用示例
.method public static void CallAddNumbers() cil managed
{
    .maxstack 2
    
    ldc.i4.5                    ; 第一个参数
    ldc.i4.3                    ; 第二个参数
    call int32 MyClass::AddNumbers(int32, int32)
    
    call void [mscorlib]System.Console::WriteLine(int32)
    ret
}
```

### 引用类型参数

```msil
.method public static void ProcessString(string input) cil managed
{
    .maxstack 2
    
    ldarg.0                     ; 加载字符串参数
    brfalse.s null_case         ; 检查是否为 null
    
    ldarg.0
    callvirt instance string [mscorlib]System.String::ToUpper()
    call void [mscorlib]System.Console::WriteLine(string)
    ret
    
null_case:
    ldstr "Input is null"
    call void [mscorlib]System.Console::WriteLine(string)
    ret
}
```

### 引用参数（ref）

```msil
.method public static void SwapIntegers(int32& a, int32& b) cil managed
{
    .maxstack 2
    .locals init ([0] int32 temp)
    
    ; temp = a
    ldarg.0                     ; 加载 a 的地址
    ldind.i4                    ; 加载 a 的值
    stloc.0                     ; temp = a
    
    ; a = b
    ldarg.0                     ; 加载 a 的地址
    ldarg.1                     ; 加载 b 的地址
    ldind.i4                    ; 加载 b 的值
    stind.i4                    ; a = b
    
    ; b = temp
    ldarg.1                     ; 加载 b 的地址
    ldloc.0                     ; 加载 temp
    stind.i4                    ; b = temp
    
    ret
}

; 调用引用参数方法
.method public static void CallSwapMethod() cil managed
{
    .maxstack 2
    .locals init (
        [0] int32 x,
        [1] int32 y
    )
    
    ldc.i4.5
    stloc.0                     ; x = 5
    ldc.i4 10
    stloc.1                     ; y = 10
    
    ldloca.s 0                  ; 加载 x 的地址
    ldloca.s 1                  ; 加载 y 的地址
    call void MyClass::SwapIntegers(int32&, int32&)
    
    ret
}
```

### 输出参数（out）

```msil
.method public static bool TryParseInteger(string input, int32& result) cil managed
{
    .maxstack 2
    
    ldarg.0                     ; 加载输入字符串
    ldarg.1                     ; 加载结果地址
    call bool [mscorlib]System.Int32::TryParse(string, int32&)
    ret                         ; 返回解析是否成功
}

; 调用输出参数方法
.method public static void CallTryParseMethod() cil managed
{
    .maxstack 2
    .locals init (
        [0] int32 result,
        [1] bool success
    )
    
    ldstr "123"                 ; 输入字符串
    ldloca.s 0                  ; result 的地址
    call bool MyClass::TryParseInteger(string, int32&)
    stloc.1                     ; success = TryParse result
    
    ret
}
```

## 可变参数方法

### params 参数

```msil
.method public static void PrintNumbers(int32[] numbers) cil managed
{
    .maxstack 3
    .locals init (
        [0] int32 i,
        [1] int32 length
    )
    
    ldarg.0                     ; 加载数组
    brfalse.s end               ; 如果数组为 null
    
    ldarg.0
    ldlen                       ; 获取数组长度
    conv.i4
    stloc.1                     ; length = array.Length
    
    ldc.i4.0
    stloc.0                     ; i = 0
    br.s condition
    
loop_start:
    ldarg.0                     ; 加载数组
    ldloc.0                     ; 加载索引
    ldelem.i4                   ; 获取元素
    call void [mscorlib]System.Console::WriteLine(int32)
    
    ldloc.0
    ldc.i4.1
    add
    stloc.0                     ; i++
    
condition:
    ldloc.0                     ; 加载 i
    ldloc.1                     ; 加载 length
    blt.s loop_start            ; 如果 i < length 继续循环
    
end:
    ret
}

; 调用可变参数方法
.method public static void CallParamsMethod() cil managed
{
    .maxstack 4
    
    ; 创建数组
    ldc.i4.3                    ; 数组长度
    newarr [mscorlib]System.Int32
    dup
    ldc.i4.0                    ; 索引 0
    ldc.i4.1                    ; 值 1
    stelem.i4
    dup
    ldc.i4.1                    ; 索引 1
    ldc.i4.2                    ; 值 2
    stelem.i4
    dup
    ldc.i4.2                    ; 索引 2
    ldc.i4.3                    ; 值 3
    stelem.i4
    
    call void MyClass::PrintNumbers(int32[])
    ret
}
```

## 泛型方法调用

### 泛型方法定义和调用

```msil
; 泛型方法定义
.method public static !!T GenericMethod<T>(!!T input) cil managed
{
    .maxstack 1
    
    ldarg.0                     ; 加载泛型参数
    ret                         ; 返回相同的值
}

; 调用泛型方法
.method public static void CallGenericMethod() cil managed
{
    .maxstack 1
    
    ; 调用 GenericMethod<int32>
    ldc.i4 42
    call !!0 MyClass::GenericMethod<int32>(!!0)
    call void [mscorlib]System.Console::WriteLine(int32)
    
    ; 调用 GenericMethod<string>
    ldstr "Hello"
    call !!0 MyClass::GenericMethod<string>(!!0)
    call void [mscorlib]System.Console::WriteLine(string)
    
    ret
}
```

### 约束泛型方法

```msil
; 带约束的泛型方法
.method public static void ConstrainedGenericMethod<(class [mscorlib]System.IComparable) T>(!!T value) cil managed
{
    .maxstack 2
    
    ldarg.0                     ; 加载泛型值
    ldarg.0                     ; 再次加载用于比较
    constrained. !!T            ; 约束调用
    callvirt instance int32 [mscorlib]System.IComparable::CompareTo(object)
    
    call void [mscorlib]System.Console::WriteLine(int32)
    ret
}
```

## 性能优化

### 内联方法

```msil
; 标记为内联的方法
.method public static aggressiveinlining int32 SimpleAdd(int32 a, int32 b) cil managed
{
    .maxstack 2
    
    ldarg.0
    ldarg.1
    add
    ret
}
```

### 尾调用优化

```msil
; 尾调用优化
.method public static int32 TailCallExample(int32 n, int32 acc) cil managed
{
    .maxstack 3
    
    ldarg.0                     ; 加载 n
    ldc.i4.0
    ble.s base_case             ; 如果 n <= 0
    
    ; 尾递归调用
    ldarg.0                     ; n
    ldc.i4.1
    sub                         ; n - 1
    ldarg.1                     ; acc
    ldarg.0                     ; n
    add                         ; acc + n
    tail.                       ; 尾调用前缀
    call int32 MyClass::TailCallExample(int32, int32)
    ret
    
base_case:
    ldarg.1                     ; 返回累加器
    ret
}
```

### 方法调用缓存

```msil
; 缓存频繁调用的方法结果
.field private static class [mscorlib]System.Collections.Generic.Dictionary`2<string,int32> cache

.method public static int32 CachedExpensiveOperation(string input) cil managed
{
    .maxstack 3
    .locals init (
        [0] int32 result,
        [1] bool found
    )
    
    ; 检查缓存
    ldsfld class [mscorlib]System.Collections.Generic.Dictionary`2<string,int32> MyClass::cache
    ldarg.0                     ; 输入键
    ldloca.s 0                  ; 结果地址
    callvirt instance bool class [mscorlib]System.Collections.Generic.Dictionary`2<string,int32>::TryGetValue(!0, !1&)
    stloc.1                     ; found = TryGetValue result
    
    ldloc.1
    brtrue.s return_cached      ; 如果找到缓存值
    
    ; 执行昂贵操作
    ldarg.0
    call int32 MyClass::ExpensiveOperation(string)
    stloc.0                     ; result = ExpensiveOperation(input)
    
    ; 存储到缓存
    ldsfld class [mscorlib]System.Collections.Generic.Dictionary`2<string,int32> MyClass::cache
    ldarg.0                     ; 键
    ldloc.0                     ; 值
    callvirt instance void class [mscorlib]System.Collections.Generic.Dictionary`2<string,int32>::set_Item(!0, !1)
    
return_cached:
    ldloc.0                     ; 返回结果
    ret
}
```

## 相关文档

- [基础指令](./basic-instructions.md)
- [控制流指令](./control-flow-instructions.md)
- [对象操作指令](./object-instructions.md)
- [异常处理指令](./exception-instructions.md)