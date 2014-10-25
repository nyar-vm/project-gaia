# JASM 异常处理指令

Java Assembly (JASM) 异常处理指令，包括异常抛出、异常表定义和异常处理机制。

## 异常抛出指令

### 基本异常抛出

```jasm
athrow                      ; 抛出异常
; 栈：[objectref] → objectref（异常对象）
; 异常对象必须是 Throwable 或其子类的实例
```

### 异常创建和抛出示例

```jasm
; 抛出 RuntimeException
new java/lang/RuntimeException
dup
ldc "Something went wrong"
invokespecial Method java/lang/RuntimeException."<init>":"(Ljava/lang/String;)V"
athrow

; 抛出 IllegalArgumentException
new java/lang/IllegalArgumentException
dup
ldc "Invalid argument"
invokespecial Method java/lang/IllegalArgumentException."<init>":"(Ljava/lang/String;)V"
athrow

; 抛出 NullPointerException
new java/lang/NullPointerException
dup
invokespecial Method java/lang/NullPointerException."<init>":"()V"
athrow
```

## 异常表（Exception Table）

### 异常表语法

```jasm
.catch java/lang/Exception from try_start to try_end using catch_handler
.catch java/io/IOException from io_start to io_end using io_handler
.catch * from finally_start to finally_end using finally_handler

; 异常表条目格式：
; .catch <exception_type> from <start_label> to <end_label> using <handler_label>
; 其中 * 表示捕获所有异常（用于 finally 块）
```

### 异常表示例

```jasm
method_start:
try_start:
    ; 可能抛出异常的代码
    aload_1
    invokevirtual Method java/io/FileInputStream.read:"()I"
    istore_2
try_end:
    goto method_end

catch_handler:
    ; 处理 IOException
    astore_3                ; 存储异常对象
    getstatic Field java/lang/System.err:"Ljava/io/PrintStream;"
    ldc "IO Error occurred"
    invokevirtual Method java/io/PrintStream.println:"(Ljava/lang/String;)V"
    goto method_end

method_end:
    return

; 异常表
.catch java/io/IOException from try_start to try_end using catch_handler
```

## try-catch 语句

### 单个 catch 块

```jasm
; Java 代码：
; try {
;     int result = Integer.parseInt(str);
; } catch (NumberFormatException e) {
;     result = -1;
; }

try_start:
    aload_1                 ; 加载字符串
    invokestatic Method java/lang/Integer.parseInt:"(Ljava/lang/String;)I"
    istore_2                ; 存储结果
try_end:
    goto end

catch_number_format:
    pop                     ; 弹出异常对象（如果不需要使用）
    iconst_m1               ; 设置默认值 -1
    istore_2
    goto end

end:
    ; 继续执行

; 异常表
.catch java/lang/NumberFormatException from try_start to try_end using catch_number_format
```

### 多个 catch 块

```jasm
; Java 代码：
; try {
;     FileInputStream fis = new FileInputStream(filename);
;     int data = fis.read();
; } catch (FileNotFoundException e) {
;     // 处理文件未找到
; } catch (IOException e) {
;     // 处理其他 IO 异常
; }

try_start:
    new java/io/FileInputStream
    dup
    aload_1                 ; 文件名
    invokespecial Method java/io/FileInputStream."<init>":"(Ljava/lang/String;)V"
    astore_2                ; 存储 FileInputStream
    aload_2
    invokevirtual Method java/io/FileInputStream.read:"()I"
    istore_3
try_end:
    goto end

catch_file_not_found:
    astore_4                ; 存储异常对象
    getstatic Field java/lang/System.err:"Ljava/io/PrintStream;"
    ldc "File not found"
    invokevirtual Method java/io/PrintStream.println:"(Ljava/lang/String;)V"
    goto end

catch_io_exception:
    astore_4                ; 存储异常对象
    getstatic Field java/lang/System.err:"Ljava/io/PrintStream;"
    ldc "IO error"
    invokevirtual Method java/io/PrintStream.println:"(Ljava/lang/String;)V"
    goto end

end:
    ; 继续执行

; 异常表（注意顺序：更具体的异常在前）
.catch java/io/FileNotFoundException from try_start to try_end using catch_file_not_found
.catch java/io/IOException from try_start to try_end using catch_io_exception
```

