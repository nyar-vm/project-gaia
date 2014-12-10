# MSIL 对象操作指令

Microsoft Intermediate Language (MSIL) 对象和类操作指令，包括对象创建、字段访问、数组操作、类型检查和转换。

## 对象创建指令

### 基本对象创建

```msil
newobj constructor          ; 创建对象并调用构造函数
initobj type               ; 初始化值类型对象

; 示例：创建引用类型对象
.method public static void CreateReferenceObject() cil managed
{
    .maxstack 2
    .locals init ([0] class [mscorlib]System.Object obj)
    
    ; 创建 Object 实例
    newobj instance void [mscorlib]System.Object::.ctor()
    stloc.0                     ; obj = new Object()
    
    ; 创建 String 实例
    ldstr "Hello"
    newobj instance void [mscorlib]System.String::.ctor(string)
    
    call void [mscorlib]System.Console::WriteLine(string)
    ret
}

; 示例：初始化值类型
.method public static void CreateValueType() cil managed
{
    .maxstack 2
    .locals init ([0] valuetype [mscorlib]System.DateTime dt)
    
    ; 初始化 DateTime 结构
    ldloca.s 0                  ; 加载 DateTime 地址
    initobj [mscorlib]System.DateTime
    
    ; 使用构造函数创建 DateTime
    ldc.i8 637000000000000000   ; ticks
    newobj instance void [mscorlib]System.DateTime::.ctor(int64)
    stloc.0
    
    ret
}
```

### 数组创建

```msil
newarr type                 ; 创建一维数组
newobj constructor          ; 创建多维数组

; 示例：创建一维数组
.method public static void CreateSingleArray() cil managed
{
    .maxstack 4
    .locals init ([0] int32[] arr)
    
    ; 创建 int32 数组，长度为 5
    ldc.i4.5
    newarr [mscorlib]System.Int32
    stloc.0                     ; arr = new int[5]
    
    ; 设置数组元素
    ldloc.0                     ; 加载数组
    ldc.i4.0                    ; 索引 0
    ldc.i4 10                   ; 值 10
    stelem.i4                   ; arr[0] = 10
    
    ldloc.0                     ; 加载数组
    ldc.i4.1                    ; 索引 1
    ldc.i4 20                   ; 值 20
    stelem.i4                   ; arr[1] = 20
    
    ret
}

; 示例：创建多维数组
.method public static void CreateMultiArray() cil managed
{
    .maxstack 5
    .locals init ([0] int32[,] matrix)
    
    ; 创建 3x3 二维数组
    ldc.i4.3                    ; 第一维长度
    ldc.i4.3                    ; 第二维长度
    newobj instance void int32[,]::.ctor(int32, int32)
    stloc.0                     ; matrix = new int[3,3]
    
    ; 设置元素 matrix[1,2] = 42
    ldloc.0                     ; 加载数组
    ldc.i4.1                    ; 第一维索引
    ldc.i4.2                    ; 第二维索引
    ldc.i4 42                   ; 值
    call instance void int32[,]::Set(int32, int32, int32)
    
    ret
}

; 示例：创建锯齿数组
.method public static void CreateJaggedArray() cil managed
{
    .maxstack 4
    .locals init ([0] int32[][] jaggedArray)
    
    ; 创建外层数组
    ldc.i4.3
    newarr int32[]
    stloc.0                     ; jaggedArray = new int[3][]
    
    ; 创建第一个内层数组
    ldloc.0                     ; 加载外层数组
    ldc.i4.0                    ; 索引 0
    ldc.i4.2                    ; 内层数组长度
    newarr [mscorlib]System.Int32
    stelem.ref                  ; jaggedArray[0] = new int[2]
    
    ; 创建第二个内层数组
    ldloc.0                     ; 加载外层数组
    ldc.i4.1                    ; 索引 1
    ldc.i4.4                    ; 内层数组长度
    newarr [mscorlib]System.Int32
    stelem.ref                  ; jaggedArray[1] = new int[4]
    
    ret
}
```

## 字段访问指令

### 实例字段访问

