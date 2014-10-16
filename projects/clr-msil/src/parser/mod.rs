//! MSIL (Microsoft Intermediate Language) 解析器模块
//!
//! 这个模块提供了 MSIL 汇编语言的解析功能，将 MSIL 源代码转换为抽象语法树 (AST)。
//! 解析器使用递归下降解析技术，支持 MSIL 的各种语法构造。
//!
//! # 特性
//!
//! - 支持 MSIL 程序集声明（`.assembly`）
//! - 支持外部程序集引用（`.assembly extern`）
//! - 支持模块声明（`.module`）
//! - 支持类声明（`.class`）
//! - 支持方法声明（`.method`）
//! - 支持方法体解析（指令、局部变量等）
//! - 支持各种修饰符和属性
//!
//! # 示例
//!
//! ```rust
//! use clr_msil::{MsilParser, ReadConfig};
//!
//! let config = ReadConfig::default();
//! let parser = MsilParser::new(&config);
//!
//! let msil_code = r#"
//! .assembly extern UnityEngine
//! .assembly MyAssembly
//! .module MyModule.dll
//! "#;
//!
//! let result = parser.parse_text(msil_code);
//! if let Ok(ast) = result.result {
//!     println!("解析成功，找到 {} 个语句", ast.statements.len());
//! }
//! ```

use crate::{
    ast::{MsilClass, MsilInstruction, MsilMethod, MsilMethodBody, MsilParameter, MsilRoot, MsilStatement},
    lexer::{MsilLexer, MsilTokenType},
    ReadConfig,
};
use gaia_types::{reader::TokenStream, GaiaDiagnostics};

/// MSIL 解析器
///
/// 负责将 MSIL 源代码解析为抽象语法树。
/// 解析器维护对配置的引用，并使用词法分析器将源代码转换为 token 流。
///
/// # 生命周期参数
///
/// - `'config`: 解析器配置的生命周期
///
/// # 示例
///
/// ```rust
/// use clr_msil::{MsilParser, ReadConfig};
///
/// let config = ReadConfig::default();
/// let parser = MsilParser::new(&config);
/// ```
#[derive(Clone, Debug)]
pub struct MsilParser<'config> {
    /// 解析器配置
    config: &'config ReadConfig,
}

impl<'config> MsilParser<'config> {
    /// 创建新的 MSIL 解析器
    ///
    /// # 参数
    ///
    /// - `config`: 解析器配置
    ///
    /// # 返回值
    ///
    /// 返回一个新的 `MsilParser` 实例
    ///
    /// # 示例
    ///
    /// ```rust
    /// use clr_msil::{MsilParser, ReadConfig};
    ///
    /// let config = ReadConfig::default();
    /// let parser = MsilParser::new(&config);
    /// ```
    pub fn new(config: &'config ReadConfig) -> Self {
        Self { config }
    }

    /// 解析 MSIL 源代码文本
    ///
    /// 这个方法将 MSIL 源代码字符串解析为抽象语法树。
    /// 它首先使用词法分析器将源代码转换为 token 流，然后进行语法分析。
    ///
    /// # 参数
    ///
    /// - `text`: MSIL 源代码字符串
    ///
    /// # 返回值
    ///
    /// 返回包含解析结果的 `GaiaDiagnostics<MsilRoot>`
    ///
    /// # 示例
    ///
    /// ```rust
    /// use clr_msil::{MsilParser, ReadConfig};
    ///
    /// let config = ReadConfig::default();
    /// let parser = MsilParser::new(&config);
    ///
    /// let result = parser.parse_text(".assembly MyAssembly");
    /// if let Ok(ast) = result.result {
    ///     println!("解析成功");
    /// }
    /// ```
    pub fn parse_text(&self, text: &str) -> GaiaDiagnostics<MsilRoot> {
        let lexer = MsilLexer::new(&self.config);
        let tokens = lexer.tokenize(text);
        self.parse(tokens.result.unwrap())
    }

