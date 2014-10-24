use crate::exports::types::*;

/// 获取版本信息
pub fn get_version() -> String {
    "Gaia Assembler v0.1.0".to_string()
}

/// 格式化描述符
pub fn format_descriptor(descriptor: GaiaDescriptor, options: FormatOptions) -> Result<GaiaDescriptor, Vec<Diagnostic>> {
    // 对描述符进行格式化处理
    let mut formatted_descriptor = descriptor;

    // 格式化指令序列
    for instruction_entry in &mut formatted_descriptor.instructions {
        format_instruction_entry(instruction_entry, &options);
    }

    // 格式化符号信息
    for symbol in &mut formatted_descriptor.symbols {
        format_symbol_info(symbol, &options);
    }

    Ok(formatted_descriptor)
}

/// 获取代码补全建议
pub fn get_completions(descriptor: GaiaDescriptor, position: SourceLocation, target: Target) -> Vec<CompletionItem> {
    let mut completions = Vec::new();

    // 基于描述符上下文提供补全建议
    let context = analyze_completion_context_from_descriptor(&descriptor, &position);

    match context {
        CompletionContext::Instruction => {
            add_instruction_completions(&mut completions);
        }
        CompletionContext::Type => {
            add_type_completions(&mut completions);
        }
        CompletionContext::Variable => {
            add_variable_completions_from_descriptor(&mut completions, &descriptor);
        }
        CompletionContext::Function => {
            add_function_completions_from_descriptor(&mut completions, &descriptor);
        }
        CompletionContext::General => {
            add_instruction_completions(&mut completions);
            add_type_completions(&mut completions);
        }
    }

    completions
}

/// 优化描述符
pub fn optimize_descriptor(
    descriptor: GaiaDescriptor,
    options: OptimizationOptions,
) -> Result<GaiaDescriptor, Vec<Diagnostic>> {
    let mut optimized = descriptor;

    // 根据优化选项应用不同的优化策略
    if options.constant_folding.unwrap_or(true) {
        optimized = fold_constants_in_descriptor(optimized);
    }

    if options.dead_code_elimination.unwrap_or(true) {
        optimized = eliminate_dead_code_in_descriptor(optimized);
    }

    if options.instruction_combining.unwrap_or(false) {
        optimized = combine_instructions(optimized);
    }

    Ok(optimized)
}

/// 验证一致性
pub fn validate_consistency(descriptor: GaiaDescriptor) -> ValidationResult {
    let mut errors = vec![];
    let mut warnings = vec![];

    // 验证符号引用的一致性
    for instruction_entry in &descriptor.instructions {
        if let Some(symbol_name) = extract_symbol_reference_from_instruction(&instruction_entry.instruction) {
            if !descriptor.symbols.iter().any(|s| s.name == symbol_name) {
                errors.push(ValidationError {
                    message: format!("未定义的符号引用: {}", symbol_name),
                    location: instruction_entry.location.clone(),
                    error_code: "E001".to_string(),
                });
            }
        }
    }

    // 验证控制流的一致性
    for edge in &descriptor.control_flow.edges {
        let from_exists = descriptor.control_flow.nodes.iter().any(|n| n.id == edge.from);
        let to_exists = descriptor.control_flow.nodes.iter().any(|n| n.id == edge.to);

        if !from_exists || !to_exists {
            errors.push(ValidationError {
                message: format!("控制流边引用了不存在的节点: {} -> {}", edge.from, edge.to),
                location: None,
                error_code: "E002".to_string(),
            });
        }
    }

    // 检查潜在的问题
    if descriptor.symbols.is_empty() {
        warnings.push(ValidationWarning {
            message: "程序没有定义任何符号".to_string(),
            location: None,
            warning_code: "W001".to_string(),
        });
    }

    ValidationResult { is_valid: errors.is_empty(), errors, warnings }
}

/// 转换格式
pub fn convert_format(descriptor: GaiaDescriptor, target_format: DescriptorFormat) -> Result<GaiaDescriptor, Vec<Diagnostic>> {
    let mut converted = descriptor;

    match target_format {
        DescriptorFormat::Binary => {
            // 转换为二进制格式的逻辑
            // 这里只是占位符实现
        }
        DescriptorFormat::Json => {
            // 转换为 JSON 格式的逻辑
            // 这里只是占位符实现
        }
        DescriptorFormat::Xml => {
            // 转换为 XML 格式的逻辑
            // 这里只是占位符实现
        }
        DescriptorFormat::Yaml => {
            // 转换为 YAML 格式的逻辑
            // 这里只是占位符实现
        }
    }

    Ok(converted)
}

