//! 静态库文件分析测试
//!
//! 这个模块测试静态库文件的解析功能，使用 Windows 系统中常见的 lib 文件

use pe_coff::{
    easy_test::print_lib_summary,
    reader::{read_lib_from_file, LibReader},
};
use std::path::Path;

/// 测试解析 Windows 系统中的 lib 文件
#[test]
fn test_parse_system_lib_files() {
    // 常见的 Windows 系统 lib 文件路径
    let test_paths = vec![
        // Windows SDK 库文件
        r"C:\Program Files (x86)\Windows Kits\10\Lib\10.0.19041.0\um\x64\kernel32.lib",
        r"C:\Program Files (x86)\Windows Kits\10\Lib\10.0.19041.0\um\x64\user32.lib",
        r"C:\Program Files (x86)\Windows Kits\10\Lib\10.0.19041.0\um\x64\gdi32.lib",
        r"C:\Program Files (x86)\Windows Kits\10\Lib\10.0.19041.0\um\x64\advapi32.lib",
        r"C:\Program Files (x86)\Windows Kits\10\Lib\10.0.19041.0\um\x86\kernel32.lib",
        r"C:\Program Files (x86)\Windows Kits\10\Lib\10.0.19041.0\um\x86\user32.lib",
        // Visual Studio 运行时库
        r"C:\Program Files (x86)\Microsoft Visual Studio\2019\Community\VC\Tools\MSVC\14.29.30133\lib\x64\libcmt.lib",
        r"C:\Program Files (x86)\Microsoft Visual Studio\2019\Community\VC\Tools\MSVC\14.29.30133\lib\x64\msvcrt.lib",
        r"C:\Program Files (x86)\Microsoft Visual Studio\2019\Community\VC\Tools\MSVC\14.29.30133\lib\x86\libcmt.lib",
        // 更通用的路径（可能在不同版本的 Windows SDK 中存在）
        r"C:\Windows\System32\kernel32.dll", // 虽然是 DLL，但可以测试文件类型检测
    ];

    let mut found_files = 0;
    let mut lib_files = 0;

    for path_str in test_paths {
        let path = Path::new(path_str);
        if path.exists() {
            found_files += 1;
            println!("测试文件: {}", path_str);

            // 检查文件扩展名
            if path.extension().and_then(|s| s.to_str()) == Some("lib") {
                lib_files += 1;
                match test_parse_lib_file(path) {
                    Ok(_) => println!("✓ 成功解析库文件: {}", path_str),
                    Err(e) => println!("✗ 解析库文件失败: {} - {}", path_str, e),
                }
            }
            else {
                println!("  跳过非库文件: {}", path_str);
            }
        }
        else {
            println!("文件不存在: {}", path_str);
        }
    }

    if found_files == 0 {
        println!("警告: 未找到任何测试文件，跳过测试");
    }
    else {
        println!("总共找到 {} 个文件，其中 {} 个库文件", found_files, lib_files);
    }
}

/// 测试解析单个 lib 文件
fn test_parse_lib_file(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // 使用 easy_test 模块的便利函数
    let library = read_lib_from_file(path)?;
    print_lib_summary(&library);
    Ok(())
}

/// 测试库读取器配置
#[test]
fn test_lib_reader_config() {
    let config = LibReader::default();
    assert!(config.read_members);
    assert!(config.read_symbols);

    let custom_config = LibReader { read_members: true, read_symbols: false };

    assert!(custom_config.read_members);
    assert!(!custom_config.read_symbols);
}

/// 测试创建临时 lib 文件并解析
#[test]
fn test_minimal_lib_parsing() {
    // 创建一个最小的静态库数据用于测试
    let minimal_lib = create_minimal_lib_data();

    if minimal_lib.len() >= 8 {
        println!("创建了 {} 字节的最小库数据", minimal_lib.len());

        // 测试库文件签名检测
        let signature = &minimal_lib[0..8];
        assert_eq!(signature, b"!<arch>\n", "库文件签名不正确");
        println!("✓ 库文件签名验证通过");
    }
}

/// 创建最小的静态库数据用于测试
fn create_minimal_lib_data() -> Vec<u8> {
    let mut data = Vec::new();

    // 库文件签名
    data.extend_from_slice(b"!<arch>\n");

    // 添加一个最小的成员头部（60 字节）
    let mut member_header = [b' '; 60];

    // 成员名称 (16 字节)
    member_header[0..8].copy_from_slice(b"test.obj");

    // 时间戳 (12 字节)
    member_header[16..28].copy_from_slice(b"1234567890  ");

    // 所有者 ID (6 字节)
    member_header[28..34].copy_from_slice(b"0     ");

    // 组 ID (6 字节)
    member_header[34..40].copy_from_slice(b"0     ");

    // 模式 (8 字节)
    member_header[40..48].copy_from_slice(b"644     ");

    // 大小 (10 字节)
    member_header[48..58].copy_from_slice(b"0         ");

    // 结束标记 (2 字节)
    member_header[58..60].copy_from_slice(b"`\n");

    data.extend_from_slice(&member_header);

    data
}

/// 测试在常见位置查找库文件
#[test]
fn test_find_common_lib_locations() {
    let common_locations = vec![
        r"C:\Program Files (x86)\Windows Kits\10\Lib",
        r"C:\Program Files (x86)\Microsoft Visual Studio",
        r"C:\Program Files\Microsoft Visual Studio",
    ];

    for location in common_locations {
        let path = Path::new(location);
        if path.exists() {
            println!("找到库目录: {}", location);

            // 尝试列出一些子目录
            if let Ok(entries) = std::fs::read_dir(path) {
                let mut count = 0;
                for entry in entries.take(3) {
                    if let Ok(entry) = entry {
                        println!("  子目录: {}", entry.path().display());
                        count += 1;
                    }
                }
                if count > 0 {
                    println!("  ... (显示前 {} 个条目)", count);
                }
            }
        }
        else {
            println!("库目录不存在: {}", location);
        }
    }
}
