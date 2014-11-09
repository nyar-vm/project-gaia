# WAT AST 到 Program 的编译器

这个模块负责将解析后的 WAT AST 转换为高层次的 Program 表示

## 编译过程

编译器的主要步骤包括：

1. **类型检查**: 验证 AST 中的类型一致性
2. **符号解析**: 解析名称引用到具体的定义
3. **指令编码**: 将 AST 指令转换为二进制指令
4. **结构转换**: 将 AST 结构转换为 Program 结构

## 支持的转换

### 模块转换
- WAT 模块 → Program 核心模块
- WAT 组件 → Program 组件
- 导入/导出定义
- 函数类型定义

### 函数转换
- 函数参数和返回值
- 局部变量定义
- 指令序列转换
- 控制流结构

### 指令转换
- 数值常量
- 算术运算
- 内存操作
- 函数调用
- 控制流指令

## 使用示例

```rust,no_run
use wasi_assembler::formats::wat::{WatParser, WatCompiler};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let wat_source = r#"
        (module
            (func $add (param $a i32) (param $b i32) (result i32)
                local.get $a
                local.get $b
                i32.add
            )
            (export "add" (func $add))
        )
    "#;
    
    // 解析 WAT
    let mut parser = WatParser::new();
    let ast = parser.parse(wat_source)?;
    
    // 编译为 Program
    let mut compiler = WatCompiler::new();
    let program = compiler.compile(ast)?;
    
    // 使用 Program 结构
    println!("编译完成，包含 {} 个函数", program.functions.len());
    Ok(())
}
```

## 错误处理

编译器提供详细的错误信息：
- 类型不匹配
- 未定义的符号
- 无效的指令
- 结构错误

```rust,no_run
use wasi_assembler::formats::wat::{WatCompiler, WatError};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut compiler = WatCompiler::new();
    match compiler.compile(ast) {
        Ok(program) => {
            // 编译成功
        }
        Err(WatError::TypeMismatch { expected, found }) => {
            eprintln!("类型错误: 期望 {:?}, 找到 {:?}", expected, found);
        }
        Err(WatError::UndefinedSymbol(name)) => {
            eprintln!("未定义符号: {}", name);
        }
        Err(e) => {
            eprintln!("编译错误: {}", e);
        }
    }
    Ok(())
}
```