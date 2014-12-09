# Python 函数和方法指令

Gaia 框架为 Python 后端提供了完整的函数定义、调用和方法操作指令集，支持高级 Python 特性如装饰器、生成器和异步函数。

## 函数定义

### 基本函数定义

```python
# Gaia 代码
def add(a: int, b: int) -> int:
    return a + b
```

编译为 Python 字节码：
```python
# 生成的 Python 代码
def add(a: int, b: int) -> int:
    return a + b
```

### 带默认参数的函数

```python
# Gaia 代码
def greet(name: str, greeting: str = "Hello") -> str:
    return f"{greeting}, {name}!"
```

### 可变参数函数

```python
# Gaia 代码
def sum_all(*numbers: int) -> int:
    return sum(numbers)
```

### 关键字参数函数

```python
# Gaia 代码
def create_user(**kwargs: any) -> dict:
    return kwargs
```

## 函数调用指令

### 直接调用

```python
# Gaia 代码
CALL_FUNCTION add 5 3 result
```

### 方法调用

```python
# Gaia 代码
CALL_METHOD obj "method_name" arg1 arg2 result
```

### 类方法调用

```python
# Gaia 代码
CALL_FUNCTION_EX cls.method arg1 arg2 result
```

## 高级函数特性

### 装饰器

```python
# Gaia 代码
def timing_decorator(func):
    import time
    def wrapper(*args, **kwargs):
        start = time.time()
        result = func(*args, **kwargs)
        end = time.time()
        print(f"Function {func.__name__} took {end - start} seconds")
        return result
    return wrapper

@timing_decorator
def slow_function() -> None:
    import time
    time.sleep(1)
```

### 生成器函数

```python
# Gaia 代码
def fibonacci() -> iter[int]:
    a, b = 0, 1
    while True:
        yield a
        a, b = b, a + b
```

### 异步函数

```python
# Gaia 代码
async def fetch_data(url: str) -> str:
    import aiohttp
    async with aiohttp.ClientSession() as session:
        async with session.get(url) as response:
            return await response.text()
```

## 参数处理

### 类型注解

```python
# Gaia 代码
from typing import List, Dict, Optional, Union

def process_data(items: List[int], 
                config: Dict[str, any], 
                optional: Optional[str] = None) -> Union[int, str]:
    # 函数体
    pass
```

### 参数验证

```python
# Gaia 代码
def validated_function(age: int) -> str:
    if not isinstance(age, int):
        raise TypeError("Age must be an integer")
    if age < 0:
        raise ValueError("Age must be non-negative")
    return f"Age: {age}"
```

### 参数解构

```python
# Gaia 代码
def process_coordinates(x: float, y: float, z: float) -> float:
    return x**2 + y**2 + z**2

# 使用元组解构调用
coords = (1.0, 2.0, 3.0)
result = process_coordinates(*coords)
```

## 返回值处理

### 单返回值

```python
# Gaia 代码
def get_value() -> int:
    return 42
```

### 多返回值

```python
# Gaia 代码
def get_stats(numbers: list[float]) -> tuple[float, float]:
    return sum(numbers), sum(numbers) / len(numbers)
```

### 返回指令

```python
# Gaia 代码
RETURN_VALUE value
RETURN_CONST value
```

## 闭包和词法作用域

### 闭包定义

```python
# Gaia 代码
def make_counter() -> callable:
    count = 0
    def counter() -> int:
        nonlocal count
        count += 1
        return count
    return counter
```

### 工厂函数

```python
# Gaia 代码
def create_multiplier(factor: float) -> callable:
    def multiplier(x: float) -> float:
        return x * factor
    return multiplier
```

## 类方法

### 实例方法

```python
# Gaia 代码
class Calculator:
    def __init__(self, initial_value: float = 0.0):
        self.value = initial_value
    
    def add(self, x: float) -> float:
        self.value += x
        return self.value
    
    def multiply(self, x: float) -> float:
        self.value *= x
        return self.value
```

### 类方法

```python
# Gaia 代码
class MathUtils:
    PI: float = 3.14159265359
    
    @classmethod
    def circle_area(cls, radius: float) -> float:
        return cls.PI * radius ** 2
    
    @staticmethod
    def add(x: float, y: float) -> float:
        return x + y
```

### 属性方法

```python
# Gaia 代码
class Temperature:
    def __init__(self, celsius: float):
        self._celsius = celsius
    
    @property
    def celsius(self) -> float:
        return self._celsius
    
    @property
    def fahrenheit(self) -> float:
        return self._celsius * 9/5 + 32
    
    @celsius.setter
    def celsius(self, value: float):
        self._celsius = value
```

## 函数注解和元数据

### 函数注解

```python
# Gaia 代码
def complex_function(x: int, y: str) -> dict[str, any]:
    """Process x and y to return a dictionary."""
    return {"x": x, "y": y}

# 访问函数注解
print(complex_function.__annotations__)
print(complex_function.__doc__)
```

### 函数属性

```python
# Gaia 代码
def traced_function(x: int) -> int:
    traced_function.call_count += 1
    return x * 2

traced_function.call_count = 0
```

## 高阶函数

### map、filter、reduce

```python
# Gaia 代码
from functools import reduce

# map 示例
numbers: list[int] = [1, 2, 3, 4, 5]
squared: list[int] = list(map(lambda x: x**2, numbers))

# filter 示例
evens: list[int] = list(filter(lambda x: x % 2 == 0, numbers))

# reduce 示例
sum_all: int = reduce(lambda x, y: x + y, numbers)
```

### 偏函数

```python
# Gaia 代码
from functools import partial

def power(base: float, exponent: float) -> float:
    return base ** exponent

# 创建偏函数
square = partial(power, exponent=2)
cube = partial(power, exponent=3)
```

## 性能优化

### 函数内联

对于简单函数，考虑手动内联：

```python
# Gaia 代码
# 原始函数
def add(a: int, b: int) -> int:
    return a + b

# 内联版本（手动优化）
result = a + b  # 直接计算，避免函数调用开销
```

### 局部变量优化

```python
# Gaia 代码
# 推荐：使用局部变量缓存属性访问
local_len = len
local_range = range

def efficient_function(n: int) -> list[int]:
    return [i**2 for i in local_range(local_len(some_list))]
```

## 最佳实践

1. **类型注解**: 使用类型注解提高代码可读性和可维护性
2. **文档字符串**: 为函数编写清晰的文档字符串
3. **单一职责**: 每个函数只负责一个明确的任务
4. **参数验证**: 在函数开始时验证参数的有效性
5. **异常处理**: 合理处理函数内部可能发生的异常
6. **性能考虑**: 避免在热路径中创建不必要的对象