# Lua 基本概念

本文档介绍 Gaia 框架中 Lua 后端的核心概念和设计理念。

## Lua 虚拟机基础

### 寄存器式虚拟机

Lua 使用基于寄存器的虚拟机架构，与基于栈的虚拟机不同：

```lua
-- 基于寄存器的指令格式
local a = 10
local b = 20  
local c = a + b
```

对应的 Gaia 指令：
```gaia
LOADK 0 10    -- 将常量 10 加载到寄存器 0
LOADK 1 20    -- 将常量 20 加载到寄存器 1
ADD 2 0 1     -- 寄存器 0 + 寄存器 1，结果存入寄存器 2
```

### 指令格式

Lua 指令是 32 位的，格式如下：

```
操作码(6位) | A(8位) | B(9位) | C(9位)
```

- **操作码**: 指令类型（如 ADD、LOADK）
- **A**: 目标寄存器
- **B, C**: 源操作数（寄存器或常量索引）

## 值类型系统

### 基本类型

Lua 有 8 种基本类型：

```lua
-- nil 类型
local nothing = nil

-- 布尔类型
local flag = true
local disabled = false

-- 数字类型
local integer = 42
local float = 3.14

-- 字符串类型
local text = "Hello, World!"

-- 函数类型
local function greet(name)
    return "Hello, " .. name
end

-- 表类型
local table = {key = "value"}

-- 线程类型
local thread = coroutine.create(function() end)

-- userdata 类型
local file = io.open("file.txt")
```

### 类型标签

每个 Lua 值都有一个类型标签：

```c
// Lua 内部表示
typedef struct {
    int type;           // 类型标签
    Value value;        // 实际值
} TValue;

// 类型常量
#define LUA_TNIL        0
#define LUA_TBOOLEAN    1
#define LUA_TLIGHTUSERDATA 2
#define LUA_TNUMBER     3
#define LUA_TSTRING     4
#define LUA_TTABLE      5
#define LUA_TFUNCTION   6
#define LUA_TUSERDATA   7
#define LUA_TTHREAD     8
```

## 表（Table）概念

### 表的结构

Lua 表是关联数组，可以同时作为数组和字典使用：

```lua
-- 数组用法
local array = {10, 20, 30, 40}
print(array[1])  -- 输出: 10

-- 字典用法
local dict = {name = "Lua", version = "5.4"}
print(dict.name)  -- 输出: Lua

-- 混合用法
local mixed = {100, 200, name = "test", active = true}
```

### 表的内部结构

表使用哈希表和数组两部分存储：

```c
// 表的内部结构
typedef struct Table {
    CommonHeader;
    lu_byte flags;      // 元方法标志
    lu_byte lsizenode;  // 哈希部分大小
    unsigned int sizearray;  // 数组部分大小
    TValue *array;      // 数组部分
    Node *node;         // 哈希部分
    Node *lastfree;     // 空闲节点指针
    struct Table *metatable;  // 元表
    GCObject *gclist;
} Table;
```

## 函数和闭包

### 函数原型

每个 Lua 函数都有一个原型（Prototype）：

```lua
-- Lua 函数
local function add(a, b)
    return a + b
end
```

对应的原型结构：
```c
typedef struct Proto {
    CommonHeader;
    TValue *k;              // 常量表
    Instruction *code;      // 指令数组
    struct Proto **p;       // 子函数原型
    int *lineinfo;          // 行号信息
    LocVar *locvars;        // 局部变量信息
    Upvaldesc *upvalues;    // 上值信息
    int sizeupvalues;       // 上值数量
    int sizek;              // 常量数量
    int sizecode;           // 指令数量
    int sizelineinfo;       // 行号数量
    int sizep;              // 子函数数量
    int sizelocvars;        // 局部变量数量
    int linedefined;        // 定义起始行
    int lastlinedefined;    // 定义结束行
    GCObject *gclist;
    TString *source;        // 源文件名
    char maxstacksize;      // 最大栈深度
} Proto;
```

### 闭包

闭包包含函数原型和上值：

```lua
-- 闭包示例
function make_counter()
    local count = 0
    return function()
        count = count + 1
        return count
    end
end

local counter1 = make_counter()
local counter2 = make_counter()
print(counter1())  -- 1
print(counter1())  -- 2
print(counter2())  -- 1
```

## 元表和元方法

### 元表概念

元表允许改变表的行为：

```lua
-- 创建元表
local mt = {
    __add = function(a, b)
        return a.value + b.value
    end,
    __tostring = function(t)
        return "Value: " .. t.value
    end
}

-- 设置元表
local t1 = {value = 10}
local t2 = {value = 20}
setmetatable(t1, mt)
setmetatable(t2, mt)

-- 使用元方法
local result = t1 + t2  -- 调用 __add
print(result)           -- 30
print(t1)               -- 调用 __tostring
```

### 常用元方法

```lua
-- 算术运算
__add(a, b)      -- 加法
__sub(a, b)      -- 减法
__mul(a, b)      -- 乘法
__div(a, b)      -- 除法
__mod(a, b)      -- 取模
__pow(a, b)      -- 幂运算
__unm(a)         -- 取负

-- 比较运算
__eq(a, b)       -- 等于
__lt(a, b)       -- 小于
__le(a, b)       -- 小于等于

-- 其他运算
__concat(a, b)   -- 连接
__len(a)         -- 长度
__tostring(a)    -- 字符串表示
__index(t, k)    -- 索引访问
__newindex(t, k, v) -- 索引赋值
__call(t, ...)   -- 函数调用
```

## 协程（Coroutine）

### 协程基础

Lua 协程是协作式多线程：

