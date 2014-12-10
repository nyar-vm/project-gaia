use serde::{Deserialize, Serialize};

/// 导入表条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportEntry {
    /// DLL 名称
    pub dll_name: String,
    /// 导入的函数列表
    pub functions: Vec<String>,
}

/// 导入表结构
///
/// 描述 PE 文件从外部 DLL 导入的函数信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportTable {
    /// 导入条目列表
    pub entries: Vec<ImportEntry>,
}

impl ImportTable {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }
}

impl Default for ImportTable {
    fn default() -> Self {
        Self::new()
    }
}

/// 导出表结构
///
/// 描述 PE 文件向外部导出的函数信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportTable {
    /// 模块名称
    pub name: String,
    /// 导出的函数列表
    pub functions: Vec<String>,
}

impl ExportTable {
    pub fn new() -> Self {
        Self { name: String::new(), functions: Vec::new() }
    }
}

impl Default for ExportTable {
    fn default() -> Self {
        Self::new()
    }
}
