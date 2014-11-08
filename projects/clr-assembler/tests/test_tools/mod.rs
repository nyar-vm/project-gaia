//! 辅助功能模块
//!
//! 提供测试辅助功能和其他工具函数

use serde::{Deserialize, Serialize};
use std::fs::File;

use clr_assembler::formats::msil::{
    ast::{MsilRoot, MsilStatement},
    parser::MsilParser,
    MsilReadConfig,
};
use gaia_types::GaiaError;
use serde_json::{ser::PrettyFormatter, Serializer};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// MSIL 文件期望结构体 - 用于定义测试期望

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MsilExpected {
    /// 期望的程序集名称
    pub assembly_name: Option<String>,
    /// 期望的外部程序集
    pub extern_assemblies: Vec<String>,
    /// 期望的模块名称
    pub module_name: Option<String>,
    /// 期望的类名
    pub class_name: Option<String>,
    /// 期望的类修饰符
    pub class_modifiers: Vec<String>,
    /// 期望的基类
    pub extends: Option<String>,
    /// 期望的方法数量
    pub method_count: usize,
    /// 期望的方法名列表
    pub method_names: Vec<String>,
    /// 期望的语句数量
    pub statement_count: usize,
    /// 文件路径（用于调试）
    #[serde(skip)]
    pub file_path: PathBuf,
}

impl MsilExpected {
    /// 从 MSIL AST 自动生成期望对象
    pub fn from_ast(ast: &MsilRoot, file_path: &Path) -> Self {
        let mut assembly_name = None;
        let mut extern_assemblies = Vec::new();
        let mut module_name = None;
        let mut class_name = None;
        let mut class_modifiers = Vec::new();
        let mut extends = None;
        let mut method_count = 0;
        let mut method_names = Vec::new();

        for statement in &ast.statements {
            match statement {
                MsilStatement::AssemblyExtern(name) => {
                    extern_assemblies.push(name.clone());
                }
                MsilStatement::Assembly(name) => {
                    assembly_name = Some(name.clone());
                }
                MsilStatement::Module(name) => {
                    module_name = Some(name.clone());
                }
                MsilStatement::Class(class) => {
                    class_name = Some(class.name.clone());
                    class_modifiers = class.modifiers.clone();
                    extends = class.extends.clone();
                    method_count = class.methods.len();
                    method_names = class.methods.iter().map(|m| m.name.clone()).collect();
                }
            }
        }

        Self {
            assembly_name,
            extern_assemblies,
            module_name,
            class_name,
            class_modifiers,
            extends,
            method_count,
            method_names,
            statement_count: ast.statements.len(),
            file_path: file_path.to_path_buf(),
        }
    }

    /// 创建新的空期望对象
    pub fn new() -> Self {
        Self {
            assembly_name: None,
            extern_assemblies: Vec::new(),
            module_name: None,
            class_name: None,
            class_modifiers: Vec::new(),
            extends: None,
            method_count: 0,
            method_names: Vec::new(),
            statement_count: 0,
            file_path: PathBuf::new(),
        }
    }

    /// 保存期望到 JSON 文件
    pub fn save_to_json(&self, json_path: &str) -> Result<(), GaiaError> {
        let file = File::create(json_path)?;
        let mut json = Serializer::with_formatter(file, PrettyFormatter::with_indent(b"    "));
        self.serialize(&mut json).map_err(GaiaError::not_implemented)?;
        Ok(())
    }

    /// 从 JSON 文件加载期望
    pub fn load_from_json(json_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let json_content = std::fs::read_to_string(json_path)?;
        let expected: Self = serde_json::from_str(&json_content)?;
        Ok(expected)
    }