```lua
-- 创建协程
local co = coroutine.create(function(a, b)
    print("协程开始", a, b)
    local c = coroutine.yield(a + b)
    print("协程继续", c)
    return c * 2
end)

-- 运行协程
local status, result = coroutine.resume(co, 10, 20)
print(status, result)  -- true, 30

status, result = coroutine.resume(co, 5)
print(status, result)  -- true, 10
```

### 协程状态

```lua
-- 协程状态
coroutine.status(co)  -- "suspended", "running", "normal", "dead"

-- 包装器函数
function wrap_coroutine(func)
    local co = coroutine.create(func)
    return function(...)
        local status, result = coroutine.resume(co, ...)
        if not status then
            error(result)
        end
        return result
    end
end
```

## 垃圾回收

### 垃圾回收机制

Lua 使用增量标记清除垃圾回收器：

```lua
-- 控制垃圾回收
collector = {
    threshold = 100,    -- 垃圾回收阈值
    pause = 200,        -- 垃圾回收暂停时间
    stepmul = 200       -- 垃圾回收步长倍数
}

-- 手动触发垃圾回收
collectgarbage("collect")      -- 完整垃圾回收
collectgarbage("count")        -- 返回内存使用量
collectgarbage("step", 1024)   -- 执行一步垃圾回收
```

### 弱引用表

```lua
-- 弱引用键
local weak_keys = setmetatable({}, {__mode = "k"})

-- 弱引用值
local weak_values = setmetatable({}, {__mode = "v"})

-- 弱引用键和值
local weak_both = setmetatable({}, {__mode = "kv"})
```

## 模块系统

### 模块定义

```lua
-- 模块定义 (mymodule.lua)
local M = {}

function M.add(a, b)
    return a + b
end

function M.multiply(a, b)
    return a * b
end

return M
```

### 模块使用

```lua
-- 使用模块
local mymodule = require("mymodule")
print(mymodule.add(10, 20))
print(mymodule.multiply(5, 6))
```

## 错误处理

### 错误处理机制

```lua
-- 错误抛出
error("错误消息")
assert(condition, "错误消息")

-- 错误捕获
local success, result = pcall(function()
    return 10 / 0
end)

if success then
    print("结果:", result)
else
    print("错误:", result)
end
```

### xpcall 和调试

```lua
-- 带调试信息的错误处理
local function error_handler(err)
    print(debug.traceback("错误: " .. err, 2))
    return err
end

local success, result = xpcall(function()
    return 10 / 0
end, error_handler)
```

## 性能优化概念

### 局部变量优化

```lua
-- 推荐：使用局部变量
local function fast_function()
    local sin = math.sin  -- 缓存全局函数
    local cos = math.cos
    
    for i = 1, 1000000 do
        local x = sin(i) + cos(i)
    end
end

-- 避免：重复全局访问
local function slow_function()
    for i = 1, 1000000 do
        local x = math.sin(i) + math.cos(i)  -- 每次都要查找全局表
    end
end
```

### 表预分配

```lua
-- 推荐：预分配表大小
local function create_array(n)
    local t = {}
    for i = 1, n do
        t[i] = i * 2
    end
    return t
end

-- 更优：使用表构造函数
local function create_array_optimized(n)
    return {n = n, [1] = 2, [2] = 4, [3] = 6}  -- 如果大小固定
end
```

## 内存管理概念

### 内存分配策略

Lua 使用不同的内存分配策略：

```c
// 内存分配函数
typedef void * (*lua_Alloc) (void *ud,
                             void *ptr,
                             size_t osize,
                             size_t nsize);

// 默认分配器
void *l_alloc (void *ud, void *ptr, size_t osize, size_t nsize) {
    if (nsize == 0) {
        free(ptr);
        return NULL;
    } else
        return realloc(ptr, nsize);
}
```

### 内存池

```lua
-- 简单的对象池实现
local ObjectPool = {}
ObjectPool.__index = ObjectPool

function ObjectPool.new(create_func, reset_func)
    local pool = setmetatable({}, ObjectPool)
    pool.create_func = create_func
    pool.reset_func = reset_func or function(obj) end
    pool.available = {}
    pool.in_use = {}
    return pool
end

function ObjectPool:acquire(...)
    local obj
    if #self.available > 0 then
        obj = table.remove(self.available)
        self.reset_func(obj)
    else
        obj = self.create_func(...)
    end
    self.in_use[obj] = true
    return obj
end

function ObjectPool:release(obj)
    if self.in_use[obj] then
        self.in_use[obj] = nil
        table.insert(self.available, obj)
    end
end
```

## 调试概念

### 调试库

```lua
-- 调试信息
local function debug_info()
    print(debug.getinfo(1, "n").name)      -- 函数名
    print(debug.getinfo(1, "S").source)    -- 源文件
    print(debug.getinfo(1, "l").currentline) -- 当前行号
end

-- 局部变量检查
local function inspect_locals()
    local i = 1
    while true do
        local name, value = debug.getlocal(2, i)
        if not name then break end
        print(name, value)
        i = i + 1
    end
end
```

### 性能分析

```lua
-- 简单的性能分析器
local Profiler = {}

function Profiler.start()
    Profiler.start_time = os.clock()
    Profiler.memory_start = collectgarbage("count")
end

function Profiler.stop()
    local elapsed = os.clock() - Profiler.start_time
    local memory_used = collectgarbage("count") - Profiler.memory_start
    return elapsed, memory_used
end

-- 使用示例
Profiler.start()
-- 被测试的代码
local result = complex_calculation()
local time, memory = Profiler.stop()
print(string.format("Time: %.3fs, Memory: %.2fKB", time, memory))
```