# SPIR-V 后端

SPIR-V (Standard Portable Intermediate Representation) 是一种用于图形和计算的中间语言，由 Khronos Group 定义。

Gaia 的 SPIR-V 后端允许将通用逻辑编译为可在 GPU 上执行的着色器或计算核函数。

## 主要特性

- **跨平台 GPU 支持**: 兼容 Vulkan、OpenGL 和 OpenCL。
- **二进制输出**: 直接生成 `.spv` 二进制文件。
- **着色器支持**: 支持顶点着色器、像素着色器、计算着色器等。
- **强类型检查**: 利用 SPIR-V 的类型系统确保内核安全。
- **优化器集成**: 支持 SPIRV-Tools 优化流程。

## 目标 API

- **Vulkan**: 1.0, 1.1, 1.2, 1.3
- **OpenGL**: 4.6 (通过 `GL_ARB_gl_spirv`)

## 使用示例

```rust
use spirv_assembler::SpirvGenerator;
```

## 优势

通过 Gaia 框架编写 SPIR-V，您可以享受到比原生 GLSL/HLSL 更强大的元编程能力和类型安全性，同时保持对底层硬件的精确控制。
