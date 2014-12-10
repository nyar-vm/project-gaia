# MSIL 异常处理指令

Microsoft Intermediate Language (MSIL) 异常处理指令，包括异常抛出、捕获、finally 块和异常表定义。

## 异常抛出指令

### 基本异常抛出

```msil
throw                       ; 抛出栈顶的异常对象
rethrow                     ; 重新抛出当前捕获的异常

; 示例：抛出异常
.method public static void ThrowExceptionExample(int32 value) cil managed
{
    .maxstack 2
    
    ldarg.0                     ; 加载参数
    ldc.i4.0
    bge.s valid_value           ; 如果 value >= 0
    
    ; 抛出 ArgumentException
    ldstr "Value cannot be negative"
    newobj instance void [mscorlib]System.ArgumentException::.ctor(string)
    throw
    
valid_value:
    ; 正常处理
    ldstr "Value is valid"
    call void [mscorlib]System.Console::WriteLine(string)
    ret
}
```

### 重新抛出异常

```msil
; 示例：重新抛出异常
.method public static void RethrowExample() cil managed
{
    .maxstack 2
    
    .try
    {
        ; 可能抛出异常的代码
        call void SomeMethodThatMightThrow()
        leave.s end
    }
    catch [mscorlib]System.Exception
    {
        ; 记录异常
        ldstr "Exception caught, logging and rethrowing"
        call void [mscorlib]System.Console::WriteLine(string)
        
        ; 重新抛出原始异常（保持堆栈跟踪）
        rethrow
    }
    
end:
    ret
}
```

## 异常处理结构

### try-catch 块

```msil
; 基本 try-catch 结构
.method public static void TryCatchExample() cil managed
{
    .maxstack 2
    .locals init ([0] class [mscorlib]System.Exception ex)
    
    .try
    {
        ; try 块中的代码
        ldstr "10"
        call int32 [mscorlib]System.Int32::Parse(string)
        call void [mscorlib]System.Console::WriteLine(int32)
        leave.s end             ; 正常退出 try 块
    }
    catch [mscorlib]System.FormatException
    {
        ; 捕获 FormatException
        stloc.0                 ; ex = caught exception
        ldstr "Format exception caught"
        call void [mscorlib]System.Console::WriteLine(string)
        leave.s end             ; 退出 catch 块
    }
    catch [mscorlib]System.Exception
    {
        ; 捕获其他异常
        stloc.0                 ; ex = caught exception
        ldstr "General exception caught"
        call void [mscorlib]System.Console::WriteLine(string)
        leave.s end             ; 退出 catch 块
    }
    
end:
    ret
}
```

### try-finally 块

```msil
; try-finally 结构
.method public static void TryFinallyExample() cil managed
{
    .maxstack 1
    .locals init ([0] class [System.IO]System.IO.FileStream fs)
    
    .try
    {
        ; 打开文件
        ldstr "test.txt"
        ldc.i4.3                ; FileMode.Open
        newobj instance void [System.IO]System.IO.FileStream::.ctor(string, valuetype [System.IO]System.IO.FileMode)
        stloc.0
        
        ; 使用文件
        ldstr "File operations"
        call void [mscorlib]System.Console::WriteLine(string)
        
        leave.s end             ; 正常退出
    }
    finally
    {
        ; 清理资源
        ldloc.0                 ; 加载文件流
        brfalse.s skip_close    ; 如果为 null 跳过
        
        ldloc.0
        callvirt instance void [System.IO]System.IO.Stream::Close()
        
skip_close:
        ldstr "Finally block executed"
        call void [mscorlib]System.Console::WriteLine(string)
        endfinally              ; 结束 finally 块
    }
    
end:
    ret
}
```

### try-catch-finally 块

```msil
; 完整的异常处理结构
.method public static void TryCatchFinallyExample() cil managed
{
    .maxstack 2
    .locals init (
        [0] class [System.IO]System.IO.FileStream fs,
        [1] class [mscorlib]System.Exception ex
    )
    
    .try
    {
        ; 尝试打开文件
        ldstr "nonexistent.txt"
        ldc.i4.3                ; FileMode.Open
        newobj instance void [System.IO]System.IO.FileStream::.ctor(string, valuetype [System.IO]System.IO.FileMode)
        stloc.0
        
        ; 文件操作
        ldstr "File opened successfully"
        call void [mscorlib]System.Console::WriteLine(string)
        
        leave.s end
    }
    catch [System.IO]System.IO.FileNotFoundException
    {
        ; 处理文件未找到异常
        stloc.1
        ldstr "File not found"
        call void [mscorlib]System.Console::WriteLine(string)
        leave.s end
    }
    catch [mscorlib]System.Exception
    {
        ; 处理其他异常
        stloc.1
        ldstr "Other exception occurred"
        call void [mscorlib]System.Console::WriteLine(string)
        leave.s end
    }
    finally
    {
        ; 清理资源
        ldloc.0
        brfalse.s skip_dispose
        
        ldloc.0
        callvirt instance void [System.IO]System.IO.Stream::Dispose()
        
skip_dispose:
        ldstr "Resources cleaned up"
        call void [mscorlib]System.Console::WriteLine(string)
        endfinally
    }
    
end:
    ret
}
```