    /// 验证当前 AST 是否符合期望
    pub fn validate_ast(&self, ast: &MsilRoot) -> Result<(), String> {
        // 验证语句数量
        if ast.statements.len() != self.statement_count {
            return Err(format!("语句数量不匹配: 期望 {}, 实际 {}", self.statement_count, ast.statements.len()));
        }

        let mut found_extern_assemblies = Vec::new();
        let mut found_assembly_name = None;
        let mut found_module_name = None;
        let mut found_class = None;

        for statement in &ast.statements {
            match statement {
                MsilStatement::AssemblyExtern(name) => {
                    found_extern_assemblies.push(name.clone());
                }
                MsilStatement::Assembly(name) => {
                    found_assembly_name = Some(name.clone());
                }
                MsilStatement::Module(name) => {
                    found_module_name = Some(name.clone());
                }
                MsilStatement::Class(class) => {
                    found_class = Some(class.clone());
                }
            }
        }

        // 验证外部程序集
        if found_extern_assemblies != self.extern_assemblies {
            return Err(format!("外部程序集不匹配: 期望 {:?}, 实际 {:?}", self.extern_assemblies, found_extern_assemblies));
        }

        // 验证程序集名称
        if found_assembly_name != self.assembly_name {
            return Err(format!("程序集名称不匹配: 期望 {:?}, 实际 {:?}", self.assembly_name, found_assembly_name));
        }

        // 验证模块名称
        if found_module_name != self.module_name {
            return Err(format!("模块名称不匹配: 期望 {:?}, 实际 {:?}", self.module_name, found_module_name));
        }

        // 验证类信息
        if let Some(expected_class_name) = &self.class_name {
            if found_class.is_none() {
                return Err("未找到期望的类定义".to_string());
            }

            let class = found_class.as_ref().unwrap();

            if &class.name != expected_class_name {
                return Err(format!("类名不匹配: 期望 '{}', 实际 '{}'", expected_class_name, class.name));
            }

            if class.modifiers != self.class_modifiers {
                return Err(format!("类修饰符不匹配: 期望 {:?}, 实际 {:?}", self.class_modifiers, class.modifiers));
            }

            if class.extends != self.extends {
                return Err(format!("基类不匹配: 期望 {:?}, 实际 {:?}", self.extends, class.extends));
            }

            if class.methods.len() != self.method_count {
                return Err(format!("方法数量不匹配: 期望 {}, 实际 {}", self.method_count, class.methods.len()));
            }

            let actual_method_names: Vec<String> = class.methods.iter().map(|m| m.name.clone()).collect();
            if actual_method_names != self.method_names {
                return Err(format!("方法名不匹配: 期望 {:?}, 实际 {:?}", self.method_names, actual_method_names));
            }
        }

        Ok(())
    }
}

/// 获取 MSIL 文件对应的 JSON 期望文件路径
pub fn get_expected_json_path(path: &Path) -> String {
    let parent = path.parent().unwrap_or(Path::new("."));
    let stem = path.file_stem().unwrap().to_string_lossy();
    parent.join(format!("{}.expected.json", stem)).to_string_lossy().to_string()
}

pub fn validate_msil_files(folder: &Path) {
    let mut test_count = 0;
    let mut success_count = 0;
    let mut failed_tests = Vec::new();

    for entry in WalkDir::new(folder)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "msil"))
    {
        let msil_path = entry.path();
        let test_name = entry.path().file_stem().unwrap_or_default().to_string_lossy().to_string();

        test_count += 1;

        match compare_msil_file(&test_name, &msil_path) {
            Ok(()) => {
                success_count += 1;
                println!("✓ 测试通过: {}", test_name);
            }
            Err(e) => {
                failed_tests.push((test_name.clone(), e.to_string()));
                println!("✗ 测试失败: {} - {}", test_name, e);
            }
        }
    }

    println!("\n=== 测试总结 ===");
    println!("总测试数: {}", test_count);
    println!("成功数: {}", success_count);
    println!("失败数: {}", failed_tests.len());

    if !failed_tests.is_empty() {
        println!("\n失败的测试:");
        for (name, error) in &failed_tests {
            println!("  - {}: {}", name, error);
        }
    }

    // 如果有失败的测试，让测试失败
    if !failed_tests.is_empty() {
        panic!("有 {} 个测试失败", failed_tests.len());
    }
}

