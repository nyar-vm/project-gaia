# WAT 语法分析器模块

WebAssembly 文本格式 (WAT) 的语法分析器，将词法单元解析为抽象语法树 (AST)。

## 功能特性

- **递归下降解析**: 高效的语法分析算法
- **错误恢复**: 从语法错误中恢复
- **错误报告**: 详细的错误信息
- **语法验证**: 完整的语法检查
- **位置跟踪**: 保留源代码位置信息

## 支持的语法

### 模块语法
```wat
(module
    (func $name (param $p i32) (result i32) ...)
    (export "name" (func $name))
    (import "module" "name" (func $name (param i32)))
)
```

### 函数语法
```wat
(func $add (param $a i32) (param $b i32) (result i32)
    local.get $a
    local.get $b
    i32.add
)
```

## 使用示例

```rust
use wasi_assembler::formats::wat::{WatLexer, WatParser};

let wat_source = r#"
    (module
        (func $add (param $a i32) (param $b i32) (result i32)
            local.get $a
            local.get $b
            i32.add)
        (export "add" (func $add))
    )
"#;

// 词法分析
let mut lexer = WatLexer::new();
let tokens = lexer.tokenize(wat_source)?;

// 语法分析
let mut parser = WatParser::new();
let ast = parser.parse(tokens)?;

println!("解析成功，模块包含 {} 个函数", ast.functions.len());
```

## 错误处理

语法分析器提供详细的错误信息：
- 意外的词法单元
- 缺少必需的元素
- 语法结构错误
- 位置信息