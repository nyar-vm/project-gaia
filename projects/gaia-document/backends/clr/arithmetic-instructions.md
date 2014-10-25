# MSIL 算术指令

Microsoft Intermediate Language (MSIL) 算术指令，包括基本算术运算、位运算、类型转换和数值比较。

## 基本算术运算

### 加法指令

```msil
add                         ; 整数/浮点数加法
add.ovf                     ; 有溢出检查的有符号加法
add.ovf.un                  ; 有溢出检查的无符号加法

; 示例：计算 a + b
ldloc.0                     ; 加载 a
ldloc.1                     ; 加载 b
add                         ; 执行加法
stloc.2                     ; 存储结果到 c
```

### 减法指令

```msil
sub                         ; 整数/浮点数减法
sub.ovf                     ; 有溢出检查的有符号减法
sub.ovf.un                  ; 有溢出检查的无符号减法

; 示例：计算 a - b
ldloc.0                     ; 加载 a
ldloc.1                     ; 加载 b
sub                         ; 执行减法
stloc.2                     ; 存储结果到 c
```

### 乘法指令

```msil
mul                         ; 整数/浮点数乘法
mul.ovf                     ; 有溢出检查的有符号乘法
mul.ovf.un                  ; 有溢出检查的无符号乘法

; 示例：计算 a * b
ldloc.0                     ; 加载 a
ldloc.1                     ; 加载 b
mul                         ; 执行乘法
stloc.2                     ; 存储结果到 c
```

### 除法指令

```msil
div                         ; 有符号整数/浮点数除法
div.un                      ; 无符号整数除法

; 示例：计算 a / b
ldloc.0                     ; 加载 a
ldloc.1                     ; 加载 b
div                         ; 执行除法
stloc.2                     ; 存储结果到 c
```

### 取模指令

```msil
rem                         ; 有符号整数取模
rem.un                      ; 无符号整数取模

; 示例：计算 a % b
ldloc.0                     ; 加载 a
ldloc.1                     ; 加载 b
rem                         ; 执行取模
stloc.2                     ; 存储结果到 c
```

### 取负指令

```msil
neg                         ; 取负值

; 示例：计算 -a
ldloc.0                     ; 加载 a
neg                         ; 取负
stloc.1                     ; 存储结果到 b
```

## 位运算指令

### 逻辑运算

```msil
and                         ; 按位与
or                          ; 按位或
xor                         ; 按位异或
not                         ; 按位取反

; 示例：按位与运算
ldloc.0                     ; 加载 a
ldloc.1                     ; 加载 b
and                         ; 执行按位与
stloc.2                     ; 存储结果
```

### 移位运算

```msil
shl                         ; 左移
shr                         ; 算术右移（有符号）
shr.un                      ; 逻辑右移（无符号）

; 示例：左移运算
ldloc.0                     ; 加载值
ldc.i4.2                    ; 加载移位位数
shl                         ; 左移 2 位
stloc.1                     ; 存储结果
```

## 类型转换指令

### 整数转换

```msil
conv.i1                     ; 转换为 int8 (sbyte)
conv.u1                     ; 转换为 uint8 (byte)
conv.i2                     ; 转换为 int16 (short)
conv.u2                     ; 转换为 uint16 (ushort)
conv.i4                     ; 转换为 int32 (int)
conv.u4                     ; 转换为 uint32 (uint)
conv.i8                     ; 转换为 int64 (long)
conv.u8                     ; 转换为 uint64 (ulong)
conv.i                      ; 转换为 native int
conv.u                      ; 转换为 native uint
```

### 浮点转换

```msil
conv.r4                     ; 转换为 float32 (float)
conv.r8                     ; 转换为 float64 (double)
conv.r.un                   ; 无符号整数转换为浮点数
```

### 溢出检查转换

```msil
conv.ovf.i1                 ; 有溢出检查转换为 int8
conv.ovf.u1                 ; 有溢出检查转换为 uint8
conv.ovf.i2                 ; 有溢出检查转换为 int16
conv.ovf.u2                 ; 有溢出检查转换为 uint16
conv.ovf.i4                 ; 有溢出检查转换为 int32
conv.ovf.u4                 ; 有溢出检查转换为 uint32
conv.ovf.i8                 ; 有溢出检查转换为 int64
conv.ovf.u8                 ; 有溢出检查转换为 uint64

; 无符号溢出检查转换
conv.ovf.i1.un              ; 无符号转换为 int8（有溢出检查）
conv.ovf.u1.un              ; 无符号转换为 uint8（有溢出检查）
; ... 其他类型类似
```

