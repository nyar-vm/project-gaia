use std::path::{Path, PathBuf};

pub fn test_path(path: &str) -> PathBuf {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    here.join("tests").join(path)
}