## try-finally 语句

### 基本 finally 块

```jasm
; Java 代码：
; try {
;     // 一些操作
; } finally {
;     // 清理代码
; }

try_start:
    ; 主要逻辑
    aload_1
    invokevirtual Method java/io/InputStream.read:"()I"
    istore_2
try_end:
    ; 正常执行 finally
    jsr finally_subroutine
    goto end

exception_handler:
    astore_3                ; 存储异常
    ; 异常情况下执行 finally
    jsr finally_subroutine
    aload_3                 ; 重新加载异常
    athrow                  ; 重新抛出异常

finally_subroutine:
    astore 4                ; 存储返回地址
    ; finally 块的代码
    aload_1
    ifnull skip_close
    aload_1
    invokevirtual Method java/io/InputStream.close:"()V"
skip_close:
    ret 4                   ; 返回

end:
    return

; 异常表
.catch * from try_start to try_end using exception_handler
```

### try-catch-finally

```jasm
; Java 代码：
; try {
;     // 主要逻辑
; } catch (IOException e) {
;     // 处理异常
; } finally {
;     // 清理资源
; }

try_start:
    ; 主要逻辑
    new java/io/FileInputStream
    dup
    aload_1
    invokespecial Method java/io/FileInputStream."<init>":"(Ljava/lang/String;)V"
    astore_2
    aload_2
    invokevirtual Method java/io/FileInputStream.read:"()I"
    istore_3
try_end:
    ; 正常情况下执行 finally
    jsr finally_subroutine
    goto end

catch_io:
    astore_4                ; 存储 IOException
    ; 处理异常
    getstatic Field java/lang/System.err:"Ljava/io/PrintStream;"
    ldc "IO Error"
    invokevirtual Method java/io/PrintStream.println:"(Ljava/lang/String;)V"
    ; catch 块后执行 finally
    jsr finally_subroutine
    goto end

any_exception:
    astore_5                ; 存储其他异常
    ; 其他异常情况下执行 finally
    jsr finally_subroutine
    aload_5
    athrow                  ; 重新抛出异常

finally_subroutine:
    astore 6                ; 存储返回地址
    ; finally 块代码
    aload_2                 ; 加载 FileInputStream
    ifnull skip_close
    aload_2
    invokevirtual Method java/io/FileInputStream.close:"()V"
skip_close:
    ret 6                   ; 返回

end:
    return

; 异常表
.catch java/io/IOException from try_start to try_end using catch_io
.catch * from try_start to try_end using any_exception
.catch * from catch_io to any_exception using any_exception
```

## try-with-resources（自动资源管理）

### 单个资源

```jasm
; Java 代码：
; try (FileInputStream fis = new FileInputStream(filename)) {
;     return fis.read();
; }

; 编译器生成的字节码模式
aconst_null
astore_2                    ; 异常变量
aconst_null
astore_3                    ; 抑制异常变量

try_resource_start:
    new java/io/FileInputStream
    dup
    aload_1                 ; 文件名
    invokespecial Method java/io/FileInputStream."<init>":"(Ljava/lang/String;)V"
    astore 4                ; 存储资源
try_main_start:
    aload 4
    invokevirtual Method java/io/FileInputStream.read:"()I"
    istore 5                ; 存储读取结果
try_main_end:
    ; 正常关闭资源
    aload 4
    ifnull after_close
    aload_2                 ; 检查是否有异常
    ifnull normal_close
    ; 有异常时关闭
    aload 4
    invokevirtual Method java/io/FileInputStream.close:"()V"
    goto after_close
normal_close:
    aload 4
    invokevirtual Method java/io/FileInputStream.close:"()V"
after_close:
    iload 5
    ireturn

catch_main:
    astore_2                ; 存储主要异常
    aload 4
    ifnull rethrow
    aload 4
    invokevirtual Method java/io/FileInputStream.close:"()V"
    goto rethrow

catch_close:
    astore_3                ; 存储关闭时的异常
    aload_2
    aload_3
    invokevirtual Method java/lang/Throwable.addSuppressed:"(Ljava/lang/Throwable;)V"

rethrow:
    aload_2
    athrow

; 异常表
.catch java/lang/Throwable from try_main_start to try_main_end using catch_main
.catch java/lang/Throwable from catch_main to rethrow using catch_close
```

