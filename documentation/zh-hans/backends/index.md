# 后端支持

Gaia 框架为多种目标平台提供统一的编译接口，每个后端都针对特定的运行时环境进行了优化。

## 支持的后端

### [CLR (.NET) 后端](./clr/)

- 生成 .NET IL (Intermediate Language) 代码
- 支持 Unity 游戏开发
- 兼容 .NET Framework 和 .NET Core
- 支持 Windows、Linux、macOS 平台

### [PE (Windows) 后端](./pe/)

- 生成 Windows PE (Portable Executable) 文件
- 支持 EXE 和 DLL 格式
- 原生 Windows 应用程序支持
- 控制台和 GUI 应用程序

### [ELF (Linux) 后端](./elf/)

- 生成 Linux ELF (Executable and Linkable Format) 文件
- 支持静态和动态链接
- 原生 Linux 应用程序支持
- 系统调用接口

### [JVM (Java) 后端](./jvm/)

- 生成 Java 字节码
- 支持 JAR 文件格式
- 跨平台 Java 应用程序
- Android 应用开发支持

### [WASM (WebAssembly) 后端](./wasm/)

- 生成 WebAssembly 模块
- 浏览器和服务器端支持
- WASI (WebAssembly System Interface) 支持
- 高性能 Web 应用程序

### [LUA (Lua) 后端](./lua/)

- 生成 Lua 字节码 (luac)
- 支持 Lua 5.1/5.2/5.3/5.4 版本
- 轻量级脚本语言支持
- 游戏开发和嵌入式系统

### [PYC (Python) 后端](./pyc/)

- 生成 Python 字节码 (.pyc)
- 支持 Python 3.x 版本
- 跨平台 Python 应用程序
- 数据科学和机器学习集成

### [Mach-O (macOS/iOS) 后端](./macho/)

- 生成 Mach-O (Mach Object) 文件
- 支持 macOS 和 iOS 平台
- 支持动态库 (.dylib) 和可执行文件
- 原生 Apple 平台支持

### [x86 (Direct) 后端](./x86/)

- 直接生成 x86 (32-bit) 和 x86_64 (64-bit) 机器码
- 高性能原生指令执行
- 支持裸机和系统级开发
- 最小化运行时开销

### [SPIR-V (Vulkan) 后端](./spirv/)

- 生成 SPIR-V 二进制指令格式
- 支持 Vulkan 和 OpenGL 渲染管线
- 跨厂商 GPU 计算支持
- 高性能着色器编译

### [GCN (AMD GPU) 后端](./gcn/)

- 针对 AMD Graphics Core Next 架构优化
- 直接生成 GCN 汇编指令
- 高性能 GPU 核函数支持
- 针对 AMD 硬件的深度优化

### [SASS (NVIDIA GPU) 后端](./sass/)

- 针对 NVIDIA Streaming Assembler 架构
- 直接生成 NVIDIA GPU 机器指令
- 极致的 GPU 性能控制
- 支持最新的 CUDA 架构特性

### [MSL (Metal) 后端](./msl/)

- 生成 Metal Shading Language 代码
- 针对 Apple Metal 框架优化
- 高性能图形和计算着色器
- 深度集成 Apple 芯片特性

## 选择合适的后端

选择后端时需要考虑以下因素：

- **目标平台**: Windows (PE)、Linux (ELF)、Web (WASM)、跨平台 (.NET/JVM/Lua/Python)
- **性能要求**: 原生代码 (PE/ELF) vs 虚拟机 (.NET/JVM/WASM) vs 解释器 (Lua/Python)
- **生态系统**: 现有库和工具的兼容性
- **部署方式**: 独立可执行文件 vs 运行时依赖
- **应用场景**: 游戏开发 (Lua)、数据科学 (Python)、企业应用 (.NET/JVM)

## 通用特性

所有后端都支持：

- 类型安全的代码生成
- 错误处理和异常管理
- 内存管理优化
- 调试信息生成
- 性能分析支持