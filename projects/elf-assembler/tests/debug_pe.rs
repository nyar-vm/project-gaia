pub mod exit_code;
pub mod hello_world;

#[cfg(test)]
mod tests {
    use crate::hello_world::generate_x86_console_log;
    use std::fs;

    #[test]
    fn debug_pe_file_structure() {
        // 生成 PE 文件
        let pe_data = generate_x86_console_log("Hello").unwrap();

        // 保存到文件
        let output_path = "debug_hello_x86.exe";
        fs::write(output_path, &pe_data).unwrap();

        // 打印前 200 字节的十六进制
        println!("PE file size: {} bytes", pe_data.len());
        println!("First 200 bytes:");
        for (i, chunk) in pe_data.chunks(16).take(12).enumerate() {
            print!("{:04X}: ", i * 16);
            for byte in chunk {
                print!("{:02X} ", byte);
            }
            println!();
        }

        // 检查 DOS 头
        if pe_data.len() >= 2 {
            let dos_magic = u16::from_le_bytes([pe_data[0], pe_data[1]]);
            println!("DOS magic: 0x{:04X} (should be 0x5A4D)", dos_magic);
        }

        // 检查 PE 头偏移
        if pe_data.len() >= 64 {
            let pe_offset = u32::from_le_bytes([pe_data[60], pe_data[61], pe_data[62], pe_data[63]]);
            println!("PE header offset: 0x{:08X}", pe_offset);

            // 检查 PE 签名
            if pe_data.len() >= (pe_offset + 4) as usize {
                let pe_signature = u32::from_le_bytes([
                    pe_data[pe_offset as usize],
                    pe_data[pe_offset as usize + 1],
                    pe_data[pe_offset as usize + 2],
                    pe_data[pe_offset as usize + 3],
                ]);
                println!("PE signature: 0x{:08X} (should be 0x00004550)", pe_signature);

                // 检查 COFF 头
                let coff_offset = pe_offset as usize + 4;
                if pe_data.len() >= coff_offset + 20 {
                    let machine = u16::from_le_bytes([pe_data[coff_offset], pe_data[coff_offset + 1]]);
                    let num_sections = u16::from_le_bytes([pe_data[coff_offset + 2], pe_data[coff_offset + 3]]);
                    let opt_header_size = u16::from_le_bytes([pe_data[coff_offset + 16], pe_data[coff_offset + 17]]);
                    let characteristics = u16::from_le_bytes([pe_data[coff_offset + 18], pe_data[coff_offset + 19]]);

                    println!("Machine type: 0x{:04X} (should be 0x014C for i386)", machine);
                    println!("Number of sections: {}", num_sections);
                    println!("Optional header size: {}", opt_header_size);
                    println!("Characteristics: 0x{:04X}", characteristics);

                    // 检查可选头魔数
                    let opt_header_offset = coff_offset + 20;
                    if pe_data.len() >= opt_header_offset + 2 {
                        let magic = u16::from_le_bytes([pe_data[opt_header_offset], pe_data[opt_header_offset + 1]]);
                        println!("Optional header magic: 0x{:04X} (should be 0x010B for PE32)", magic);

                        // 检查入口点
                        if pe_data.len() >= opt_header_offset + 16 {
                            let entry_point = u32::from_le_bytes([
                                pe_data[opt_header_offset + 16],
                                pe_data[opt_header_offset + 17],
                                pe_data[opt_header_offset + 18],
                                pe_data[opt_header_offset + 19],
                            ]);
                            println!("Entry point: 0x{:08X}", entry_point);
                        }
                    }
                }
            }
        }
    }
}
