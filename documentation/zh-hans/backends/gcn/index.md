# GCN (AMD GPU) 后端

GCN (Graphics Core Next) 是 AMD 开发的一系列 GPU 微架构。Gaia 的 GCN 后端专门针对 AMD 硬件提供底层指令生成支持。

## 主要特性

- **硬件级控制**: 直接操作 GCN 寄存器和标量/矢量单元。
- **ISA 覆盖**: 支持主流的 GCN 指令集（如 GFX6 到 GFX9）。
- **核函数生成**: 生成适用于 ROCm 或 OpenCL 环境的内核代码。
- **内存模型优化**: 针对 AMD GPU 的高速缓存和本地内存（LDS）进行优化。

## 支持的架构

- **Vega (GFX9)**
- **Polaris (GFX8)**
- **GCN 1.0/2.0/3.0**

## 应用场景

- **高性能计算 (HPC)**: 针对 AMD 显卡集群的大规模并行计算。
- **挖矿算法**: 极致优化的加密哈希运算。
- **自定义驱动开发**: 需要直接与硬件交互的系统。

## 相关资源

- [AMD GCN ISA 文档](https://developer.amd.com/resources/developer-guides-manuals/)
