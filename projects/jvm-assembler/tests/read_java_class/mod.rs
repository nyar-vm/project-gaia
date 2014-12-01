use crate::test_tools::test_path;
use gaia_types::{
    helpers::{open_file, save_json},
    GaiaError,
};
use jvm_assembler::{formats::class::ClassReadConfig, program::JvmVersion};
use serde::{Deserialize, Serialize};
use std::{path::Path, process::Command};
use walkdir::WalkDir;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClassExpected {
    /// 期望的类名
    class_name: String,
    /// 类版本信息
    class_version: JvmVersion,
    /// 访问标志（是否为 public、final、abstract 等）
    access_flags: Vec<String>,
    /// 超类名称
    super_class: Option<String>,
    /// 实现的接口列表
    interfaces: Vec<String>,
    /// 方法数量
    method_count: usize,
    /// 字段数量
    field_count: usize,
    /// 源文件名
    source_file: Option<String>,
    /// 常量池数量
    constant_pool_count: usize,
    /// 字段详细信息
    fields: Vec<FieldInfo>,
    /// 方法详细信息
    methods: Vec<MethodInfo>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldInfo {
    /// 字段名
    name: String,
    /// 字段描述符
    descriptor: String,
    /// 访问标志
    access_flags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MethodInfo {
    /// 方法名
    name: String,
    /// 方法描述符
    descriptor: String,
    /// 访问标志
    access_flags: Vec<String>,
}

fn generate_class() -> Result<(), GaiaError> {
    let base_dir = test_path("read_java_class");
    let generated_dir = base_dir.join("generated");
    std::fs::create_dir_all(&generated_dir)?;

    // Compile JavaClassGenerator.java itself
    let javac_creator_output = Command::new("javac").current_dir(&base_dir).arg(base_dir.join("JavaClassGenerator.java")).output()?;

    if !javac_creator_output.status.success() {
        eprintln!("stdout: {}", String::from_utf8_lossy(&javac_creator_output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&javac_creator_output.stderr));
        return Err(GaiaError::custom_error(format!(
            "Failed to compile JavaClassGenerator.java: stdout: {}, stderr: {}",
            String::from_utf8_lossy(&javac_creator_output.stdout),
            String::from_utf8_lossy(&javac_creator_output.stderr)
        )));
    }

    // Run ClassCreator to generate and compile .java files into generated_dir
    let java_creator_output = Command::new("java").current_dir(&base_dir).arg("JavaClassGenerator").output()?;

    if !java_creator_output.status.success() {
        eprintln!("stdout: {}", String::from_utf8_lossy(&java_creator_output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&java_creator_output.stderr));
        return Err(GaiaError::custom_error(format!(
            "Failed to run ClassCreator: stdout: {}, stderr: {}",
            String::from_utf8_lossy(&java_creator_output.stdout),
            String::from_utf8_lossy(&java_creator_output.stderr)
        )));
    }

    Ok(())
}

#[test]
fn assert_classes_info() -> Result<(), GaiaError> {
    if let Err(e) = generate_class() {
        // 即便生成失败，也继续测试已缓存的 class 文件
        eprintln!("{}", e);
    };
    let mut is_success = true;
    let test_dir = test_path("read_java_class");
    std::fs::remove_file(test_dir.join("JavaClassGenerator.class"))?;
    for entry in WalkDir::new(test_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "class"))
    {
        let class_path = entry.path();
        let class_name = class_path.file_stem().unwrap_or_default().to_string_lossy().to_string();

        println!("解析 class 文件: {}", class_path.display());

        // 读取 class 文件
        match read_class(class_path) {
            Ok(actual) => {
                // 生成对应的 JSON 文件路径
                let json_path = class_path.with_extension("expected.json");

                // 检查 JSON 文件是否存在
                if json_path.exists() {
                    // 如果存在，比较结果
                    let expected_content = std::fs::read_to_string(&json_path)?;
                    let expected: ClassExpected = serde_json::from_str(&expected_content)?;

                    if actual == expected {
                        println!("✓ {} 测试通过", class_name);
                    }
                    else {
                        println!("✗ {} 测试失败 - 结果不匹配", class_name);
                        is_success = false;
                    }
                }
                else {
                    save_json(&actual, &json_path)?;
                }
            }
            Err(e) => {
                println!("✗ {} 读取失败: {}", class_name, e);
                return Err(e);
            }
        }
    }
    assert!(is_success);
    Ok(())
}

fn read_class(path: &Path) -> Result<ClassExpected, GaiaError> {
    let (file, _url) = open_file(path)?;
    let config = ClassReadConfig {};
    let reader = config.as_reader(file);
    let info = reader.get_info()?;
    let prog = reader.get_program()?;

    // 提取字段信息
    let fields: Vec<FieldInfo> = prog.fields.iter().map(|field| {
        FieldInfo {
            name: field.name.clone(),
            descriptor: field.descriptor.clone(),
            access_flags: field.access_flags.to_modifiers(),
        }
    }).collect();

    // 提取方法信息
    let methods: Vec<MethodInfo> = prog.methods.iter().map(|method| {
        MethodInfo {
            name: method.name.clone(),
            descriptor: method.descriptor.clone(),
            access_flags: method.access_flags.to_modifiers(),
        }
    }).collect();

    // 创建 ClassExpected 结构体，包含更多关键信息
    let class_expected = ClassExpected {
        class_name: prog.name.clone(),
        class_version: info.version.clone(),
        access_flags: prog.access_flags.to_modifiers(),
        super_class: prog.super_class.clone(),
        interfaces: prog.interfaces.clone(),
        method_count: prog.methods.len(),
        field_count: prog.fields.len(),
        source_file: prog.source_file.clone(),
        constant_pool_count: prog.constant_pool.entries.len(),
        fields,
        methods,
    };
    Ok(class_expected)
}
