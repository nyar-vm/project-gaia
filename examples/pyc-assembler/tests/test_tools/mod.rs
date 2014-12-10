use std::path::{Path, PathBuf};

/// 获取测试文件路径
pub fn test_path(path: &str) -> PathBuf {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    here.join("tests").join(path)
}