    /// 解析 MSIL token 流
    ///
    /// 这个方法将已经词法分析过的 token 流解析为抽象语法树。
    /// 它处理各种 MSIL 语句，包括程序集声明、模块声明、类声明等。
    ///
    /// # 参数
    ///
    /// - `tokens`: 词法分析器生成的 token 流
    ///
    /// # 返回值
    ///
    /// 返回包含解析结果的 `GaiaDiagnostics<MsilRoot>`
    ///
    /// # 解析过程
    ///
    /// 1. 跳过空白字符和注释
    /// 2. 识别以 `.` 开头的指令
    /// 3. 根据指令类型调用相应的解析函数
    /// 4. 构建抽象语法树
    ///
    /// # 支持的语法构造
    ///
    /// - `.assembly extern <name>` - 外部程序集引用
    /// - `.assembly <name>` - 程序集声明
    /// - `.module <name>` - 模块声明
    /// - `.class <modifiers> <name> extends <base>` - 类声明
    /// - `.method <modifiers> <return_type> <name>(<parameters>)` - 方法声明
    ///
    /// # 示例
    ///
    /// ```rust
    /// use clr_msil::{MsilParser, ReadConfig};
    /// use gaia_types::reader::TokenStream;
    ///
    /// let config = ReadConfig::default();
    /// let parser = MsilParser::new(&config);
    ///
    /// // 通常通过 parse_text 方法使用，这个方法主要用于内部处理
    /// ```
    pub fn parse(&self, tokens: TokenStream<MsilTokenType>) -> GaiaDiagnostics<MsilRoot> {
        let mut statements = Vec::new();
        let token_vec = tokens.tokens.get_ref();
        let mut current_index = 0;

        // 跳过注释和空白字符的辅助函数
        let skip_ignored = |index: &mut usize| {
            while *index < token_vec.len() {
                match token_vec[*index].token_type {
                    MsilTokenType::Whitespace | MsilTokenType::Comment => {
                        *index += 1;
                    }
                    _ => break,
                }
            }
        };

        while current_index < token_vec.len() {
            skip_ignored(&mut current_index);

            if current_index >= token_vec.len() {
                break;
            }
            let token = &token_vec[current_index];
            match &token.token_type {
                MsilTokenType::Dot => {
                    println!("找到 Dot token");
                    // 处理以 . 开头的指令
                    if current_index + 1 < token_vec.len() {
                        let next_token = &token_vec[current_index + 1];
                        println!("Dot 后的 token: {:?}", next_token.token_type);
                        match &next_token.token_type {
                            MsilTokenType::Assembly => {
                                println!("找到 Assembly 指令");
                                current_index += 2; // 跳过 '.' 和 'assembly'
                                skip_ignored(&mut current_index);

                                // 检查是否是 extern assembly
                                if current_index < token_vec.len() {
                                    let peek_token = &token_vec[current_index];
                                    if peek_token.token_type == MsilTokenType::Extern {
                                        current_index += 1; // 消费 extern token
                                        skip_ignored(&mut current_index);

                                        if current_index < token_vec.len() {
                                            let name_token = &token_vec[current_index];
                                            if name_token.token_type == MsilTokenType::Identifier {
                                                let name = tokens.get_text(name_token).unwrap_or("unknown").to_string();
                                                statements.push(MsilStatement::AssemblyExtern(name.clone()));
                                                current_index += 1;
                                                println!("创建了 AssemblyExtern 语句: {}", name);
                                            }
                                        }
                                    }
                                    else if peek_token.token_type == MsilTokenType::Identifier {
                                        let name = tokens.get_text(peek_token).unwrap_or("unknown").to_string();
                                        statements.push(MsilStatement::Assembly(name.clone()));
                                        current_index += 1;
                                        println!("创建了 Assembly 语句: {}", name);
                                    }
                                }
                            }
                            MsilTokenType::Module => {
                                println!("找到 Module 指令");
                                current_index += 2; // 跳过 '.' 和 'module'
                                skip_ignored(&mut current_index);

                                if current_index < token_vec.len() {
                                    let name_token = &token_vec[current_index];
                                    if name_token.token_type == MsilTokenType::Identifier {
                                        let name = tokens.get_text(name_token).unwrap_or("unknown").to_string();
                                        statements.push(MsilStatement::Module(name.clone()));
                                        current_index += 1;
                                        println!("创建了 Module 语句: {}", name);
                                    }
                                }
                            }
                            MsilTokenType::Class => {
                                println!("找到 Class 指令");
                                current_index += 2; // 跳过 '.' 和 'class'
                                skip_ignored(&mut current_index);

                                let mut modifiers = Vec::new();
                                let mut class_name = String::new();
                                let mut extends = None;
                                let mut methods = Vec::new();

                                // 收集修饰符
                                while current_index < token_vec.len() {
                                    let peek_token = &token_vec[current_index];
                                    match peek_token.token_type {
                                        MsilTokenType::Public => {
                                            modifiers.push("public".to_string());
                                            current_index += 1;
                                            skip_ignored(&mut current_index);
                                        }
                                        MsilTokenType::Private => {
                                            modifiers.push("private".to_string());
                                            current_index += 1;
                                            skip_ignored(&mut current_index);
                                        }
                                        MsilTokenType::Auto => {
                                            modifiers.push("auto".to_string());
                                            current_index += 1;
                                            skip_ignored(&mut current_index);
                                        }
                                        MsilTokenType::Ansi => {
                                            modifiers.push("ansi".to_string());
                                            current_index += 1;
                                            skip_ignored(&mut current_index);
                                        }
                                        MsilTokenType::Beforefieldinit => {
                                            modifiers.push("beforefieldinit".to_string());
                                            current_index += 1;
                                            skip_ignored(&mut current_index);
                                        }
                                        MsilTokenType::Identifier => {
                                            let name = tokens.get_text(peek_token).unwrap_or("unknown").to_string();
                                            class_name = name;
                                            current_index += 1;

                                            // 检查是否有更多的点和标识符组成完整的类名
                                            while current_index < token_vec.len() {
                                                if token_vec[current_index].token_type == MsilTokenType::Dot {
                                                    current_index += 1; // 跳过 '.'
                                                    if current_index < token_vec.len()
                                                        && token_vec[current_index].token_type == MsilTokenType::Identifier
                                                    {
                                                        class_name.push('.');
                                                        class_name
                                                            .push_str(tokens.get_text(&token_vec[current_index]).unwrap_or(""));
                                                        current_index += 1;
                                                    }
                                                    else {
                                                        current_index -= 1; // 回退，这个点不是类名的一部分
                                                        break;
                                                    }
                                                }
                                                else {
                                                    break;
                                                }
                                            }
                                            break;
                                        }
                                        _ => break,
                                    }
                                }

                                skip_ignored(&mut current_index);

                                // 检查是否有 extends
                                if current_index < token_vec.len() {
                                    let peek_token = &token_vec[current_index];
                                    if peek_token.token_type == MsilTokenType::Extends {
                                        current_index += 1; // 消费 extends token
                                        skip_ignored(&mut current_index);

                                        if current_index < token_vec.len() {
                                            let base_token = &token_vec[current_index];
                                            match base_token.token_type {
                                                MsilTokenType::Identifier => {
                                                    extends =
                                                        Some(tokens.get_text(base_token).unwrap_or("unknown").to_string());
                                                    current_index += 1;
                                                }
                                                MsilTokenType::LeftBracket => {
                                                    // 处理 [Assembly]Type 格式的基类
                                                    let mut base_type = String::new();
                                                    while current_index < token_vec.len()
                                                        && token_vec[current_index].token_type != MsilTokenType::RightBracket
                                                    {
                                                        base_type
                                                            .push_str(tokens.get_text(&token_vec[current_index]).unwrap_or(""));
                                                        current_index += 1;
                                                    }
                                                    if current_index < token_vec.len()
                                                        && token_vec[current_index].token_type == MsilTokenType::RightBracket
                                                    {
                                                        current_index += 1; // 跳过 ']'
                                                        base_type.push(']');

                                                        // 继续读取类型名
                                                        while current_index < token_vec.len() {
                                                            let type_token = &token_vec[current_index];
                                                            match type_token.token_type {
                                                                MsilTokenType::Identifier => {
                                                                    base_type
                                                                        .push_str(tokens.get_text(type_token).unwrap_or(""));
                                                                    current_index += 1;
                                                                }
                                                                MsilTokenType::Dot => {
                                                                    base_type.push('.');
                                                                    current_index += 1;
                                                                }
                                                                _ => break,
                                                            }
                                                        }

                                                        extends = Some(format!("[{}", base_type));
                                                    }
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                }

                                // 跳过到类体开始 '{'
                                skip_ignored(&mut current_index);
                                if current_index < token_vec.len()
                                    && token_vec[current_index].token_type == MsilTokenType::LeftBrace
                                {
                                    current_index += 1;

                                    // 解析类体中的方法
                                    while current_index < token_vec.len() {
                                        skip_ignored(&mut current_index);

                                        if current_index >= token_vec.len() {
                                            break;
                                        }

                                        let token = &token_vec[current_index];
                                        if token.token_type == MsilTokenType::RightBrace {
                                            current_index += 1; // 消费 '}'
                                            break;
                                        }

                                        if token.token_type == MsilTokenType::Dot {
                                            if current_index + 1 < token_vec.len()
                                                && token_vec[current_index + 1].token_type == MsilTokenType::Method
                                            {
                                                // 解析方法
                                                if let Some(method) = self.parse_method(&tokens, &mut current_index) {
                                                    methods.push(method);
                                                }
                                            }
                                            else {
                                                current_index += 1; // 跳过其他 . 指令
                                            }
                                        }
                                        else {
                                            current_index += 1; // 跳过其他 token
                                        }
                                    }
                                }

                                statements.push(MsilStatement::Class(MsilClass {
                                    modifiers,
                                    name: class_name.clone(),
                                    extends,
                                    methods,
                                }));
                                println!("创建了 Class 语句: {}", class_name);
                            }
                            MsilTokenType::Identifier => {
                                // 检查标识符的内容来确定是什么指令
                                let identifier_text = tokens.get_text(next_token).unwrap_or("");
                                println!("标识符内容: {}", identifier_text);
                                println!("忽略其他以 . 开头的指令: {}", identifier_text);
                                current_index += 2; // 跳过 '.' 和标识符
                            }
                            _ => {
                                println!("忽略其他以 . 开头的指令: {:?}", next_token.token_type);
                                current_index += 1; // 只跳过当前的 '.' token，让下一次循环处理下一个 token
                            }
                        }
                    }
                    else {
                        println!("Dot token 后没有更多 token");
                        current_index += 1; // 跳过当前 token
                    }
                }

                MsilTokenType::Eof => break,

                _ => {
                    // 忽略其他 token
                    current_index += 1;
                }
            }
        }

        println!("解析完成，语句数量: {}", statements.len());
        GaiaDiagnostics::success(MsilRoot { statements })
    }

    /// 解析 MSIL 方法声明
    ///
    /// 这个方法解析 `.method` 指令，提取方法的修饰符、返回类型、名称、参数和方法体。
    ///
    /// # 参数
    ///
    /// - `tokens`: token 流
    /// - `current_index`: 当前解析位置的索引（会被更新）
    ///
    /// # 返回值
    ///
    /// 返回解析出的 `MsilMethod` 结构，如果解析失败则返回 `None`
    ///
    /// # 解析过程
    ///
    /// 1. 跳过 `.method` 指令
    /// 2. 收集方法修饰符（public, static, virtual 等）
    /// 3. 解析返回类型
    /// 4. 解析方法名（支持特殊方法名如 `.ctor`）
    /// 5. 解析参数列表
    /// 6. 解析调用约定
    /// 7. 解析方法体（如果存在）
    ///
    /// # 示例
    ///
    /// 解析以下方法声明：
    /// ```msil
    /// .method public hidebysig virtual instance void Start() cil managed
    /// ```
    fn parse_method(&self, tokens: &TokenStream<MsilTokenType>, current_index: &mut usize) -> Option<MsilMethod> {
        let token_vec = tokens.tokens.get_ref();

        // 跳过注释和空白字符的辅助函数
        let skip_ignored = |index: &mut usize| {
            while *index < token_vec.len() {
                match token_vec[*index].token_type {
                    MsilTokenType::Whitespace | MsilTokenType::Comment => {
                        *index += 1;
                    }
                    _ => break,
                }
            }
        };

        // 跳过 '.' 和 'method'
        *current_index += 2;
        skip_ignored(current_index);

        let mut modifiers = Vec::new();
        let mut return_type = String::new();
        let mut method_name = String::new();
        let mut parameters = Vec::new();

        // 收集修饰符
        while *current_index < token_vec.len() {
            let token = &token_vec[*current_index];
            match token.token_type {
                MsilTokenType::Public => {
                    modifiers.push("public".to_string());
                    *current_index += 1;
                    skip_ignored(current_index);
                }
                MsilTokenType::Private => {
                    modifiers.push("private".to_string());
                    *current_index += 1;
                    skip_ignored(current_index);
                }
                MsilTokenType::Hidebysig => {
                    modifiers.push("hidebysig".to_string());
                    *current_index += 1;
                    skip_ignored(current_index);
                }
                MsilTokenType::Static => {
                    modifiers.push("static".to_string());
                    *current_index += 1;
                    skip_ignored(current_index);
                }
                MsilTokenType::Virtual => {
                    modifiers.push("virtual".to_string());
                    *current_index += 1;
                    skip_ignored(current_index);
                }
                MsilTokenType::Specialname => {
                    modifiers.push("specialname".to_string());
                    *current_index += 1;
                    skip_ignored(current_index);
                }
                MsilTokenType::Rtspecialname => {
                    modifiers.push("rtspecialname".to_string());
                    *current_index += 1;
                    skip_ignored(current_index);
                }
                MsilTokenType::Instance => {
                    modifiers.push("instance".to_string());
                    *current_index += 1;
                    skip_ignored(current_index);
                }
                MsilTokenType::Identifier => {
                    // 这应该是返回类型
                    return_type = tokens.get_text(token).unwrap_or("void").to_string();
                    *current_index += 1;
                    skip_ignored(current_index);
                    break;
                }
                _ => break,
            }
        }

        // 获取方法名
        if *current_index < token_vec.len() {
            let token = &token_vec[*current_index];
            match token.token_type {
                MsilTokenType::Identifier => {
                    method_name = tokens.get_text(token).unwrap_or("unknown").to_string();
                    *current_index += 1;
                    skip_ignored(current_index);
                }
                MsilTokenType::Dot => {
                    // 处理 .ctor 等特殊方法名
                    if *current_index + 1 < token_vec.len() {
                        let next_token = &token_vec[*current_index + 1];
                        if next_token.token_type == MsilTokenType::Ctor {
                            method_name = ".ctor".to_string();
                            *current_index += 2; // 跳过 '.' 和 'ctor'
                            skip_ignored(current_index);
                        }
                        else {
                            method_name = "unknown".to_string();
                            *current_index += 1;
                        }
                    }
                }
                _ => {
                    method_name = "unknown".to_string();
                }
            }
        }

        // 解析参数列表 (简化版本)
        if *current_index < token_vec.len() && token_vec[*current_index].token_type == MsilTokenType::LeftParen {
            *current_index += 1; // 跳过 '('
            skip_ignored(current_index);

            // 简单解析参数，这里可以进一步扩展
            while *current_index < token_vec.len() && token_vec[*current_index].token_type != MsilTokenType::RightParen {
                if token_vec[*current_index].token_type == MsilTokenType::Identifier {
                    let param_type = tokens.get_text(&token_vec[*current_index]).unwrap_or("unknown").to_string();
                    parameters.push(MsilParameter { param_type, name: None });
                }
                *current_index += 1;
                skip_ignored(current_index);
            }

            if *current_index < token_vec.len() && token_vec[*current_index].token_type == MsilTokenType::RightParen {
                *current_index += 1; // 跳过 ')'
                skip_ignored(current_index);
            }
        }

        // 跳过调用约定 (cil managed)
        while *current_index < token_vec.len() {
            let token = &token_vec[*current_index];
            match token.token_type {
                MsilTokenType::Cil | MsilTokenType::Managed => {
                    *current_index += 1;
                    skip_ignored(current_index);
                }
                MsilTokenType::LeftBrace => {
                    break;
                }
                _ => {
                    *current_index += 1;
                    skip_ignored(current_index);
                }
            }
        }

        // 解析方法体
        let body = if *current_index < token_vec.len() && token_vec[*current_index].token_type == MsilTokenType::LeftBrace {
            self.parse_method_body(tokens, current_index)
        }
        else {
            None
        };

        Some(MsilMethod { modifiers, return_type, name: method_name, parameters, body })
    }

    /// 解析 MSIL 方法体
    ///
    /// 这个方法解析方法体中的指令序列，包括 `.maxstack` 指令、IL 指令等。
    ///
    /// # 参数
    ///
    /// - `tokens`: token 流
    /// - `current_index`: 当前解析位置的索引（会被更新）
    ///
    /// # 返回值
    ///
    /// 返回解析出的 `MsilMethodBody` 结构，如果解析失败则返回 `None`
    ///
    /// # 解析过程
    ///
    /// 1. 跳过方法体开始的 `{`
    /// 2. 解析 `.maxstack` 指令（如果存在）
    /// 3. 解析 `.entrypoint` 指令（如果存在）
    /// 4. 解析 IL 指令序列
    /// 5. 遇到 `}` 时结束解析
    ///
    /// # 支持的指令
    ///
    /// - `.maxstack <size>` - 设置最大栈大小
    /// - `.entrypoint` - 标记入口点
    /// - IL 指令（ldstr, call, ret, nop, ldarg 等）
    ///
    /// # 示例
    ///
    /// 解析以下方法体：
    /// ```msil
    /// {
    ///     .maxstack 8
    ///     ldstr "Hello World!"
    ///     call void [UnityEngine]UnityEngine.Debug::Log(object)
    ///     ret
    /// }
    /// ```
    fn parse_method_body(&self, tokens: &TokenStream<MsilTokenType>, current_index: &mut usize) -> Option<MsilMethodBody> {
        let token_vec = tokens.tokens.get_ref();

        // 跳过注释和空白字符的辅助函数
        let skip_ignored = |index: &mut usize| {
            while *index < token_vec.len() {
                match token_vec[*index].token_type {
                    MsilTokenType::Whitespace | MsilTokenType::Comment => {
                        *index += 1;
                    }
                    _ => break,
                }
            }
        };

        // 跳过 '{'
        *current_index += 1;
        skip_ignored(current_index);

        let mut maxstack = None;
        let mut instructions = Vec::new();

        while *current_index < token_vec.len() {
            skip_ignored(current_index);

            if *current_index >= token_vec.len() {
                break;
            }

            let token = &token_vec[*current_index];

            if token.token_type == MsilTokenType::RightBrace {
                *current_index += 1; // 消费 '}'
                break;
            }

            if token.token_type == MsilTokenType::Dot {
                if *current_index + 1 < token_vec.len() {
                    let next_token = &token_vec[*current_index + 1];
                    match next_token.token_type {
                        MsilTokenType::Maxstack => {
                            // 解析 .maxstack
                            *current_index += 2; // 跳过 '.' 和 'maxstack'
                            skip_ignored(current_index);

                            if *current_index < token_vec.len() && token_vec[*current_index].token_type == MsilTokenType::Number
                            {
                                if let Ok(stack_size) =
                                    tokens.get_text(&token_vec[*current_index]).unwrap_or("8").parse::<u32>()
                                {
                                    maxstack = Some(stack_size);
                                }
                                *current_index += 1;
                            }
                        }
                        MsilTokenType::Entrypoint => {
                            // 解析 .entrypoint
                            *current_index += 2; // 跳过 '.' 和 'entrypoint'
                            skip_ignored(current_index);

                            // 添加 entrypoint 指令到指令列表
                            instructions.push(MsilInstruction {
                                opcode: ".entrypoint".to_string(),
                                operands: Vec::new(),
                                label: None,
                            });
                        }
                        _ => {
                            *current_index += 1; // 跳过其他 . 指令
                        }
                    }
                }
                else {
                    *current_index += 1; // 跳过其他 . 指令
                }
            }
            else {
                // 解析 IL 指令
                match token.token_type {
                    MsilTokenType::Ldstr
                    | MsilTokenType::Call
                    | MsilTokenType::Ret
                    | MsilTokenType::Nop
                    | MsilTokenType::Ldarg => {
                        let opcode = tokens.get_text(token).unwrap_or("unknown").to_string();
                        *current_index += 1;
                        skip_ignored(current_index);

                        let mut operands = Vec::new();

                        // 收集操作数
                        while *current_index < token_vec.len() {
                            let operand_token = &token_vec[*current_index];
                            match operand_token.token_type {
                                MsilTokenType::StringLiteral | MsilTokenType::Identifier | MsilTokenType::Number => {
                                    operands.push(tokens.get_text(operand_token).unwrap_or("").to_string());
                                    *current_index += 1;
                                    skip_ignored(current_index);
                                }
                                MsilTokenType::LeftBracket => {
                                    // 处理类型引用 [Assembly]Type
                                    let mut type_ref = String::new();
                                    while *current_index < token_vec.len()
                                        && token_vec[*current_index].token_type != MsilTokenType::RightBracket
                                    {
                                        type_ref.push_str(tokens.get_text(&token_vec[*current_index]).unwrap_or(""));
                                        *current_index += 1;
                                    }
                                    if *current_index < token_vec.len()
                                        && token_vec[*current_index].token_type == MsilTokenType::RightBracket
                                    {
                                        *current_index += 1; // 跳过 ']'
                                        type_ref.push(']');

                                        // 继续读取类型名
                                        if *current_index < token_vec.len()
                                            && token_vec[*current_index].token_type == MsilTokenType::Identifier
                                        {
                                            type_ref.push_str(tokens.get_text(&token_vec[*current_index]).unwrap_or(""));
                                            *current_index += 1;
                                        }

                                        operands.push(format!("[{}", type_ref));
                                        skip_ignored(current_index);
                                    }
                                }
                                MsilTokenType::DoubleColon => {
                                    operands.push("::".to_string());
                                    *current_index += 1;
                                    skip_ignored(current_index);
                                }
                                _ => break,
                            }
                        }

                        instructions.push(MsilInstruction { opcode, operands, label: None });
                    }
                    _ => {
                        *current_index += 1; // 跳过其他 token
                    }
                }
            }
        }

        Some(MsilMethodBody {
            maxstack,
            locals: Vec::new(), // 暂时不解析 locals
            instructions,
        })
    }
}
