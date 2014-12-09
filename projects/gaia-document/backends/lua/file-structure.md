# Lua 文件结构

本文档详细介绍 Gaia 框架中 Lua 后端的文件组织结构、字节码格式和模块系统。

## Lua 源文件结构

### 基本文件格式

Lua 源文件使用 UTF-8 编码，支持 Unicode 字符：

```lua
-- 文件头注释
-- 项目名称: MyLuaProject
-- 作者: Developer
-- 创建日期: 2024-01-01

-- 模块声明
local M = {}

-- 依赖导入
local json = require("json")
local utils = require("utils")

-- 常量定义
local VERSION = "1.0.0"
local MAX_SIZE = 1000

-- 局部函数
local function helper_function()
    return "helper"
end

-- 模块函数
function M.main_function()
    return helper_function()
end

-- 返回模块
return M
```

### 文件命名约定

```
模块文件:        module_name.lua
类文件:         ClassName.lua  
工具文件:       utils.lua, helpers.lua
配置文件:       config.lua, settings.lua
测试文件:       test_module_name.lua, spec_module_name.lua
主文件:         main.lua, init.lua
```

## Lua 字节码文件格式

### 二进制块结构

Lua 字节码文件（.luac）使用二进制格式：

```
+-------------+
| 头部 (Header) |
+-------------+
| 函数原型 (Function Prototype) |
+-------------+
```

### 文件头部结构

```c
typedef struct {
    char signature[4];      // "\x1bLua" (0x1B4C7561)
    char version;           // 版本号 (0x54 for Lua 5.4)
    char format;            // 格式版本 (0)
    char luac_data[6];      // 0x19930900 (Lua 创建日期)
    uchar cint_size;        // sizeof(int)
    uchar sizet_size;       // sizeof(size_t)
    uchar instruction_size; // sizeof(Instruction)
    uchar lua_integer_size; // sizeof(lua_Integer)
    uchar lua_number_size;  // sizeof(lua_Number)
    lua_Integer luac_int;   // 0x5678 (测试整数)
    lua_Number luac_num;    // 370.5 (测试浮点数)
} Header;
```

### 函数原型结构

```c
typedef struct {
    TString *source;           // 源文件名
    int linedefined;           // 定义起始行
    int lastlinedefined;       // 定义结束行
    uchar numparams;           // 参数数量
    uchar is_vararg;           // 是否可变参数
    uchar maxstacksize;        // 最大栈大小
    Instruction *code;         // 指令数组
    int sizecode;              // 指令数量
    TValue *k;                 // 常量表
    int sizek;                 // 常量数量
    struct Proto **p;          // 子函数原型
    int sizep;                 // 子函数数量
    int *lineinfo;             // 行号信息
    int abslineinfo;           // 绝对行号信息
    LocVar *locvars;           // 局部变量信息
    int sizelocvars;           // 局部变量数量
    Upvaldesc *upvalues;       // 上值信息
    int sizeupvalues;          // 上值数量
} Proto;
```

## 字节码指令格式

### 指令编码

Lua 使用 32 位指令，格式如下：

```
操作码(6位) | A(8位) | B(9位) | C(9位)
```

### 指令类型