## 异常过滤器

### 异常过滤器（when 子句）

```msil
; 带过滤器的异常处理
.method public static void ExceptionFilterExample(int32 errorCode) cil managed
{
    .maxstack 2
    .locals init ([0] class [mscorlib]System.Exception ex)
    
    .try
    {
        ldarg.0
        call void ThrowBasedOnErrorCode(int32)
        leave.s end
    }
    filter
    {
        ; 过滤器代码：只处理特定的异常
        dup                     ; 复制异常对象
        isinst [mscorlib]System.ArgumentException
        brfalse.s not_argument_exception
        
        ; 检查异常消息
        castclass [mscorlib]System.ArgumentException
        callvirt instance string [mscorlib]System.Exception::get_Message()
        ldstr "specific error"
        callvirt instance bool [mscorlib]System.String::Contains(string)
        br.s filter_end
        
not_argument_exception:
        ldc.i4.0                ; 不处理此异常
        
filter_end:
        endfilter               ; 返回过滤器结果
    }
    {
        ; 处理通过过滤器的异常
        stloc.0
        ldstr "Filtered exception handled"
        call void [mscorlib]System.Console::WriteLine(string)
        leave.s end
    }
    
end:
    ret
}
```

## 自定义异常

### 定义自定义异常类

```msil
; 自定义异常类
.class public CustomException extends [mscorlib]System.Exception
{
    .field private int32 errorCode
    
    ; 默认构造函数
    .method public hidebysig specialname rtspecialname 
            instance void .ctor() cil managed
    {
        .maxstack 1
        
        ldarg.0
        call instance void [mscorlib]System.Exception::.ctor()
        
        ldarg.0
        ldc.i4.0
        stfld int32 CustomException::errorCode
        
        ret
    }
    
    ; 带消息的构造函数
    .method public hidebysig specialname rtspecialname 
            instance void .ctor(string message) cil managed
    {
        .maxstack 2
        
        ldarg.0
        ldarg.1
        call instance void [mscorlib]System.Exception::.ctor(string)
        
        ldarg.0
        ldc.i4.0
        stfld int32 CustomException::errorCode
        
        ret
    }
    
    ; 带消息和错误代码的构造函数
    .method public hidebysig specialname rtspecialname 
            instance void .ctor(string message, int32 errorCode) cil managed
    {
        .maxstack 2
        
        ldarg.0
        ldarg.1
        call instance void [mscorlib]System.Exception::.ctor(string)
        
        ldarg.0
        ldarg.2
        stfld int32 CustomException::errorCode
        
        ret
    }
    
    ; ErrorCode 属性
    .method public hidebysig specialname instance int32 get_ErrorCode() cil managed
    {
        .maxstack 1
        
        ldarg.0
        ldfld int32 CustomException::errorCode
        ret
    }
    
    .property instance int32 ErrorCode()
    {
        .get instance int32 CustomException::get_ErrorCode()
    }
}

; 使用自定义异常
.method public static void UseCustomException(int32 value) cil managed
{
    .maxstack 3
    
    ldarg.0
    ldc.i4.0
    bge.s valid_input
    
    ; 抛出自定义异常
    ldstr "Invalid input value"
    ldarg.0
    newobj instance void CustomException::.ctor(string, int32)
    throw
    
valid_input:
    ldstr "Input is valid"
    call void [mscorlib]System.Console::WriteLine(string)
    ret
}
```

## 异常链

### 内部异常处理

```msil
; 异常链示例
.method public static void ExceptionChainingExample() cil managed
{
    .maxstack 3
    .locals init ([0] class [mscorlib]System.Exception innerEx)
    
    .try
    {
        ; 调用可能抛出异常的方法
        call void MethodThatThrowsException()
        leave.s end
    }
    catch [mscorlib]System.Exception
    {
        ; 捕获内部异常
        stloc.0
        
        ; 创建包装异常，保留内部异常
        ldstr "Operation failed"
        ldloc.0                 ; 内部异常
        newobj instance void [mscorlib]System.ApplicationException::.ctor(string, class [mscorlib]System.Exception)
        throw                   ; 抛出包装异常
    }
    
end:
    ret
}
```

