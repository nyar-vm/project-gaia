# Python 对象和类指令

Gaia 框架为 Python 后端提供了完整的面向对象编程指令集，支持类定义、继承、多态、数据类、抽象类等高级特性。

## 类和对象基础

### 基本类定义

```python
# Gaia 代码
class Person:
    def __init__(self, name: str, age: int):
        self.name = name
        self.age = age
    
    def greet(self) -> str:
        return f"Hello, I'm {self.name}"
    
    def get_age(self) -> int:
        return self.age
```

### 对象创建和使用

```python
# Gaia 代码
CREATE_OBJECT person Person "Alice" 25
call_method person "greet" greeting
call_method person "get_age" age
```

编译为 Python 代码：
```python
# 生成的 Python 代码
person = Person("Alice", 25)
greeting = person.greet()
age = person.get_age()
```

## 继承和多态

### 单继承

```python
# Gaia 代码
class Employee(Person):
    def __init__(self, name: str, age: int, employee_id: str):
        super().__init__(name, age)
        self.employee_id = employee_id
    
    def greet(self) -> str:
        return f"Hello, I'm {self.name}, employee {self.employee_id}"
    
    def work(self) -> str:
        return f"{self.name} is working"
```

### 多继承

```python
# Gaia 代码
class Manager(Employee, Leader):
    def __init__(self, name: str, age: int, employee_id: str, department: str):
        Employee.__init__(self, name, age, employee_id)
        Leader.__init__(self, department)
    
    def manage(self) -> str:
        return f"{self.name} is managing {self.department}"
```

### 方法重写

```python
# Gaia 代码
class Student(Person):
    def __init__(self, name: str, age: int, student_id: str):
        super().__init__(name, age)
        self.student_id = student_id
    
    def greet(self) -> str:
        return f"Hi, I'm {self.name}, student ID: {self.student_id}"
```

## 特殊方法和运算符重载

### 字符串表示

```python
# Gaia 代码
class Book:
    def __init__(self, title: str, author: str, pages: int):
        self.title = title
        self.author = author
        self.pages = pages
    
    def __str__(self) -> str:
        return f"{self.title} by {self.author}"
    
    def __repr__(self) -> str:
        return f"Book('{self.title}', '{self.author}', {self.pages})"
```

### 运算符重载

```python
# Gaia 代码
class Vector:
    def __init__(self, x: float, y: float):
        self.x = x
        self.y = y
    
    def __add__(self, other: 'Vector') -> 'Vector':
        return Vector(self.x + other.x, self.y + other.y)
    
    def __mul__(self, scalar: float) -> 'Vector':
        return Vector(self.x * scalar, self.y * scalar)
    
    def __len__(self) -> int:
        return 2
    
    def __getitem__(self, index: int) -> float:
        if index == 0:
            return self.x
        elif index == 1:
            return self.y
        raise IndexError("Vector index out of range")
```

### 比较运算符

```python
# Gaia 代码
class Product:
    def __init__(self, name: str, price: float):
        self.name = name
        self.price = price
    
    def __eq__(self, other: object) -> bool:
        if not isinstance(other, Product):
            return False
        return self.name == other.name and self.price == other.price
    
    def __lt__(self, other: 'Product') -> bool:
        return self.price < other.price
    
    def __hash__(self) -> int:
        return hash((self.name, self.price))
```

## 属性管理

### 属性装饰器

```python
# Gaia 代码
class Circle:
    def __init__(self, radius: float):
        self._radius = radius
    
    @property
    def radius(self) -> float:
        return self._radius
    
    @radius.setter
    def radius(self, value: float):
        if value < 0:
            raise ValueError("Radius cannot be negative")
        self._radius = value
    
    @property
    def area(self) -> float:
        return 3.14159 * self._radius ** 2
    
    @property
    def circumference(self) -> float:
        return 2 * 3.14159 * self._radius
```

### 属性描述符

```python
# Gaia 代码
class TemperatureDescriptor:
    def __init__(self):
        self._value = 0
    
    def __get__(self, obj, objtype=None):
        return self._value
    
    def __set__(self, obj, value):
        if value < -273.15:
            raise ValueError("Temperature cannot be below absolute zero")
        self._value = value

class Thermometer:
    celsius = TemperatureDescriptor()
```

## 数据类

### 基本数据类

```python
# Gaia 代码
from dataclasses import dataclass

@dataclass
class Point:
    x: float
    y: float
    
    def distance_to(self, other: 'Point') -> float:
        return ((self.x - other.x) ** 2 + (self.y - other.y) ** 2) ** 0.5
```

### 高级数据类