```c
// 指令操作码枚举
typedef enum {
    OP_MOVE,      // A B     R(A) := R(B)
    OP_LOADK,     // A Bx    R(A) := K(Bx)
    OP_LOADKX,    // A       R(A) := K(extra arg)
    OP_LOADBOOL,  // A B C   R(A) := (Bool)B; if (C) pc++
    OP_LOADNIL,   // A B     R(A) := ... := R(B) := nil
    OP_GETUPVAL,  // A B     R(A) := UpValue[B]
    OP_GETTABUP,  // A B C   R(A) := UpValue[B][RK(C)]
    OP_GETTABLE,  // A B C   R(A) := R(B)[RK(C)]
    OP_SETTABUP,  // A B C   UpValue[A][RK(B)] := RK(C)
    OP_SETUPVAL,  // A B     UpValue[B] := R(A)
    OP_SETTABLE,  // A B C   R(A)[RK(B)] := RK(C)
    OP_NEWTABLE,  // A B C   R(A) := {} (size = B,C)
    OP_SELF,      // A B C   R(A+1) := R(B); R(A) := R(B)[RK(C)]
    OP_ADD,       // A B C   R(A) := RK(B) + RK(C)
    OP_SUB,       // A B C   R(A) := RK(B) - RK(C)
    OP_MUL,       // A B C   R(A) := RK(B) * RK(C)
    OP_DIV,       // A B C   R(A) := RK(B) / RK(C)
    OP_MOD,       // A B C   R(A) := RK(B) % RK(C)
    OP_POW,       // A B C   R(A) := RK(B) ^ RK(C)
    OP_UNM,       // A B     R(A) := -R(B)
    OP_NOT,       // A B     R(A) := not R(B)
    OP_LEN,       // A B     R(A) := length of R(B)
    OP_CONCAT,    // A B C   R(A) := R(B).. ... ..R(C)
    OP_JMP,       // A sBx   pc += sBx; if (A) close all upvalues >= A-1
    OP_EQ,        // A B C   if ((RK(B) == RK(C)) ~= A) then pc++
    OP_LT,        // A B C   if ((RK(B) <  RK(C)) ~= A) then pc++
    OP_LE,        // A B C   if ((RK(B) <= RK(C)) ~= A) then pc++
    OP_TEST,      // A C     if not (R(A) <=> C) then pc++
    OP_TESTSET,   // A B C   if (R(B) <=> C) then R(A) := R(B) else pc++
    OP_CALL,      // A B C   R(A), ... ,R(A+C-2) := R(A)(R(A+1), ... ,R(A+B-1))
    OP_TAILCALL,  // A B C   return R(A)(R(A+1), ... ,R(A+B-1))
    OP_RETURN,    // A B     return R(A), ... ,R(A+B-2)
    OP_FORLOOP,   // A sBx   R(A)+=R(A+2); if R(A) <?= R(A+1) then { pc+=sBx; R(A+3)=R(A) }
    OP_FORPREP,   // A sBx   R(A)-=R(A+2); pc+=sBx
    OP_TFORCALL,  // A C     R(A+3), ... ,R(A+2+C) := R(A)(R(A+1), R(A+2))
    OP_TFORLOOP,  // A sBx   if R(A+1) ~= nil then { R(A)=R(A+1); pc += sBx }
    OP_SETLIST,   // A B C   R(A)[(C-1)*FPF+i] := R(A+i), 1 <= i <= B
    OP_CLOSURE,   // A Bx    R(A) := closure(KPROTO[Bx], R(A), ... ,A+n)
    OP_VARARG,    // A B     R(A), R(A+1), ..., R(A+B-1) = vararg
    OP_EXTRAARG   //         extra (larger) argument for previous opcode
} OpCode;
```

## 常量表结构

### 常量类型

```c
// 常量类型枚举
typedef enum {
    LUA_TNIL = 0,
    LUA_TBOOLEAN = 1,
    LUA_TLIGHTUSERDATA = 2,
    LUA_TNUMBER = 3,
    LUA_TSTRING = 4,
    LUA_TTABLE = 5,
    LUA_TFUNCTION = 6,
    LUA_TUSERDATA = 7,
    LUA_TTHREAD = 8,
    LUA_TNUMFLT = 9,    // 浮点数
    LUA_TNUMINT = 10,   // 整数
    LUA_TSHRSTR = 11,   // 短字符串
    LUA_TLNGSTR = 12    // 长字符串
} LuaType;
```

### 常量编码

```c
// 常量值结构
typedef union {
    struct {
        Value value_;    // 值
        int tt_;         // 类型标签
    } i;
    double d;            // 双精度浮点数（用于对齐）
} TValue;

// 值结构
typedef union {
    GCObject *gc;     // 可回收对象
    void *p;          // 轻量级用户数据
    int b;            // 布尔值
    lua_CFunction f;  // C 函数
    lua_Integer i;    // 整数
    lua_Number n;     // 浮点数
} Value;
```

## 模块加载系统

### require 函数

```lua
-- 模块加载
local module = require("module_name")

-- 搜索路径
local path = package.path  -- Lua 模块搜索路径
local cpath = package.cpath -- C 模块搜索路径
```

### 模块搜索器

```lua
-- 自定义搜索器
table.insert(package.searchers, function(name)
    local path = "custom/" .. name .. ".lua"
    local file = io.open(path, "r")
    if file then
        file:close()
        return loadfile(path)
    end
    return nil
end)
```

### 模块缓存

```lua
-- 查看已加载模块
for name, module in pairs(package.loaded) do
    print(name, module)
end

-- 卸载模块
package.loaded["module_name"] = nil
```

## 包管理结构

### LuaRocks 包结构

```
my-package/
├── rockspec/
│   └── my-package-1.0.0-1.rockspec
├── src/
│   ├── main.lua
│   └── utils.lua
├── test/
│   ├── test_main.lua
│   └── test_utils.lua
├── doc/
│   ├── README.md
│   └── API.md
├── examples/
│   └── example1.lua
└── .luacheckrc
```

### Rockspec 文件格式