## 资源管理

### using 语句模式

```msil
; 实现 using 语句的模式
.method public static void UsingPatternExample() cil managed
{
    .maxstack 2
    .locals init ([0] class [System.IO]System.IO.FileStream fs)
    
    .try
    {
        ; 获取资源
        ldstr "test.txt"
        ldc.i4.3                ; FileMode.Open
        newobj instance void [System.IO]System.IO.FileStream::.ctor(string, valuetype [System.IO]System.IO.FileMode)
        stloc.0
        
        ; 使用资源
        ldstr "Using resource"
        call void [mscorlib]System.Console::WriteLine(string)
        
        leave.s end
    }
    finally
    {
        ; 自动释放资源
        ldloc.0
        brfalse.s skip_dispose
        
        ldloc.0
        callvirt instance void [mscorlib]System.IDisposable::Dispose()
        
skip_dispose:
        endfinally
    }
    
end:
    ret
}
```

### 多个资源的管理

```msil
; 管理多个 IDisposable 资源
.method public static void MultipleResourcesExample() cil managed
{
    .maxstack 2
    .locals init (
        [0] class [System.IO]System.IO.FileStream fs1,
        [1] class [System.IO]System.IO.FileStream fs2
    )
    
    .try
    {
        ; 获取第一个资源
        ldstr "file1.txt"
        ldc.i4.3
        newobj instance void [System.IO]System.IO.FileStream::.ctor(string, valuetype [System.IO]System.IO.FileMode)
        stloc.0
        
        ; 获取第二个资源
        ldstr "file2.txt"
        ldc.i4.3
        newobj instance void [System.IO]System.IO.FileStream::.ctor(string, valuetype [System.IO]System.IO.FileMode)
        stloc.1
        
        ; 使用资源
        ldstr "Using multiple resources"
        call void [mscorlib]System.Console::WriteLine(string)
        
        leave.s end
    }
    finally
    {
        ; 释放第二个资源
        ldloc.1
        brfalse.s dispose_first
        
        ldloc.1
        callvirt instance void [mscorlib]System.IDisposable::Dispose()
        
dispose_first:
        ; 释放第一个资源
        ldloc.0
        brfalse.s end_finally
        
        ldloc.0
        callvirt instance void [mscorlib]System.IDisposable::Dispose()
        
end_finally:
        endfinally
    }
    
end:
    ret
}
```

## 异常性能优化

### 避免异常的性能开销

```msil
; 使用 TryParse 模式避免异常
.method public static int32 SafeParseExample(string input) cil managed
{
    .maxstack 2
    .locals init (
        [0] int32 result,
        [1] bool success
    )
    
    ; 使用 TryParse 而不是 Parse（避免异常）
    ldarg.0
    ldloca.s 0                  ; result 的地址
    call bool [mscorlib]System.Int32::TryParse(string, int32&)
    stloc.1                     ; success = TryParse result
    
    ldloc.1
    brtrue.s return_result      ; 如果解析成功
    
    ; 解析失败，返回默认值
    ldc.i4.0
    ret
    
return_result:
    ldloc.0                     ; 返回解析结果
    ret
}
```

### 异常缓存

```msil
; 缓存常用异常实例
.class public ExceptionCache extends [mscorlib]System.Object
{
    .field private static class [mscorlib]System.ArgumentNullException cachedArgumentNullException
    .field private static class [mscorlib]System.ArgumentException cachedArgumentException
    
    .method private hidebysig specialname rtspecialname static 
            void .cctor() cil managed
    {
        .maxstack 1
        
        ; 预创建常用异常
        ldstr "Value cannot be null"
        newobj instance void [mscorlib]System.ArgumentNullException::.ctor(string)
        stsfld class [mscorlib]System.ArgumentNullException ExceptionCache::cachedArgumentNullException
        
        ldstr "Invalid argument"
        newobj instance void [mscorlib]System.ArgumentException::.ctor(string)
        stsfld class [mscorlib]System.ArgumentException ExceptionCache::cachedArgumentException
        
        ret
    }
    
    .method public static void ThrowArgumentNull() cil managed
    {
        .maxstack 1
        
        ldsfld class [mscorlib]System.ArgumentNullException ExceptionCache::cachedArgumentNullException
        throw
    }
    
    .method public static void ThrowArgumentException() cil managed
    {
        .maxstack 1
        
        ldsfld class [mscorlib]System.ArgumentException ExceptionCache::cachedArgumentException
        throw
    }
}
```

## 调试和诊断

### 异常信息收集