```msil
ldfld field                 ; 加载实例字段值
ldflda field                ; 加载实例字段地址
stfld field                 ; 存储到实例字段

; 示例类定义
.class public MyClass extends [mscorlib]System.Object
{
    .field public int32 instanceField
    .field private string privateField
    
    ; 构造函数
    .method public hidebysig specialname rtspecialname 
            instance void .ctor() cil managed
    {
        .maxstack 2
        
        ldarg.0                 ; 加载 this
        call instance void [mscorlib]System.Object::.ctor()
        
        ; 初始化字段
        ldarg.0                 ; 加载 this
        ldc.i4.0
        stfld int32 MyClass::instanceField
        
        ldarg.0                 ; 加载 this
        ldstr "default"
        stfld string MyClass::privateField
        
        ret
    }
    
    ; 访问实例字段的方法
    .method public instance void AccessInstanceFields() cil managed
    {
        .maxstack 2
        
        ; 读取字段
        ldarg.0                 ; 加载 this
        ldfld int32 MyClass::instanceField
        ldc.i4.1
        add
        
        ; 写入字段
        ldarg.0                 ; 加载 this
        swap                    ; 交换栈顶两个值
        stfld int32 MyClass::instanceField
        
        ret
    }
}
```

### 静态字段访问

```msil
ldsfld field                ; 加载静态字段值
ldsflda field               ; 加载静态字段地址
stsfld field                ; 存储到静态字段

; 示例：静态字段
.class public StaticExample extends [mscorlib]System.Object
{
    .field public static int32 staticCounter
    .field private static string staticMessage
    
    ; 静态构造函数
    .method private hidebysig specialname rtspecialname static 
            void .cctor() cil managed
    {
        .maxstack 1
        
        ldc.i4.0
        stsfld int32 StaticExample::staticCounter
        
        ldstr "Initialized"
        stsfld string StaticExample::staticMessage
        
        ret
    }
    
    ; 访问静态字段
    .method public static void AccessStaticFields() cil managed
    {
        .maxstack 2
        
        ; 递增计数器
        ldsfld int32 StaticExample::staticCounter
        ldc.i4.1
        add
        stsfld int32 StaticExample::staticCounter
        
        ; 读取消息
        ldsfld string StaticExample::staticMessage
        call void [mscorlib]System.Console::WriteLine(string)
        
        ret
    }
}
```

### 字段地址操作

```msil
; 获取字段地址用于引用传递
.method public static void FieldAddressExample() cil managed
{
    .maxstack 2
    .locals init ([0] class MyClass obj)
    
    newobj instance void MyClass::.ctor()
    stloc.0
    
    ; 获取实例字段地址
    ldloc.0
    ldflda int32 MyClass::instanceField
    
    ; 获取静态字段地址
    ldsflda int32 StaticExample::staticCounter
    
    ; 可以将这些地址传递给需要引用参数的方法
    ret
}
```

## 数组操作指令

### 数组元素访问

```msil
ldelem type                 ; 加载数组元素
ldelem.i1, ldelem.i2, ldelem.i4, ldelem.i8    ; 加载整数元素
ldelem.r4, ldelem.r8        ; 加载浮点元素
ldelem.ref                  ; 加载引用类型元素
ldelema type                ; 加载数组元素地址

stelem type                 ; 存储数组元素
stelem.i1, stelem.i2, stelem.i4, stelem.i8    ; 存储整数元素
stelem.r4, stelem.r8        ; 存储浮点元素
stelem.ref                  ; 存储引用类型元素

; 示例：数组元素操作
.method public static void ArrayElementOperations() cil managed
{
    .maxstack 4
    .locals init (
        [0] int32[] intArray,
        [1] string[] stringArray,
        [2] float64[] doubleArray
    )
    
    ; 创建并初始化整数数组
    ldc.i4.3
    newarr [mscorlib]System.Int32
    stloc.0
    
    ; 设置 intArray[0] = 10
    ldloc.0                     ; 加载数组
    ldc.i4.0                    ; 索引
    ldc.i4 10                   ; 值
    stelem.i4
    
    ; 设置 intArray[1] = 20
    ldloc.0
    ldc.i4.1
    ldc.i4 20
    stelem.i4
    
    ; 读取 intArray[0]
    ldloc.0
    ldc.i4.0
    ldelem.i4                   ; 结果在栈顶
    
    ; 创建字符串数组
    ldc.i4.2
    newarr [mscorlib]System.String
    stloc.1
    
    ; 设置 stringArray[0] = "Hello"
    ldloc.1
    ldc.i4.0
    ldstr "Hello"
    stelem.ref
    
    ; 读取 stringArray[0]
    ldloc.1
    ldc.i4.0
    ldelem.ref
    
    call void [mscorlib]System.Console::WriteLine(string)
    ret
}
```

