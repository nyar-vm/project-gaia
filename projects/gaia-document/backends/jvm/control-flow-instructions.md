# JASM 控制流指令

Java Assembly (JASM) 控制流指令，包括条件跳转、无条件跳转、比较指令和循环控制。

## 无条件跳转指令

### 基本跳转

```jasm
goto label_name             ; 无条件跳转到标签
goto_w label_name           ; 宽索引无条件跳转（用于远距离跳转）

; 示例
start:
    iconst_1
    istore_1
    goto end
    iconst_2                ; 这行代码不会执行
    istore_1
end:
    ; 程序继续执行
```

## 条件跳转指令

### 零值比较跳转

```jasm
; 整数零值比较
ifeq label                  ; 如果值等于 0 则跳转
ifne label                  ; 如果值不等于 0 则跳转
iflt label                  ; 如果值小于 0 则跳转
ifle label                  ; 如果值小于等于 0 则跳转
ifgt label                  ; 如果值大于 0 则跳转
ifge label                  ; 如果值大于等于 0 则跳转

; 引用零值比较
ifnull label                ; 如果引用为 null 则跳转
ifnonnull label             ; 如果引用不为 null 则跳转
```

### 两值比较跳转

```jasm
; 整数比较
if_icmpeq label             ; 如果两个 int 相等则跳转
if_icmpne label             ; 如果两个 int 不相等则跳转
if_icmplt label             ; 如果第一个 int 小于第二个则跳转
if_icmple label             ; 如果第一个 int 小于等于第二个则跳转
if_icmpgt label             ; 如果第一个 int 大于第二个则跳转
if_icmpge label             ; 如果第一个 int 大于等于第二个则跳转

; 引用比较
if_acmpeq label             ; 如果两个引用相等则跳转
if_acmpne label             ; 如果两个引用不相等则跳转
```

## 比较指令

### 数值比较

```jasm
; long 比较
lcmp                        ; 比较两个 long 值
; 栈：[value1, value2] → [result]
; result: -1 (value1 < value2), 0 (相等), 1 (value1 > value2)

; float 比较
fcmpl                       ; 比较两个 float 值（NaN 时返回 -1）
fcmpg                       ; 比较两个 float 值（NaN 时返回 1）

; double 比较
dcmpl                       ; 比较两个 double 值（NaN 时返回 -1）
dcmpg                       ; 比较两个 double 值（NaN 时返回 1）
```

## 控制流示例

### if-else 语句

```jasm
; Java 代码：if (x > 0) { y = 1; } else { y = -1; }
iload_1                     ; 加载 x
ifle else_branch            ; 如果 x <= 0 跳转到 else
    iconst_1                ; x > 0 的情况
    istore_2                ; y = 1
    goto end
else_branch:
    iconst_m1               ; x <= 0 的情况
    istore_2                ; y = -1
end:
```

### while 循环

```jasm
; Java 代码：while (i < 10) { sum += i; i++; }
goto loop_condition
loop_start:
    iload_2                 ; 加载 sum
    iload_1                 ; 加载 i
    iadd                    ; sum + i
    istore_2                ; sum = sum + i
    iinc 1, 1               ; i++
loop_condition:
    iload_1                 ; 加载 i
    bipush 10               ; 加载常量 10
    if_icmplt loop_start    ; 如果 i < 10 继续循环
```

### for 循环

```jasm
; Java 代码：for (int i = 0; i < 10; i++) { sum += i; }
iconst_0                    ; 初始化 i = 0
istore_1
goto for_condition
for_start:
    iload_2                 ; 加载 sum
    iload_1                 ; 加载 i
    iadd                    ; sum + i
    istore_2                ; sum = sum + i
    iinc 1, 1               ; i++
for_condition:
    iload_1                 ; 加载 i
    bipush 10               ; 加载常量 10
    if_icmplt for_start     ; 如果 i < 10 继续循环
```

### do-while 循环

```jasm
; Java 代码：do { sum += i; i++; } while (i < 10);
do_start:
    iload_2                 ; 加载 sum
    iload_1                 ; 加载 i
    iadd                    ; sum + i
    istore_2                ; sum = sum + i
    iinc 1, 1               ; i++
    iload_1                 ; 加载 i
    bipush 10               ; 加载常量 10
    if_icmplt do_start      ; 如果 i < 10 继续循环
```

## switch 语句

### tableswitch 指令

```jasm
; Java 代码：switch (x) { case 1: y = 10; break; case 2: y = 20; break; default: y = 0; }
iload_1                     ; 加载 x
tableswitch 1 to 2 {        ; 处理 case 1 到 2
    case 1: case_1
    case 2: case_2
    default: default_case
}

case_1:
    bipush 10
    istore_2
    goto switch_end

case_2:
    bipush 20
    istore_2
    goto switch_end

default_case:
    iconst_0
    istore_2

switch_end:
```

### lookupswitch 指令

