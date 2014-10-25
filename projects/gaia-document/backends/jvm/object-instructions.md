# JASM 对象和类操作指令

Java Assembly (JASM) 对象和类操作指令，包括对象创建、字段访问、数组操作、类型检查和转换。

## 对象创建指令

### 基本对象创建

```jasm
new java/lang/Object        ; 创建新对象实例
; 栈：[] → [objectref]
; 注意：对象尚未初始化，需要调用构造函数

; 完整的对象创建和初始化
new java/lang/StringBuilder
dup                         ; 复制引用用于构造函数调用
invokespecial Method java/lang/StringBuilder."<init>":"()V"
; 栈：[] → [objectref]（已初始化）
```

### 数组创建

```jasm
; 一维数组
newarray int                ; 创建 int 数组
; 栈：[count] → [arrayref]

anewarray java/lang/String  ; 创建引用类型数组
; 栈：[count] → [arrayref]

; 多维数组
multianewarray [[I, 2       ; 创建二维 int 数组
; 栈：[count1, count2] → [arrayref]
; 第二个参数是维度数
```

## 字段访问指令

### 实例字段

```jasm
; 获取实例字段
getfield Field MyClass.instanceField:"I"
; 栈：[objectref] → [value]

; 设置实例字段
putfield Field MyClass.instanceField:"I"
; 栈：[objectref, value] → []
```

### 静态字段

```jasm
; 获取静态字段
getstatic Field MyClass.staticField:"I"
; 栈：[] → [value]

; 设置静态字段
putstatic Field MyClass.staticField:"I"
; 栈：[value] → []
```

## 数组操作指令

### 数组元素访问

```jasm
; 加载数组元素
iaload                      ; 加载 int 数组元素
laload                      ; 加载 long 数组元素
faload                      ; 加载 float 数组元素
daload                      ; 加载 double 数组元素
aaload                      ; 加载引用数组元素
baload                      ; 加载 byte/boolean 数组元素
caload                      ; 加载 char 数组元素
saload                      ; 加载 short 数组元素
; 栈：[arrayref, index] → [value]

; 存储数组元素
iastore                     ; 存储到 int 数组
lastore                     ; 存储到 long 数组
fastore                     ; 存储到 float 数组
dastore                     ; 存储到 double 数组
aastore                     ; 存储到引用数组
bastore                     ; 存储到 byte/boolean 数组
castore                     ; 存储到 char 数组
sastore                     ; 存储到 short 数组
; 栈：[arrayref, index, value] → []
```

### 数组长度

```jasm
arraylength                 ; 获取数组长度
; 栈：[arrayref] → [length]
```

## 类型检查和转换

### 类型检查

```jasm
instanceof java/lang/String ; 检查对象是否为指定类型的实例
; 栈：[objectref] → [result]
; result: 1 (是), 0 (否)

checkcast java/lang/String  ; 类型转换检查
; 栈：[objectref] → [objectref]
; 如果类型不匹配抛出 ClassCastException
```

## 对象操作示例

### 创建和初始化对象

```jasm
; 创建 ArrayList 对象
new java/util/ArrayList
dup
invokespecial Method java/util/ArrayList."<init>":"()V"
astore_1                    ; 存储到局部变量

; 创建带参数的对象
new java/lang/StringBuilder
dup
ldc "Hello"                 ; 初始字符串
invokespecial Method java/lang/StringBuilder."<init>":"(Ljava/lang/String;)V"
astore_2
```

### 字段操作示例

```jasm
; 设置实例字段
aload_0                     ; 加载 this
iconst_5                    ; 加载值 5
putfield Field MyClass.count:"I"

; 获取实例字段
aload_0                     ; 加载 this
getfield Field MyClass.count:"I"
istore_1                    ; 存储到局部变量

; 操作静态字段
getstatic Field java/lang/System.out:"Ljava/io/PrintStream;"
ldc "Hello World"
invokevirtual Method java/io/PrintStream.println:"(Ljava/lang/String;)V"
```

### 数组操作示例

```jasm
; 创建和初始化 int 数组
bipush 10                   ; 数组大小
newarray int                ; 创建 int 数组
astore_1                    ; 存储数组引用

; 设置数组元素：array[0] = 42
aload_1                     ; 加载数组引用
iconst_0                    ; 索引 0
bipush 42                   ; 值 42
iastore                     ; 存储到数组

; 获取数组元素：int value = array[0]
aload_1                     ; 加载数组引用
iconst_0                    ; 索引 0
iaload                      ; 加载数组元素
istore_2                    ; 存储到局部变量

; 获取数组长度
aload_1                     ; 加载数组引用
arraylength                 ; 获取长度
istore_3                    ; 存储长度
```

### 字符串数组操作

