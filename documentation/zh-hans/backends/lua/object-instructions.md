# Lua 对象操作指令

Gaia 框架为 Lua 后端提供了完整的对象操作指令集，用于表操作、元表和面向对象编程。

## 表操作

### 表创建

```lua
-- Gaia 代码
local tbl: table = {}
local arr: table = {1, 2, 3, 4, 5}
```

编译为 Lua 字节码：
```lua
-- 生成的 Lua 代码
local tbl = {}
local arr = {1, 2, 3, 4, 5}
```

### 表字段访问

```lua
-- Gaia 代码
local value = tbl.key
local item = arr[1]
```

### 表字段赋值

```lua
-- Gaia 代码
tbl.name = "Lua"
tbl[1] = "first"
```

## 表操作指令

### 创建表

```lua
-- Gaia 代码
NEW_TABLE result
```

### 获取表字段

```lua
-- Gaia 代码
GET_TABLE table key result
```

### 设置表字段

```lua
-- Gaia 代码
SET_TABLE table key value
```

### 表长度

```lua
-- Gaia 代码
GET_LEN table result
```

## 元表操作

### 设置元表

```lua
-- Gaia 代码
local mt: table = {
    __add = function(a, b) return a.value + b.value end,
    __tostring = function(obj) return "Object: " .. obj.name end
}
setmetatable(obj, mt)
```

### 获取元表

```lua
-- Gaia 代码
local mt: table = getmetatable(obj)
```

### 元方法定义

```lua
-- Gaia 代码
local mt: table = {
    __index = function(table, key)
        return rawget(table, "_" .. key)
    end,
    __newindex = function(table, key, value)
        rawset(table, "_" .. key, value)
    end
}
```

## 面向对象编程

### 类定义

```lua
-- Gaia 代码
local Person = {}
Person.__index = Person

function Person:new(name: string, age: int)
    local obj = {}
    setmetatable(obj, Person)
    obj.name = name
    obj.age = age
    return obj
end

function Person:greet(): string
    return "Hello, I'm " .. self.name
end
```

### 继承实现

```lua
-- Gaia 代码
local Student = {}
Student.__index = Student
setmetatable(Student, Person)

function Student:new(name: string, age: int, grade: int)
    local obj = Person:new(name, age)
    setmetatable(obj, Student)
    obj.grade = grade
    return obj
end
```

## 对象操作指令

### 创建对象

```lua
-- Gaia 代码
NEW_OBJECT class_name result
```

### 方法调用

```lua
-- Gaia 代码
CALL_METHOD obj "method_name" arg1 arg2 result
```

### 字段访问

```lua
-- Gaia 代码
GET_FIELD obj "field_name" result
```

### 字段赋值

```lua
-- Gaia 代码
SET_FIELD obj "field_name" value
```

## 高级表操作

### 表遍历

```lua
-- Gaia 代码
for key, value in pairs(tbl) do
    print(key, value)
end

for index, value in ipairs(arr) do
    print(index, value)
end
```

### 表连接

```lua
-- Gaia 代码
local combined: table = {}
for i, v in ipairs(arr1) do table.insert(combined, v) end
for i, v in ipairs(arr2) do table.insert(combined, v) end
```

### 表排序

```lua
-- Gaia 代码
local sorted: table = {table.unpack(arr)}
table.sort(sorted, function(a, b) return a < b end)
```

## 内存管理

### 弱引用表

```lua
-- Gaia 代码
local weak_table: table = {}
setmetatable(weak_table, {__mode = "kv"})  -- 键和值都是弱引用
```

### 对象池

```lua
-- Gaia 代码
local ObjectPool = {
    pool = {},
    
    acquire = function(self)
        if #self.pool > 0 then
            return table.remove(self.pool)
        else
            return self:create_new()
        end
    end,
    
    release = function(self, obj)
        self:reset(obj)
        table.insert(self.pool, obj)
    end
}
```

## 性能优化

### 表预分配

```lua
-- Gaia 代码
local tbl: table = {}
tbl.size = 100  -- 预分配提示
```

### 避免表创建

```lua
-- 推荐：重用表
local function process()
    local temp = {}
    -- 使用 temp
end

-- 避免：频繁创建
local function process()
    local temp = {a = 1, b = 2}  -- 每次调用都创建新表
    -- 使用 temp
end
```

### 使用局部变量缓存

```lua
-- Gaia 代码
local t = some_table
local v1 = t.field1
local v2 = t.field2
-- 使用 v1, v2 而不是重复访问 t.field1, t.field2
```

## 最佳实践

1. **使用局部变量**: 避免重复的全局表访问
2. **预分配表**: 当知道大小时预分配表
3. **重用对象**: 使用对象池模式
4. **合理使用元表**: 不要过度使用元方法
5. **类型注解**: 使用类型注解提高代码可维护性