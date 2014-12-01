use byteorder::{LittleEndian, ReadBytesExt};
use gaia_types::GaiaError;
use serde::{Deserialize, Serialize};
use std::io::Read;

/// DOS 头结构
///
/// 包含 DOS 可执行文件的基本信息，是 PE 文件的第一个结构。
/// 虽然现代 Windows 程序不运行在 DOS 模式下，但 PE 格式仍保留这个结构用于兼容性。
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct DosHeader {
    /// DOS 签名，通常为 0x5A4D（"MZ"）
    pub e_magic: u16,
    /// 文件最后页的字节数
    pub e_cblp: u16,
    /// 文件的总页数
    pub e_cp: u16,
    /// 重定位项的数量
    pub e_crlc: u16,
    /// 段落中头部的大小
    pub e_cparhdr: u16,
    /// 所需的最小段落数
    pub e_min_allocate: u16,
    /// 所需的最大段落数
    pub e_max_allocate: u16,
    /// 初始的 SS 寄存器值
    pub e_ss: u16,
    /// 初始的 SP 寄存器值
    pub e_sp: u16,
    /// 校验和
    pub e_check_sum: u16,
    /// 初始的 IP 寄存器值
    pub e_ip: u16,
    /// 初始的 CS 寄存器值
    pub e_cs: u16,
    /// 重定位表的文件偏移
    pub e_lfarlc: u16,
    /// 覆盖号
    pub e_ovno: u16,
    /// 保留字段，通常为 0
    pub e_res: [u16; 4],
    /// OEM 标识符
    pub e_oem_id: u16,
    /// OEM 信息
    pub e_oem_info: u16,
    /// 保留字段，通常为 0
    pub e_res2: [u16; 10],
    /// PE 头的文件偏移，指向真正的 PE 结构
    pub e_lfanew: u32,
}

impl Default for DosHeader {
    fn default() -> Self {
        Self {
            e_magic: 0x5A4D, // "MZ"
            e_cblp: 0x90,
            e_cp: 0x03,
            e_crlc: 0x00,
            e_cparhdr: 0x04,
            e_min_allocate: 0x00,
            e_max_allocate: 0xFFFF,
            e_ss: 0x00,
            e_sp: 0xB8,
            e_check_sum: 0x00,
            e_ip: 0x00,
            e_cs: 0x00,
            e_lfarlc: 0x40,
            e_ovno: 0x00,
            e_res: [0; 4],
            e_oem_id: 0x00,
            e_oem_info: 0x00,
            e_res2: [0; 10],
            e_lfanew: 0x80, // PE header offset
        }
    }
}

impl DosHeader {
    /// 从二进制读取器中读取 DOS 头
    ///
    /// # Arguments
    /// * `lexer` - 二进制读取器
    ///
    /// # Returns
    /// 返回 DOS 头结构或错误
    pub fn read<R: Read>(mut reader: R) -> Result<Self, GaiaError> {
        let e_magic = reader.read_u16::<LittleEndian>()?;
        let e_cblp = reader.read_u16::<LittleEndian>()?;
        let e_cp = reader.read_u16::<LittleEndian>()?;
        let e_crlc = reader.read_u16::<LittleEndian>()?;
        let e_cparhdr = reader.read_u16::<LittleEndian>()?;
        let e_min_allocate = reader.read_u16::<LittleEndian>()?;
        let e_max_allocate = reader.read_u16::<LittleEndian>()?;
        let e_ss = reader.read_u16::<LittleEndian>()?;
        let e_sp = reader.read_u16::<LittleEndian>()?;
        let e_check_sum = reader.read_u16::<LittleEndian>()?;
        let e_ip = reader.read_u16::<LittleEndian>()?;
        let e_cs = reader.read_u16::<LittleEndian>()?;
        let e_lfarlc = reader.read_u16::<LittleEndian>()?;
        let e_ovno = reader.read_u16::<LittleEndian>()?;

        let mut e_res = [0u16; 4];
        for i in 0..4 {
            e_res[i] = reader.read_u16::<LittleEndian>()?;
        }

        let e_oem_id = reader.read_u16::<LittleEndian>()?;
        let e_oem_info = reader.read_u16::<LittleEndian>()?;

        let mut e_res2 = [0u16; 10];
        for i in 0..10 {
            e_res2[i] = reader.read_u16::<LittleEndian>()?;
        }

        let e_lfanew = reader.read_u32::<LittleEndian>()?;

        Ok(DosHeader {
            e_magic,
            e_cblp,
            e_cp,
            e_crlc,
            e_cparhdr,
            e_min_allocate,
            e_max_allocate,
            e_ss,
            e_sp,
            e_check_sum,
            e_ip,
            e_cs,
            e_lfarlc,
            e_ovno,
            e_res,
            e_oem_id,
            e_oem_info,
            e_res2,
            e_lfanew,
        })
    }
}

impl DosHeader {
    /// 创建一个标准的 DOS 头，指定 PE 头的偏移
    pub fn new(header_offset: u32) -> Self {
        Self { e_lfanew: header_offset, ..Default::default() }
    }
}
