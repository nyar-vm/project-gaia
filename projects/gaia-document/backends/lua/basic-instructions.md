# Lua 基础指令

Gaia 框架为 Lua 后端提供了一系列基础指令，用于变量操作、栈管理和基本程序结构。

## 变量声明和赋值

### 局部变量声明

```lua
-- Gaia 代码
local x: int = 10
local name: string = "Hello"
```

编译为 Lua 字节码：
```lua
-- 生成的 Lua 代码
local x = 10
local name = "Hello"
```

### 变量赋值

```lua
-- Gaia 代码
x = 20
name = "World"
```

## 栈操作指令

Lua 使用基于栈的虚拟机，Gaia 提供了相应的栈操作指令。

### 加载常量

```lua
-- Gaia 代码
LOAD_CONST 10
LOAD_CONST "string"
```

### 栈操作

```lua
-- Gaia 代码
PUSH 5
POP
DUP
SWAP
```

## 基本类型操作

### 数值类型

```lua
-- Gaia 代码
local a: int = 5
local b: float = 3.14
```

### 字符串类型

```lua
-- Gaia 代码
local str: string = "Hello Lua"
```

### 布尔类型

```lua
-- Gaia 代码
local flag: bool = true
```

### 表类型

```lua
-- Gaia 代码
local tbl: table = {}
```

## 控制结构

### 条件语句

```lua
-- Gaia 代码
if x > 0 then
    print("Positive")
end
```

### 循环结构

```lua
-- Gaia 代码
for i = 1, 10 do
    print(i)
end
```

## 函数定义和调用

### 函数定义

```lua
-- Gaia 代码
function add(a: int, b: int): int
    return a + b
end
```

### 函数调用

```lua
-- Gaia 代码
CALL add 5 3
```

## 优化技巧

1. **局部变量优先**: Lua 中局部变量访问速度更快
2. **避免全局变量**: 减少全局变量的使用
3. **表预分配**: 提前指定表的大小
4. **字符串缓存**: 重用字符串对象

## 性能考虑

- Lua 的局部变量比全局变量快约 30%
- 表操作是 Lua 的核心，需要特别注意性能
- 字符串是不可变的，频繁拼接会影响性能