use crate::instructions::SassInstruction;
use serde::{Deserialize, Serialize};

/// SASS 程序 (Cubin 包装)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SassProgram {
    pub name: String,
    pub kernels: Vec<SassKernel>,
}

impl SassProgram {
    pub fn new(name: String) -> Self {
        Self { name, kernels: Vec::new() }
    }
}

/// SASS Kernel 定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SassKernel {
    pub name: String,
    pub instructions: Vec<SassInstruction>,
}