### 数组长度和边界

```msil
ldlen                       ; 获取数组长度

; 示例：数组长度操作
.method public static void ArrayLengthExample() cil managed
{
    .maxstack 3
    .locals init (
        [0] int32[] array,
        [1] int32 length,
        [2] int32 i
    )
    
    ; 创建数组
    ldc.i4 10
    newarr [mscorlib]System.Int32
    stloc.0
    
    ; 获取数组长度
    ldloc.0
    ldlen
    conv.i4                     ; 转换为 int32
    stloc.1                     ; length = array.Length
    
    ; 遍历数组
    ldc.i4.0
    stloc.2                     ; i = 0
    br.s condition
    
loop_start:
    ; 设置 array[i] = i * 2
    ldloc.0                     ; 加载数组
    ldloc.2                     ; 加载索引
    ldloc.2                     ; 加载 i
    ldc.i4.2
    mul                         ; i * 2
    stelem.i4                   ; array[i] = i * 2
    
    ; i++
    ldloc.2
    ldc.i4.1
    add
    stloc.2
    
condition:
    ldloc.2                     ; 加载 i
    ldloc.1                     ; 加载 length
    blt.s loop_start            ; 如果 i < length 继续循环
    
    ret
}
```

### 多维数组操作

```msil
; 多维数组特殊方法
call instance type [,]::Get(int32, int32)      ; 获取元素
call instance void [,]::Set(int32, int32, type) ; 设置元素
call instance int32 [,]::GetLength(int32)       ; 获取指定维度长度

; 示例：多维数组操作
.method public static void MultiDimensionalArrayExample() cil managed
{
    .maxstack 5
    .locals init (
        [0] int32[,] matrix,
        [1] int32 rows,
        [2] int32 cols,
        [3] int32 i,
        [4] int32 j
    )
    
    ; 创建 3x4 矩阵
    ldc.i4.3
    ldc.i4.4
    newobj instance void int32[,]::.ctor(int32, int32)
    stloc.0
    
    ; 获取维度信息
    ldloc.0
    ldc.i4.0                    ; 第 0 维
    call instance int32 int32[,]::GetLength(int32)
    stloc.1                     ; rows = matrix.GetLength(0)
    
    ldloc.0
    ldc.i4.1                    ; 第 1 维
    call instance int32 int32[,]::GetLength(int32)
    stloc.2                     ; cols = matrix.GetLength(1)
    
    ; 初始化矩阵
    ldc.i4.0
    stloc.3                     ; i = 0
    br.s outer_condition
    
outer_loop:
    ldc.i4.0
    stloc.s 4                   ; j = 0
    br.s inner_condition
    
inner_loop:
    ; matrix[i,j] = i * cols + j
    ldloc.0                     ; 加载矩阵
    ldloc.3                     ; i
    ldloc.s 4                   ; j
    ldloc.3                     ; i
    ldloc.2                     ; cols
    mul                         ; i * cols
    ldloc.s 4                   ; j
    add                         ; i * cols + j
    call instance void int32[,]::Set(int32, int32, int32)
    
    ; j++
    ldloc.s 4
    ldc.i4.1
    add
    stloc.s 4
    
inner_condition:
    ldloc.s 4                   ; j
    ldloc.2                     ; cols
    blt.s inner_loop
    
    ; i++
    ldloc.3
    ldc.i4.1
    add
    stloc.3
    
outer_condition:
    ldloc.3                     ; i
    ldloc.1                     ; rows
    blt.s outer_loop
    
    ret
}
```

## 类型检查和转换指令

### 类型检查