```lua
-- my-package-1.0.0-1.rockspec
package = "my-package"
version = "1.0.0-1"

source = {
    url = "git://github.com/user/my-package.git",
    tag = "v1.0.0"
}

description = {
    summary = "A sample Lua package",
    detailed = [[
        This package provides useful utilities for Lua applications.
    ]],
    homepage = "https://github.com/user/my-package",
    license = "MIT"
}

dependencies = {
    "lua >= 5.1, < 5.5",
    "luasocket >= 3.0"
}

build = {
    type = "builtin",
    modules = {
        ["my_package.main"] = "src/main.lua",
        ["my_package.utils"] = "src/utils.lua"
    }
}
```

## 项目目录结构

### 标准项目布局

```
lua-project/
├── src/                    # 源代码
│   ├── main.lua           # 主程序
│   ├── config.lua         # 配置
│   ├── core/              # 核心模块
│   │   ├── init.lua
│   │   ├── engine.lua
│   │   └── utils.lua
│   ├── modules/           # 功能模块
│   │   ├── database.lua
│   │   ├── network.lua
│   │   └── ui.lua
│   └── third_party/       # 第三方库
│       └── json.lua
├── test/                  # 测试代码
│   ├── unit/              # 单元测试
│   │   ├── test_core.lua
│   │   └── test_modules.lua
│   ├── integration/       # 集成测试
│   └── test_helper.lua    # 测试辅助函数
├── docs/                  # 文档
│   ├── README.md
│   ├── API.md
│   └── tutorial.md
├── examples/              # 示例代码
│   ├── basic_usage.lua
│   └── advanced_usage.lua
├── tools/                 # 工具脚本
│   ├── build.lua
│   └── deploy.lua
├── .luacheckrc            # Luacheck 配置
├── .busted                 # Busted 测试配置
└── run.lua                # 启动脚本
```

### 模块组织

```lua
-- src/core/init.lua
local core = {}

core.VERSION = "1.0.0"
core.AUTHOR = "Developer"

-- 子模块
local engine = require("core.engine")
core.Engine = engine

local utils = require("core.utils")
core.Utils = utils

return core
```

## 字节码优化

### 常量折叠

```lua
-- 源代码
local result = 10 + 20 * 3

-- 优化后的字节码（常量折叠）
LOADK 0 70    -- 直接加载计算结果 70
```

### 死代码消除

```lua
-- 源代码
local function test(x)
    if false then
        return 1  -- 死代码，会被消除
    end
    return x + 1
end

-- 优化后只保留有效代码
```

### 寄存器分配优化

```lua
-- 源代码
local a = 10
local b = a + 5
local c = b * 2

-- 优化的寄存器使用
LOADK 0 10    -- R0 = 10
ADD 0 0 5     -- R0 = R0 + 5 (重用寄存器)
MUL 0 0 2     -- R0 = R0 * 2 (继续重用)
```

## 调试信息格式

### 行号信息

```c
// 行号信息编码
typedef struct {
    int startline;    // 起始行号
    int endline;      // 结束行号
    int *lineinfo;    // 指令到行号的映射
    int sizelineinfo; // 行号信息数量
} LineInfo;
```

### 局部变量信息

```c
typedef struct {
    TString *varname;    // 变量名
    int startpc;         // 开始指令位置
    int endpc;           // 结束指令位置
} LocVar;
```

### 上值信息

```c
typedef struct {
    TString *name;       // 上值名称
    uchar instack;       // 是否在栈中
    uchar idx;           // 索引
} Upvaldesc;
```

## 性能分析文件

### 性能数据格式

```lua
-- profile_data.lua
return {
    functions = {
        {
            name = "main",
            calls = 1,
            total_time = 0.001,
            self_time = 0.0005,
            line_time = {
                [10] = 0.0001,
                [15] = 0.0002,
                [20] = 0.0002
            }
        },
        {
            name = "process_data",
            calls = 1000,
            total_time = 0.5,
            self_time = 0.3,
            line_time = {
                [5] = 0.1,
                [8] = 0.2,
                [12] = 0.2
            }
        }
    },
    memory = {
        allocations = 15000,
        deallocations = 12000,
        peak_usage = 2048,  -- KB
        current_usage = 1024  -- KB
    }
}
```

## 版本兼容性

### Lua 版本差异

```lua
-- Lua 5.1 vs 5.2+ 模块定义
-- Lua 5.1
module("mymodule", package.seeall)

-- Lua 5.2+
local M = {}
return M
```

### 兼容性检查

```lua
-- 版本检查
local lua_version = _VERSION  -- "Lua 5.1", "Lua 5.2", etc.

-- 特性检测
local has_bit32 = bit32 ~= nil        -- Lua 5.2
local has_5_3_features = utf8 ~= nil  -- Lua 5.3+
local has_5_4_features = <  close> ~= nil  -- Lua 5.4+
```