fn apply_formatting(node: &AstNode, options: &FormatOptions, depth: usize) -> AstNode {
    let mut formatted_node = node.clone();

    // 格式化子节点
    formatted_node.children = node.children.iter().map(|child| apply_formatting(child, options, depth + 1)).collect();

    // 根据节点类型应用特定的格式化规则
    match node.node_type {
        AstNodeType::FunctionDef => {
            // 函数定义的格式化
            format_function_definition(&mut formatted_node, options);
        }
        AstNodeType::Block => {
            // 代码块的格式化
            format_block(&mut formatted_node, options, depth);
        }
        AstNodeType::VariableDecl => {
            // 变量声明的格式化
            format_variable_declaration(&mut formatted_node, options);
        }
        _ => {}
    }

    formatted_node
}

fn format_function_definition(node: &mut AstNode, _options: &FormatOptions) {
    // 函数定义格式化逻辑
    if let Some(ref mut value) = node.value {
        *value = value.trim().to_string();
    }
}

fn format_block(node: &mut AstNode, _options: &FormatOptions, _depth: usize) {
    // 代码块格式化逻辑
    // 确保块内语句正确缩进
}

fn format_variable_declaration(node: &mut AstNode, _options: &FormatOptions) {
    // 变量声明格式化逻辑
    if let Some(ref mut value) = node.value {
        *value = value.trim().to_string();
    }
}

#[derive(Debug)]
enum CompletionContext {
    Instruction,
    Type,
    Variable,
    Function,
    General,
}

fn analyze_completion_context(ast: &AstNode, position: &SourceLocation) -> CompletionContext {
    // 分析光标位置的上下文
    if let Some(node) = find_node_at_position(ast, position) {
        match node.node_type {
            AstNodeType::FunctionDef => CompletionContext::Function,
            AstNodeType::VariableDecl => CompletionContext::Type,
            AstNodeType::Instruction => CompletionContext::Instruction,
            _ => CompletionContext::General,
        }
    }
    else {
        CompletionContext::General
    }
}

fn find_node_at_position(node: &AstNode, position: &SourceLocation) -> Option<&AstNode> {
    // 查找指定位置的 AST 节点
    if let Some(ref range) = node.location {
        if is_position_in_range(position, range) {
            // 首先检查子节点
            for child in &node.children {
                if let Some(found) = find_node_at_position(child, position) {
                    return Some(found);
                }
            }
            // 如果子节点中没有找到，返回当前节点
            return Some(node);
        }
    }
    None
}

fn is_position_in_range(position: &SourceLocation, range: &SourceRange) -> bool {
    position.line >= range.start.line
        && position.line <= range.end.line
        && position.column >= range.start.column
        && position.column <= range.end.column
}

fn add_instruction_completions(completions: &mut Vec<CompletionItem>) {
    let instructions = vec![
        ("load", "加载指令"),
        ("store", "存储指令"),
        ("add", "加法指令"),
        ("sub", "减法指令"),
        ("mul", "乘法指令"),
        ("div", "除法指令"),
        ("br", "跳转指令"),
        ("call", "调用指令"),
        ("ret", "返回指令"),
    ];

    for (name, desc) in instructions {
        completions.push(CompletionItem {
            label: name.to_string(),
            kind: CompletionItemKind::Keyword,
            detail: Some(desc.to_string()),
            documentation: Some(format!("Gaia 汇编指令: {}", desc)),
            insert_text: Some(name.to_string()),
        });
    }
}

fn add_type_completions(completions: &mut Vec<CompletionItem>) {
    let data_types = vec![
        ("i32", "32位整数"),
        ("i64", "64位整数"),
        ("f32", "32位浮点数"),
        ("f64", "64位浮点数"),
        ("bool", "布尔值"),
        ("string", "字符串"),
    ];

    for (name, desc) in data_types {
        completions.push(CompletionItem {
            label: name.to_string(),
            kind: CompletionItemKind::Type,
            detail: Some(desc.to_string()),
            documentation: Some(format!("数据类型: {}", desc)),
            insert_text: Some(name.to_string()),
        });
    }
}

fn add_variable_completions(completions: &mut Vec<CompletionItem>, ast: &AstNode) {
    // 从 AST 中提取变量名
    let variables = extract_variables(ast);
    for var_name in variables {
        completions.push(CompletionItem {
            label: var_name.clone(),
            kind: CompletionItemKind::Variable,
            detail: Some("变量".to_string()),
            documentation: Some(format!("变量: {}", var_name)),
            insert_text: Some(var_name),
        });
    }
}

