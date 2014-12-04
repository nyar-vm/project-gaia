use crate::types::{MachoProgram, MachoReadConfig};
use gaia_types::{BinaryReader, GaiaError};
use std::io::{Read, Seek};

/// Mach-O 读取器 trait
///
/// 定义了读取 Mach-O 文件的通用接口。
pub trait MachoReader<R: Read + Seek> {
    /// 读取 Mach-O 程序
    fn read_program(&mut self) -> Result<MachoProgram, GaiaError>;
    
    /// 获取内部读取器的引用
    fn reader(&mut self) -> &mut BinaryReader<R, byteorder::LittleEndian>;
    
    /// 获取配置
    fn config(&self) -> &MachoReadConfig;
}