//! 测试 Hello World PE 文件生成和保存

use std::fs;

// 引入 hello_world 模块
mod hello_world;
use hello_world::generate_hello_world_pe;

#[test]
fn test_generate_and_save_hello_world_pe() {
    println!("正在生成 Hello World PE 文件...");

    // 生成 PE 文件数据
    let pe_data = generate_hello_world_pe();

    // 创建输出目录
    let output_dir = "tests/generated";
    fs::create_dir_all(output_dir).expect("无法创建输出目录");

    // 保存 PE 文件
    let output_path = format!("{}/hello_world.exe", output_dir);
    fs::write(&output_path, &pe_data).expect("无法写入PE文件");

    println!("✅ PE 文件已生成: {}", output_path);
    println!("   文件大小: {} 字节", pe_data.len());

    // 打印一些基本信息
    println!("\n=== PE 文件基本信息 ===");
    println!("DOS 头魔数: 0x{:04X}", u16::from_le_bytes([pe_data[0], pe_data[1]]));

    // 获取 PE 头偏移
    let pe_offset = u32::from_le_bytes([pe_data[60], pe_data[61], pe_data[62], pe_data[63]]) as usize;
    println!("PE 头偏移: 0x{:08X}", pe_offset);

    // 检查 PE 签名
    if pe_data.len() >= pe_offset + 4 {
        let pe_signature = &pe_data[pe_offset..pe_offset + 4];
        println!("PE 签名: {:?}", std::str::from_utf8(pe_signature).unwrap_or("无效"));
    }

    // 获取机器类型
    if pe_data.len() >= pe_offset + 6 {
        let machine_type = u16::from_le_bytes([pe_data[pe_offset + 4], pe_data[pe_offset + 5]]);
        println!(
            "机器类型: 0x{:04X} ({})",
            machine_type,
            match machine_type {
                0x014C => "i386",
                0x8664 => "x86_64",
                _ => "未知",
            }
        );
    }

    // 获取节数量
    if pe_data.len() >= pe_offset + 8 {
        let num_sections = u16::from_le_bytes([pe_data[pe_offset + 6], pe_data[pe_offset + 7]]);
        println!("节数量: {}", num_sections);
    }

    println!("\n现在可以使用以下命令分析 PE 文件:");
    println!("dumpbin /headers {}", output_path);
    println!("dumpbin /imports {}", output_path);
    println!("dumpbin /all {}", output_path);

    // 验证基本的PE结构
    assert_eq!(pe_data[0], 0x4D); // 'M'
    assert_eq!(pe_data[1], 0x5A); // 'Z'
    assert_eq!(&pe_data[pe_offset..pe_offset + 4], b"PE\0\0");
}
