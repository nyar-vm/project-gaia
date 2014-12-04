#[cfg(test)]
mod tests {
    use macho_assembler::{
        builder::DylibBuilder,
        types::{CpuType, MachoReadConfig},
        formats::dylib::{DylibWriter, DylibReader},
        helpers::MachoWriter,
    };
    use gaia_types::BinaryReader;
    use std::io::Cursor;
    use byteorder::LittleEndian;

    #[test]
    fn test_dylib_builder() {
        let builder = DylibBuilder::new(CpuType::X86_64);
        
        // 添加一些测试加载命令
        let program = builder
            .add_load_command(0x1, vec![0x00, 0x01, 0x02, 0x03])
            .set_flags(0x2000)
            .build()
            .expect("Failed to build dylib");
        
        assert_eq!(program.header.cpu_type, CpuType::X86_64 as u32);
        assert_eq!(program.header.ncmds, 1);
        assert_eq!(program.load_commands.len(), 1);
        assert_eq!(program.header.flags, 0x2000);
    }

    #[test]
    fn test_dylib_write_read_roundtrip() {
        let builder = DylibBuilder::new(CpuType::X86_64);
        let original_program = builder
            .add_load_command(0x1, vec![0x00, 0x01, 0x02, 0x03])
            .build()
            .expect("Failed to build dylib");
        
        // 写入到内存缓冲区
        let mut buffer = Vec::new();
        {
            let mut writer = DylibWriter::new(Cursor::new(&mut buffer));
            writer.write_program(&original_program).expect("Failed to write program");
        }
        
        // 从缓冲区读取
        let config = MachoReadConfig::default();
        let binary_reader = BinaryReader::<_, LittleEndian>::new(Cursor::new(&buffer));
        let reader = DylibReader::new(binary_reader, config);
        
        let read_program = reader.get_program().expect("Failed to read program");
        
        // 验证读取的数据与原始数据一致
        assert_eq!(read_program.header.magic, original_program.header.magic);
        assert_eq!(read_program.header.cpu_type, original_program.header.cpu_type);
        assert_eq!(read_program.header.ncmds, original_program.header.ncmds);
        assert_eq!(read_program.load_commands.len(), original_program.load_commands.len());
    }

    #[test]
    fn test_dylib_lazy_loading() {
        let builder = DylibBuilder::new(CpuType::Arm64);
        let program = builder
            .add_load_command(0x2, vec![0x10, 0x20, 0x30, 0x40])
            .build()
            .expect("Failed to build dylib");
        
        let mut buffer = Vec::new();
        {
            let mut writer = DylibWriter::new(Cursor::new(&mut buffer));
            writer.write_program(&program).expect("Failed to write program");
        }
        
        let config = MachoReadConfig::default();
        let binary_reader = BinaryReader::<_, LittleEndian>::new(Cursor::new(&buffer));
        let reader = DylibReader::new(binary_reader, config);
        
        // 测试延迟加载 - 多次调用应该返回相同的结果
        let info1 = reader.get_info().expect("Failed to get info first time");
        let info2 = reader.get_info().expect("Failed to get info second time");
        
        assert_eq!(info1.cpu_type, info2.cpu_type);
        assert_eq!(info1.file_type, info2.file_type);
    }
}