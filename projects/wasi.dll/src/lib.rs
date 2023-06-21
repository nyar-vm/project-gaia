#![deny(missing_debug_implementations, missing_copy_implementations)]
#![warn(missing_docs, rustdoc::missing_crate_level_docs)]
#![doc = include_str!("../readme.md")]
#![doc(html_logo_url = "https://raw.githubusercontent.com/oovm/shape-rs/dev/projects/images/Trapezohedron.svg")]
#![doc(html_favicon_url = "https://raw.githubusercontent.com/oovm/shape-rs/dev/projects/images/Trapezohedron.svg")]


pub use crate::vm::WasiRunner;

mod vm;

#[no_mangle]
pub extern "win64" fn run_wasi(buffer: *const u8, length: u64) {
    let bytes = unsafe { std::slice::from_raw_parts(buffer, length as usize) };
    WasiRunner::run_wasm(bytes).unwrap();
}

#[test]
fn ready() {
    println!("it works!")
}
