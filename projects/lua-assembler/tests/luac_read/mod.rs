//! # Luac Read 测试模块
//! 
//! 本模块测试 `LuacReader` 的功能，包括：
//! - 读取 `.luac` 文件头部信息
//! - 解析 Lua 字节码数据
//! - 惰性加载机制验证
//! - 错误处理测试

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use lua_assembler::formats::luac::LuacReadConfig;
use lua_assembler::program::LuaVersion;
use gaia_types::GaiaError;

/// 测试 LuacReader 的基本功能
/// 
/// 验证 LuacReader 能够正确读取 `.luac` 文件并解析基本信息
#[test]
fn test_luac_reader_basic() {
    let luac_path = Path::new("tests/luac_read/hello.luac");
    
    // 如果 .luac 文件不存在，跳过测试
    if !luac_path.exists() {
        println!("跳过测试：{} 文件不存在", luac_path.display());
        return;
    }
    
    let file = File::open(luac_path).expect("无法打开 .luac 文件");
    let config = LuacReadConfig::default();
    let reader = config.as_reader(BufReader::new(file));
    
    // 测试获取基本信息
    let info_result = reader.get_info();
    assert!(info_result.is_ok(), "获取 LuacInfo 失败: {:?}", info_result.err());
    
    let info = info_result.unwrap();
    println!("Lua 版本: {:?}", info.version);
    println!("格式版本: {}", info.header.format_version);
    println!("字节序: {}", info.header.endianness);
    
    // 测试获取完整程序
    let result = reader.finish();
    assert!(result.result.is_ok(), "解析 LuaProgram 失败: {:?}", result.result.err());
    
    let program = result.result.unwrap();
    println!("程序版本: {:?}", program.header.version);
    assert_ne!(program.header.version, LuaVersion::Unknown, "程序版本不应为 Unknown");
}

/// 测试 LuacReader 的惰性加载机制
/// 
/// 验证 LuacReader 只在需要时才解析数据
#[test]
fn test_luac_reader_lazy_loading() {
    let luac_path = Path::new("tests/luac_read/hello.luac");
    
    // 如果 .luac 文件不存在，跳过测试
    if !luac_path.exists() {
        println!("跳过测试：{} 文件不存在", luac_path.display());
        return;
    }
    
    let file = File::open(luac_path).expect("无法打开 .luac 文件");
    let config = LuacReadConfig::default();
    let reader = config.as_reader(BufReader::new(file));
    
    // 第一次调用 get_info 应该触发解析
    let info1 = reader.get_info().expect("第一次获取 info 失败");
    
    // 第二次调用 get_info 应该返回缓存的结果
    let info2 = reader.get_info().expect("第二次获取 info 失败");
    
    // 验证两次调用返回相同的数据
    assert_eq!(info1.version, info2.version);
    assert_eq!(info1.header.format_version, info2.header.format_version);
    assert_eq!(info1.header.endianness, info2.header.endianness);
}

/// 测试无效文件的错误处理
/// 
/// 验证 LuacReader 能够正确处理无效的 `.luac` 文件
#[test]
fn test_luac_reader_invalid_file() {
    // 创建一个包含无效数据的临时文件
    let invalid_data = b"invalid luac data";
    let reader = std::io::Cursor::new(invalid_data);
    
    // 启用魔数检查
    let mut config = LuacReadConfig::default();
    config.check_magic_head = true;
    let luac_reader = config.as_reader(reader);
    
    // 尝试读取应该失败
    let info_result = luac_reader.get_info();
    println!("Info result: {:?}", info_result);
    assert!(info_result.is_err(), "读取无效文件应该失败，但实际成功了: {:?}", info_result);
    
    let result = luac_reader.finish();
    println!("Finish result: {:?}", result.result);
    assert!(result.result.is_err(), "解析无效文件应该失败，但实际成功了: {:?}", result.result);
}

/// 测试空文件的错误处理
/// 
/// 验证 LuacReader 能够正确处理空文件
#[test]
fn test_luac_reader_empty_file() {
    let empty_data = b"";
    let reader = std::io::Cursor::new(empty_data);
    
    let config = LuacReadConfig::default();
    let luac_reader = config.as_reader(reader);
    
    // 尝试读取空文件应该失败
    let info_result = luac_reader.get_info();
    println!("Info result: {:?}", info_result);
    
    // 检查是否真的失败了
    match info_result {
        Ok(info) => {
            panic!("读取空文件应该失败，但实际成功了: {:?}", info);
        }
        Err(err) => {
            println!("Error message: {}", err);
            assert!(err.to_string().contains("File is empty"), "错误消息应该包含 'File is empty'");
        }
    }
    
    let result = luac_reader.finish();
    println!("Finish result: {:?}", result.result);
    assert!(result.result.is_err(), "解析空文件应该失败，但实际成功了: {:?}", result.result);
}

/// 测试 LuacReader 与 luac_read_path 函数的集成
/// 
/// 验证通过 luac_read_path 函数读取文件的功能
#[test]
fn test_luac_read_path_integration() {
    use lua_assembler::formats::luac::luac_read_path;
    
    let luac_path = Path::new("tests/luac_read/hello.luac");
    
    // 如果 .luac 文件不存在，跳过测试
    if !luac_path.exists() {
        println!("跳过测试：{} 文件不存在", luac_path.display());
        return;
    }
    
    let result = luac_read_path(luac_path);
    assert!(result.is_ok(), "luac_read_path 失败: {:?}", result.err());
    
    let program = result.unwrap();
    println!("通过 luac_read_path 读取的程序版本: {:?}", program.header.version);
    assert_ne!(program.header.version, LuaVersion::Unknown, "程序版本不应为 Unknown");
}

/// 辅助函数：创建测试用的 .luac 文件
/// 
/// 注意：这个函数需要系统安装了 Lua 编译器
#[allow(dead_code)]
fn create_test_luac_file() {
    use std::process::Command;
    
    let lua_file = "tests/luac_read/hello.lua";
    let luac_file = "tests/luac_read/hello.luac";
    
    // 尝试使用 luac 编译 .lua 文件
    let output = Command::new("luac")
        .args(&["-o", luac_file, lua_file])
        .output();
    
    match output {
        Ok(output) => {
            if output.status.success() {
                println!("成功创建测试文件: {}", luac_file);
            } else {
                println!("创建测试文件失败: {}", String::from_utf8_lossy(&output.stderr));
            }
        }
        Err(e) => {
            println!("无法执行 luac 命令: {}", e);
            println!("请确保系统已安装 Lua 编译器");
        }
    }
}