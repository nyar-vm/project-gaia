# JASM 基础指令

Java Assembly (JASM) 基础指令集，包括常量加载、局部变量操作和栈操作指令。

## 常量加载指令

### 空值和数值常量

```jasm
aconst_null     ; 将 null 引用推入栈
iconst_m1       ; 将 int 常量 -1 推入栈
iconst_0        ; 将 int 常量 0 推入栈
iconst_1        ; 将 int 常量 1 推入栈
iconst_2        ; 将 int 常量 2 推入栈
iconst_3        ; 将 int 常量 3 推入栈
iconst_4        ; 将 int 常量 4 推入栈
iconst_5        ; 将 int 常量 5 推入栈
lconst_0        ; 将 long 常量 0L 推入栈
lconst_1        ; 将 long 常量 1L 推入栈
fconst_0        ; 将 float 常量 0.0f 推入栈
fconst_1        ; 将 float 常量 1.0f 推入栈
fconst_2        ; 将 float 常量 2.0f 推入栈
dconst_0        ; 将 double 常量 0.0d 推入栈
dconst_1        ; 将 double 常量 1.0d 推入栈
```

### 常量池加载

```jasm
ldc "Hello"     ; 从常量池加载字符串常量
ldc 100         ; 从常量池加载 int 常量
ldc 3.14f       ; 从常量池加载 float 常量
ldc_w #index    ; 从常量池加载常量（宽索引）
ldc2_w 3.14159d ; 从常量池加载 long/double 常量
```

### 立即数加载

```jasm
bipush 127      ; 将 byte 值推入栈（-128 到 127）
sipush 32767    ; 将 short 值推入栈（-32768 到 32767）
```

## 局部变量操作指令

### 加载指令 (Load)

```jasm
; 整型加载
iload 0         ; 从局部变量 0 加载 int
iload_0         ; 从局部变量 0 加载 int（快速形式）
iload_1         ; 从局部变量 1 加载 int
iload_2         ; 从局部变量 2 加载 int
iload_3         ; 从局部变量 3 加载 int

; 长整型加载
lload 0         ; 从局部变量 0 加载 long
lload_0         ; 从局部变量 0 加载 long（快速形式）
lload_1         ; 从局部变量 1 加载 long
lload_2         ; 从局部变量 2 加载 long
lload_3         ; 从局部变量 3 加载 long

; 浮点型加载
fload 0         ; 从局部变量 0 加载 float
fload_0         ; 从局部变量 0 加载 float（快速形式）
fload_1         ; 从局部变量 1 加载 float
fload_2         ; 从局部变量 2 加载 float
fload_3         ; 从局部变量 3 加载 float

; 双精度浮点型加载
dload 0         ; 从局部变量 0 加载 double
dload_0         ; 从局部变量 0 加载 double（快速形式）
dload_1         ; 从局部变量 1 加载 double
dload_2         ; 从局部变量 2 加载 double
dload_3         ; 从局部变量 3 加载 double

; 引用类型加载
aload 0         ; 从局部变量 0 加载引用
aload_0         ; 从局部变量 0 加载引用（快速形式，通常是 this）
aload_1         ; 从局部变量 1 加载引用
aload_2         ; 从局部变量 2 加载引用
aload_3         ; 从局部变量 3 加载引用
```

### 存储指令 (Store)

```jasm
; 整型存储
istore 0        ; 将栈顶 int 存储到局部变量 0
istore_0        ; 将栈顶 int 存储到局部变量 0（快速形式）
istore_1        ; 将栈顶 int 存储到局部变量 1
istore_2        ; 将栈顶 int 存储到局部变量 2
istore_3        ; 将栈顶 int 存储到局部变量 3

; 长整型存储
lstore 0        ; 将栈顶 long 存储到局部变量 0
lstore_0        ; 将栈顶 long 存储到局部变量 0（快速形式）
lstore_1        ; 将栈顶 long 存储到局部变量 1
lstore_2        ; 将栈顶 long 存储到局部变量 2
lstore_3        ; 将栈顶 long 存储到局部变量 3

; 浮点型存储
fstore 0        ; 将栈顶 float 存储到局部变量 0
fstore_0        ; 将栈顶 float 存储到局部变量 0（快速形式）
fstore_1        ; 将栈顶 float 存储到局部变量 1
fstore_2        ; 将栈顶 float 存储到局部变量 2
fstore_3        ; 将栈顶 float 存储到局部变量 3

; 双精度浮点型存储
dstore 0        ; 将栈顶 double 存储到局部变量 0
dstore_0        ; 将栈顶 double 存储到局部变量 0（快速形式）
dstore_1        ; 将栈顶 double 存储到局部变量 1
dstore_2        ; 将栈顶 double 存储到局部变量 2
dstore_3        ; 将栈顶 double 存储到局部变量 3

; 引用类型存储
astore 0        ; 将栈顶引用存储到局部变量 0
astore_0        ; 将栈顶引用存储到局部变量 0（快速形式）
astore_1        ; 将栈顶引用存储到局部变量 1
astore_2        ; 将栈顶引用存储到局部变量 2
astore_3        ; 将栈顶引用存储到局部变量 3
```

## 栈操作指令

### 栈管理

```jasm
pop             ; 弹出栈顶值（1 个字）
pop2            ; 弹出栈顶值（2 个字，或两个 1 字值）
dup             ; 复制栈顶值
dup_x1          ; 复制栈顶值并插入到栈顶下两个值之下
dup_x2          ; 复制栈顶值并插入到栈顶下三个值之下
dup2            ; 复制栈顶两个值
dup2_x1         ; 复制栈顶两个值并插入到栈顶下三个值之下
dup2_x2         ; 复制栈顶两个值并插入到栈顶下四个值之下
swap            ; 交换栈顶两个值
```

### 栈操作示例

```jasm
; 示例：交换两个变量的值
iload_1         ; 加载变量 1
iload_2         ; 加载变量 2
istore_1        ; 存储到变量 1（原来的变量 2 值）
istore_2        ; 存储到变量 2（原来的变量 1 值）

; 示例：复制栈顶值用于多次使用
iload_1         ; 加载变量 1
dup             ; 复制栈顶值
istore_2        ; 存储副本到变量 2
istore_3        ; 存储原值到变量 3
```

## 无操作指令

```jasm
nop             ; 无操作，不执行任何动作
```

## 使用注意事项

1. **局部变量索引**：局部变量从索引 0 开始，对于实例方法，索引 0 通常是 `this` 引用
2. **类型安全**：确保加载和存储指令与变量类型匹配
3. **栈平衡**：确保方法执行过程中栈的平衡性
4. **快速形式**：对于索引 0-3 的局部变量，使用快速形式指令可以减少字节码大小

## 相关文档

- [算术运算指令](./arithmetic-instructions.md)
- [方法调用指令](./method-instructions.md)
- [控制流指令](./control-flow-instructions.md)