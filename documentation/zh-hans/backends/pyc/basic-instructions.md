# Python 基础指令

Gaia 框架为 Python 后端提供了一系列基础指令，用于变量操作、栈管理和基本程序结构。

## 变量声明和赋值

### 变量声明

```python
# Gaia 代码
x: int = 10
name: str = "Hello"
flag: bool = True
```

编译为 Python 字节码：
```python
# 生成的 Python 代码
x = 10
name = "Hello"
flag = True
```

### 动态类型

Python 支持动态类型，但 Gaia 提供类型注解：

```python
# Gaia 代码
value = 42  # 推断为 int 类型
text = "Python"  # 推断为 str 类型
```

## 栈操作指令

### 加载常量

```python
# Gaia 代码
LOAD_CONST 10
LOAD_CONST "string"
LOAD_CONST True
```

### 栈操作

```python
# Gaia 代码
PUSH 5
POP
DUP
ROT_TWO
ROT_THREE
```

## 基本类型操作

### 数值类型

```python
# Gaia 代码
a: int = 5
b: float = 3.14
c: complex = 1 + 2j
```

### 字符串类型

```python
# Gaia 代码
str_val: str = "Hello Python"
multiline: str = """Multi
line
string"""
```

### 布尔类型

```python
# Gaia 代码
is_valid: bool = True
is_empty: bool = False
```

### 列表类型

```python
# Gaia 代码
numbers: list[int] = [1, 2, 3, 4, 5]
mixed: list = [1, "hello", True]
```

### 字典类型

```python
# Gaia 代码
person: dict[str, any] = {"name": "Alice", "age": 30}
scores: dict[str, int] = {"math": 95, "english": 88}
```

## 控制结构

### 条件语句

```python
# Gaia 代码
if x > 0:
    print("Positive")
elif x < 0:
    print("Negative")
else:
    print("Zero")
```

### 循环结构

```python
# Gaia 代码
for i in range(10):
    print(i)

while x > 0:
    x -= 1
```

## 函数定义和调用

### 函数定义

```python
# Gaia 代码
def add(a: int, b: int) -> int:
    return a + b
```

### 函数调用

```python
# Gaia 代码
result: int = add(5, 3)
```

## 特殊指令

### 导入模块

```python
# Gaia 代码
IMPORT math
IMPORT_FROM os import path
```

### 异常处理

```python
# Gaia 代码
TRY
    risky_operation()
EXCEPT Exception as e
    handle_error(e)
END_TRY
```

## 类型注解和检查

### 类型注解

```python
# Gaia 代码
def process_data(data: list[int]) -> dict[str, int]:
    result: dict[str, int] = {}
    # 处理逻辑
    return result
```

### 类型检查

```python
# Gaia 代码
ISINSTANCE obj int
TYPE obj result
```

## 优化技巧

1. **使用局部变量**: Python 中局部变量访问速度更快
2. **避免全局变量**: 减少全局变量的使用
3. **列表推导式**: 使用列表推导式替代循环
4. **生成器表达式**: 对于大数据使用生成器
5. **内置函数**: 优先使用 Python 内置函数

## 性能考虑

- 局部变量比全局变量快约 15-30%
- 列表推导式通常比等效的 for 循环快
- 字符串拼接使用 join 比 + 操作符更高效
- 避免在循环中进行重复的属性访问

## 字节码优化

Python 会自动进行一些优化：

```python
# Gaia 代码
# 常量折叠
result = 2 + 3 * 4  # 编译时常量计算

# 死代码消除
if False:
    unreachable_code()
```