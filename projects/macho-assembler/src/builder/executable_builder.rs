use crate::types::{MachoHeader, MachoProgram, LoadCommand, CpuType, MachoType};
use gaia_types::GaiaError;

/// Mach-O 可执行文件构建器
///
/// 提供了构建 Mach-O 可执行文件的高级接口。
#[derive(Debug)]
pub struct ExecutableBuilder {
    header: MachoHeader,
    load_commands: Vec<LoadCommand>,
    entry_point: Option<u64>,
}

impl ExecutableBuilder {
    /// 创建一个新的可执行文件构建器
    pub fn new(cpu_type: CpuType) -> Self {
        let header = MachoHeader {
            magic: match cpu_type {
                CpuType::X86_64 | CpuType::Arm64 => 0xfeedfacf, // MH_MAGIC_64
                _ => 0xfeedface, // MH_MAGIC
            },
            cpu_type: cpu_type as u32,
            cpu_subtype: 0,
            file_type: MachoType::Execute as u32,
            ncmds: 0,
            sizeofcmds: 0,
            flags: 0,
            reserved: match cpu_type {
                CpuType::X86_64 | CpuType::Arm64 => Some(0),
                _ => None,
            },
        };

        Self {
            header,
            load_commands: Vec::new(),
            entry_point: None,
        }
    }

    /// 设置程序入口点
    pub fn set_entry_point(&mut self, entry_point: u64) -> &mut Self {
        self.entry_point = Some(entry_point);
        self
    }

    /// 添加一个加载命令
    pub fn add_load_command(&mut self, cmd: u32, data: Vec<u8>) -> &mut Self {
        let cmdsize = 8 + data.len() as u32; // cmd + cmdsize + data
        let load_cmd = LoadCommand {
            cmd,
            cmdsize,
            data,
        };
        
        self.load_commands.push(load_cmd);
        self.header.ncmds += 1;
        self.header.sizeofcmds += cmdsize;
        
        self
    }

    /// 设置文件标志
    pub fn set_flags(&mut self, flags: u32) -> &mut Self {
        self.header.flags = flags;
        self
    }

    /// 构建 MachoProgram
    pub fn build(self) -> Result<MachoProgram, GaiaError> {
        Ok(MachoProgram {
            header: self.header,
            load_commands: self.load_commands,
            segments: Vec::new(),
            sections: Vec::new(),
        })
    }
}

impl Default for ExecutableBuilder {
    fn default() -> Self {
        Self::new(CpuType::X86_64)
    }
}