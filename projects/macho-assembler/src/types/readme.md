# Mach-O 类型定义

本模块包含了 Mach-O 文件格式的所有核心类型定义。

## 主要类型

### MachoHeader
Mach-O 文件头，包含文件的基本信息：
- 魔数（magic number）
- CPU 类型和子类型
- 文件类型
- 加载命令信息

### LoadCommand
加载命令基础结构，用于描述如何加载和链接文件的各个部分。

### SegmentCommand64
64位段命令，定义了内存段的布局和属性。

### Section64
64位节结构，定义了段内各个节的详细信息。

### MachoProgram
完整的 Mach-O 程序结构，包含所有必要的组件。

## 支持的架构

- x86_64: Intel/AMD 64位处理器
- ARM64: Apple Silicon 处理器
- ARM64e: 带指针认证的 ARM64
- i386: 32位 Intel 处理器
- ARM: 32位 ARM 处理器

## 支持的文件类型

- Object: 目标文件 (.o)
- Execute: 可执行文件
- Dylib: 动态库 (.dylib)
- Bundle: 动态加载包
- 其他各种 Mach-O 文件类型