# Python 异常处理指令

Gaia 框架为 Python 后端提供了完整的异常处理机制，支持传统的 try-except 模式、异常链、上下文管理器和自定义异常类型。

## 基本异常处理

### Try-Except 块

```python
# Gaia 代码
def divide_numbers(a: float, b: float) -> float:
    try:
        result = a / b
        return result
    except ZeroDivisionError:
        return float('inf')
    except TypeError:
        raise ValueError("Arguments must be numbers")
```

编译为 Python 代码：
```python
# 生成的 Python 代码
def divide_numbers(a: float, b: float) -> float:
    try:
        result = a / b
        return result
    except ZeroDivisionError:
        return float('inf')
    except TypeError:
        raise ValueError("Arguments must be numbers")
```

### Try-Except-Finally

```python
# Gaia 代码
def process_file(filename: str) -> str:
    file_handle = None
    try:
        file_handle = open(filename, 'r')
        content = file_handle.read()
        return content
    except FileNotFoundError:
        return "File not found"
    except IOError:
        return "Error reading file"
    finally:
        if file_handle:
            file_handle.close()
```

### Try-Except-Else

```python
# Gaia 代码
def safe_divide(a: float, b: float) -> tuple[float, bool]:
    try:
        result = a / b
    except ZeroDivisionError:
        return 0.0, False
    else:
        # 只有在没有异常时执行
        return result, True
```

## 异常类型

### 内置异常类型

```python
# Gaia 代码
# 算术异常
raise ZeroDivisionError("Cannot divide by zero")
raise ArithmeticError("Arithmetic operation failed")

# 类型异常
raise TypeError("Invalid type")
raise ValueError("Invalid value")

# 索引异常
raise IndexError("List index out of range")
raise KeyError("Dictionary key not found")

# 属性异常
raise AttributeError("Object has no attribute")

# 文件异常
raise FileNotFoundError("File not found")
raise IOError("I/O operation failed")
```

### 自定义异常

```python
# Gaia 代码
class ValidationError(Exception):
    """数据验证异常"""
    def __init__(self, message: str, field: str = None):
        super().__init__(message)
        self.field = field

class BusinessLogicError(Exception):
    """业务逻辑异常"""
    def __init__(self, message: str, error_code: int = None):
        super().__init__(message)
        self.error_code = error_code

class ConfigurationError(Exception):
    """配置异常"""
    def __init__(self, message: str, config_key: str = None):
        super().__init__(message)
        self.config_key = config_key
```

## 异常处理指令

### TRY 指令

```python
# Gaia 代码
TRY
    # 可能抛出异常的代码
    DIVIDE a b result
END_TRY
```

### CATCH 指令

```python
# Gaia 代码
CATCH ZeroDivisionError
    # 处理除零异常
    LOAD_CONST 0.0 result
END_CATCH
```

### THROW 指令

```python
# Gaia 代码
THROW ValueError "Invalid input value"
```

### ASSERT 指令

```python
# Gaia 代码
ASSERT condition "Assertion failed message"
```

## 异常链和上下文

### 异常链

```python
# Gaia 代码
def process_data(data: str) -> dict:
    try:
        parsed = json.loads(data)
        return parsed
    except json.JSONDecodeError as e:
        raise DataProcessingError("Failed to parse JSON data") from e
```

### 异常上下文

```python
# Gaia 代码
def validate_and_process(data: any) -> any:
    try:
        if not isinstance(data, dict):
            raise TypeError("Data must be a dictionary")
        
        if "required_field" not in data:
            raise KeyError("Missing required field")
        
        return process_data(data)
    except (TypeError, KeyError) as e:
        # 添加上下文信息
        e.add_note(f"Error processing data: {data}")
        raise
```

## 上下文管理器和异常

### 自定义上下文管理器

```python
# Gaia 代码
class DatabaseConnection:
    def __init__(self, connection_string: str):
        self.connection_string = connection_string
        self.connection = None
    
    def __enter__(self):
        self.connection = self._connect()
        return self.connection
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        if self.connection:
            if exc_type is not None:
                # 发生异常时回滚
                self.connection.rollback()
            else:
                # 正常完成时提交
                self.connection.commit()
            self.connection.close()
        # 返回 False 让异常继续传播
        return False
```

### 使用上下文管理器

```python
# Gaia 代码
def query_database(query: str) -> list:
    try:
        with DatabaseConnection("db://localhost") as conn:
            result = conn.execute(query)
            return result.fetchall()
    except DatabaseError as e:
        logger.error(f"Database query failed: {e}")
        return []
```

## 异常处理模式

### 重试模式

```python
# Gaia 代码
import time
from typing import Callable, Any

def retry_operation(operation: Callable[[], Any], 
                   max_attempts: int = 3, 
                   delay: float = 1.0) -> Any:
    """带重试的操作执行"""
    last_exception = None
    
    for attempt in range(max_attempts):
        try:
            return operation()
        except (NetworkError, TimeoutError) as e:
            last_exception = e
            if attempt < max_attempts - 1:
                time.sleep(delay * (2 ** attempt))  # 指数退避
            else:
                break
    
    raise RetryExhaustedError(f"Operation failed after {max_attempts} attempts") from last_exception
```

### 断路器模式

```python
# Gaia 代码
class CircuitBreaker:
    def __init__(self, failure_threshold: int = 5, timeout: float = 60.0):
        self.failure_threshold = failure_threshold
        self.timeout = timeout
        self.failure_count = 0
        self.last_failure_time = None
        self.state = "closed"  # closed, open, half-open
    
    def call(self, operation: Callable[[], Any]) -> Any:
        if self.state == "open":
            if time.time() - self.last_failure_time > self.timeout:
                self.state = "half-open"
            else:
                raise CircuitOpenError("Circuit breaker is open")
        
        try:
            result = operation()
            if self.state == "half-open":
                self.state = "closed"
                self.failure_count = 0
            return result
        except Exception as e:
            self.failure_count += 1
            self.last_failure_time = time.time()
            
            if self.failure_count >= self.failure_threshold:
                self.state = "open"
            
            raise
```

