use crate::{
    helpers::MachoReader,
    types::{MachoProgram, MachoReadConfig},
};
use gaia_types::{BinaryReader, GaiaError};
use std::{cell::RefCell, sync::OnceLock};
use byteorder::LittleEndian;
use std::io::{Read, Seek};

/// Mach-O 动态库信息
#[derive(Debug, Clone, Copy)]
pub struct DylibInfo {
    /// CPU类型
    pub cpu_type: u32,
    /// 文件类型
    pub file_type: u32,
    /// 加载命令数量
    pub ncmds: u32,
    /// 加载命令总大小
    pub sizeofcmds: u32,
    /// 标志位
    pub flags: u32,
}

/// Mach-O 动态库延迟读取器
#[derive(Debug)]
pub struct DylibReader<R: Read + Seek> {
    reader: RefCell<BinaryReader<R, LittleEndian>>,
    config: MachoReadConfig,
    program: OnceLock<Result<MachoProgram, GaiaError>>,
    info: OnceLock<Result<DylibInfo, GaiaError>>,
}

impl<R: Read + Seek> DylibReader<R> {
    /// 创建一个新的动态库读取器
    pub fn new(reader: BinaryReader<R, LittleEndian>, config: MachoReadConfig) -> Self {
        Self {
            reader: RefCell::new(reader),
            config,
            program: OnceLock::new(),
            info: OnceLock::new(),
        }
    }

    /// 获取程序信息（延迟加载）
    pub fn get_program(&self) -> Result<&MachoProgram, &GaiaError> {
        self.program.get_or_init(|| {
            self.read_program_internal()
        }).as_ref()
    }

    /// 获取文件信息（延迟加载）
    pub fn get_info(&self) -> Result<&DylibInfo, &GaiaError> {
        self.info.get_or_init(|| {
            self.read_info()
        }).as_ref()
    }

    /// 读取完整程序（内部方法）
    fn read_program_internal(&self) -> Result<MachoProgram, GaiaError> {
        let mut reader = self.reader.borrow_mut();
        
        // 读取 Mach-O 文件头
        let magic = reader.read_u32()?;
        let cpu_type = reader.read_u32()?;
        let cpu_subtype = reader.read_u32()?;
        let file_type = reader.read_u32()?;
        let ncmds = reader.read_u32()?;
        let sizeofcmds = reader.read_u32()?;
        let flags = reader.read_u32()?;
        
        let reserved = if magic == 0xfeedfacf {
            Some(reader.read_u32()?)
        } else {
            None
        };

        let header = crate::types::MachoHeader {
            magic,
            cpu_type,
            cpu_subtype,
            file_type,
            ncmds,
            sizeofcmds,
            flags,
            reserved,
        };

        // 读取加载命令
        let mut load_commands = Vec::new();
        for _ in 0..ncmds {
            let cmd = reader.read_u32()?;
            let cmdsize = reader.read_u32()?;
            
            let data_size = cmdsize.saturating_sub(8) as usize;
            let mut data = vec![0u8; data_size];
            reader.read_exact(&mut data)?;
            
            load_commands.push(crate::types::LoadCommand {
                cmd,
                cmdsize,
                data,
            });
        }

        Ok(MachoProgram {
            header,
            load_commands,
            segments: Vec::new(),
            sections: Vec::new(),
        })
    }

    /// 读取文件基本信息
    fn read_info(&self) -> Result<DylibInfo, GaiaError> {
        let mut reader = self.reader.borrow_mut();
        
        let _magic = reader.read_u32()?;
        let cpu_type = reader.read_u32()?;
        let _cpu_subtype = reader.read_u32()?;
        let file_type = reader.read_u32()?;
        let ncmds = reader.read_u32()?;
        let sizeofcmds = reader.read_u32()?;
        let flags = reader.read_u32()?;
        
        Ok(DylibInfo {
            cpu_type,
            file_type,
            ncmds,
            sizeofcmds,
            flags,
        })
    }
}

impl<R: Read + Seek> MachoReader<R> for DylibReader<R> {
    fn read_program(&mut self) -> Result<MachoProgram, GaiaError> {
        self.read_program_internal()
    }

    fn reader(&mut self) -> &mut BinaryReader<R, LittleEndian> {
        self.reader.get_mut()
    }

    fn config(&self) -> &MachoReadConfig {
        &self.config
    }
}