# MSIL 控制流指令

Microsoft Intermediate Language (MSIL) 控制流指令，包括条件分支、无条件跳转、循环控制和 switch 语句。

## 无条件跳转指令

### 基本跳转

```msil
br label                    ; 无条件跳转到标签
br.s label                  ; 短距离无条件跳转（优化版本）

; 示例
start:
    ldc.i4.1
    stloc.0
    br end                  ; 跳转到 end 标签
    ldc.i4.2                ; 这行代码不会执行
    stloc.0
end:
    ret
```

### 跳转指令优化

```msil
; 短距离跳转（-128 到 +127 字节）
br.s short_label            ; 使用 1 字节偏移

; 长距离跳转
br long_label               ; 使用 4 字节偏移
```

## 条件跳转指令

### 真值条件跳转

```msil
brtrue label                ; 如果值为真（非零）则跳转
brtrue.s label              ; 短距离真值跳转
brfalse label               ; 如果值为假（零）则跳转
brfalse.s label             ; 短距离假值跳转

; 示例：条件检查
ldloc.0                     ; 加载布尔值
brtrue true_branch          ; 如果为真跳转
    ; false 分支代码
    ldstr "False"
    br end
true_branch:
    ; true 分支代码
    ldstr "True"
end:
    ; 继续执行
```

### 比较跳转指令

```msil
; 相等比较跳转
beq label                   ; 如果相等则跳转
beq.s label                 ; 短距离相等跳转
bne.un label                ; 如果不相等则跳转（无符号）
bne.un.s label              ; 短距离不相等跳转

; 大小比较跳转
bgt label                   ; 如果大于则跳转（有符号）
bgt.s label                 ; 短距离大于跳转
bgt.un label                ; 如果大于则跳转（无符号）
bgt.un.s label              ; 短距离无符号大于跳转

bge label                   ; 如果大于等于则跳转（有符号）
bge.s label                 ; 短距离大于等于跳转
bge.un label                ; 如果大于等于则跳转（无符号）
bge.un.s label              ; 短距离无符号大于等于跳转

blt label                   ; 如果小于则跳转（有符号）
blt.s label                 ; 短距离小于跳转
blt.un label                ; 如果小于则跳转（无符号）
blt.un.s label              ; 短距离无符号小于跳转

ble label                   ; 如果小于等于则跳转（有符号）
ble.s label                 ; 短距离小于等于跳转
ble.un label                ; 如果小于等于则跳转（无符号）
ble.un.s label              ; 短距离无符号小于等于跳转
```

## 控制流结构示例

### if-else 语句

```msil
; C# 代码：if (x > 0) { y = 1; } else { y = -1; }
.method public static void IfElseExample(int32 x) cil managed
{
    .maxstack 2
    .locals init ([0] int32 y)
    
    ldarg.0                     ; 加载 x
    ldc.i4.0                    ; 加载 0
    ble.s else_branch           ; 如果 x <= 0 跳转到 else
    
    ; if 分支 (x > 0)
    ldc.i4.1                    ; 加载 1
    stloc.0                     ; y = 1
    br.s end                    ; 跳转到结束
    
else_branch:
    ; else 分支 (x <= 0)
    ldc.i4.m1                   ; 加载 -1
    stloc.0                     ; y = -1
    
end:
    ret
}
```

### while 循环

```msil
; C# 代码：while (i < 10) { sum += i; i++; }
.method public static int32 WhileLoop(int32 start) cil managed
{
    .maxstack 2
    .locals init (
        [0] int32 i,
        [1] int32 sum
    )
    
    ldarg.0                     ; 加载 start
    stloc.0                     ; i = start
    ldc.i4.0
    stloc.1                     ; sum = 0
    
    br.s condition              ; 跳转到条件检查
    
loop_start:
    ldloc.1                     ; 加载 sum
    ldloc.0                     ; 加载 i
    add                         ; sum + i
    stloc.1                     ; sum = sum + i
    
    ldloc.0                     ; 加载 i
    ldc.i4.1
    add                         ; i + 1
    stloc.0                     ; i = i + 1
    
condition:
    ldloc.0                     ; 加载 i
    ldc.i4 10                   ; 加载 10
    blt.s loop_start            ; 如果 i < 10 继续循环
    
    ldloc.1                     ; 返回 sum
    ret
}
```

### for 循环

```msil
; C# 代码：for (int i = 0; i < 10; i++) { sum += i; }
.method public static int32 ForLoop() cil managed
{
    .maxstack 2
    .locals init (
        [0] int32 i,
        [1] int32 sum
    )
    
    ; 初始化
    ldc.i4.0
    stloc.0                     ; i = 0
    ldc.i4.0
    stloc.1                     ; sum = 0
    
    br.s condition              ; 跳转到条件检查
    
loop_start:
    ; 循环体
    ldloc.1                     ; 加载 sum
    ldloc.0                     ; 加载 i
    add                         ; sum + i
    stloc.1                     ; sum = sum + i
    
    ; 增量
    ldloc.0                     ; 加载 i
    ldc.i4.1
    add                         ; i + 1
    stloc.0                     ; i++
    
condition:
    ; 条件检查
    ldloc.0                     ; 加载 i
    ldc.i4 10                   ; 加载 10
    blt.s loop_start            ; 如果 i < 10 继续循环
    
    ldloc.1                     ; 返回 sum
    ret
}
```

