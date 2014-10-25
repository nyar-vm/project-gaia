# JASM 方法调用指令

Java Assembly (JASM) 方法调用指令，包括各种方法调用类型、返回指令和动态调用指令。

## 方法调用指令

### 实例方法调用

```jasm
invokevirtual Method java/lang/Object.toString:"()Ljava/lang/String;"
; 调用虚方法（支持多态）
; 栈：[objectref, arg1, arg2, ...] → [result]

invokespecial Method java/lang/Object."<init>":"()V"
; 调用特殊方法（构造函数、私有方法、父类方法）
; 栈：[objectref, arg1, arg2, ...] → [result]

invokeinterface InterfaceMethod java/util/List.size:"()I", 1
; 调用接口方法
; 栈：[objectref, arg1, arg2, ...] → [result]
; 最后的数字是参数计数
```

### 静态方法调用

```jasm
invokestatic Method java/lang/Math.max:"(II)I"
; 调用静态方法
; 栈：[arg1, arg2, ...] → [result]
```

### 动态方法调用

```jasm
invokedynamic InvokeDynamic REF_invokeStatic:Method java/lang/invoke/LambdaMetafactory.metafactory:
    "(Ljava/lang/invoke/MethodHandles$Lookup;Ljava/lang/String;Ljava/lang/invoke/MethodType;
     Ljava/lang/invoke/MethodType;Ljava/lang/invoke/MethodHandle;Ljava/lang/invoke/MethodType;)
     Ljava/lang/invoke/CallSite;":
    apply:"(LLambdaExample;)Ljava/util/function/Function;" {
        MethodType "(Ljava/lang/Object;)Ljava/lang/Object;",
        MethodHandle REF_invokeVirtual:Method LambdaExample.lambda$0:"(Ljava/lang/String;)Ljava/lang/String;",
        MethodType "(Ljava/lang/String;)Ljava/lang/String;"
    };
; 动态方法调用（用于 Lambda 表达式、方法引用等）
```

## 返回指令

### 基本返回指令

```jasm
return          ; 从 void 方法返回
ireturn         ; 返回 int 值
lreturn         ; 返回 long 值
freturn         ; 返回 float 值
dreturn         ; 返回 double 值
areturn         ; 返回引用类型值
```

## 方法调用示例

### 构造函数调用

```jasm
; 创建新对象：new StringBuilder()
new java/lang/StringBuilder
dup
invokespecial Method java/lang/StringBuilder."<init>":"()V"
; 栈顶现在有一个 StringBuilder 实例
```

### 链式方法调用

```jasm
; StringBuilder.append("Hello").append(" World")
aload_0         ; 加载 StringBuilder 引用
ldc "Hello"     ; 加载字符串 "Hello"
invokevirtual Method java/lang/StringBuilder.append:"(Ljava/lang/String;)Ljava/lang/StringBuilder;"
ldc " World"    ; 加载字符串 " World"
invokevirtual Method java/lang/StringBuilder.append:"(Ljava/lang/String;)Ljava/lang/StringBuilder;"
```

### 静态方法调用

```jasm
; Math.max(a, b)
iload_1         ; 加载变量 a
iload_2         ; 加载变量 b
invokestatic Method java/lang/Math.max:"(II)I"
istore_3        ; 存储结果
```

### 接口方法调用

```jasm
; List.size()
aload_1         ; 加载 List 引用
invokeinterface InterfaceMethod java/util/List.size:"()I", 1
istore_2        ; 存储大小
```

## 方法描述符

### 基本类型描述符

| Java 类型 | 描述符 | 示例 |
|---------|-----|----|
| boolean | Z   | Z  |
| byte    | B   | B  |
| char    | C   | C  |
| short   | S   | S  |
| int     | I   | I  |
| long    | J   | J  |
| float   | F   | F  |
| double  | D   | D  |
| void    | V   | V  |

### 引用类型描述符

```jasm
; 类类型：L<classname>;
Ljava/lang/String;          ; String 类型
Ljava/util/List;            ; List 类型
Ljava/lang/Object;          ; Object 类型

; 数组类型：[<type>
[I                          ; int[] 数组
[Ljava/lang/String;         ; String[] 数组
[[I                         ; int[][] 二维数组
```

### 方法描述符格式

```jasm
; 格式：(<parameter_types>)<return_type>
"()V"                       ; 无参数，返回 void
"(I)V"                      ; 一个 int 参数，返回 void
"(II)I"                     ; 两个 int 参数，返回 int
"(Ljava/lang/String;)V"     ; 一个 String 参数，返回 void
"([Ljava/lang/String;)V"    ; 一个 String[] 参数，返回 void
"(ILjava/lang/String;)Ljava/lang/Object;" ; int 和 String 参数，返回 Object
```

