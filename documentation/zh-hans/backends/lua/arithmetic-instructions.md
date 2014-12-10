# Lua 算术指令

Gaia 框架为 Lua 后端提供了完整的算术运算指令集，支持数值计算和数学运算。

## 基本算术运算

### 加法运算

```lua
-- Gaia 代码
local a: int = 5
local b: int = 3
local c: int = a + b
```

编译为 Lua 字节码：
```lua
-- 生成的 Lua 代码
local a = 5
local b = 3
local c = a + b
```

### 减法运算

```lua
-- Gaia 代码
local result: int = a - b
```

### 乘法运算

```lua
-- Gaia 代码
local product: int = a * b
```

### 除法运算

```lua
-- Gaia 代码
local quotient: float = a / b
```

### 取模运算

```lua
-- Gaia 代码
local remainder: int = a % b
```

### 幂运算

```lua
-- Gaia 代码
local power: int = a ^ 2
```

## 算术指令集

### ADD 指令

```lua
-- Gaia 代码
ADD local_a local_b result
```

### SUB 指令

```lua
-- Gaia 代码
SUB local_a local_b result
```

### MUL 指令

```lua
-- Gaia 代码
MUL local_a local_b result
```

### DIV 指令

```lua
-- Gaia 代码
DIV local_a local_b result
```

### MOD 指令

```lua
-- Gaia 代码
MOD local_a local_b result
```

### POW 指令

```lua
-- Gaia 代码
POW local_a 2 result
```

## 数值类型

### 整数类型

Lua 5.3+ 支持真正的整数类型：

```lua
-- Gaia 代码
local int_val: int = 42
```

### 浮点数类型

```lua
-- Gaia 代码
local float_val: float = 3.14159
```

### 类型转换

```lua
-- Gaia 代码
local int_from_float: int = tointeger(float_val)
local float_from_int: float = tonumber(int_val)
```

## 数学库函数

Lua 提供了丰富的数学库函数：

```lua
-- Gaia 代码
local abs_val: int = math.abs(-5)
local sqrt_val: float = math.sqrt(16)
local sin_val: float = math.sin(math.pi/2)
local cos_val: float = math.cos(0)
local floor_val: int = math.floor(3.7)
local ceil_val: int = math.ceil(3.2)
```

## 位运算

Lua 5.3+ 支持位运算：

```lua
-- Gaia 代码
local bit_and: int = a & b
local bit_or: int = a | b
local bit_xor: int = a ~ b
local bit_not: int = ~a
local left_shift: int = a << 2
local right_shift: int = a >> 1
```

## 优化建议

1. **使用局部变量**: 局部变量比全局变量性能更好
2. **避免不必要的类型转换**: 保持类型一致性
3. **利用数学库**: 使用 Lua 内置的数学函数
4. **整数运算优先**: 整数运算比浮点数运算更快

## 性能考虑

- 整数运算通常比浮点数运算快
- 位运算在 Lua 5.3+ 中非常高效
- 避免在循环中进行重复计算
- 考虑使用查表法替代复杂计算