```msil
isinst type                 ; 检查对象是否为指定类型
castclass type              ; 强制类型转换

; 示例：类型检查
.method public static void TypeCheckingExample(object obj) cil managed
{
    .maxstack 2
    .locals init ([0] string str)
    
    ; 检查对象是否为字符串
    ldarg.0                     ; 加载对象
    isinst [mscorlib]System.String
    stloc.0                     ; str = obj as string
    
    ldloc.0
    brfalse.s not_string        ; 如果不是字符串
    
    ; 是字符串的情况
    ldstr "Object is a string: "
    ldloc.0
    call string [mscorlib]System.String::Concat(string, string)
    call void [mscorlib]System.Console::WriteLine(string)
    br.s end
    
not_string:
    ; 不是字符串的情况
    ldstr "Object is not a string"
    call void [mscorlib]System.Console::WriteLine(string)
    
end:
    ret
}

; 示例：强制类型转换
.method public static void CastingExample(object obj) cil managed
{
    .maxstack 1
    .locals init ([0] string str)
    
    ; 强制转换为字符串（可能抛出异常）
    ldarg.0
    castclass [mscorlib]System.String
    stloc.0                     ; str = (string)obj
    
    ldloc.0
    call void [mscorlib]System.Console::WriteLine(string)
    ret
}
```

### 装箱和拆箱

```msil
box type                    ; 装箱值类型
unbox type                  ; 拆箱到值类型地址
unbox.any type              ; 拆箱到值类型值

; 示例：装箱和拆箱
.method public static void BoxingUnboxingExample() cil managed
{
    .maxstack 2
    .locals init (
        [0] int32 value,
        [1] object boxedValue,
        [2] int32 unboxedValue
    )
    
    ; 装箱
    ldc.i4 42
    stloc.0                     ; value = 42
    
    ldloc.0
    box [mscorlib]System.Int32  ; 装箱
    stloc.1                     ; boxedValue = (object)value
    
    ; 拆箱方法1：使用 unbox + ldind
    ldloc.1                     ; 加载装箱的对象
    unbox [mscorlib]System.Int32 ; 获取值的地址
    ldind.i4                    ; 加载值
    stloc.2                     ; unboxedValue = (int)boxedValue
    
    ; 拆箱方法2：使用 unbox.any
    ldloc.1                     ; 加载装箱的对象
    unbox.any [mscorlib]System.Int32
    stloc.2                     ; unboxedValue = (int)boxedValue
    
    ret
}
```

## 对象比较指令

### 引用比较

```msil
ceq                         ; 比较相等性
cgt, clt                    ; 比较大小（用于引用比较时比较地址）

; 示例：引用比较
.method public static void ReferenceComparisonExample() cil managed
{
    .maxstack 3
    .locals init (
        [0] string str1,
        [1] string str2,
        [2] bool areEqual
    )
    
    ldstr "Hello"
    stloc.0                     ; str1 = "Hello"
    
    ldstr "Hello"
    stloc.1                     ; str2 = "Hello"
    
    ; 引用比较
    ldloc.0
    ldloc.1
    ceq                         ; str1 == str2 (引用比较)
    stloc.2                     ; areEqual = (str1 == str2)
    
    ldloc.2
    call void [mscorlib]System.Console::WriteLine(bool)
    ret
}
```

### 值比较

```msil
; 示例：调用 Equals 方法进行值比较
.method public static void ValueComparisonExample() cil managed
{
    .maxstack 2
    .locals init (
        [0] string str1,
        [1] string str2,
        [2] bool areEqual
    )
    
    ldstr "Hello"
    stloc.0
    
    ldstr "Hello"
    stloc.1
    
    ; 值比较
    ldloc.0
    ldloc.1
    callvirt instance bool [mscorlib]System.String::Equals(string)
    stloc.2                     ; areEqual = str1.Equals(str2)
    
    ldloc.2
    call void [mscorlib]System.Console::WriteLine(bool)
    ret
}
```

## null 检查

### null 检查和处理

```msil
; 示例：null 检查
.method public static void NullCheckExample(object obj) cil managed
{
    .maxstack 1
    
    ldarg.0                     ; 加载对象
    brfalse.s is_null           ; 如果为 null
    
    ; 对象不为 null
    ldstr "Object is not null"
    call void [mscorlib]System.Console::WriteLine(string)
    br.s end
    
is_null:
    ; 对象为 null
    ldstr "Object is null"
    call void [mscorlib]System.Console::WriteLine(string)
    
end:
    ret
}

; 示例：null 合并操作
.method public static string NullCoalescingExample(string input) cil managed
{
    .maxstack 1
    
    ldarg.0                     ; 加载输入
    brtrue.s not_null           ; 如果不为 null
    
    ; 输入为 null，返回默认值
    ldstr "default value"
    ret
    
not_null:
    ; 输入不为 null，返回输入
    ldarg.0
    ret
}
```

