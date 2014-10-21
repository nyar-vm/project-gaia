mod x64;
mod x86;

use serde::{Deserialize, Serialize};

// 定义 ImportTable 类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportTable {
    pub dll_name: String,
    pub functions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportTable {
    pub name: String,
    pub functions: Vec<String>,
}

// 定义编码后的指令
#[derive(Debug, Clone)]
pub struct EncodedInstruction {
    pub bytes: Vec<u8>,
    pub length: usize,
}