```jasm
; Java 代码：switch (x) { case 10: y = 1; break; case 100: y = 2; break; default: y = 0; }
iload_1                     ; 加载 x
lookupswitch {              ; 处理稀疏的 case 值
    10: case_10
    100: case_100
    default: default_case
}

case_10:
    iconst_1
    istore_2
    goto switch_end

case_100:
    iconst_2
    istore_2
    goto switch_end

default_case:
    iconst_0
    istore_2

switch_end:
```

## 复杂控制流示例

### 嵌套循环

```jasm
; Java 代码：for (int i = 0; i < 3; i++) { for (int j = 0; j < 3; j++) { sum += i * j; } }
iconst_0                    ; i = 0
istore_1
goto outer_condition

outer_start:
    iconst_0                ; j = 0
    istore_2
    goto inner_condition

inner_start:
    iload_3                 ; 加载 sum
    iload_1                 ; 加载 i
    iload_2                 ; 加载 j
    imul                    ; i * j
    iadd                    ; sum + (i * j)
    istore_3                ; sum = sum + (i * j)
    iinc 2, 1               ; j++

inner_condition:
    iload_2                 ; 加载 j
    iconst_3                ; 加载常量 3
    if_icmplt inner_start   ; 如果 j < 3 继续内层循环

    iinc 1, 1               ; i++

outer_condition:
    iload_1                 ; 加载 i
    iconst_3                ; 加载常量 3
    if_icmplt outer_start   ; 如果 i < 3 继续外层循环
```

### 条件表达式（三元运算符）

```jasm
; Java 代码：result = (x > 0) ? x : -x
iload_1                     ; 加载 x
dup                         ; 复制 x
ifle negative               ; 如果 x <= 0 跳转
    goto end                ; x > 0，直接使用 x
negative:
    ineg                    ; x <= 0，取负值
end:
    istore_2                ; 存储结果
```

### break 和 continue

```jasm
; Java 代码：for (int i = 0; i < 10; i++) { if (i == 5) continue; if (i == 8) break; sum += i; }
iconst_0                    ; i = 0
istore_1
goto for_condition

for_start:
    iload_1                 ; 加载 i
    iconst_5                ; 加载常量 5
    if_icmpeq continue_point ; 如果 i == 5 跳转到 continue

    iload_1                 ; 加载 i
    bipush 8                ; 加载常量 8
    if_icmpeq break_point   ; 如果 i == 8 跳转到 break

    iload_2                 ; 加载 sum
    iload_1                 ; 加载 i
    iadd                    ; sum + i
    istore_2                ; sum = sum + i

continue_point:
    iinc 1, 1               ; i++

for_condition:
    iload_1                 ; 加载 i
    bipush 10               ; 加载常量 10
    if_icmplt for_start     ; 如果 i < 10 继续循环

break_point:
    ; 循环结束
```

## 标签和跳转优化

### 标签命名约定

```jasm
; 推荐的标签命名
method_start:               ; 方法开始
loop_start:                 ; 循环开始
loop_condition:             ; 循环条件检查
loop_end:                   ; 循环结束
if_true:                    ; if 条件为真
if_false:                   ; if 条件为假
switch_case_1:              ; switch case 1
switch_default:             ; switch 默认情况
exception_handler:          ; 异常处理器
method_end:                 ; 方法结束
```

### 跳转距离优化

```jasm
; 短距离跳转（推荐）
iload_1
ifne short_jump
iconst_0
istore_2
short_jump:

; 长距离跳转（必要时使用）
iload_1
ifne long_jump_helper
iconst_0
istore_2
goto after_long_jump
long_jump_helper:
goto_w very_far_label       ; 使用宽索引跳转
after_long_jump:
```

## 性能考虑

### 分支预测友好的代码

```jasm
; 优化前：频繁的条件跳转
iload_1
ifne case1
iload_2
ifne case2
; ... 更多条件

; 优化后：减少跳转层次
iload_1
ifeq check_second
    ; 处理第一个条件
    goto end
check_second:
iload_2
ifeq end
    ; 处理第二个条件
end:
```

### 循环优化

```jasm
; 优化前：在循环内重复计算
loop_start:
    iload_1                 ; 加载 i
    iload_2                 ; 加载 n
    if_icmpge loop_end      ; 每次都比较
    ; 循环体
    iinc 1, 1
    goto loop_start

; 优化后：减少循环内计算
iload_2                     ; 预加载 n
istore_3                    ; 存储到局部变量
loop_start:
    iload_1                 ; 加载 i
    iload_3                 ; 加载预存的 n
    if_icmpge loop_end      ; 比较
    ; 循环体
    iinc 1, 1
    goto loop_start
```

## 相关文档

- [基础指令](./basic-instructions.md)
- [算术指令](./arithmetic-instructions.md)
- [方法调用指令](./method-instructions.md)
- [异常处理指令](./exception-instructions.md)