## 自定义异常

### 创建自定义异常类

```jasm
; 自定义异常类的构造函数
.class public MyCustomException
.super java/lang/Exception

.method public "<init>"()V
    aload_0
    invokespecial Method java/lang/Exception."<init>":"()V"
    return
.end method

.method public "<init>"(Ljava/lang/String;)V
    aload_0
    aload_1
    invokespecial Method java/lang/Exception."<init>":"(Ljava/lang/String;)V"
    return
.end method
```

### 抛出自定义异常

```jasm
; 抛出自定义异常
new MyCustomException
dup
ldc "Custom error message"
invokespecial Method MyCustomException."<init>":"(Ljava/lang/String;)V"
athrow
```

## 异常链（Exception Chaining）

### 包装异常

```jasm
; Java 代码：
; try {
;     // 一些操作
; } catch (IOException e) {
;     throw new RuntimeException("Operation failed", e);
; }

try_start:
    ; 可能抛出 IOException 的操作
    aload_1
    invokevirtual Method java/io/FileInputStream.read:"()I"
    pop
try_end:
    goto end

catch_io:
    astore_2                ; 存储原始异常
    new java/lang/RuntimeException
    dup
    ldc "Operation failed"
    aload_2                 ; 原始异常作为 cause
    invokespecial Method java/lang/RuntimeException."<init>":"(Ljava/lang/String;Ljava/lang/Throwable;)V"
    athrow

end:
    return

.catch java/io/IOException from try_start to try_end using catch_io
```

## 异常处理最佳实践

### 资源清理

```jasm
; 确保资源被正确清理
aconst_null
astore_2                    ; 资源变量

try_start:
    new java/io/FileInputStream
    dup
    aload_1
    invokespecial Method java/io/FileInputStream."<init>":"(Ljava/lang/String;)V"
    astore_2                ; 存储资源
    ; 使用资源
    aload_2
    invokevirtual Method java/io/FileInputStream.read:"()I"
    istore_3
try_end:
    ; 正常情况下清理资源
    aload_2
    ifnull end
    aload_2
    invokevirtual Method java/io/FileInputStream.close:"()V"
    goto end

exception_handler:
    astore_4                ; 存储异常
    ; 异常情况下也要清理资源
    aload_2
    ifnull rethrow
    aload_2
    invokevirtual Method java/io/FileInputStream.close:"()V"
rethrow:
    aload_4
    athrow

end:
    return

.catch * from try_start to try_end using exception_handler
```

### 异常信息记录

```jasm
; 记录异常信息
catch_handler:
    astore_1                ; 存储异常对象
    
    ; 记录异常信息
    getstatic Field java/lang/System.err:"Ljava/io/PrintStream;"
    aload_1
    invokevirtual Method java/lang/Throwable.getMessage:"()Ljava/lang/String;"
    invokevirtual Method java/io/PrintStream.println:"(Ljava/lang/String;)V"
    
    ; 打印堆栈跟踪
    aload_1
    invokevirtual Method java/lang/Throwable.printStackTrace:"()V"
    
    ; 处理异常或重新抛出
    aload_1
    athrow
```

## 性能考虑

### 异常处理开销

```jasm
; 避免在循环中使用异常控制流
; 错误示例：
loop_start:
    try_start:
        aload_1
        iload_2
        aaload              ; 可能抛出 ArrayIndexOutOfBoundsException
        ; 处理元素
    try_end:
        iinc 2, 1
        goto loop_start
    catch_bounds:
        ; 使用异常来检测数组结束
        goto loop_end
    .catch java/lang/ArrayIndexOutOfBoundsException from try_start to try_end using catch_bounds

; 正确示例：
aload_1
arraylength
istore_3                    ; 获取数组长度
loop_start:
    iload_2
    iload_3
    if_icmpge loop_end      ; 正常的边界检查
    aload_1
    iload_2
    aaload
    ; 处理元素
    iinc 2, 1
    goto loop_start
loop_end:
```

## 相关文档

- [基础指令](./basic-instructions.md)
- [方法调用指令](./method-instructions.md)
- [对象操作指令](./object-instructions.md)
- [控制流指令](./control-flow-instructions.md)