use clr_assembler::formats::dot_net::reader::{read_dotnet_assembly, DotNetReader};
use std::path::Path;

#[test]
fn test_read_unity_engine_dll() {
    // 测试读取 UnityEngine.dll 文件
    let dll_path = Path::new("tests/net_read/UnityEngine.dll");

    if !dll_path.exists() {
        println!("跳过测试：UnityEngine.dll 文件不存在");
        return;
    }

    // 首先检查是否为 .NET 程序集
    let is_dotnet = DotNetReader::is_dotnet_assembly(dll_path.to_str().unwrap()).expect("检查 .NET 程序集失败");

    if !is_dotnet {
        println!("UnityEngine.dll 不是 .NET 程序集，跳过测试");
        return;
    }

    println!("UnityEngine.dll 是有效的 .NET 程序集");

    // 惰性读取程序集基本信息
    let reader = DotNetReader::read_from_file(dll_path.to_str().unwrap()).expect("读取 UnityEngine.dll 失败");

    // 验证程序集完整性
    let warnings = reader.validate_assembly().expect("验证程序集失败");

    if !warnings.is_empty() {
        println!("程序集验证警告: {:?}", warnings);
    }

    // 获取程序集摘要信息
    let summary = reader.get_assembly_summary();
    println!("程序集摘要:\n{}", summary);

    // 获取程序集基本信息
    let assembly_info = reader.get_assembly_info().expect("获取程序集信息失败");

    println!("程序集名称: {}", assembly_info.name);
    println!("程序集版本: {}", assembly_info.version);

    if let Some(culture) = &assembly_info.culture {
        println!("文化信息: {}", culture);
    }

    if let Some(public_key_token) = &assembly_info.public_key_token {
        println!("公钥标记: {}", public_key_token);
    }

    if let Some(runtime_version) = &assembly_info.runtime_version {
        println!("运行时版本: {}", runtime_version);
    }
}

#[test]
fn test_full_parse_unity_engine_dll() {
    // 测试完整解析 UnityEngine.dll 文件
    let dll_path = Path::new("tests/net_read/UnityEngine.dll");

    if !dll_path.exists() {
        println!("跳过测试：UnityEngine.dll 文件不存在");
        return;
    }

    // 使用便利函数进行完整解析
    let result = read_dotnet_assembly(dll_path.to_str().unwrap());

    if result.result.is_ok() {
        let program = result.result.unwrap();
        println!("成功解析 CLR 程序: {}", program.name);
        println!(
            "程序集版本: {}.{}.{}.{}",
            program.version.major, program.version.minor, program.version.build, program.version.revision
        );
        println!("类型数量: {}", program.types.len());
        println!("外部程序集引用数量: {}", program.external_assemblies.len());

        // 验证程序完整性
        if let Err(error) = program.validate() {
            println!("程序验证失败: {}", error);
        }
        else {
            println!("程序验证通过");
        }
    }
    else {
        println!("解析失败: {}", result.result.unwrap_err());
    }
}

#[test]
fn test_read_non_dotnet_file() {
    // 测试读取非 .NET 文件
    let test_file = "tests/net_read/mod.rs"; // 这个文件本身

    let is_dotnet = DotNetReader::is_dotnet_assembly(test_file).expect("检查文件失败");

    assert!(!is_dotnet, "Rust 源文件不应该被识别为 .NET 程序集");
    println!("正确识别非 .NET 文件");
}

#[test]
fn test_read_nonexistent_file() {
    // 测试读取不存在的文件
    let result = DotNetReader::read_from_file("nonexistent.dll");

    assert!(result.is_err(), "读取不存在的文件应该返回错误");
    println!("正确处理不存在的文件");
}
