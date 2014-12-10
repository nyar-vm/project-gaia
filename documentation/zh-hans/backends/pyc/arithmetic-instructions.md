# Python 算术指令

Gaia 框架为 Python 后端提供了完整的算术运算指令集，支持数值计算、数学运算和位运算。

## 基本算术运算

### 加法运算

```python
# Gaia 代码
a: int = 5
b: int = 3
c: int = a + b
```

编译为 Python 字节码：
```python
# 生成的 Python 代码
a = 5
b = 3
c = a + b
```

### 减法运算

```python
# Gaia 代码
result: int = a - b
```

### 乘法运算

```python
# Gaia 代码
product: int = a * b
```

### 除法运算

```python
# Gaia 代码
quotient: float = a / b
```

### 地板除

```python
# Gaia 代码
floor_div: int = a // b
```

### 取模运算

```python
# Gaia 代码
remainder: int = a % b
```

### 幂运算

```python
# Gaia 代码
power: int = a ** 2
```

## 算术指令集

### BINARY_ADD 指令

```python
# Gaia 代码
BINARY_ADD left right result
```

### BINARY_SUBTRACT 指令

```python
# Gaia 代码
BINARY_SUBTRACT left right result
```

### BINARY_MULTIPLY 指令

```python
# Gaia 代码
BINARY_MULTIPLY left right result
```

### BINARY_TRUE_DIVIDE 指令

```python
# Gaia 代码
BINARY_TRUE_DIVIDE left right result
```

### BINARY_FLOOR_DIVIDE 指令

```python
# Gaia 代码
BINARY_FLOOR_DIVIDE left right result
```

### BINARY_MODULO 指令

```python
# Gaia 代码
BINARY_MODULO left right result
```

### BINARY_POWER 指令

```python
# Gaia 代码
BINARY_POWER base exponent result
```

## 数值类型

### 整数类型

Python 3 中的整数是任意精度的：

```python
# Gaia 代码
big_int: int = 123456789012345678901234567890
```

### 浮点数类型

```python
# Gaia 代码
float_val: float = 3.14159265359
scientific: float = 1.23e-4
```

### 复数类型

```python
# Gaia 代码
complex_val: complex = 1 + 2j
real_part: float = complex_val.real
imag_part: float = complex_val.imag
```

### 类型转换

```python
# Gaia 代码
int_from_float: int = int(3.14)
float_from_int: float = float(42)
complex_from_nums: complex = complex(1, 2)
```

## 数学库函数

Python 提供了丰富的数学库：

```python
# Gaia 代码
import math

abs_val: int = abs(-5)
sqrt_val: float = math.sqrt(16)
sin_val: float = math.sin(math.pi/2)
cos_val: float = math.cos(0)
floor_val: int = math.floor(3.7)
ceil_val: int = math.ceil(3.2)
log_val: float = math.log(10)
exp_val: float = math.exp(1)
```

## 位运算

Python 支持位运算：

```python
# Gaia 代码
bit_and: int = a & b
bit_or: int = a | b
bit_xor: int = a ^ b
bit_not: int = ~a
left_shift: int = a << 2
right_shift: int = a >> 1
```

## 增强赋值运算

```python
# Gaia 代码
a += 1  # 等价于 a = a + 1
b -= 2  # 等价于 b = b - 2
c *= 3  # 等价于 c = c * 3
d /= 4  # 等价于 d = d / 4
```

## 数值精度

### 浮点数精度

```python
# Gaia 代码
from decimal import Decimal

precise: Decimal = Decimal('0.1') + Decimal('0.2')  # 精确计算
```

### 分数运算

```python
# Gaia 代码
from fractions import Fraction

frac1: Fraction = Fraction(1, 3)
frac2: Fraction = Fraction(1, 4)
result: Fraction = frac1 + frac2
```

## 优化建议

1. **使用内置函数**: Python 内置的数学函数经过优化
2. **避免不必要的类型转换**: 保持类型一致性
3. **使用适当的数据类型**: 根据需求选择 int、float 或 Decimal
4. **利用运算符重载**: Python 支持自定义类型的运算符重载

## 性能考虑

- 整数运算是任意精度的，但大整数运算会影响性能
- 浮点数运算遵循 IEEE 754 标准
- 位运算在 Python 中非常高效
- 复数运算会自动处理实部和虚部

## 特殊数值

```python
# Gaia 代码
infinity: float = float('inf')
negative_inf: float = float('-inf')
nan: float = float('nan')
pi: float = math.pi
e: float = math.e