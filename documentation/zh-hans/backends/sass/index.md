# SASS (NVIDIA GPU) 后端

SASS (Streaming Assembler) 是 NVIDIA GPU 的底层机器指令集。与 PTX（虚拟指令集）不同，SASS 是直接在硬件上运行的代码。

## 主要特性

- **极致性能**: 绕过编译器优化限制，手动控制流水线。
- **架构针对性**: 支持 Maxwell、Pascal、Volta、Turing、Ampere 及更高架构。
- **调度控制**: 精确控制指令调度和延迟隐藏。
- **寄存器优化**: 手动管理寄存器使用量，提高线程并行度（Occupancy）。

## 架构支持

- **Ampere (SM 80, 86)**
- **Turing (SM 75)**
- **Volta (SM 70)**
- **Pascal (SM 60, 61)**

## 为什么使用 SASS？

虽然 CUDA C++ 或 PTX 已经足够高效，但在某些特定的高性能库（如 cuBLAS、CUTLASS）中，为了压榨出最后一丝硬件性能，必须使用 SASS 进行手工优化。

## 注意事项

SASS 是高度平台相关的，不同代际的 NVIDIA GPU 指令集可能完全不兼容。
