# Lua 异常处理指令

Gaia 框架为 Lua 后端提供了完整的异常处理机制，支持错误捕获、异常传播和资源清理。

## 错误处理基础

### pcall 函数

Lua 使用 `pcall` (protected call) 进行错误捕获：

```lua
-- Gaia 代码
local success, result = pcall(function()
    return risky_operation()
end)

if success then
    print("Success: " .. result)
else
    print("Error: " .. result)
end
```

### xpcall 函数

`xpcall` 允许指定错误处理函数：

```lua
-- Gaia 代码
local function error_handler(err)
    print("Error caught: " .. err)
    return "handled"
end

local success, result = xpcall(function()
    error("Something went wrong")
end, error_handler)
```

## 异常指令集

### TRY 指令

```lua
-- Gaia 代码
TRY
    -- 可能抛出异常的代码
    risky_operation()
CATCH exception
    -- 异常处理代码
    handle_error(exception)
END_TRY
```

### THROW 指令

```lua
-- Gaia 代码
THROW "Custom error message"
THROW error_object
```

### ASSERT 指令

```lua
-- Gaia 代码
ASSERT condition "Assertion failed"
```

## 错误对象

### 创建错误对象

```lua
-- Gaia 代码
local error_obj = {
    message = "Custom error",
    code = 500,
    stack_trace = debug.traceback()
}
```

### 错误类型

```lua
-- Gaia 代码
local ErrorTypes = {
    VALIDATION_ERROR = "ValidationError",
    NETWORK_ERROR = "NetworkError",
    DATABASE_ERROR = "DatabaseError"
}
```

## 异常处理模式

### 包装模式

```lua
-- Gaia 代码
local function safe_divide(a: number, b: number): (boolean, number|string)
    if b == 0 then
        return false, "Division by zero"
    end
    return true, a / b
end
```

### 错误传播

```lua
-- Gaia 代码
local function process_data(data: table): (boolean, table|string)
    local success, validated = validate_data(data)
    if not success then
        return false, "Validation failed: " .. validated
    end
    
    local success, processed = transform_data(validated)
    if not success then
        return false, "Processing failed: " .. processed
    end
    
    return true, processed
end
```

## 资源管理

### 使用 finally 模式

```lua
-- Gaia 代码
local function with_resource(resource: any): (boolean, any)
    local success, result = pcall(function()
        -- 使用资源的操作
        return process_resource(resource)
    end)
    
    -- 清理资源（finally 块）
    cleanup_resource(resource)
    
    return success, result
end
```

### RAII 模式

```lua
-- Gaia 代码
local ResourceManager = {}
Resource.__index = Resource

function ResourceManager:new(resource: any)
    local obj = {resource = resource}
    setmetatable(obj, ResourceManager)
    return obj
end

function ResourceManager:__gc()
    if self.resource then
        self.resource:cleanup()
    end
end
```

## 调试支持

### 堆栈跟踪

```lua
-- Gaia 代码
local function get_stack_trace(): string
    return debug.traceback()
end
```

### 错误定位

```lua
-- Gaia 代码
local function detailed_error(message: string): string
    local info = debug.getinfo(2)
    return string.format("Error at %s:%d: %s", 
        info.short_src, info.currentline, message)
end
```

## 性能优化

### 错误处理性能

```lua
-- Gaia 代码
-- 推荐：预先检查条件
if divisor ~= 0 then
    result = dividend / divisor
end

-- 避免：依赖异常处理
local success, result = pcall(function()
    return dividend / divisor
end)
```

### 异常安全

```lua
-- Gaia 代码
local function atomic_operation(): (boolean, any)
    local backup = get_state()
    
    local success, result = pcall(function()
        return perform_operation()
    end)
    
    if not success then
        restore_state(backup)
        return false, result
    end
    
    return true, result
end
```

## 最佳实践

### 错误处理策略

1. **预防优于治疗**: 预先检查条件，避免异常
2. **具体错误信息**: 提供详细的错误描述
3. **错误分类**: 使用错误类型进行分类处理
4. **资源清理**: 确保异常发生时资源得到清理

### 异常使用原则

1. **异常情况**: 只在真正异常的情况下使用异常
2. **性能考虑**: 异常处理有性能开销，避免在热路径中过度使用
3. **错误传播**: 合理传播错误，不要吞掉异常
4. **日志记录**: 记录异常信息，便于调试

### 代码示例

```lua
-- Gaia 代码
local function robust_function(input: any): (boolean, any)
    -- 参数验证
    if not input then
        return false, "Input is required"
    end
    
    -- 使用 pcall 保护关键操作
    local success, result = pcall(function()
        return process_input(input)
    end)
    
    if not success then
        -- 记录错误并返回友好信息
        log_error("Process failed", result)
        return false, "Processing failed, please try again"
    end
    
    return true, result
end
```