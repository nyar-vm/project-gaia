# JASM 算术运算指令

Java Assembly (JASM) 算术运算指令，包括加法、减法、乘法、除法、取模、位运算和类型转换指令。

## 算术运算指令

### 加法指令

```jasm
iadd            ; int 加法：value1 + value2 → result
ladd            ; long 加法：value1 + value2 → result
fadd            ; float 加法：value1 + value2 → result
dadd            ; double 加法：value1 + value2 → result
```

### 减法指令

```jasm
isub            ; int 减法：value1 - value2 → result
lsub            ; long 减法：value1 - value2 → result
fsub            ; float 减法：value1 - value2 → result
dsub            ; double 减法：value1 - value2 → result
```

### 乘法指令

```jasm
imul            ; int 乘法：value1 * value2 → result
lmul            ; long 乘法：value1 * value2 → result
fmul            ; float 乘法：value1 * value2 → result
dmul            ; double 乘法：value1 * value2 → result
```

### 除法指令

```jasm
idiv            ; int 除法：value1 / value2 → result
ldiv            ; long 除法：value1 / value2 → result
fdiv            ; float 除法：value1 / value2 → result
ddiv            ; double 除法：value1 / value2 → result
```

### 取模指令

```jasm
irem            ; int 取模：value1 % value2 → result
lrem            ; long 取模：value1 % value2 → result
frem            ; float 取模：value1 % value2 → result
drem            ; double 取模：value1 % value2 → result
```

### 取负指令

```jasm
ineg            ; int 取负：-value → result
lneg            ; long 取负：-value → result
fneg            ; float 取负：-value → result
dneg            ; double 取负：-value → result
```

## 位运算指令

### 位移指令

```jasm
ishl            ; int 左移：value1 << value2 → result
lshl            ; long 左移：value1 << value2 → result
ishr            ; int 算术右移：value1 >> value2 → result
lshr            ; long 算术右移：value1 >> value2 → result
iushr           ; int 逻辑右移：value1 >>> value2 → result
lushr           ; long 逻辑右移：value1 >>> value2 → result
```

### 位逻辑指令

```jasm
iand            ; int 按位与：value1 & value2 → result
land            ; long 按位与：value1 & value2 → result
ior             ; int 按位或：value1 | value2 → result
lor             ; long 按位或：value1 | value2 → result
ixor            ; int 按位异或：value1 ^ value2 → result
lxor            ; long 按位异或：value1 ^ value2 → result
```

## 类型转换指令

### 整型转换

```jasm
i2l             ; int → long
i2f             ; int → float
i2d             ; int → double
i2b             ; int → byte
i2c             ; int → char
i2s             ; int → short
```

### 长整型转换

```jasm
l2i             ; long → int
l2f             ; long → float
l2d             ; long → double
```

### 浮点型转换

```jasm
f2i             ; float → int
f2l             ; float → long
f2d             ; float → double
```

### 双精度浮点型转换

```jasm
d2i             ; double → int
d2l             ; double → long
d2f             ; double → float
```

## 增量指令

```jasm
iinc 1, 5       ; 将局部变量 1 增加 5
iinc 0, -1      ; 将局部变量 0 减少 1
```

## 算术运算示例

### 基本算术运算

```jasm
; 计算 (a + b) * c
iload_1         ; 加载变量 a
iload_2         ; 加载变量 b
iadd            ; a + b
iload_3         ; 加载变量 c
imul            ; (a + b) * c
istore 4        ; 存储结果到变量 4
```

### 复杂表达式

```jasm
; 计算 (a * b + c) / d
iload_1         ; 加载变量 a
iload_2         ; 加载变量 b
imul            ; a * b
iload_3         ; 加载变量 c
iadd            ; a * b + c
iload 4         ; 加载变量 d
idiv            ; (a * b + c) / d
istore 5        ; 存储结果
```

### 位运算示例

```jasm
; 检查数字是否为偶数 (n & 1 == 0)
iload_1         ; 加载数字 n
iconst_1        ; 加载常量 1
iand            ; n & 1
; 结果为 0 表示偶数，非 0 表示奇数
```

### 类型转换示例

```jasm
; 将 int 转换为 double 进行精确除法
iload_1         ; 加载 int 值 a
i2d             ; 转换为 double
iload_2         ; 加载 int 值 b
i2d             ; 转换为 double
ddiv            ; 执行 double 除法
dstore_3        ; 存储 double 结果
```

### 增量操作示例

```jasm
; 实现 for 循环的计数器递增
; for (int i = 0; i < 10; i++)
iconst_0        ; 初始化 i = 0
istore_1        ; 存储到局部变量 1

loop_start:
iload_1         ; 加载 i
bipush 10       ; 加载 10
if_icmpge loop_end ; 如果 i >= 10 跳出循环

; 循环体代码...

iinc 1, 1       ; i++
goto loop_start ; 跳回循环开始

loop_end:
```

## 运算优先级和栈操作

### 栈操作顺序

```jasm
; 表达式：a - b + c
; 等价于：(a - b) + c
iload_1         ; 栈：[a]
iload_2         ; 栈：[a, b]
isub            ; 栈：[a-b]
iload_3         ; 栈：[a-b, c]
iadd            ; 栈：[(a-b)+c]
```

### 复杂表达式的栈管理

```jasm
; 表达式：a * (b + c) - d
iload_1         ; 栈：[a]
iload_2         ; 栈：[a, b]
iload_3         ; 栈：[a, b, c]
iadd            ; 栈：[a, b+c]
imul            ; 栈：[a*(b+c)]
iload 4         ; 栈：[a*(b+c), d]
isub            ; 栈：[a*(b+c)-d]
```

## 异常处理

### 算术异常

- **ArithmeticException**：除零操作（idiv, ldiv, irem, lrem）
- **ClassCastException**：不当的类型转换

### 异常处理示例

```jasm
; 安全的除法操作
iload_1         ; 加载被除数
iload_2         ; 加载除数
dup             ; 复制除数用于检查
ifne safe_div   ; 如果除数不为 0，跳转到安全除法
pop             ; 弹出被除数
pop             ; 弹出除数
iconst_0        ; 返回 0 作为默认值
goto end

safe_div:
idiv            ; 执行除法

end:
```

## 性能优化建议

1. **使用适当的常量指令**：优先使用 `iconst_0` 到 `iconst_5` 而不是 `ldc`
2. **避免不必要的类型转换**：在可能的情况下保持相同类型的运算
3. **利用增量指令**：使用 `iinc` 而不是 load-add-store 序列
4. **注意栈深度**：避免过深的表达式嵌套

## 相关文档

- [基础指令](./basic-instructions.md)
- [比较和控制流指令](./control-flow-instructions.md)
- [数组操作指令](./array-instructions.md)