```msil
; 收集详细的异常信息
.method public static void DetailedExceptionHandling() cil managed
{
    .maxstack 3
    .locals init ([0] class [mscorlib]System.Exception ex)
    
    .try
    {
        call void RiskyOperation()
        leave.s end
    }
    catch [mscorlib]System.Exception
    {
        stloc.0                 ; 保存异常
        
        ; 输出异常类型
        ldstr "Exception Type: "
        ldloc.0
        callvirt instance class [mscorlib]System.Type [mscorlib]System.Object::GetType()
        callvirt instance string [mscorlib]System.Type::get_FullName()
        call string [mscorlib]System.String::Concat(string, string)
        call void [mscorlib]System.Console::WriteLine(string)
        
        ; 输出异常消息
        ldstr "Message: "
        ldloc.0
        callvirt instance string [mscorlib]System.Exception::get_Message()
        call string [mscorlib]System.String::Concat(string, string)
        call void [mscorlib]System.Console::WriteLine(string)
        
        ; 输出堆栈跟踪
        ldstr "Stack Trace: "
        ldloc.0
        callvirt instance string [mscorlib]System.Exception::get_StackTrace()
        call string [mscorlib]System.String::Concat(string, string)
        call void [mscorlib]System.Console::WriteLine(string)
        
        ; 检查内部异常
        ldloc.0
        callvirt instance class [mscorlib]System.Exception [mscorlib]System.Exception::get_InnerException()
        brfalse.s no_inner_exception
        
        ldstr "Inner Exception: "
        ldloc.0
        callvirt instance class [mscorlib]System.Exception [mscorlib]System.Exception::get_InnerException()
        callvirt instance string [mscorlib]System.Exception::get_Message()
        call string [mscorlib]System.String::Concat(string, string)
        call void [mscorlib]System.Console::WriteLine(string)
        
no_inner_exception:
        leave.s end
    }
    
end:
    ret
}
```

## 最佳实践

### 异常处理最佳实践

```msil
; 异常处理最佳实践示例
.method public static void ExceptionBestPractices() cil managed
{
    .maxstack 2
    .locals init ([0] class [mscorlib]System.Exception ex)
    
    .try
    {
        ; 1. 只捕获你能处理的异常
        ; 2. 尽可能具体地捕获异常类型
        ; 3. 不要忽略异常
        
        call void OperationThatMightFail()
        leave.s end
    }
    catch [System.IO]System.IO.FileNotFoundException
    {
        ; 处理特定的文件未找到异常
        stloc.0
        ldstr "Required file not found, using default configuration"
        call void [mscorlib]System.Console::WriteLine(string)
        
        ; 执行恢复操作
        call void LoadDefaultConfiguration()
        leave.s end
    }
    catch [mscorlib]System.UnauthorizedAccessException
    {
        ; 处理权限异常
        stloc.0
        ldstr "Access denied, please check permissions"
        call void [mscorlib]System.Console::WriteLine(string)
        
        ; 记录安全事件
        ldloc.0
        call void LogSecurityEvent(class [mscorlib]System.Exception)
        
        ; 重新抛出，因为这是不可恢复的错误
        rethrow
    }
    
end:
    ret
}
```

### 资源清理模式

```msil
; 正确的资源清理模式
.method public static void ProperResourceCleanup() cil managed
{
    .maxstack 2
    .locals init (
        [0] class [System.IO]System.IO.FileStream fs,
        [1] class [System.IO]System.IO.StreamReader reader
    )
    
    .try
    {
        ; 获取资源
        ldstr "data.txt"
        ldc.i4.3                ; FileMode.Open
        newobj instance void [System.IO]System.IO.FileStream::.ctor(string, valuetype [System.IO]System.IO.FileMode)
        stloc.0
        
        ldloc.0
        newobj instance void [System.IO]System.IO.StreamReader::.ctor(class [System.IO]System.IO.Stream)
        stloc.1
        
        ; 使用资源
        ldloc.1
        callvirt instance string [System.IO]System.IO.StreamReader::ReadToEnd()
        call void [mscorlib]System.Console::WriteLine(string)
        
        leave.s end
    }
    finally
    {
        ; 按相反顺序释放资源
        ldloc.1
        brfalse.s dispose_stream
        
        ldloc.1
        callvirt instance void [System.IO]System.IO.StreamReader::Dispose()
        
dispose_stream:
        ldloc.0
        brfalse.s end_finally
        
        ldloc.0
        callvirt instance void [System.IO]System.IO.FileStream::Dispose()
        
end_finally:
        endfinally
    }
    
end:
    ret
}
```

## 相关文档

- [基础指令](./basic-instructions.md)
- [控制流指令](./control-flow-instructions.md)
- [方法调用指令](./method-instructions.md)
- [对象操作指令](./object-instructions.md)