## 比较指令

### 相等比较

```msil
ceq                         ; 相等比较（返回 0 或 1）

; 示例：比较两个值是否相等
ldloc.0                     ; 加载 a
ldloc.1                     ; 加载 b
ceq                         ; 比较是否相等
stloc.2                     ; 存储结果（0 或 1）
```

### 大小比较

```msil
cgt                         ; 大于比较（有符号）
cgt.un                      ; 大于比较（无符号）
clt                         ; 小于比较（有符号）
clt.un                      ; 小于比较（无符号）

; 示例：检查 a > b
ldloc.0                     ; 加载 a
ldloc.1                     ; 加载 b
cgt                         ; 比较 a > b
stloc.2                     ; 存储结果（0 或 1）
```

## 算术运算示例

### 基本计算器

```msil
.method public static int32 Calculate(int32 a, int32 b, int32 operation) cil managed
{
    .maxstack 2
    .locals init ([0] int32 result)
    
    ; 加载操作数
    ldarg.0                     ; 加载 a
    ldarg.1                     ; 加载 b
    
    ; 根据操作类型执行运算
    ldarg.2                     ; 加载 operation
    ldc.i4.1
    beq.s add_operation
    
    ldarg.2
    ldc.i4.2
    beq.s sub_operation
    
    ldarg.2
    ldc.i4.3
    beq.s mul_operation
    
    ldarg.2
    ldc.i4.4
    beq.s div_operation
    
    ; 默认返回 0
    pop
    pop
    ldc.i4.0
    ret
    
add_operation:
    add
    ret
    
sub_operation:
    sub
    ret
    
mul_operation:
    mul
    ret
    
div_operation:
    div
    ret
}
```

### 数学函数示例

```msil
.method public static int32 Abs(int32 value) cil managed
{
    .maxstack 2
    
    ldarg.0                     ; 加载 value
    dup                         ; 复制值
    ldc.i4.0                    ; 加载 0
    clt                         ; 检查是否小于 0
    brfalse.s positive          ; 如果不小于 0，跳转到 positive
    
    neg                         ; 取负值
    ret
    
positive:
    ret                         ; 直接返回正值
}
```

### 位操作示例

```msil
.method public static bool IsPowerOfTwo(int32 value) cil managed
{
    .maxstack 3
    
    ; 检查 value > 0 && (value & (value - 1)) == 0
    ldarg.0                     ; 加载 value
    ldc.i4.0
    cgt                         ; value > 0
    brfalse.s not_power_of_two
    
    ldarg.0                     ; 加载 value
    ldarg.0                     ; 加载 value
    ldc.i4.1
    sub                         ; value - 1
    and                         ; value & (value - 1)
    ldc.i4.0
    ceq                         ; 结果是否等于 0
    ret
    
not_power_of_two:
    ldc.i4.0                    ; 返回 false
    ret
}
```

## 浮点运算

### 浮点数算术

```msil
.method public static float64 CalculateDistance(float64 x1, float64 y1, float64 x2, float64 y2) cil managed
{
    .maxstack 4
    
    ; 计算 sqrt((x2-x1)^2 + (y2-y1)^2)
    ldarg.2                     ; x2
    ldarg.0                     ; x1
    sub                         ; x2 - x1
    dup
    mul                         ; (x2 - x1)^2
    
    ldarg.3                     ; y2
    ldarg.1                     ; y1
    sub                         ; y2 - y1
    dup
    mul                         ; (y2 - y1)^2
    
    add                         ; (x2-x1)^2 + (y2-y1)^2
    call float64 [mscorlib]System.Math::Sqrt(float64)
    ret
}
```

### 浮点数比较

