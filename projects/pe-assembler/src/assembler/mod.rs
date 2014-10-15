mod x64;
mod x86;

// 定义 ImportTable 类型
#[derive(Debug, Clone)]
pub struct ImportTable {
    pub dll_name: String,
    pub functions: Vec<String>,
}

// 定义编码后的指令
#[derive(Debug, Clone)]
pub struct EncodedInstruction {
    pub bytes: Vec<u8>,
    pub length: usize,
}
