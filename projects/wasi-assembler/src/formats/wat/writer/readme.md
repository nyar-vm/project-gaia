# WAT 写入器模块

WebAssembly 文本格式 (WAT) 的写入器，将抽象语法树 (AST) 转换回格式化的 WAT 文本。

## 功能特性

- **格式化输出**: 生成美观的 WAT 文本
- **注释生成**: 自动添加有用的注释
- **代码美化**: 一致的代码风格
- **可配置**: 支持不同的输出格式

## 输出格式

### 标准格式
```wat
(module
    (func $add (param $a i32) (param $b i32) (result i32)
        local.get $a
        local.get $b
        i32.add
    )
    (export "add" (func $add))
)
```

### 紧凑格式
```wat
(module (func $add (param i32 i32) (result i32) local.get 0 local.get 1 i32.add) (export "add" (func $add)))
```

## 使用示例

```rust
use wasi_assembler::formats::wat::{WatParser, WatWriter};

let wat_source = r#"
    (module
        (func $add (param $a i32) (param $b i32) (result i32)
            local.get $a
            local.get $b
            i32.add)
    )
"#;

// 解析 WAT
let mut parser = WatParser::new();
let ast = parser.parse(wat_source)?;

// 写回 WAT
let mut writer = WatWriter::new();
let formatted_wat = writer.write(&ast)?;

println!("格式化后的 WAT:\n{}", formatted_wat);
```

## 配置选项

- **缩进**: 空格或制表符
- **括号风格**: 紧凑或展开
- **注释**: 是否生成注释
- **行长度**: 最大行长度限制