fn add_function_completions(completions: &mut Vec<CompletionItem>, ast: &AstNode) {
    // 从 AST 中提取函数名
    let functions = extract_functions(ast);
    for func_name in functions {
        completions.push(CompletionItem {
            label: func_name.clone(),
            kind: CompletionItemKind::Function,
            detail: Some("函数".to_string()),
            documentation: Some(format!("函数: {}", func_name)),
            insert_text: Some(func_name),
        });
    }
}

fn extract_variables(node: &AstNode) -> Vec<String> {
    let mut variables = Vec::new();

    if node.node_type == AstNodeType::VariableDecl {
        if let Some(ref name) = node.value {
            variables.push(name.clone());
        }
    }

    for child in &node.children {
        variables.extend(extract_variables(child));
    }

    variables
}

fn extract_functions(node: &AstNode) -> Vec<String> {
    let mut functions = Vec::new();

    if node.node_type == AstNodeType::FunctionDef {
        if let Some(ref name) = node.value {
            functions.push(name.clone());
        }
    }

    for child in &node.children {
        functions.extend(extract_functions(child));
    }

    functions
}

fn fold_constants(ast: AstNode) -> AstNode {
    // 常量折叠优化
    let mut optimized = ast;

    // 递归处理子节点
    optimized.children = optimized.children.into_iter().map(fold_constants).collect();

    // 如果是算术表达式且操作数都是常量，则计算结果
    if optimized.node_type == AstNodeType::Expression && optimized.children.len() == 2 {
        if let (Some(left_val), Some(right_val)) =
            (get_constant_value(&optimized.children[0]), get_constant_value(&optimized.children[1]))
        {
            // 根据操作符计算结果
            if let Some(result) = evaluate_constant_expression(&optimized.value, left_val, right_val) {
                optimized.value = Some(result.to_string());
                optimized.children.clear();
            }
        }
    }

    optimized
}

fn eliminate_dead_code(ast: AstNode) -> AstNode {
    // 死代码消除
    let mut optimized = ast;

    // 递归处理子节点
    optimized.children = optimized.children.into_iter().map(eliminate_dead_code).filter(|node| !is_dead_code(node)).collect();

    optimized
}

fn inline_small_functions(ast: AstNode) -> AstNode {
    // 内联小函数
    let mut optimized = ast;

    // 递归处理子节点
    optimized.children = optimized.children.into_iter().map(inline_small_functions).collect();

    optimized
}

fn get_constant_value(node: &AstNode) -> Option<i64> {
    if let Some(ref value) = node.value {
        value.parse().ok()
    }
    else {
        None
    }
}

fn evaluate_constant_expression(op: &Option<String>, left: i64, right: i64) -> Option<i64> {
    match op.as_ref()?.as_str() {
        "add" => Some(left + right),
        "sub" => Some(left - right),
        "mul" => Some(left * right),
        "div" if right != 0 => Some(left / right),
        _ => None,
    }
}

fn is_dead_code(node: &AstNode) -> bool {
    // 简化的死代码检测
    match node.node_type {
        AstNodeType::VariableDecl => {
            // 检查变量是否被使用
            false // 简化实现
        }
        _ => false,
    }
}

fn generate_clr_hello_world() -> String {
    r#"// CLR Hello World 示例
.assembly extern mscorlib {}
.assembly hello {}

.method static void Main() cil managed
{
    .entrypoint
    ldstr "Hello, World!"
    call void [mscorlib]System.Console::WriteLine(string)
    ret
}"#
    .to_string()
}