## 性能优化

### 对象池模式

```msil
; 对象池实现示例
.class public ObjectPool extends [mscorlib]System.Object
{
    .field private static class [mscorlib]System.Collections.Generic.Stack`1<class MyObject> pool
    
    .method private hidebysig specialname rtspecialname static 
            void .cctor() cil managed
    {
        .maxstack 1
        
        newobj instance void class [mscorlib]System.Collections.Generic.Stack`1<class MyObject>::.ctor()
        stsfld class [mscorlib]System.Collections.Generic.Stack`1<class MyObject> ObjectPool::pool
        ret
    }
    
    .method public static class MyObject GetObject() cil managed
    {
        .maxstack 1
        
        ldsfld class [mscorlib]System.Collections.Generic.Stack`1<class MyObject> ObjectPool::pool
        callvirt instance int32 class [mscorlib]System.Collections.Generic.Stack`1<class MyObject>::get_Count()
        brfalse.s create_new
        
        ; 从池中获取对象
        ldsfld class [mscorlib]System.Collections.Generic.Stack`1<class MyObject> ObjectPool::pool
        callvirt instance !0 class [mscorlib]System.Collections.Generic.Stack`1<class MyObject>::Pop()
        ret
        
create_new:
        ; 创建新对象
        newobj instance void MyObject::.ctor()
        ret
    }
    
    .method public static void ReturnObject(class MyObject obj) cil managed
    {
        .maxstack 2
        
        ldarg.0
        brfalse.s end               ; 如果对象为 null，直接返回
        
        ; 重置对象状态
        ldarg.0
        callvirt instance void MyObject::Reset()
        
        ; 返回到池中
        ldsfld class [mscorlib]System.Collections.Generic.Stack`1<class MyObject> ObjectPool::pool
        ldarg.0
        callvirt instance void class [mscorlib]System.Collections.Generic.Stack`1<class MyObject>::Push(!0)
        
end:
        ret
    }
}
```

### 字段访问优化

```msil
; 缓存频繁访问的字段
.method public instance void OptimizedFieldAccess() cil managed
{
    .maxstack 2
    .locals init ([0] int32 cachedField)
    
    ; 缓存字段值，避免重复访问
    ldarg.0
    ldfld int32 MyClass::instanceField
    stloc.0                     ; cachedField = this.instanceField
    
    ; 使用缓存的值进行多次计算
    ldloc.0                     ; 使用缓存值
    ldc.i4.2
    mul
    
    ldloc.0                     ; 再次使用缓存值
    ldc.i4.3
    add
    
    ; 最后更新字段
    ldarg.0
    ldloc.0
    ldc.i4.1
    add
    stfld int32 MyClass::instanceField
    
    ret
}
```

### 数组访问优化

```msil
; 边界检查优化
.method public static void OptimizedArrayAccess(int32[] array) cil managed
{
    .maxstack 3
    .locals init (
        [0] int32 length,
        [1] int32 i
    )
    
    ldarg.0
    brfalse.s end               ; 检查数组是否为 null
    
    ; 缓存数组长度
    ldarg.0
    ldlen
    conv.i4
    stloc.0                     ; length = array.Length
    
    ldc.i4.0
    stloc.1                     ; i = 0
    br.s condition
    
loop_start:
    ; 直接使用 ldelem/stelem，JIT 会优化边界检查
    ldarg.0
    ldloc.1
    ldarg.0
    ldloc.1
    ldelem.i4
    ldc.i4.1
    add
    stelem.i4                   ; array[i] = array[i] + 1
    
    ldloc.1
    ldc.i4.1
    add
    stloc.1                     ; i++
    
condition:
    ldloc.1
    ldloc.0
    blt.s loop_start            ; i < length
    
end:
    ret
}
```

## 相关文档

- [基础指令](./basic-instructions.md)
- [算术指令](./arithmetic-instructions.md)
- [控制流指令](./control-flow-instructions.md)
- [方法调用指令](./method-instructions.md)
- [异常处理指令](./exception-instructions.md)