/// 自动化的 MSIL 文件测试函数
///
/// # 参数
/// * `test_name` - 测试名称，用于显示
/// * `msil_file_path` - MSIL 文件路径
/// * `force_regenerate` - 是否强制重新生成期望文件
pub fn compare_msil_file(test_name: &str, file_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 测试 {} ===", test_name);

    // 读取 MSIL 文件
    let msil_content = std::fs::read_to_string(file_path)?;

    // 创建解析器并解析
    let config = MsilReadConfig::default();
    let parser = MsilParser::new(&config);
    let ast_result = parser.parse_text(&msil_content);

    if let Err(e) = ast_result.result.as_ref() {
        return Err(format!("语法分析失败: {:?}", e).into());
    }

    let ast = ast_result.result.unwrap();
    let json_path = get_expected_json_path(file_path);

    // 检查是否存在期望文件
    if !Path::new(&json_path).exists() {
        // 首次运行或强制重新生成，创建期望文件
        let expected = MsilExpected::from_ast(&ast, file_path);
        expected.save_to_json(&json_path)?;
        println!("✓ 已生成期望文件: {}", json_path);
        println!("✓ 解析成功: {} 个语句", ast.statements.len());

        // 显示解析结果摘要
        print_ast_summary(&ast);
        return Ok(());
    }

    // 加载现有期望文件并验证
    let expected = MsilExpected::load_from_json(&json_path)?;
    println!("✓ 已加载期望文件: {}", json_path);

    match expected.validate_ast(&ast) {
        Ok(()) => {
            println!("✓ 验证通过: 解析结果符合期望");
            print_ast_summary(&ast);
        }
        Err(e) => {
            println!("✗ 验证失败: {}", e);
            println!("\n当前解析结果:");
            print_ast_summary(&ast);
            println!("\n期望结果:");
            println!("{:#?}", expected);
            return Err(e.into());
        }
    }

    Ok(())
}

/// 打印 AST 摘要信息
pub fn print_ast_summary(ast: &MsilRoot) {
    println!("解析结果摘要:");
    for (i, statement) in ast.statements.iter().enumerate() {
        match statement {
            MsilStatement::AssemblyExtern(name) => {
                println!("  {}. 外部程序集: {}", i + 1, name);
            }
            MsilStatement::Assembly(name) => {
                println!("  {}. 程序集: {}", i + 1, name);
            }
            MsilStatement::Module(name) => {
                println!("  {}. 模块: {}", i + 1, name);
            }
            MsilStatement::Class(class) => {
                println!(
                    "  {}. 类: {} (修饰符: {:?}, 基类: {:?}, 方法数: {})",
                    i + 1,
                    class.name,
                    class.modifiers,
                    class.extends,
                    class.methods.len()
                );
                for (j, method) in class.methods.iter().enumerate() {
                    println!(
                        "    {}.{} 方法: {} (修饰符: {:?}, 返回类型: {}, 参数数: {})",
                        i + 1,
                        j + 1,
                        method.name,
                        method.modifiers,
                        method.return_type,
                        method.parameters.len()
                    );
                }
            }
        }
    }
}

/// 使用期望对象的 MSIL 文件测试函数（保留用于向后兼容）
///
/// # 参数
/// * `test_name` - 测试名称，用于显示
/// * `msil_file_path` - MSIL 文件路径
/// * `expected` - 期望对象
pub fn test_msil_file_with_expectation(test_name: &str, msil_file_path: &str, expected: &MsilExpected) {
    println!("\n=== 测试 {} ===", test_name);

    // 读取 MSIL 文件
    let msil_content = std::fs::read_to_string(msil_file_path).expect(&format!("无法读取文件: {}", msil_file_path));

    // 创建解析器并解析
    let config = MsilReadConfig::default();
    let parser = MsilParser::new(&config);
    let ast_result = parser.parse_text(&msil_content);

    assert!(ast_result.result.is_ok(), "语法分析失败: {:?}", ast_result.result.as_ref().err());
    let ast = ast_result.result.unwrap();

    // 使用新的验证方法
    match expected.validate_ast(&ast) {
        Ok(()) => {
            println!("✓ 验证通过: 解析结果符合期望");
            print_ast_summary(&ast);
        }
        Err(e) => {
            panic!("验证失败: {}", e);
        }
    }
}
