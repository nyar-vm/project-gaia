//! 神经网络算子定义

use serde::{Deserialize, Serialize};

/// 矩阵乘法组配置
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatMulGroup {
    /// M 维度
    pub m: u32,
    /// N 维度
    pub n: u32,
    /// K 维度
    pub k: u32,
    /// 是否转置 A
    pub transpose_a: bool,
    /// 是否转置 B
    pub transpose_b: bool,
}

/// 神经网络节点指令
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NeuralNode {
    /// 基础矩阵乘法
    MatMul(MatMulGroup),
    /// 批量矩阵乘法
    BatchMatMul(MatMulGroup, u32),
    /// Flash Attention 算子
    FlashAttention {
        /// 序列长度
        seq_len: u32,
        /// 头数量
        num_heads: u32,
        /// 每个头的维度
        head_dim: u32,
        /// 是否有掩码
        has_mask: bool,
    },
    /// 多头注意力
    MultiHeadAttention {
        /// 序列长度
        seq_len: u32,
        /// 头数量
        num_heads: u32,
        /// 嵌入维度
        embed_dim: u32,
    },
    /// 卷积算子
    Convolution {
        /// 输入通道
        in_channels: u32,
        /// 输出通道
        out_channels: u32,
        /// 卷积核大小
        kernel_size: (u32, u32),
        /// 步长
        stride: (u32, u32),
        /// 填充
        padding: (u32, u32),
    },
}
