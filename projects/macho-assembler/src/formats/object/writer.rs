use crate::{
    helpers::MachoWriter,
    types::{MachoProgram, MachoHeader, LoadCommand},
};
use gaia_types::{BinaryWriter, GaiaError};
use byteorder::LittleEndian;
use std::io::Write;

/// Mach-O 目标文件写入器
#[derive(Debug)]
pub struct ObjectWriter<W: Write> {
    writer: BinaryWriter<W, LittleEndian>,
}

impl<W: Write> ObjectWriter<W> {
    /// 创建一个新的目标文件写入器
    pub fn new(writer: BinaryWriter<W, LittleEndian>) -> Self {
        Self { writer }
    }

    /// 写入目标文件头
    pub fn write_header(&mut self, header: &MachoHeader) -> Result<(), GaiaError> {
        self.writer.write_u32(header.magic)?;
        self.writer.write_u32(header.cpu_type)?;
        self.writer.write_u32(header.cpu_subtype)?;
        self.writer.write_u32(header.file_type)?;
        self.writer.write_u32(header.ncmds)?;
        self.writer.write_u32(header.sizeofcmds)?;
        self.writer.write_u32(header.flags)?;
        
        if let Some(reserved) = header.reserved {
            self.writer.write_u32(reserved)?;
        }
        
        Ok(())
    }

    /// 写入加载命令
    pub fn write_load_commands(&mut self, load_commands: &[LoadCommand]) -> Result<(), GaiaError> {
        for cmd in load_commands {
            self.writer.write_u32(cmd.cmd)?;
            self.writer.write_u32(cmd.cmdsize)?;
            self.writer.write_all(&cmd.data)?;
        }
        Ok(())
    }
}

impl<W: Write> MachoWriter<W> for ObjectWriter<W> {
    fn write_program(&mut self, program: &MachoProgram) -> Result<(), GaiaError> {
        self.write_header(&program.header)?;
        self.write_load_commands(&program.load_commands)?;
        Ok(())
    }

    fn writer(&mut self) -> &mut BinaryWriter<W, LittleEndian> {
        &mut self.writer
    }
}