### do-while 循环

```msil
; C# 代码：do { sum += i; i++; } while (i < 10);
.method public static int32 DoWhileLoop(int32 start) cil managed
{
    .maxstack 2
    .locals init (
        [0] int32 i,
        [1] int32 sum
    )
    
    ldarg.0                     ; 加载 start
    stloc.0                     ; i = start
    ldc.i4.0
    stloc.1                     ; sum = 0
    
loop_start:
    ; 循环体（至少执行一次）
    ldloc.1                     ; 加载 sum
    ldloc.0                     ; 加载 i
    add                         ; sum + i
    stloc.1                     ; sum = sum + i
    
    ldloc.0                     ; 加载 i
    ldc.i4.1
    add                         ; i + 1
    stloc.0                     ; i++
    
    ; 条件检查
    ldloc.0                     ; 加载 i
    ldc.i4 10                   ; 加载 10
    blt.s loop_start            ; 如果 i < 10 继续循环
    
    ldloc.1                     ; 返回 sum
    ret
}
```

## switch 语句

### 简单 switch

```msil
; C# 代码：switch (value) { case 1: return "One"; case 2: return "Two"; default: return "Other"; }
.method public static string SwitchExample(int32 value) cil managed
{
    .maxstack 1
    
    ldarg.0                     ; 加载 value
    ldc.i4.1
    beq.s case_one              ; 如果 value == 1
    
    ldarg.0                     ; 加载 value
    ldc.i4.2
    beq.s case_two              ; 如果 value == 2
    
    br.s default_case           ; 跳转到默认情况
    
case_one:
    ldstr "One"
    ret
    
case_two:
    ldstr "Two"
    ret
    
default_case:
    ldstr "Other"
    ret
}
```

### 优化的 switch（跳转表）

```msil
; 对于连续的整数值，编译器可能生成跳转表
.method public static string OptimizedSwitch(int32 value) cil managed
{
    .maxstack 1
    
    ldarg.0                     ; 加载 value
    switch (case_0, case_1, case_2, case_3)
    br.s default_case
    
case_0:
    ldstr "Zero"
    ret
    
case_1:
    ldstr "One"
    ret
    
case_2:
    ldstr "Two"
    ret
    
case_3:
    ldstr "Three"
    ret
    
default_case:
    ldstr "Other"
    ret
}
```

## 嵌套控制结构

### 嵌套循环

```msil
; C# 代码：for (int i = 0; i < 3; i++) { for (int j = 0; j < 3; j++) { sum += i * j; } }
.method public static int32 NestedLoops() cil managed
{
    .maxstack 3
    .locals init (
        [0] int32 i,
        [1] int32 j,
        [2] int32 sum
    )
    
    ldc.i4.0
    stloc.2                     ; sum = 0
    ldc.i4.0
    stloc.0                     ; i = 0
    br.s outer_condition
    
outer_loop:
    ldc.i4.0
    stloc.1                     ; j = 0
    br.s inner_condition
    
inner_loop:
    ; sum += i * j
    ldloc.2                     ; 加载 sum
    ldloc.0                     ; 加载 i
    ldloc.1                     ; 加载 j
    mul                         ; i * j
    add                         ; sum + (i * j)
    stloc.2                     ; sum = sum + (i * j)
    
    ; j++
    ldloc.1
    ldc.i4.1
    add
    stloc.1
    
inner_condition:
    ldloc.1                     ; 加载 j
    ldc.i4.3                    ; 加载 3
    blt.s inner_loop            ; 如果 j < 3 继续内层循环
    
    ; i++
    ldloc.0
    ldc.i4.1
    add
    stloc.0
    
outer_condition:
    ldloc.0                     ; 加载 i
    ldc.i4.3                    ; 加载 3
    blt.s outer_loop            ; 如果 i < 3 继续外层循环
    
    ldloc.2                     ; 返回 sum
    ret
}
```

### 条件嵌套

```msil
; C# 代码：if (x > 0) { if (y > 0) { return 1; } else { return 2; } } else { return 0; }
.method public static int32 NestedConditions(int32 x, int32 y) cil managed
{
    .maxstack 2
    
    ldarg.0                     ; 加载 x
    ldc.i4.0
    ble.s x_not_positive        ; 如果 x <= 0
    
    ; x > 0 的情况
    ldarg.1                     ; 加载 y
    ldc.i4.0
    ble.s y_not_positive        ; 如果 y <= 0
    
    ; x > 0 && y > 0
    ldc.i4.1
    ret
    
y_not_positive:
    ; x > 0 && y <= 0
    ldc.i4.2
    ret
    
x_not_positive:
    ; x <= 0
    ldc.i4.0
    ret
}
```

