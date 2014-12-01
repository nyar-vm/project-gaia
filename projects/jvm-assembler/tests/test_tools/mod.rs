use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// JASM 文件期望结构体 - 用于定义测试期望
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JasmExpected {
    /// 期望的类名
    pub class_name: String,
    /// 期望的类修饰符
    pub class_modifiers: Vec<String>,
    /// 期望的方法数量
    pub method_count: usize,
    /// 期望的字段数量
    pub field_count: usize,
    /// 期望包含的特定方法（方法名和描述符）
    pub expected_methods: Vec<String>,
    /// 期望包含的特定字段
    pub expected_fields: Vec<String>,
    /// 期望包含的语法糖特征
    pub syntactic_sugar_features: Vec<(String, bool)>, // (特征, 是否应该存在)
    /// 是否检查 token 类型
    pub check_tokens: bool,
    /// 期望的源文件名
    pub source_file: Option<String>,
    /// 文件路径（用于调试）
    #[serde(skip)]
    pub file_path: PathBuf,
}
/// 获取测试文件路径
pub fn test_path(path: &str) -> PathBuf {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    here.join("tests").join(path)
}
