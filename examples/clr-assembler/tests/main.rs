use crate::test_tools::validate_msil_files;
pub use gaia_types::Result;
use std::path::Path;

mod msil_read;
mod msil_write;
mod net_read;
mod net_write;
mod test_tools;

#[test]
fn ready() {
    println!("it works!")
}

fn here() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn parse_msil_files() {
    println!("开始自动化测试所有 MSIL 文件...");
    validate_msil_files(&here().join("tests"))
}

#[test]
fn hello_dll() -> Result<()> {
    // let program =
    //     IlContext::read(include_bytes!("HelloUnity.msil_read").to_vec(), ReadConfig { format: IlFormat::Auto, url: None }).program?;
    // let bytes = IlContext::write(program, WriterConfig { format: IlFormat::Dll })?;
    // // 验证DLL基本结构，不再依赖外部二进制文件
    // assert!(bytes.len() > 0, "DLL字节数组不能为空");
    // // 验证DOS头部
    // assert_eq!(&bytes[0..2], b"MZ", "DOS签名不正确");
    // // 验证PE签名位置
    // let pe_offset = u32::from_le_bytes([bytes[0x3C], bytes[0x3D], bytes[0x3E], bytes[0x3F]]);
    // let pe_start = pe_offset as usize;
    // assert_eq!(&bytes[pe_start..pe_start + 4], b"PE\0\0", "PE签名不正确");
    Ok(())
}
