use byteorder::{LittleEndian, ReadBytesExt};
use gaia_types::GaiaError;
use serde::{Deserialize, Serialize};
use std::io::{Read, Seek, SeekFrom};

/// NT 头结构
///
/// 包含 PE 文件的主要签名和基本信息，标识这是一个有效的 PE 文件。
/// signature 字段必须为 0x00004550（"PE\0\0"）。
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct NtHeader {
    /// PE 签名，必须为 0x00004550（"PE\0\0"）
    pub signature: u32,
}

impl NtHeader {
    /// 从二进制读取器中读取 NT 头
    ///
    /// # Arguments
    /// * `lexer` - 二进制读取器
    ///
    /// # Returns
    /// 返回 NT 头结构或错误
    pub fn read<R>(mut reader: R) -> Result<Self, GaiaError>
    where
        R: Read,
    {
        let signature = reader.read_u32::<LittleEndian>()?;
        Ok(NtHeader { signature })
    }

    pub fn read_at<R, E>(mut reader: R, offset: u64) -> Result<Self, GaiaError>
    where
        R: Read + Seek,
    {
        reader.seek(SeekFrom::Start(offset))?;
        Self::read(reader)
    }
}

impl Default for NtHeader {
    fn default() -> Self {
        Self {
            // "PE\0\0"
            signature: 0x00004550,
        }
    }
}