```python
# Gaia 代码
@dataclass(frozen=True)
class ImmutablePoint:
    x: float
    y: float
    
@dataclass
class Config:
    host: str = "localhost"
    port: int = 8080
    debug: bool = False
    
    def __post_init__(self):
        if self.port < 1024:
            raise ValueError("Port must be >= 1024")
```

## 抽象类和接口

### 抽象基类

```python
# Gaia 代码
from abc import ABC, abstractmethod

class Shape(ABC):
    @abstractmethod
    def area(self) -> float:
        pass
    
    @abstractmethod
    def perimeter(self) -> float:
        pass
    
    def describe(self) -> str:
        return f"Shape with area {self.area()} and perimeter {self.perimeter()}"

class Rectangle(Shape):
    def __init__(self, width: float, height: float):
        self.width = width
        self.height = height
    
    def area(self) -> float:
        return self.width * self.height
    
    def perimeter(self) -> float:
        return 2 * (self.width + self.height)
```

### 接口协议

```python
# Gaia 代码
from typing import Protocol

class Drawable(Protocol):
    def draw(self) -> str:
        ...

class Circle:
    def __init__(self, radius: float):
        self.radius = radius
    
    def draw(self) -> str:
        return f"Drawing circle with radius {self.radius}"
```

## 元类和动态类创建

### 元类

```python
# Gaia 代码
class SingletonMeta(type):
    _instances = {}
    
    def __call__(cls, *args, **kwargs):
        if cls not in cls._instances:
            cls._instances[cls] = super().__call__(*args, **kwargs)
        return cls._instances[cls]

class Database(metaclass=SingletonMeta):
    def __init__(self):
        self.connection = "Connected"
```

### 动态类创建

```python
# Gaia 代码
def create_model_class(name: str, fields: dict) -> type:
    return type(name, (), fields)

User = create_model_class("User", {
    "__init__": lambda self, name: setattr(self, "name", name),
    "__str__": lambda self: f"User({self.name})"
})
```

## 对象序列化

### Pickle 支持

```python
# Gaia 代码
import pickle

class SerializableObject:
    def __init__(self, data: any):
        self.data = data
        self.created_at = time.time()
    
    def __getstate__(self):
        state = self.__dict__.copy()
        # 自定义序列化逻辑
        return state
    
    def __setstate__(self, state):
        self.__dict__.update(state)
        # 自定义反序列化逻辑
```

### JSON 序列化

```python
# Gaia 代码
import json

class JSONMixin:
    def to_json(self) -> str:
        return json.dumps(self.__dict__)
    
    @classmethod
    def from_json(cls, json_str: str):
        data = json.loads(json_str)
        return cls(**data)
```

## 内存管理

### 弱引用

```python
# Gaia 代码
import weakref

class Cache:
    def __init__(self):
        self._cache = weakref.WeakValueDictionary()
    
    def get(self, key: str):
        return self._cache.get(key)
    
    def set(self, key: str, value: object):
        self._cache[key] = value
```

### 对象池

```python
# Gaia 代码
class ObjectPool:
    def __init__(self, factory_func, max_size: int = 100):
        self.factory_func = factory_func
        self.pool = []
        self.max_size = max_size
    
    def acquire(self):
        if self.pool:
            return self.pool.pop()
        return self.factory_func()
    
    def release(self, obj):
        if len(self.pool) < self.max_size:
            self.pool.append(obj)
```

## 性能优化

### 类属性缓存

```python
# Gaia 代码
class CachedProperty:
    def __init__(self, func):
        self.func = func
        self.name = func.__name__
    
    def __get__(self, obj, objtype=None):
        if obj is None:
            return self
        value = self.func(obj)
        setattr(obj, self.name, value)
        return value

class ExpensiveCalculation:
    @CachedProperty
    def expensive_result(self):
        # 昂贵的计算
        return sum(i**2 for i in range(1000000))
```

### 对象复用

```python
# Gaia 代码
class Flyweight:
    _instances = {}
    
    def __new__(cls, *args):
        key = args
        if key not in cls._instances:
            cls._instances[key] = super().__new__(cls)
        return cls._instances[key]
```

## 最佳实践

1. **封装性**: 使用私有属性和方法保护内部实现
2. **继承层次**: 保持继承层次简单，优先考虑组合
3. **类型注解**: 为类属性和方法参数添加类型注解
4. **文档字符串**: 为类和方法编写清晰的文档
5. **异常处理**: 在构造函数和方法中适当处理异常
6. **资源管理**: 使用上下文管理器管理资源
7. **性能考虑**: 避免在热路径中频繁创建对象