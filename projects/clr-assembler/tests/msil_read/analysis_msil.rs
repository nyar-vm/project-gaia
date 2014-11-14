use crate::test_tools::validate_msil_files;
use std::path::Path;

fn here() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn parse_msil_files() {
    println!("开始自动化测试所有 MSIL 文件...");
    validate_msil_files(&here().join("tests"))
}