### 异常转换

```python
# Gaia 代码
def convert_exceptions(source_exceptions: tuple[type, ...], 
                      target_exception: type):
    """异常类型转换装饰器"""
    def decorator(func: Callable) -> Callable:
        def wrapper(*args, **kwargs):
            try:
                return func(*args, **kwargs)
            except source_exceptions as e:
                raise target_exception(f"Converted from {type(e).__name__}: {e}") from e
        return wrapper
    return decorator

@convert_exceptions((IOError, OSError), DataAccessError)
def read_file_safely(filename: str) -> str:
    with open(filename, 'r') as f:
        return f.read()
```

## 异常日志记录

### 结构化日志记录

```python
# Gaia 代码
import logging
from datetime import datetime

class ExceptionLogger:
    def __init__(self, logger_name: str):
        self.logger = logging.getLogger(logger_name)
    
    def log_exception(self, exception: Exception, context: dict = None):
        """记录异常信息"""
        log_data = {
            "timestamp": datetime.now().isoformat(),
            "exception_type": type(exception).__name__,
            "exception_message": str(exception),
            "exception_args": exception.args,
            "context": context or {}
        }
        
        self.logger.error(f"Exception occurred: {log_data}", exc_info=True)
    
    def log_with_recovery(self, operation: Callable, 
                         recovery_func: Callable,
                         context: dict = None):
        """记录异常并尝试恢复"""
        try:
            return operation()
        except Exception as e:
            self.log_exception(e, context)
            return recovery_func(e)
```

### 异常监控

```python
# Gaia 代码
class ExceptionMonitor:
    def __init__(self):
        self.exception_stats = defaultdict(int)
        self.exception_timestamps = []
    
    def record_exception(self, exception: Exception):
        """记录异常统计"""
        exception_type = type(exception).__name__
        self.exception_stats[exception_type] += 1
        self.exception_timestamps.append({
            "type": exception_type,
            "time": time.time()
        })
    
    def get_exception_rate(self, time_window: float = 300.0) -> float:
        """获取异常率（每分钟的异常数）"""
        current_time = time.time()
        recent_exceptions = [
            ts for ts in self.exception_timestamps 
            if current_time - ts["time"] <= time_window
        ]
        return len(recent_exceptions) / (time_window / 60.0)
    
    def should_alert(self, threshold: int = 10) -> bool:
        """判断是否需要告警"""
        return self.get_exception_rate() > threshold
```

## 资源管理异常

### 文件资源管理

```python
# Gaia 代码
class SafeFileManager:
    def __init__(self, filename: str, mode: str = 'r'):
        self.filename = filename
        self.mode = mode
        self.file = None
    
    def __enter__(self):
        try:
            self.file = open(self.filename, self.mode)
            return self.file
        except FileNotFoundError:
            raise FileOperationError(f"File not found: {self.filename}")
        except PermissionError:
            raise FileOperationError(f"Permission denied: {self.filename}")
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        if self.file:
            try:
                self.file.close()
            except Exception as e:
                # 记录关闭文件时的异常，但不阻止原始异常的传播
                logging.warning(f"Error closing file: {e}")
        return False  # 不抑制异常
```

### 网络资源管理

```python
# Gaia 代码
class NetworkConnection:
    def __init__(self, host: str, port: int, timeout: float = 30.0):
        self.host = host
        self.port = port
        self.timeout = timeout
        self.socket = None
        self.connected = False
    
    def __enter__(self):
        try:
            self.socket = socket.create_connection(
                (self.host, self.port), 
                timeout=self.timeout
            )
            self.connected = True
            return self
        except socket.timeout:
            raise NetworkTimeoutError(f"Connection timeout to {self.host}:{self.port}")
        except socket.gaierror:
            raise NetworkError(f"Failed to resolve host: {self.host}")
        except ConnectionRefusedError:
            raise NetworkError(f"Connection refused by {self.host}:{self.port}")
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        if self.socket:
            try:
                self.socket.close()
            except Exception as e:
                logging.warning(f"Error closing socket: {e}")
        self.connected = False
        return False
```

## 性能优化

### 异常性能考虑

```python
# Gaia 代码
# 不推荐：使用异常进行控制流
def find_item_bad(items: list, target: any) -> bool:
    try:
        items.index(target)
        return True
    except ValueError:
        return False

# 推荐：使用条件判断
def find_item_good(items: list, target: any) -> bool:
    return target in items
```

### 异常缓存

```python
# Gaia 代码
class CachedException:
    """缓存异常实例以避免重复创建"""
    def __init__(self, exception_type: type, message: str):
        self.exception = exception_type(message)
    
    def raise_exception(self):
        raise self.exception

# 使用缓存的异常
validation_error = CachedException(ValueError, "Validation failed")
# 在需要时抛出
try:
    validation_error.raise_exception()
except ValueError as e:
    # 处理异常
    pass
```

## 最佳实践

1. **具体异常类型**: 使用具体的异常类型而不是通用的 Exception
2. **异常链**: 使用 `from` 保留异常链信息
3. **异常文档**: 在函数文档中说明可能抛出的异常
4. **异常测试**: 编写测试用例验证异常处理逻辑
5. **资源清理**: 使用 finally 或上下文管理器确保资源清理
6. **异常日志**: 记录异常信息便于调试和监控
7. **性能考虑**: 避免在热路径中频繁抛出和捕获异常