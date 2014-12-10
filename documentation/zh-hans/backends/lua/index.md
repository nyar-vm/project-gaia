# Lua 后端

Gaia 框架的 Lua 后端提供了生成标准 Lua 字节码的能力，兼容 Lua 5.1、5.2、5.3 和 5.4 版本。

## 主要特性

- **字节码生成**: 生成优化的 Lua 字节码 (.luac)
- **版本兼容**: 支持 Lua 5.1/5.2/5.3/5.4
- **轻量级**: 适合嵌入式系统和游戏开发
- **高性能**: 优化的字节码执行效率
- **跨平台**: 支持所有 Lua 支持的平台

## 应用场景

- **游戏开发**: 游戏脚本和逻辑编写
- **嵌入式系统**: 资源受限的环境
- **配置文件**: 复杂配置和 DSL
- **快速原型**: 快速开发和测试

## 指令集

Lua 后端支持完整的 Lua 虚拟机指令集，包括：
- [算术指令](./arithmetic-instructions.md)
- [基础指令](./basic-instructions.md)
- [控制流指令](./control-flow-instructions.md)
- [异常处理](./exception-instructions.md)
- [方法调用](./method-instructions.md)
- [对象操作](./object-instructions.md)

## 实现细节

1. **字节码 vs 源码**: 优先生成 Lua 字节码以提高加载速度。
2. **类型系统**: 将 Gaia 的强类型系统映射到 Lua 的动态类型。
3. **内存管理**: 利用 Lua 的自动垃圾回收机制。

## 相关资源

- [Lua 官方文档](https://www.lua.org/manual/)
- [Lua 字节码格式](https://www.lua.org/source/5.4/lobject.h.html)
- [Lua 5.4 源代码](https://www.lua.org/source/5.4/)