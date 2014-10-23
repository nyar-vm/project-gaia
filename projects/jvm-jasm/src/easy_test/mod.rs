//! 辅助功能模块
//!
//! 提供测试辅助功能和其他工具函数

use crate::{ast::JasmRoot, lexer::JasmLexer, parser::JasmParser};
use gaia_types::GaiaError;
use serde::{Deserialize, Serialize};
use serde_json::{ser::PrettyFormatter, Serializer};
use std::{
    fs::File,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

/// JASM 文件期望结构体 - 用于定义测试期望
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JasmExpected {
    /// 期望的类名
    pub class_name: String,
    /// 期望的类修饰符
    pub class_modifiers: Vec<String>,
    /// 期望的方法数量
    pub method_count: usize,
    /// 期望的字段数量
    pub field_count: usize,
    /// 期望包含的特定方法（方法名和描述符）
    pub expected_methods: Vec<String>,
    /// 期望包含的特定字段
    pub expected_fields: Vec<String>,
    /// 期望包含的语法糖特征
    pub syntactic_sugar_features: Vec<(String, bool)>, // (特征, 是否应该存在)
    /// 是否检查 token 类型
    pub check_tokens: bool,
    /// 期望的源文件名
    pub source_file: Option<String>,
    /// 文件路径（用于调试）
    #[serde(skip)]
    pub file_path: PathBuf,
}

impl JasmExpected {
    /// 从 JASM AST 自动生成期望对象
    pub fn from_ast(ast: &JasmRoot, file_path: &Path) -> Self {
        let mut expected = Self {
            class_name: ast.class.name.clone(),
            class_modifiers: ast.class.modifiers.clone(),
            method_count: ast.class.methods.len(),
            field_count: ast.class.fields.len(),
            expected_methods: ast.class.methods.iter().map(|m| m.name_and_descriptor.clone()).collect(),
            expected_fields: ast.class.fields.iter().map(|f| f.name_and_descriptor.clone()).collect(),
            syntactic_sugar_features: Vec::new(),
            check_tokens: true,
            source_file: ast.class.source_file.clone(),
            file_path: file_path.to_path_buf(),
        };

        expected
    }

    /// 创建新的空期望对象
    pub fn new() -> Self {
        Self {
            class_name: String::new(),
            class_modifiers: Vec::new(),
            method_count: 0,
            field_count: 0,
            expected_methods: Vec::new(),
            expected_fields: Vec::new(),
            syntactic_sugar_features: Vec::new(),
            check_tokens: true,
            source_file: None,
            file_path: PathBuf::new(),
        }
    }

    /// 保存期望到 JSON 文件
    pub fn save_to_json(&self, json_path: &str) -> Result<(), Box<dyn std::error::Error>> {
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
    pub fn validate_ast(&self, ast: &JasmRoot) -> Result<(), String> {
        // 验证类名
        if ast.class.name != self.class_name {
            return Err(format!("类名不匹配: 期望 '{}', 实际 '{}'", self.class_name, ast.class.name));
        }

        // 验证类修饰符
        for modifier in &self.class_modifiers {
            if !ast.class.modifiers.contains(modifier) {
                return Err(format!("缺少类修饰符 '{}'，实际修饰符: {:?}", modifier, ast.class.modifiers));
            }
        }

        // 验证方法数量
        if ast.class.methods.len() != self.method_count {
            return Err(format!("方法数量不匹配: 期望 {}, 实际 {}", self.method_count, ast.class.methods.len()));
        }

        // 验证字段数量
        if ast.class.fields.len() != self.field_count {
            return Err(format!("字段数量不匹配: 期望 {}, 实际 {}", self.field_count, ast.class.fields.len()));
        }

        // 验证特定方法
        for expected_method in &self.expected_methods {
            if !ast.class.methods.iter().any(|m| m.name_and_descriptor.contains(expected_method)) {
                return Err(format!("缺少期望的方法: {}", expected_method));
            }
        }

        // 验证特定字段
        for expected_field in &self.expected_fields {
            if !ast.class.fields.iter().any(|f| f.name_and_descriptor.contains(expected_field)) {
                return Err(format!("缺少期望的字段: {}", expected_field));
            }
        }

        // 验证源文件
        if self.source_file != ast.class.source_file {
            return Err(format!("源文件名不匹配: 期望 '{:?}', 实际 '{:?}'", self.source_file, ast.class.source_file));
        }

        Ok(())
    }
}

/// 获取 JASM 文件对应的 JSON 期望文件路径
pub fn get_expected_json_path(path: &Path) -> String {
    let parent = path.parent().unwrap_or(Path::new("."));
    let stem = path.file_stem().unwrap().to_string_lossy();
    parent.join(format!("{}.expected.json", stem)).to_string_lossy().to_string()
}

/// 验证指定文件夹中的所有 JASM 文件
pub fn validate_jasm_files(folder: &Path) {
    let mut test_count = 0;
    let mut success_count = 0;
    let mut failed_tests = Vec::new();

    for entry in WalkDir::new(folder)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "jasm"))
    {
        let jasm_path = entry.path();
        let test_name = entry.path().file_stem().unwrap_or_default().to_string_lossy().to_string();

        test_count += 1;

        match compare_jasm_file(&test_name, &jasm_path) {
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

/// 自动化的 JASM 文件测试函数
///
/// # 参数
/// * `test_name` - 测试名称，用于显示
/// * `file_path` - JASM 文件路径
pub fn compare_jasm_file(test_name: &str, file_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n--- 测试文件: {} ---", file_path.display());

    // 读取 JASM 文件
    let file_bytes = std::fs::read(file_path)?;

    // 检查并处理BOM
    let jasm_content = if file_bytes.len() >= 2 && file_bytes[0] == 0xFF && file_bytes[1] == 0xFE {
        // UTF-16 LE BOM
        let utf16_bytes: Vec<u16> =
            file_bytes[2..].chunks_exact(2).map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]])).collect();
        String::from_utf16(&utf16_bytes)?
    }
    else if file_bytes.len() >= 3 && file_bytes[0] == 0xEF && file_bytes[1] == 0xBB && file_bytes[2] == 0xBF {
        // UTF-8 BOM
        String::from_utf8(file_bytes[3..].to_vec())?
    }
    else {
        // 无BOM，尝试UTF-8
        String::from_utf8(file_bytes)?
    };

    // 创建 lexer 和 parser
    let lexer = JasmLexer::new();
    let parser = JasmParser::new();

    // 测试词法分析
    let tokens_result = lexer.tokenize(&jasm_content);
    if let Err(e) = tokens_result.result.as_ref() {
        return Err(format!("词法分析失败: {:?}", e).into());
    }

    let tokens = tokens_result.result.unwrap();
    println!("词法分析成功: {} 个 tokens", tokens.tokens.get_ref().len());

    // 测试语法分析
    let ast_result = parser.parse(tokens);
    if let Err(e) = ast_result.result.as_ref() {
        return Err(format!("语法分析失败: {:?}", e).into());
    }

    let ast = ast_result.result.unwrap();
    let json_path = get_expected_json_path(file_path);

    // 检查是否存在期望文件
    if !Path::new(&json_path).exists() {
        // 首次运行，创建期望文件
        let expected = JasmExpected::from_ast(&ast, file_path);
        expected.save_to_json(&json_path)?;
        println!("✓ 已生成期望文件: {}", json_path);
        println!("解析成功: {} 类, {} 个方法, {} 个字段", ast.class.name, ast.class.methods.len(), ast.class.fields.len());
        print_ast_summary(&ast);
        return Ok(());
    }

    // 加载现有期望文件并验证
    let expected = JasmExpected::load_from_json(&json_path)?;
    println!("✓ 已加载期望文件: {}", json_path);

    // 验证方法数量（允许不匹配但给出警告）
    if ast.class.methods.len() != expected.method_count {
        println!("警告: 方法数量不匹配: 期望 {}, 实际 {} (继续测试)", expected.method_count, ast.class.methods.len());
    }

    match expected.validate_ast(&ast) {
        Ok(()) => {
            println!("解析成功: {} 类, {} 个方法, {} 个字段", ast.class.name, ast.class.methods.len(), ast.class.fields.len());
            println!("✓ {} 测试通过", test_name);
        }
        Err(e) => {
            println!("✗ 验证失败: {}", e);
            println!("\n当前解析结果:");
            print_ast_summary(&ast);
            return Err(e.into());
        }
    }

    Ok(())
}

