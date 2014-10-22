#[cfg(test)]
mod tests {
    use debug_pe::hello_world::{generate_x86_console_log, generate_x86_console_program};
    use pe_assembler::writer::PeAssembler;
    use std::{fs, process::Command};

    #[test]
    #[cfg(feature = "easy-test")]
    fn test_minimal_pe_file() {
        // 生成一个最小的 PE 程序结构
        let pe_program = generate_x86_console_program("Minimal PE Test").expect("Failed to generate PE program");

        // 添加调试信息：打印生成的 PE 程序的节信息
        println!("=== Generated PE Program Debug Info ===");
        for (i, section) in pe_program.sections.iter().enumerate() {
            println!("Section {}: {}", i, section.name);
            println!("  virtual_size: 0x{:X}", section.virtual_size);
            println!("  virtual_address: 0x{:X}", section.virtual_address);
            println!("  size_of_raw_data: 0x{:X}", section.size_of_raw_data);
            println!("  pointer_to_raw_data: 0x{:X}", section.pointer_to_raw_data);
            println!("  characteristics: 0x{:X}", section.characteristics);
            println!("  data.len(): {}", section.data.len());
        }
        println!("========================================");

        // 写入二进制数据
        let pe_data = PeAssembler::write_program(&pe_program).expect("Failed to write PE program");

        // 写入文件
        let output_path = "minimal_pe.exe";
        fs::write(output_path, &pe_data).expect("Failed to write PE file");

        println!("Generated minimal PE file: {}", output_path);
        println!("File size: {} bytes", pe_data.len());

        // 尝试使用 file 命令检查文件类型（如果可用）
        if let Ok(output) = Command::new("file").arg(output_path).output() {
            println!("File type: {}", String::from_utf8_lossy(&output.stdout));
        }

        // 使用 dumpbin 分析 PE 文件
        let dumpbin_path =
            r"C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Tools\MSVC\14.43.34808\bin\Hostx86\x64\dumpbin.exe";
        let output = Command::new(dumpbin_path).args(&["/headers", output_path]).output();

        match output {
            Ok(result) => {
                let stdout = String::from_utf8_lossy(&result.stdout);
                let stderr = String::from_utf8_lossy(&result.stderr);

                if result.status.success() {
                    println!("✓ PE signature verified by dumpbin");
                    println!("=== dumpbin output ===");
                    println!("{}", stdout);
                    println!("======================");
                }
                else {
                    println!("✗ dumpbin failed:");
                    println!("stdout: {}", stdout);
                    println!("stderr: {}", stderr);
                }
            }
            Err(e) => {
                println!("✗ Failed to run dumpbin: {}", e);
            }
        }

        // 检查文件是否被识别为可执行文件
        let metadata = fs::metadata(output_path).expect("Failed to get file metadata");
        println!("File permissions: {:?}", metadata.permissions());

        // 不清理文件，以便后续检查
        // let _ = fs::remove_file(output_path);
    }

    #[test]
    #[cfg(feature = "easy-test")]
    fn test_pe_structure_validation() {
        let pe_data = generate_x86_console_log("Structure Test").expect("Failed to generate PE program");

        // 验证 DOS 头
        assert_eq!(&pe_data[0..2], b"MZ", "DOS magic number should be MZ");

        // 获取 PE 头偏移
        let pe_offset = u32::from_le_bytes([pe_data[60], pe_data[61], pe_data[62], pe_data[63]]) as usize;
        println!("PE header offset: 0x{:X}", pe_offset);

        // 验证 PE 签名
        assert_eq!(&pe_data[pe_offset..pe_offset + 4], b"PE\0\0", "PE signature should be PE\\0\\0");

        // 验证机器类型 (x86 = 0x014C)
        let machine_type = u16::from_le_bytes([pe_data[pe_offset + 4], pe_data[pe_offset + 5]]);
        assert_eq!(machine_type, 0x014C, "Machine type should be 0x014C for x86");

        // 验证节数量
        let num_sections = u16::from_le_bytes([pe_data[pe_offset + 6], pe_data[pe_offset + 7]]);
        println!("Number of sections: {}", num_sections);
        assert!(num_sections > 0, "Should have at least one section");

        // 验证可选头大小
        let optional_header_size = u16::from_le_bytes([pe_data[pe_offset + 20], pe_data[pe_offset + 21]]);
        println!("Optional header size: {}", optional_header_size);
        assert_eq!(optional_header_size, 224, "Optional header size should be 224 for PE32");

        // 验证可选头魔数 (PE32 = 0x010B)
        let optional_header_offset = pe_offset + 24;
        let optional_header_magic = u16::from_le_bytes([pe_data[optional_header_offset], pe_data[optional_header_offset + 1]]);
        assert_eq!(optional_header_magic, 0x010B, "Optional header magic should be 0x010B for PE32");

        println!("All PE structure validations passed!");
    }
}
