use crate::{
    helpers::MachoWriter,
    types::MachoProgram,
};
use gaia_types::{BinaryWriter, GaiaError};
use byteorder::LittleEndian;
use std::io::{Write, Seek};

/// Mach-O 动态库写入器
///
/// 专门用于写入 Mach-O 动态库文件的写入器。
#[derive(Debug)]
pub struct DylibWriter<W: Write + Seek> {
    writer: BinaryWriter<W, LittleEndian>,
}

impl<W: Write + Seek> DylibWriter<W> {
    /// 创建一个新的动态库写入器
    pub fn new(writer: W) -> Self {
        Self {
            writer: BinaryWriter::new(writer),
        }
    }
}

impl<W: Write + Seek> MachoWriter<W> for DylibWriter<W> {
    fn write_program(&mut self, program: &MachoProgram) -> Result<(), GaiaError> {
        // 写入 Mach-O 文件头
        self.writer.write_u32(program.header.magic)?;
        self.writer.write_u32(program.header.cpu_type)?;
        self.writer.write_u32(program.header.cpu_subtype)?;
        self.writer.write_u32(program.header.file_type)?;
        self.writer.write_u32(program.header.ncmds)?;
        self.writer.write_u32(program.header.sizeofcmds)?;
        self.writer.write_u32(program.header.flags)?;
        
        // 如果是64位格式，写入保留字段
        if let Some(reserved) = program.header.reserved {
            self.writer.write_u32(reserved)?;
        }

        // 写入加载命令
        for load_cmd in &program.load_commands {
            self.writer.write_u32(load_cmd.cmd)?;
            self.writer.write_u32(load_cmd.cmdsize)?;
            self.writer.write_all(&load_cmd.data)?;
        }

        Ok(())
    }

    fn writer(&mut self) -> &mut BinaryWriter<W, LittleEndian> {
        &mut self.writer
    }
}