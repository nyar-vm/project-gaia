use crate::types::MachoProgram;
use gaia_types::{BinaryWriter, GaiaError};
use std::io::Write;

/// Mach-O 写入器 trait
///
/// 定义了写入 Mach-O 文件的通用接口。
pub trait MachoWriter<W: Write> {
    /// 写入 Mach-O 程序
    fn write_program(&mut self, program: &MachoProgram) -> Result<(), GaiaError>;
    
    /// 获取内部写入器的引用
    fn writer(&mut self) -> &mut BinaryWriter<W, byteorder::LittleEndian>;
}