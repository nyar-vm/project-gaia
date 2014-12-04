#[cfg(test)]
mod tests {
    use macho_assembler::{
        builder::{DylibBuilder, ExecutableBuilder},
        types::CpuType,
    };

    #[test]
    fn test_dylib_builder_default() {
        let builder = DylibBuilder::default();
        let program = builder.build().expect("Failed to build dylib");
        
        assert_eq!(program.header.cpu_type, CpuType::X86_64 as u32);
        assert_eq!(program.header.ncmds, 0);
        assert_eq!(program.load_commands.len(), 0);
    }

    #[test]
    fn test_executable_builder_with_entry_point() {
        let mut builder = ExecutableBuilder::new(CpuType::Arm64);
        builder.set_entry_point(0x1000);
        builder.add_load_command(0x5, vec![0xaa, 0xbb, 0xcc, 0xdd]);
        
        let program = builder.build().expect("Failed to build executable");
        
        assert_eq!(program.header.cpu_type, CpuType::Arm64 as u32);
        assert_eq!(program.header.ncmds, 1);
        assert_eq!(program.load_commands.len(), 1);
        assert_eq!(program.load_commands[0].cmd, 0x5);
    }

    #[test]
    fn test_builder_chaining() {
        let program = DylibBuilder::new(CpuType::X86_64)
            .add_load_command(0x1, vec![0x01, 0x02])
            .add_load_command(0x2, vec![0x03, 0x04, 0x05])
            .set_flags(0x1000)
            .build()
            .expect("Failed to build with chaining");
        
        assert_eq!(program.header.ncmds, 2);
        assert_eq!(program.header.flags, 0x1000);
        assert_eq!(program.load_commands.len(), 2);
    }

    #[test]
    fn test_load_command_size_calculation() {
        let builder = DylibBuilder::new(CpuType::X86_64);
        
        // 添加一个4字节数据的加载命令
        let program = builder
            .add_load_command(0x1, vec![0x00, 0x01, 0x02, 0x03])
            .build()
            .expect("Failed to build");
        
        // cmdsize 应该是 8 (cmd + cmdsize) + 4 (data) = 12
        assert_eq!(program.load_commands[0].cmdsize, 12);
        assert_eq!(program.header.sizeofcmds, 12);
    }

    #[test]
    fn test_multiple_architectures() {
        let architectures = vec![
            CpuType::X86_64,
            CpuType::Arm64,
            CpuType::I386,
            CpuType::Arm,
        ];

        for arch in architectures {
            let builder = DylibBuilder::new(arch);
            let program = builder.build().expect("Failed to build for architecture");
            
            assert_eq!(program.header.cpu_type, arch as u32);
            
            // 检查魔数是否正确设置
            match arch {
                CpuType::X86_64 | CpuType::Arm64 => {
                    assert_eq!(program.header.magic, 0xfeedfacf); // MH_MAGIC_64
                    assert!(program.header.reserved.is_some());
                }
                _ => {
                    assert_eq!(program.header.magic, 0xfeedface); // MH_MAGIC
                    assert!(program.header.reserved.is_none());
                }
            }
        }
    }
}