```msil
.method public static bool IsNearlyEqual(float64 a, float64 b, float64 epsilon) cil managed
{
    .maxstack 3
    
    ; 计算 |a - b| < epsilon
    ldarg.0                     ; a
    ldarg.1                     ; b
    sub                         ; a - b
    call float64 [mscorlib]System.Math::Abs(float64)
    ldarg.2                     ; epsilon
    clt                         ; |a - b| < epsilon
    ret
}
```

## 溢出处理

### 安全算术运算

```msil
.method public static int32 SafeAdd(int32 a, int32 b) cil managed
{
    .maxstack 2
    
    .try
    {
        ldarg.0
        ldarg.1
        add.ovf                 ; 有溢出检查的加法
        ret
    }
    catch [mscorlib]System.OverflowException
    {
        pop                     ; 弹出异常对象
        ldc.i4 2147483647       ; 返回 int.MaxValue
        ret
    }
}
```

### 无符号运算

```msil
.method public static uint32 UnsignedDivide(uint32 a, uint32 b) cil managed
{
    .maxstack 2
    
    ldarg.0
    ldarg.1
    div.un                      ; 无符号除法
    ret
}
```

## 类型转换示例

### 安全类型转换

```msil
.method public static int32 SafeFloatToInt(float64 value) cil managed
{
    .maxstack 1
    
    .try
    {
        ldarg.0
        conv.ovf.i4             ; 有溢出检查的转换
        ret
    }
    catch [mscorlib]System.OverflowException
    {
        pop                     ; 弹出异常对象
        ldc.i4.0                ; 返回默认值
        ret
    }
}
```

### 字节数组转换

```msil
.method public static int32 BytesToInt(uint8[] bytes, int32 startIndex) cil managed
{
    .maxstack 4
    
    ; 假设小端序：bytes[0] + (bytes[1] << 8) + (bytes[2] << 16) + (bytes[3] << 24)
    ldarg.0                     ; bytes
    ldarg.1                     ; startIndex
    ldelem.u1                   ; bytes[startIndex]
    conv.i4
    
    ldarg.0                     ; bytes
    ldarg.1                     ; startIndex
    ldc.i4.1
    add                         ; startIndex + 1
    ldelem.u1                   ; bytes[startIndex + 1]
    conv.i4
    ldc.i4.8
    shl                         ; << 8
    or
    
    ldarg.0                     ; bytes
    ldarg.1                     ; startIndex
    ldc.i4.2
    add                         ; startIndex + 2
    ldelem.u1                   ; bytes[startIndex + 2]
    conv.i4
    ldc.i4 16
    shl                         ; << 16
    or
    
    ldarg.0                     ; bytes
    ldarg.1                     ; startIndex
    ldc.i4.3
    add                         ; startIndex + 3
    ldelem.u1                   ; bytes[startIndex + 3]
    conv.i4
    ldc.i4 24
    shl                         ; << 24
    or
    
    ret
}
```

## 性能优化

### 常量折叠

```msil
; 编译时优化：常量表达式
; 源代码：int result = 10 + 20 * 3;
; 优化前：
ldc.i4 10
ldc.i4 20
ldc.i4.3
mul
add

; 优化后：
ldc.i4 70                   ; 编译器计算 10 + 20 * 3 = 70
```

### 强度削减

```msil
; 乘以 2 的幂次优化
; 优化前：乘以 8
ldloc.0
ldc.i4.8
mul

; 优化后：左移 3 位
ldloc.0
ldc.i4.3
shl                         ; 更快的位移操作
```

### 除法优化

```msil
; 除以 2 的幂次优化
; 优化前：除以 4
ldloc.0
ldc.i4.4
div

; 优化后：右移 2 位（仅适用于正数）
ldloc.0
ldc.i4.2
shr
```

## 数值精度考虑

### 浮点数精度

```msil
; 避免浮点数精度问题
.method public static bool FloatEquals(float32 a, float32 b) cil managed
{
    .maxstack 3
    
    ldarg.0
    ldarg.1
    sub                         ; a - b
    call float32 [mscorlib]System.Math::Abs(float32)
    ldc.r4 1e-6                 ; 小的 epsilon 值
    clt                         ; |a - b| < epsilon
    ret
}
```

## 相关文档

- [基础指令](./basic-instructions.md)
- [控制流指令](./control-flow-instructions.md)
- [方法调用指令](./method-instructions.md)
- [对象操作指令](./object-instructions.md)