## break 和 continue

### break 语句

```msil
; C# 代码：for (int i = 0; i < 10; i++) { if (i == 5) break; sum += i; }
.method public static int32 BreakExample() cil managed
{
    .maxstack 2
    .locals init (
        [0] int32 i,
        [1] int32 sum
    )
    
    ldc.i4.0
    stloc.0                     ; i = 0
    ldc.i4.0
    stloc.1                     ; sum = 0
    br.s condition
    
loop_start:
    ; 检查 break 条件
    ldloc.0                     ; 加载 i
    ldc.i4.5                    ; 加载 5
    beq.s loop_end              ; 如果 i == 5，跳出循环
    
    ; 循环体
    ldloc.1                     ; 加载 sum
    ldloc.0                     ; 加载 i
    add                         ; sum + i
    stloc.1                     ; sum = sum + i
    
    ; i++
    ldloc.0
    ldc.i4.1
    add
    stloc.0
    
condition:
    ldloc.0                     ; 加载 i
    ldc.i4 10                   ; 加载 10
    blt.s loop_start            ; 如果 i < 10 继续循环
    
loop_end:
    ldloc.1                     ; 返回 sum
    ret
}
```

### continue 语句

```msil
; C# 代码：for (int i = 0; i < 10; i++) { if (i % 2 == 0) continue; sum += i; }
.method public static int32 ContinueExample() cil managed
{
    .maxstack 3
    .locals init (
        [0] int32 i,
        [1] int32 sum
    )
    
    ldc.i4.0
    stloc.0                     ; i = 0
    ldc.i4.0
    stloc.1                     ; sum = 0
    br.s condition
    
loop_start:
    ; 检查 continue 条件
    ldloc.0                     ; 加载 i
    ldc.i4.2                    ; 加载 2
    rem                         ; i % 2
    ldc.i4.0
    beq.s continue_point        ; 如果 i % 2 == 0，跳到 continue
    
    ; 循环体（只有奇数执行）
    ldloc.1                     ; 加载 sum
    ldloc.0                     ; 加载 i
    add                         ; sum + i
    stloc.1                     ; sum = sum + i
    
continue_point:
    ; i++
    ldloc.0
    ldc.i4.1
    add
    stloc.0
    
condition:
    ldloc.0                     ; 加载 i
    ldc.i4 10                   ; 加载 10
    blt.s loop_start            ; 如果 i < 10 继续循环
    
    ldloc.1                     ; 返回 sum
    ret
}
```

## 三元运算符

```msil
; C# 代码：result = (condition) ? trueValue : falseValue
.method public static int32 TernaryOperator(bool condition, int32 trueValue, int32 falseValue) cil managed
{
    .maxstack 1
    
    ldarg.0                     ; 加载 condition
    brfalse.s false_branch      ; 如果条件为假
    
    ldarg.1                     ; 加载 trueValue
    ret
    
false_branch:
    ldarg.2                     ; 加载 falseValue
    ret
}
```

## 性能优化

### 分支预测优化

```msil
; 将最可能的分支放在前面
; 优化前：
ldloc.0
brfalse.s unlikely_case     ; 不太可能的情况
; 常见情况的代码
br.s end
unlikely_case:
; 不常见情况的代码
end:

; 优化后：
ldloc.0
brtrue.s likely_case        ; 最可能的情况
; 不常见情况的代码
br.s end
likely_case:
; 常见情况的代码
end:
```

### 短距离跳转优化

```msil
; 使用短距离跳转指令（.s 后缀）
br.s short_label            ; 1 字节偏移
beq.s short_label           ; 1 字节偏移
brtrue.s short_label        ; 1 字节偏移

; 而不是：
br long_label               ; 4 字节偏移
beq long_label              ; 4 字节偏移
brtrue long_label           ; 4 字节偏移
```

### 循环优化

```msil
; 循环不变量提升
; 优化前：在循环内重复计算
loop_start:
    ldloc.0                 ; 加载 i
    ldloc.1                 ; 加载 array
    ldlen                   ; 获取数组长度（每次都计算）
    bge.s loop_end
    ; 循环体
    br.s loop_start

; 优化后：将不变量移到循环外
ldloc.1                     ; 加载 array
ldlen                       ; 获取数组长度（只计算一次）
stloc.2                     ; 存储长度
loop_start:
    ldloc.0                 ; 加载 i
    ldloc.2                 ; 加载预计算的长度
    bge.s loop_end
    ; 循环体
    br.s loop_start
```

## 相关文档

- [基础指令](./basic-instructions.md)
- [算术指令](./arithmetic-instructions.md)
- [方法调用指令](./method-instructions.md)
- [异常处理指令](./exception-instructions.md)