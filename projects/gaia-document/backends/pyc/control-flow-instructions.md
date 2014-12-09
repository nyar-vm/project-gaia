# Python 控制流指令

Gaia 框架为 Python 后端提供了完整的控制流指令集，支持条件判断、循环和跳转操作。

## 条件语句

### if 语句

```python
# Gaia 代码
if x > 0:
    print("Positive")
elif x < 0:
    print("Negative")
else:
    print("Zero")
```

### 条件表达式

```python
# Gaia 代码
result: str = "positive" if x > 0 else "non-positive"
```

### 匹配语句 (Python 3.10+)

```python
# Gaia 代码
match value:
    case 1:
        print("One")
    case 2:
        print("Two")
    case _:
        print("Other")
```

## 比较指令

### 相等比较

```python
# Gaia 代码
EQ local_a local_b result
```

### 不等比较

```python
# Gaia 代码
NEQ local_a local_b result
```

### 小于比较

```python
# Gaia 代码
LT local_a local_b result
```

### 小于等于比较

```python
# Gaia 代码
LE local_a local_b result
```

### 大于比较

```python
# Gaia 代码
GT local_a local_b result
```

### 大于等于比较

```python
# Gaia 代码
GE local_a local_b result
```

## 循环结构

### for 循环

```python
# Gaia 代码
for i in range(10):
    print(i)

for item in items:
    process(item)
```

### while 循环

```python
# Gaia 代码
while x > 0:
    x -= 1
```

### 循环控制

```python
# Gaia 代码
for i in range(10):
    if i == 5:
        continue  # 跳过本次循环
    if i == 8:
        break     # 退出循环
    print(i)
```

## 跳转指令

### 无条件跳转

```python
# Gaia 代码
JMP label_name
```

### 条件跳转

```python
# Gaia 代码
JZ condition label_name    # 如果为零则跳转
JNZ condition label_name   # 如果不为零则跳转
```

### 比较跳转

```python
# Gaia 代码
JE local_a local_b label_name    # 相等则跳转
JNE local_a local_b label_name   # 不等则跳转
JL local_a local_b label_name    # 小于则跳转
JLE local_a local_b label_name   # 小于等于则跳转
JG local_a local_b label_name    # 大于则跳转
JGE local_a local_b label_name   # 大于等于则跳转
```

## 逻辑运算

### 逻辑与

```python
# Gaia 代码
AND condition1 condition2 result
```

### 逻辑或

```python
# Gaia 代码
OR condition1 condition2 result
```

### 逻辑非

```python
# Gaia 代码
NOT condition result
```

### 短路求值

```python
# Gaia 代码
# Python 的 and 和 or 运算符支持短路求值
result1 = condition1 and condition2
result2 = condition1 or condition2
```

## 列表推导式

### 基本列表推导式

```python
# Gaia 代码
squares: list[int] = [x**2 for x in range(10)]
```

### 条件列表推导式

```python
# Gaia 代码
evens: list[int] = [x for x in range(10) if x % 2 == 0]
```

### 嵌套列表推导式

```python
# Gaia 代码
matrix: list[list[int]] = [[i+j for j in range(3)] for i in range(3)]
```

## 生成器表达式

### 基本生成器

```python
# Gaia 代码
gen = (x**2 for x in range(10))
```

### 条件生成器

```python
# Gaia 代码
even_gen = (x for x in range(10) if x % 2 == 0)
```

## 循环优化

### 循环展开

```python
# Gaia 代码
# 手动展开循环
for i in range(0, 8, 2):
    process(i)
    process(i+1)
```

### 循环不变量外提

```python
# Gaia 代码
constant_value = expensive_calculation()
for i in range(n):
    # 使用 constant_value，避免重复计算
    process(i, constant_value)
```

## 异常处理控制流

### try-except 结构

```python
# Gaia 代码
try:
    risky_operation()
except ValueError as e:
    handle_value_error(e)
except Exception as e:
    handle_generic_error(e)
finally:
    cleanup()
```

### else 子句

```python
# Gaia 代码
try:
    result = risky_operation()
except Exception as e:
    handle_error(e)
else:
    # 如果没有异常发生
    process_result(result)
```

## 上下文管理器

### with 语句

```python
# Gaia 代码
with open('file.txt', 'r') as f:
    content = f.read()
```

### 自定义上下文管理器

```python
# Gaia 代码
from contextlib import contextmanager

@contextmanager
def managed_resource():
    resource = acquire_resource()
    try:
        yield resource
    finally:
        release_resource(resource)
```

## 性能考虑

1. **循环优化**: 尽量减少循环内部的计算量
2. **条件预测**: 尽量让最可能的分支放在前面
3. **短路求值**: 利用 and/or 的短路特性
4. **列表推导式**: 通常比等效的 for 循环快
5. **生成器**: 对于大数据集使用生成器节省内存

## 最佳实践

- 使用 early return 减少嵌套层级
- 避免过深的条件嵌套
- 合理使用循环，考虑算法复杂度
- 使用列表推导式和生成器表达式提高可读性和性能
- 利用 Python 的上下文管理器进行资源管理