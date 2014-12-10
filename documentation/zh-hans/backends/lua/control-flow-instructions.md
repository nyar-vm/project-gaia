# Lua 控制流指令

Gaia 框架为 Lua 后端提供了完整的控制流指令集，支持条件判断、循环和跳转操作。

## 条件语句

### if 语句

```lua
-- Gaia 代码
if x > 0 then
    print("Positive")
elseif x < 0 then
    print("Negative")
else
    print("Zero")
end
```

### 条件表达式

```lua
-- Gaia 代码
local result: string = x > 0 ? "positive" : "non-positive"
```

## 比较指令

### 相等比较

```lua
-- Gaia 代码
EQ local_a local_b result
```

### 不等比较

```lua
-- Gaia 代码
NEQ local_a local_b result
```

### 小于比较

```lua
-- Gaia 代码
LT local_a local_b result
```

### 小于等于比较

```lua
-- Gaia 代码
LE local_a local_b result
```

### 大于比较

```lua
-- Gaia 代码
GT local_a local_b result
```

### 大于等于比较

```lua
-- Gaia 代码
GE local_a local_b result
```

## 循环结构

### for 循环

```lua
-- Gaia 代码
for i: int = 1, 10, 1 do
    print(i)
end
```

### while 循环

```lua
-- Gaia 代码
while x > 0 do
    x = x - 1
end
```

### repeat-until 循环

```lua
-- Gaia 代码
repeat
    x = x + 1
until x >= 10
```

## 跳转指令

### 无条件跳转

```lua
-- Gaia 代码
JMP label_name
```

### 条件跳转

```lua
-- Gaia 代码
JZ condition label_name    -- 如果为零则跳转
JNZ condition label_name   -- 如果不为零则跳转
```

### 比较跳转

```lua
-- Gaia 代码
JE local_a local_b label_name    -- 相等则跳转
JNE local_a local_b label_name   -- 不等则跳转
JL local_a local_b label_name    -- 小于则跳转
JLE local_a local_b label_name   -- 小于等于则跳转
JG local_a local_b label_name    -- 大于则跳转
JGE local_a local_b label_name   -- 大于等于则跳转
```

## 逻辑运算

### 逻辑与

```lua
-- Gaia 代码
AND condition1 condition2 result
```

### 逻辑或

```lua
-- Gaia 代码
OR condition1 condition2 result
```

### 逻辑非

```lua
-- Gaia 代码
NOT condition result
```

## 循环优化

### 循环展开

```lua
-- Gaia 代码
for i: int = 1, 4 do
    -- 手动展开循环体
    process(i)
end
```

### 循环不变量外提

```lua
-- Gaia 代码
local constant_value: int = expensive_calculation()
for i: int = 1, n do
    -- 使用 constant_value，避免重复计算
    process(i, constant_value)
end
```

## 性能考虑

1. **条件预测**: 尽量让最可能的分支放在前面
2. **循环优化**: 减少循环内部的计算量
3. **跳转最小化**: 减少不必要的跳转指令
4. **局部变量**: 使用局部变量存储中间结果

## 最佳实践

- 使用 early return 减少嵌套层级
- 避免过深的条件嵌套
- 合理使用循环，考虑算法复杂度
- 使用表驱动的方法替代复杂的条件分支