```jasm
; 创建 String 数组
iconst_3                    ; 数组大小
anewarray java/lang/String  ; 创建 String 数组
astore_1                    ; 存储数组引用

; 设置数组元素：array[0] = "Hello"
aload_1                     ; 加载数组引用
iconst_0                    ; 索引 0
ldc "Hello"                 ; 字符串值
aastore                     ; 存储到引用数组

; 获取数组元素：String str = array[0]
aload_1                     ; 加载数组引用
iconst_0                    ; 索引 0
aaload                      ; 加载引用数组元素
astore_2                    ; 存储到局部变量
```

### 多维数组操作

```jasm
; 创建二维 int 数组：int[][] array = new int[3][4]
iconst_3                    ; 第一维大小
iconst_4                    ; 第二维大小
multianewarray [[I, 2       ; 创建二维数组
astore_1                    ; 存储数组引用

; 访问二维数组元素：array[1][2] = 100
aload_1                     ; 加载二维数组引用
iconst_1                    ; 第一维索引
aaload                      ; 获取一维数组
iconst_2                    ; 第二维索引
bipush 100                  ; 值
iastore                     ; 存储到一维数组

; 获取二维数组元素：int value = array[1][2]
aload_1                     ; 加载二维数组引用
iconst_1                    ; 第一维索引
aaload                      ; 获取一维数组
iconst_2                    ; 第二维索引
iaload                      ; 加载元素值
istore_2                    ; 存储到局部变量
```

## 类型检查和转换示例

### instanceof 检查

```jasm
; 检查对象类型：if (obj instanceof String)
aload_1                     ; 加载对象引用
instanceof java/lang/String ; 类型检查
ifeq not_string             ; 如果不是 String 跳转
    ; 是 String 类型的处理
    aload_1
    checkcast java/lang/String ; 安全转换
    invokevirtual Method java/lang/String.length:"()I"
    istore_2
    goto end
not_string:
    ; 不是 String 类型的处理
    iconst_m1
    istore_2
end:
```

### 类型转换

```jasm
; 安全的类型转换
aload_1                     ; 加载 Object 引用
instanceof java/util/List   ; 检查是否为 List
ifeq not_list
    aload_1
    checkcast java/util/List ; 转换为 List
    invokeinterface InterfaceMethod java/util/List.size:"()I", 1
    istore_2
    goto end
not_list:
    iconst_0
    istore_2
end:
```

## 对象比较

### 引用比较

```jasm
; 比较两个对象引用是否相同
aload_1                     ; 加载第一个对象
aload_2                     ; 加载第二个对象
if_acmpeq objects_equal     ; 如果引用相同跳转
    iconst_0                ; 不相同
    istore_3
    goto end
objects_equal:
    iconst_1                ; 相同
    istore_3
end:
```

### equals 方法调用

```jasm
; 使用 equals 方法比较对象内容
aload_1                     ; 加载第一个对象
aload_2                     ; 加载第二个对象
invokevirtual Method java/lang/Object.equals:"(Ljava/lang/Object;)Z"
istore_3                    ; 存储比较结果
```

## 空值检查

### null 检查

```jasm
; 检查对象是否为 null
aload_1                     ; 加载对象引用
ifnull is_null              ; 如果为 null 跳转
    ; 对象不为 null 的处理
    aload_1
    invokevirtual Method java/lang/Object.toString:"()Ljava/lang/String;"
    astore_2
    goto end
is_null:
    ; 对象为 null 的处理
    ldc "null"
    astore_2
end:
```

### 防御性编程

```jasm
; 防御性的方法调用
aload_1                     ; 加载可能为 null 的对象
dup                         ; 复制引用
ifnull handle_null          ; 如果为 null 跳转
    ; 对象不为 null，安全调用方法
    invokevirtual Method java/lang/Object.hashCode:"()I"
    istore_2
    goto end
handle_null:
    pop                     ; 弹出 null 引用
    iconst_0                ; 返回默认值
    istore_2
end:
```

## 性能优化建议

### 对象创建优化

```jasm
; 避免不必要的对象创建
; 优化前：每次都创建新的 StringBuilder
new java/lang/StringBuilder
dup
invokespecial Method java/lang/StringBuilder."<init>":"()V"

; 优化后：重用已有的 StringBuilder
aload_1                     ; 加载已有的 StringBuilder
iconst_0                    ; 重置长度
invokevirtual Method java/lang/StringBuilder.setLength:"(I)V"
```

### 数组访问优化

```jasm
; 优化前：重复计算数组长度
aload_1                     ; 加载数组
arraylength                 ; 获取长度
istore_2                    ; 存储长度
; ... 在循环中重复使用 arraylength

; 优化后：缓存数组长度
aload_1                     ; 加载数组
arraylength                 ; 获取长度
istore_2                    ; 缓存长度
; ... 在循环中使用缓存的长度
```

## 相关文档

- [基础指令](./basic-instructions.md)
- [方法调用指令](./method-instructions.md)
- [异常处理指令](./exception-instructions.md)
- [同步指令](./synchronization-instructions.md)