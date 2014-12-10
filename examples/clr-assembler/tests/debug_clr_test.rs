#[cfg(test)]
mod debug_clr_tests {
    use clr_assembler::formats::dll::{reader::DllReader, DllReadConfig};
    use std::{fs::File, io::BufReader};

    #[test]
    fn test_debug_clr_header() {
        let file_path = r"C:\Windows\Microsoft.NET\Framework64\v4.0.30319\mscorlib.dll";

        println!("正在尝试读取文件: {}", file_path);

        let file = match File::open(file_path) {
            Ok(f) => f,
            Err(e) => {
                println!("无法打开文件: {}", e);
                return;
            }
        };

        let reader = BufReader::new(file);
        let config = DllReadConfig::default();
        let mut dll_reader = DllReader::new(reader, &config);

        println!("成功创建 DllReader");

        match dll_reader.get_assembly_info() {
            Ok(info) => {
                println!("成功读取程序集信息:");
                println!("  名称: {}", info.name);
                println!("  版本: {}", info.version);
                println!("  文化区域: {:?}", info.culture);
                println!("  公钥标记: {:?}", info.public_key_token);
                println!("  运行时版本: {:?}", info.runtime_version);

                // 验证基本信息
                assert!(!info.name.is_empty(), "程序集名称不应为空");
                assert!(!info.version.is_empty(), "程序集版本不应为空");
            }
            Err(e) => {
                println!("读取程序集信息失败: {}", e);
                panic!("测试失败: {}", e);
            }
        }
    }
}