## 特殊方法调用

### 构造函数调用模式

```jasm
; 完整的对象创建和初始化
new java/util/ArrayList     ; 创建对象
dup                         ; 复制引用
invokespecial Method java/util/ArrayList."<init>":"()V"
; 栈顶现在有初始化完成的 ArrayList 实例
```

### 父类方法调用

```jasm
; 在子类中调用父类方法
aload_0                     ; 加载 this
invokespecial Method java/lang/Object.toString:"()Ljava/lang/String;"
; 调用父类的 toString 方法
```

### 私有方法调用

```jasm
; 调用私有方法
aload_0                     ; 加载 this
iload_1                     ; 加载参数
invokespecial Method MyClass.privateMethod:"(I)V"
```

## Lambda 表达式和方法引用

### Lambda 表达式字节码

```jasm
; Java 代码：list.forEach(x -> System.out.println(x))
aload_1                     ; 加载 list
invokedynamic InvokeDynamic REF_invokeStatic:Method java/lang/invoke/LambdaMetafactory.metafactory:
    "(Ljava/lang/invoke/MethodHandles$Lookup;Ljava/lang/String;Ljava/lang/invoke/MethodType;
     Ljava/lang/invoke/MethodType;Ljava/lang/invoke/MethodHandle;Ljava/lang/invoke/MethodType;)
     Ljava/lang/invoke/CallSite;":
    accept:"()Ljava/util/function/Consumer;" {
        MethodType "(Ljava/lang/Object;)V",
        MethodHandle REF_invokeStatic:Method java/io/PrintStream.println:"(Ljava/lang/Object;)V",
        MethodType "(Ljava/lang/String;)V"
    };
invokeinterface InterfaceMethod java/util/List.forEach:"(Ljava/util/function/Consumer;)V", 2
```

### 方法引用字节码

```jasm
; Java 代码：list.forEach(System.out::println)
aload_1                     ; 加载 list
getstatic Field java/lang/System.out:"Ljava/io/PrintStream;"
invokedynamic InvokeDynamic REF_invokeStatic:Method java/lang/invoke/LambdaMetafactory.metafactory:
    "(Ljava/lang/invoke/MethodHandles$Lookup;Ljava/lang/String;Ljava/lang/invoke/MethodType;
     Ljava/lang/invoke/MethodType;Ljava/lang/invoke/MethodHandle;Ljava/lang/invoke/MethodType;)
     Ljava/lang/invoke/CallSite;":
    accept:"(Ljava/io/PrintStream;)Ljava/util/function/Consumer;" {
        MethodType "(Ljava/lang/Object;)V",
        MethodHandle REF_invokeVirtual:Method java/io/PrintStream.println:"(Ljava/lang/Object;)V",
        MethodType "(Ljava/lang/Object;)V"
    };
invokeinterface InterfaceMethod java/util/List.forEach:"(Ljava/util/function/Consumer;)V", 2
```

## 方法调用性能考虑

### 调用类型性能对比

1. **invokestatic** - 最快，编译时绑定
2. **invokespecial** - 快速，编译时绑定
3. **invokevirtual** - 中等，运行时虚方法查找
4. **invokeinterface** - 较慢，接口方法查找
5. **invokedynamic** - 可变，取决于 bootstrap 方法

### 优化建议

```jasm
; 优化前：重复的方法调用
aload_0
invokevirtual Method MyClass.getValue:"()I"
istore_1
aload_0
invokevirtual Method MyClass.getValue:"()I"
istore_2

; 优化后：缓存方法调用结果
aload_0
invokevirtual Method MyClass.getValue:"()I"
dup
istore_1
istore_2
```

## 异常处理

### 方法调用异常

```jasm
; 可能抛出异常的方法调用
try_start:
    aload_1
    invokevirtual Method java/lang/String.charAt:"(I)C"
    istore_2
    goto end
try_end:

catch_start:
    ; 处理 StringIndexOutOfBoundsException
    pop             ; 弹出异常对象
    iconst_m1       ; 返回 -1 表示错误
    istore_2
catch_end:

end:
```

## 相关文档

- [基础指令](./basic-instructions.md)
- [控制流指令](./control-flow-instructions.md)
- [对象和类指令](./object-instructions.md)
- [异常处理指令](./exception-instructions.md)