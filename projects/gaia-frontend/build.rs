fn main() {
    // 告诉 Cargo 当 WIT 文件改变时重新构建
    println!("cargo:rerun-if-changed=wit");
}
