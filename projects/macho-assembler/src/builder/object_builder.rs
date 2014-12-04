use crate::types::{MachoProgram, MachoHeader, LoadCommand, CpuType};
use gaia_types::GaiaError;

/// Mach-O 目标文件构建器
///
/// 提供高级接口来构建 Mach-O 目标文件
#[derive(Debug)]
pub struct ObjectBuilder {
    cpu_type: CpuType,
    load_commands: Vec<LoadCommand>,
    flags: u32,
}

impl ObjectBuilder {
    /// 创建一个新的目标文件构建器
    pub fn new(cpu_type: CpuType) -> Self {
        Self {
            cpu_type,
            load_commands: Vec::new(),
            flags: 0,
        }
    }

    /// 添加加载命令
    pub fn add_load_command(mut self, command: LoadCommand) -> Self {
        self.load_commands.push(command);
        self
    }

    /// 设置标志位
    pub fn set_flags(mut self, flags: u32) -> Self {
        self.flags = flags;
        self
    }

    /// 添加符号表命令
    pub fn add_symtab(self, symoff: u32, nsyms: u32, stroff: u32, strsize: u32) -> Self {
        let mut data = Vec::new();
        data.extend_from_slice(&symoff.to_le_bytes());
        data.extend_from_slice(&nsyms.to_le_bytes());
        data.extend_from_slice(&stroff.to_le_bytes());
        data.extend_from_slice(&strsize.to_le_bytes());

        let command = LoadCommand {
            cmd: 0x2, // LC_SYMTAB
            cmdsize: 24,
            data,
        };

        self.add_load_command(command)
    }

    /// 构建 MachoProgram
    pub fn build(self) -> Result<MachoProgram, GaiaError> {
        let sizeofcmds = self.load_commands.iter()
            .map(|cmd| cmd.cmdsize)
            .sum::<u32>();

        let header = MachoHeader {
            magic: match self.cpu_type {
                CpuType::X86_64 | CpuType::Arm64 => 0xfeedfacf, // MH_MAGIC_64
                _ => 0xfeedface, // MH_MAGIC
            },
            cpu_type: self.cpu_type as u32,
            cpu_subtype: 0,
            file_type: 1, // MH_OBJECT
            ncmds: self.load_commands.len() as u32,
            sizeofcmds,
            flags: self.flags,
            reserved: match self.cpu_type {
                CpuType::X86_64 | CpuType::Arm64 => Some(0),
                _ => None,
            },
        };

        Ok(MachoProgram {
            header,
            load_commands: self.load_commands,
            segments: Vec::new(),
            sections: Vec::new(),
        })
    }
}