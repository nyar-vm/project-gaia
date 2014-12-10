# Lua 函数和方法指令

Gaia 框架为 Lua 后端提供了完整的函数定义、调用和方法操作指令集。

## 函数定义

### 基本函数定义

```lua
-- Gaia 代码
function add(a: int, b: int): int
    return a + b
end
```

编译为 Lua 字节码：
```lua
-- 生成的 Lua 代码
local function add(a, b)
    return a + b
end
```

### 局部函数定义

```lua
-- Gaia 代码
local function multiply(x: int, y: int): int
    return x * y
end
```

### 匿名函数

```lua
-- Gaia 代码
local square = function(x: int): int
    return x * x
end
```

## 函数调用指令

### 直接调用

```lua
-- Gaia 代码
CALL add 5 3 result
```

### 间接调用

```lua
-- Gaia 代码
CALL_VAR func_obj arg1 arg2 result
```

### 方法调用

```lua
-- Gaia 代码
CALL_METHOD obj "method_name" arg1 arg2 result
```

## 参数处理

### 固定参数

```lua
-- Gaia 代码
function greet(name: string, age: int): string
    return "Hello " .. name .. ", age: " .. age
end
```

### 可变参数

```lua
-- Gaia 代码
function sum(...): int
    local total: int = 0
    for i: int = 1, #arg do
        total = total + arg[i]
    end
    return total
end
```

### 默认参数

```lua
-- Gaia 代码
function greet(name: string, greeting: string = "Hello"): string
    return greeting .. " " .. name
end
```

## 返回值处理

### 单返回值

```lua
-- Gaia 代码
function get_value(): int
    return 42
end
```

### 多返回值

```lua
-- Gaia 代码
function get_coords(): (int, int)
    return 10, 20
end
```

### 返回指令

```lua
-- Gaia 代码
RETURN value
RETURN_MUL value1 value2
```

## 闭包和词法作用域

### 闭包定义

```lua
-- Gaia 代码
function make_counter(): function
    local count: int = 0
    return function(): int
        count = count + 1
        return count
    end
end
```

### 上值操作

```lua
-- Gaia 代码
CLOSURE proto_id upvalue1 upvalue2 result
GET_UPVALUE closure index result
SET_UPVALUE closure index value
```

## 尾调用优化

### 尾调用

```lua
-- Gaia 代码
function factorial(n: int, acc: int = 1): int
    if n <= 1 then
        return acc
    else
        return factorial(n - 1, n * acc)  -- 尾调用
    end
end
```

### 尾调用指令

```lua
-- Gaia 代码
TAIL_CALL func arg1 arg2
```

## 函数表和元方法

### 函数表

```lua
-- Gaia 代码
local operations: table = {
    add = function(a: int, b: int): int return a + b end,
    sub = function(a: int, b: int): int return a - b end,
    mul = function(a: int, b: int): int return a * b end,
    div = function(a: int, b: int): float return a / b end
}
```

### 元方法定义

```lua
-- Gaia 代码
local mt: table = {
    __add = function(a, b)
        return a.value + b.value
    end
}
```

## 性能优化

### 局部函数优化

```lua
-- 推荐：使用局部函数
local function fast_function()
    -- 函数体
end

-- 避免：全局函数
function slow_function()
    -- 函数体
end
```

### 内联优化

对于简单的函数，考虑手动内联：

```lua
-- Gaia 代码
-- 原始函数
function add(a: int, b: int): int
    return a + b
end

-- 内联版本（手动优化）
local result: int = a + b  -- 直接计算，避免函数调用
```

## 最佳实践

1. **使用局部函数**: 提高访问速度
2. **减少函数调用开销**: 合并简单操作
3. **合理使用尾调用**: 避免栈溢出
4. **避免过度嵌套**: 保持代码清晰
5. **类型注解**: 提高代码可读性和可维护性