/// 打印 AST 摘要信息
pub fn print_ast_summary(ast: &JasmRoot) {
    println!("解析结果摘要:");
    println!("  类名: {}", ast.class.name);
    println!("  类修饰符: {:?}", ast.class.modifiers);
    if let Some(source_file) = &ast.class.source_file {
        println!("  源文件: {}", source_file);
    }

    println!("  方法数: {}", ast.class.methods.len());
    for (i, method) in ast.class.methods.iter().enumerate() {
        println!("    {}. {}", i + 1, method.name_and_descriptor);
    }

    println!("  字段数: {}", ast.class.fields.len());
    for (i, field) in ast.class.fields.iter().enumerate() {
        println!("    {}. {}", i + 1, field.name_and_descriptor);
    }
}

/// 使用期望对象的 JASM 文件测试函数（保留用于向后兼容）
///
/// # 参数
/// * `test_name` - 测试名称，用于显示
/// * `jasm_file_path` - JASM 文件路径
/// * `expected` - 期望对象
pub fn test_jasm_file_with_expectation(test_name: &str, jasm_file_path: &str, expected: &JasmExpected) {
    println!("\n=== 测试 {} ===", test_name);

    // 读取 JASM 文件
    let file_bytes = std::fs::read(jasm_file_path).expect(&format!("无法读取文件: {}", jasm_file_path));

    // 检查并处理BOM
    let jasm_content = if file_bytes.len() >= 2 && file_bytes[0] == 0xFF && file_bytes[1] == 0xFE {
        // UTF-16 LE BOM
        let utf16_bytes: Vec<u16> =
            file_bytes[2..].chunks_exact(2).map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]])).collect();
        String::from_utf16(&utf16_bytes).expect(&format!("无法解析UTF-16文件: {}", jasm_file_path))
    }
    else if file_bytes.len() >= 3 && file_bytes[0] == 0xEF && file_bytes[1] == 0xBB && file_bytes[2] == 0xBF {
        // UTF-8 BOM
        String::from_utf8(file_bytes[3..].to_vec()).expect(&format!("无法解析UTF-8文件: {}", jasm_file_path))
    }
    else {
        // 无BOM，尝试UTF-8
        String::from_utf8(file_bytes).expect(&format!("无法解析文件: {}", jasm_file_path))
    };

    // 创建 lexer 和 parser
    let lexer = JasmLexer::new();
    let parser = JasmParser::new();

    // 测试词法分析
    let tokens_result = lexer.tokenize(&jasm_content);
    assert!(tokens_result.result.is_ok(), "词法分析失败: {:?}", tokens_result.result.as_ref().err());

    let tokens = tokens_result.result.unwrap();
    println!("词法分析成功: {} 个 tokens", tokens.tokens.get_ref().len());

    // 测试语法分析
    let ast_result = parser.parse(tokens);
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
