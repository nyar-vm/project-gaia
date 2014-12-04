use crate::types::{MachoHeader, MachoProgram, LoadCommand, CpuType, MachoType};
use gaia_types::GaiaError;

/// Mach-O 动态库构建器
///
/// 提供了构建 Mach-O 动态库文件的高级接口。
#[derive(Debug)]
pub struct DylibBuilder {
    header: MachoHeader,
    load_commands: Vec<LoadCommand>,
}

impl DylibBuilder {
    /// 创建一个新的动态库构建器
    pub fn new(cpu_type: CpuType) -> Self {
        let header = MachoHeader {
            magic: match cpu_type {
                CpuType::X86_64 | CpuType::Arm64 => 0xfeedfacf, // MH_MAGIC_64
                _ => 0xfeedface, // MH_MAGIC
            },
            cpu_type: cpu_type as u32,
            cpu_subtype: 0,
            file_type: MachoType::Dylib as u32,
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
        }
    }

    /// 添加一个加载命令
    pub fn add_load_command(mut self, cmd: u32, data: Vec<u8>) -> Self {
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
    pub fn set_flags(mut self, flags: u32) -> Self {
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

impl Default for DylibBuilder {
    fn default() -> Self {
        Self::new(CpuType::X86_64)
    }
}