fn generate_jvm_hello_world() -> String {
    r#"// JVM Hello World 示例
.class public HelloWorld
.super java/lang/Object

.method public static main([Ljava/lang/String;)V
    .limit stack 2
    .limit locals 1
    
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Hello, World!"
    invokevirtual java/io/PrintStream/println(Ljava/lang/String;)V
    return
.end method"#
        .to_string()
}

fn generate_pe_hello_world() -> String {
    r#"// PE Hello World 示例
.section .data
    hello_msg db "Hello, World!", 0

.section .text
    global _start

_start:
    // 调用 Windows API 输出字符串
    push hello_msg
    call print_string
    
    // 退出程序
    push 0
    call exit"#
        .to_string()
}

fn generate_wasi_hello_world() -> String {
    r#"// WASI Hello World 示例
(module
    (import "wasi_snapshot_preview1" "fd_write" 
        (func $fd_write (param i32 i32 i32 i32) (result i32)))
    
    (memory 1)
    (export "memory" (memory 0))
    
    (data (i32.const 8) "Hello, World!\n")
    
    (func $main (export "_start")
        (i32.store (i32.const 0) (i32.const 8))  ;; iov.iov_base
        (i32.store (i32.const 4) (i32.const 14)) ;; iov.iov_len
        
        (call $fd_write
            (i32.const 1)  ;; stdout
            (i32.const 0)  ;; iov
            (i32.const 1)  ;; iovcnt
            (i32.const 20) ;; nwritten
        )
        drop
    )
)"#
    .to_string()
}

fn generate_clr_arithmetic() -> String {
    r#"// CLR 算术测试示例
.assembly extern mscorlib {}
.assembly arithmetic {}

.method static void Main() cil managed
{
    .entrypoint
    .locals init (int32 a, int32 b, int32 result)
    
    ldc.i4 10
    stloc.0     // a = 10
    
    ldc.i4 5
    stloc.1     // b = 5
    
    ldloc.0
    ldloc.1
    add
    stloc.2     // result = a + b
    
    ldloc.2
    call void [mscorlib]System.Console::WriteLine(int32)
    ret
}"#
    .to_string()
}

fn generate_jvm_arithmetic() -> String {
    r#"// JVM 算术测试示例
.class public ArithmeticTest
.super java/lang/Object

.method public static main([Ljava/lang/String;)V
    .limit stack 3
    .limit locals 4
    
    bipush 10
    istore_1    // a = 10
    
    bipush 5
    istore_2    // b = 5
    
    iload_1
    iload_2
    iadd
    istore_3    // result = a + b
    
    getstatic java/lang/System/out Ljava/io/PrintStream;
    iload_3
    invokevirtual java/io/PrintStream/println(I)V
    return
.end method"#
        .to_string()
}

fn generate_pe_arithmetic() -> String {
    r#"// PE 算术测试示例
.section .data
    a dd 10
    b dd 5
    result dd 0

.section .text
    global _start

_start:
    mov eax, [a]
    add eax, [b]
    mov [result], eax
    
    // 输出结果
    push [result]
    call print_number
    
    // 退出程序
    push 0
    call exit"#
        .to_string()
}

fn generate_wasi_arithmetic() -> String {
    r#"// WASI 算术测试示例
(module
    (func $add (param $a i32) (param $b i32) (result i32)
        local.get $a
        local.get $b
        i32.add
    )
    
    (func $main (export "_start")
        (local $result i32)
        
        i32.const 10
        i32.const 5
        call $add
        local.set $result
        
        ;; 这里可以添加输出逻辑
    )
)"#
    .to_string()
}

fn add_comments_to_template(template: String, target: Target) -> String {
    let header_comment = format!(
        "// {} 目标平台代码示例\n// 由 Gaia 汇编器生成\n\n",
        match target {
            Target::Clr => "CLR",
            Target::Jvm => "JVM",
            Target::Pe => "PE",
            Target::Wasi => "WASI",
        }
    );

    header_comment + &template
}

fn is_block_start_instruction(instruction: &str) -> bool {
    let parts: Vec<&str> = instruction.split_whitespace().collect();
    if parts.is_empty() {
        return false;
    }

    matches!(parts[0], "if" | "loop" | "function" | ".method")
}

fn is_block_end_instruction(instruction: &str) -> bool {
    let parts: Vec<&str> = instruction.split_whitespace().collect();
    if parts.is_empty() {
        return false;
    }

    matches!(parts[0], "end" | ".end" | "}")
}

fn classify_token(word: &str) -> TokenType {
    if word.starts_with("//") {
        TokenType::Comment
    }
    else if word.ends_with(':') {
        TokenType::Label
    }
    else if word.starts_with('"') && word.ends_with('"') {
        TokenType::String
    }
    else if word.parse::<i32>().is_ok() || word.parse::<f32>().is_ok() {
        TokenType::Number
    }
    else if is_instruction(word) {
        TokenType::Instruction
    }
    else if is_keyword(word) {
        TokenType::Keyword
    }
    else {
        TokenType::Identifier
    }
}

fn is_instruction(word: &str) -> bool {
    let instructions = [
        "load", "store", "add", "sub", "mul", "div", "mod", "and", "or", "xor", "not", "shl", "shr", "eq", "ne", "lt", "le",
        "gt", "ge", "br", "br_if", "call", "ret", "nop", "dup", "pop", "swap",
    ];
    instructions.contains(&word)
}

fn is_keyword(word: &str) -> bool {
    let keywords = [
        "function",
        "end",
        "if",
        "else",
        "loop",
        "break",
        "continue",
        ".assembly",
        ".method",
        ".class",
        ".section",
        ".data",
        ".text",
    ];
    keywords.contains(&word)
}

fn get_instruction_description(instruction: &str) -> String {
    match instruction {
        "load" => "加载常量或变量值".to_string(),
        "store" => "存储值到指定位置".to_string(),
        "add" => "加法运算".to_string(),
        "sub" => "减法运算".to_string(),
        "mul" => "乘法运算".to_string(),
        "div" => "除法运算".to_string(),
        "mod" => "取模运算".to_string(),
        "and" => "按位与运算".to_string(),
        "or" => "按位或运算".to_string(),
        "xor" => "按位异或运算".to_string(),
        "not" => "按位非运算".to_string(),
        "shl" => "左移运算".to_string(),
        "shr" => "右移运算".to_string(),
        "eq" => "相等比较".to_string(),
        "ne" => "不等比较".to_string(),
        "lt" => "小于比较".to_string(),
        "le" => "小于等于比较".to_string(),
        "gt" => "大于比较".to_string(),
        "ge" => "大于等于比较".to_string(),
        "br" => "无条件跳转".to_string(),
        "br_if" => "条件跳转".to_string(),
        "call" => "函数调用".to_string(),
        "ret" => "函数返回".to_string(),
        "nop" => "空操作".to_string(),
        "dup" => "复制栈顶值".to_string(),
        "pop" => "弹出栈顶值".to_string(),
        "swap" => "交换栈顶两个值".to_string(),
        _ => "未知指令".to_string(),
    }
}

fn get_instruction_documentation(instruction: &str) -> Option<String> {
    match instruction {
        "load" => Some("将常量或变量值加载到栈顶\n语法：load <value>\n示例：load 42".to_string()),
        "add" => Some("弹出栈顶两个值，相加后压入栈\n语法：add\n示例：load 1; load 2; add".to_string()),
        _ => None,
    }
}

// 新的辅助函数
fn format_instruction_entry(instruction_entry: &mut InstructionEntry, _options: &FormatOptions) {
    // 格式化指令条目的逻辑
    // 这里只是占位符实现
}

fn format_symbol_info(symbol: &mut SymbolInfo, _options: &FormatOptions) {
    // 格式化符号信息的逻辑
    // 这里只是占位符实现
}

fn analyze_completion_context_from_descriptor(descriptor: &GaiaDescriptor, position: &SourceLocation) -> CompletionContext {
    // 基于描述符分析补全上下文
    // 简化实现，实际应该根据位置分析上下文
    CompletionContext::General
}

fn add_variable_completions_from_descriptor(completions: &mut Vec<CompletionItem>, descriptor: &GaiaDescriptor) {
    for symbol in &descriptor.symbols {
        if symbol.symbol_type == SymbolType::Variable {
            completions.push(CompletionItem {
                label: symbol.name.clone(),
                kind: CompletionType::Variable,
                detail: Some(format!("变量: {:?}", symbol.data_type)),
                documentation: None,
                insert_text: Some(symbol.name.clone()),
                filter_text: Some(symbol.name.clone()),
                sort_text: Some(format!("1_{}", symbol.name)),
                additional_text_edits: vec![],
            });
        }
    }
}

fn add_function_completions_from_descriptor(completions: &mut Vec<CompletionItem>, descriptor: &GaiaDescriptor) {
    for symbol in &descriptor.symbols {
        if symbol.symbol_type == SymbolType::Function {
            completions.push(CompletionItem {
                label: symbol.name.clone(),
                kind: CompletionType::Function,
                detail: Some("函数".to_string()),
                documentation: None,
                insert_text: Some(format!("{}()", symbol.name)),
                filter_text: Some(symbol.name.clone()),
                sort_text: Some(format!("2_{}", symbol.name)),
                additional_text_edits: vec![],
            });
        }
    }
}

fn fold_constants_in_descriptor(descriptor: GaiaDescriptor) -> GaiaDescriptor {
    // 常量折叠优化的占位符实现
    descriptor
}

fn eliminate_dead_code_in_descriptor(descriptor: GaiaDescriptor) -> GaiaDescriptor {
    // 死代码消除优化的占位符实现
    descriptor
}

fn combine_instructions(descriptor: GaiaDescriptor) -> GaiaDescriptor {
    // 指令合并优化的占位符实现
    descriptor
}

fn extract_symbol_reference_from_instruction(instruction: &Instruction) -> Option<String> {
    // 从指令中提取符号引用
    match instruction {
        Instruction::Call(call_op) => {
            // 简化实现，实际应该从操作数中提取符号名
            Some(format!("call_target_{:?}", call_op))
        }
        _ => None,
    }
}
