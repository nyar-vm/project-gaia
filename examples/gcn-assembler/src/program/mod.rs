use crate::instructions::GcnInstruction;
use serde::{Deserialize, Serialize};

/// GCN 程序 (HSACO/ELF 包装)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GcnProgram {
    pub name: String,
    pub kernels: Vec<GcnKernel>,
}

impl GcnProgram {
    pub fn new(name: String) -> Self {
        Self { name, kernels: Vec::new() }
    }
}

/// GCN Kernel 定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GcnKernel {
    pub name: String,
    pub instructions: Vec<GcnInstruction>,
    pub args: Vec<GcnKernelArg>,
}

/// GCN Kernel 参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GcnKernelArg {
    pub name: String,
    pub size: usize,
    pub offset: usize,
}
