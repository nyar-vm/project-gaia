use jvm_jasm::easy_test::validate_jasm_files;
use std::path::Path;

fn here() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

/// 自动测试所有 JASM 文件
#[test]
fn test_all_jasm_files_automatically() {
    let test_folder = Path::new("../../../../tests");
    validate_jasm_files(test_folder);
}
