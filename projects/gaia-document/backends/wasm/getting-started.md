# WASM 后端入门指南

本指南将帮助您快速开始使用 Gaia WASM 后端，从安装配置到创建您的第一个 WebAssembly 模块。

## 环境准备

### 系统要求

- **Rust 1.70+** - 支持最新的 WebAssembly 特性
- **Node.js 16+** - 用于 JavaScript 绑定和测试
- **现代浏览器** - 支持 WebAssembly 1.0+

### 安装依赖

在您的 `Cargo.toml` 中添加以下依赖：

```toml
[dependencies]
gaia-assembler = { path = "../gaia-assembler" }
gaia-types = { path = "../gaia-types" }
wasmtime = "14.0"  # WASM 运行时
wasm-bindgen = "0.2"  # JavaScript 绑定

[dependencies.web-sys]
version = "0.3"
features = [
    "console",
    "Document",
    "Element",
    "HtmlElement",
    "Window",
]
```

### 开发工具（可选）

```bash
# WebAssembly 二进制工具包
cargo install wabt

# WASM 优化工具
cargo install wasm-opt

# WASM 绑定生成器
cargo install wasm-pack
```

## 第一个 WASM 模块

### 1. 创建项目结构

```
my-wasm-project/
├── src/
│   ├── lib.rs
│   └── main.rs
├── Cargo.toml
├── index.html
└── package.json
```

### 2. 编写基础代码

**src/lib.rs**:

```rust
use gaia_assembler::backends::wasm::*;
use gaia_types::*;

pub fn create_calculator_module() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // 创建 WASM 汇编器
    let mut assembler = WasmAssembler::new();

    // 设置模块信息
    assembler.set_module_name("calculator");

    // 定义函数类型：(i32, i32) -> i32
    let binary_op_type = assembler.add_function_type(
        vec![ValType::I32, ValType::I32],
        vec![ValType::I32]
    );

    // 创建加法函数
    let add_func = assembler.add_function(binary_op_type);
    add_func.add_instructions(&[
        Instruction::LocalGet(0),    // 获取第一个参数
        Instruction::LocalGet(1),    // 获取第二个参数
        Instruction::I32Add,         // 执行加法
        Instruction::End,            // 函数结束
    ]);

    // 创建减法函数
    let sub_func = assembler.add_function(binary_op_type);
    sub_func.add_instructions(&[
        Instruction::LocalGet(0),    // 获取第一个参数
        Instruction::LocalGet(1),    // 获取第二个参数
        Instruction::I32Sub,         // 执行减法
        Instruction::End,            // 函数结束
    ]);

    // 创建乘法函数
    let mul_func = assembler.add_function(binary_op_type);
    mul_func.add_instructions(&[
        Instruction::LocalGet(0),    // 获取第一个参数
        Instruction::LocalGet(1),    // 获取第二个参数
        Instruction::I32Mul,         // 执行乘法
        Instruction::End,            // 函数结束
    ]);

    // 导出函数
    assembler.add_export("add", ExportKind::Function(add_func.index()));
    assembler.add_export("sub", ExportKind::Function(sub_func.index()));
    assembler.add_export("mul", ExportKind::Function(mul_func.index()));

    // 生成 WASM 字节码
    let wasm_bytes = assembler.build()?;

    Ok(wasm_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasmtime::*;

    #[test]
    fn test_calculator_module() {
        let wasm_bytes = create_calculator_module().unwrap();

        // 创建 WASM 运行时
        let engine = Engine::default();
        let module = Module::new(&engine, &wasm_bytes).unwrap();
        let mut store = Store::new(&engine, ());
        let instance = Instance::new(&mut store, &module, &[]).unwrap();

        // 获取导出的函数
        let add = instance.get_typed_func::<(i32, i32), i32>(&mut store, "add").unwrap();
        let sub = instance.get_typed_func::<(i32, i32), i32>(&mut store, "sub").unwrap();
        let mul = instance.get_typed_func::<(i32, i32), i32>(&mut store, "mul").unwrap();

        // 测试函数
        assert_eq!(add.call(&mut store, (10, 20)).unwrap(), 30);
        assert_eq!(sub.call(&mut store, (20, 10)).unwrap(), 10);
        assert_eq!(mul.call(&mut store, (5, 6)).unwrap(), 30);
    }
}
```

**src/main.rs**:

```rust
use std::fs;

mod lib;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 生成 WASM 模块
    let wasm_bytes = lib::create_calculator_module()?;

    // 保存到文件
    fs::write("calculator.wasm", &wasm_bytes)?;
    println!("✅ WASM 模块已生成: calculator.wasm ({} bytes)", wasm_bytes.len());

    // 生成 JavaScript 绑定
    generate_js_bindings()?;

    Ok(())
}

fn generate_js_bindings() -> Result<(), Box<dyn std::error::Error>> {
    let js_code = r#"
class Calculator {
    constructor() {
        this.instance = null;
    }
    
    async init() {
        const wasmBytes = await fetch('./calculator.wasm').then(r => r.arrayBuffer());
        const wasmModule = await WebAssembly.compile(wasmBytes);
        this.instance = await WebAssembly.instantiate(wasmModule);
    }
    
    add(a, b) {
        return this.instance.exports.add(a, b);
    }
    
    sub(a, b) {
        return this.instance.exports.sub(a, b);
    }
    
    mul(a, b) {
        return this.instance.exports.mul(a, b);
    }
}

// 导出给浏览器使用
if (typeof window !== 'undefined') {
    window.Calculator = Calculator;
}

// 导出给 Node.js 使用
if (typeof module !== 'undefined') {
    module.exports = Calculator;
}
"#;

    fs::write("calculator.js", js_code)?;
    println!("✅ JavaScript 绑定已生成: calculator.js");

    Ok(())
}
```

### 3. 创建 HTML 测试页面

**index.html**:

```html
<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>WASM 计算器演示</title>
    <style>
        body {
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            max-width: 600px;
            margin: 50px auto;
            padding: 20px;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
        }

        .calculator {
            background: rgba(255, 255, 255, 0.1);
            padding: 30px;
            border-radius: 15px;
            backdrop-filter: blur(10px);
            box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
        }

        .input-group {
            margin: 15px 0;
        }

        label {
            display: block;
            margin-bottom: 5px;
            font-weight: bold;
        }

        input {
            width: 100%;
            padding: 10px;
            border: none;
            border-radius: 5px;
            font-size: 16px;
            background: rgba(255, 255, 255, 0.9);
            color: #333;
        }

        .buttons {
            display: grid;
            grid-template-columns: repeat(3, 1fr);
            gap: 10px;
            margin: 20px 0;
        }

        button {
            padding: 15px;
            border: none;
            border-radius: 8px;
            font-size: 18px;
            font-weight: bold;
            cursor: pointer;
            transition: all 0.3s ease;
            background: linear-gradient(45deg, #ff6b6b, #ee5a24);
            color: white;
        }

        button:hover {
            transform: translateY(-2px);
            box-shadow: 0 5px 15px rgba(0, 0, 0, 0.3);
        }

        .result {
            margin-top: 20px;
            padding: 15px;
            background: rgba(255, 255, 255, 0.2);
            border-radius: 8px;
            font-size: 24px;
            font-weight: bold;
            text-align: center;
            min-height: 30px;
        }

        .loading {
            text-align: center;
            color: #ffd700;
        }
    </style>
</head>
<body>
<div class="calculator">
    <h1>🚀 WASM 计算器演示</h1>
    <p>使用 Gaia 生成的 WebAssembly 模块</p>

    <div class="input-group">
        <label for="num1">第一个数字:</label>
        <input type="number" id="num1" value="10" placeholder="输入第一个数字">
    </div>

    <div class="input-group">
        <label for="num2">第二个数字:</label>
        <input type="number" id="num2" value="5" placeholder="输入第二个数字">
    </div>

    <div class="buttons">
        <button onclick="calculate('add')">➕ 加法</button>
        <button onclick="calculate('sub')">➖ 减法</button>
        <button onclick="calculate('mul')">✖️ 乘法</button>
    </div>

    <div class="result" id="result">
        <div class="loading">正在加载 WASM 模块...</div>
    </div>
</div>

<script src="calculator.js"></script>
<script>
    let calculator = null;

    async function initCalculator() {
        try {
            calculator = new Calculator();
            await calculator.init();
            document.getElementById('result').innerHTML = '✅ WASM 模块加载成功！选择操作开始计算';
        } catch (error) {
            document.getElementById('result').innerHTML = `❌ 加载失败: ${error.message}`;
            console.error('WASM 加载失败:', error);
        }
    }

    function calculate(operation) {
        if (!calculator) {
            document.getElementById('result').innerHTML = '❌ WASM 模块未加载';
            return;
        }

        const num1 = parseInt(document.getElementById('num1').value) || 0;
        const num2 = parseInt(document.getElementById('num2').value) || 0;

        let result;
        let operationSymbol;

        try {
            switch (operation) {
                case 'add':
                    result = calculator.add(num1, num2);
                    operationSymbol = '+';
                    break;
                case 'sub':
                    result = calculator.sub(num1, num2);
                    operationSymbol = '-';
                    break;
                case 'mul':
                    result = calculator.mul(num1, num2);
                    operationSymbol = '×';
                    break;
                default:
                    throw new Error('未知操作');
            }

            document.getElementById('result').innerHTML =
                    `🎯 ${num1} ${operationSymbol} ${num2} = <strong>${result}</strong>`;
        } catch (error) {
            document.getElementById('result').innerHTML = `❌ 计算错误: ${error.message}`;
        }
    }

    // 页面加载时初始化
    window.addEventListener('load', initCalculator);

    // 支持回车键计算
    document.addEventListener('keypress', function (e) {
        if (e.key === 'Enter') {
            calculate('add');
        }
    });
</script>
</body>
</html>
```

## 构建和运行

### 1. 编译 WASM 模块

```bash
# 构建项目
cargo build --release

# 运行生成器
cargo run --release
```

### 2. 启动本地服务器

```bash
# 使用 Python
python -m http.server 8000

# 或使用 Node.js
npx serve .

# 或使用 Rust
cargo install basic-http-server
basic-http-server .
```

### 3. 在浏览器中测试

打开 `http://localhost:8000`，您应该看到一个漂亮的计算器界面。

## 验证结果

### 1. 检查生成的文件

```bash
ls -la
# 应该看到:
# calculator.wasm  - WASM 二进制文件
# calculator.js    - JavaScript 绑定
# index.html       - 测试页面
```

### 2. 查看 WASM 模块信息

```bash
# 使用 wabt 工具查看模块结构
wasm-objdump -x calculator.wasm

# 反汇编查看指令
wasm2wat calculator.wasm
```

### 3. 运行单元测试

```bash
cargo test
```

## 常见问题解决

### 问题 1: CORS 错误

**错误**: `Access to fetch at 'file://...' from origin 'null' has been blocked by CORS policy`

**解决**: 必须通过 HTTP 服务器访问，不能直接打开 HTML 文件。

### 问题 2: WASM 模块加载失败

**错误**: `WebAssembly.compile(): Compiling function failed`

**解决**: 检查生成的 WASM 文件是否有效：

```bash
wasm-validate calculator.wasm
```

### 问题 3: 函数未导出

**错误**: `TypeError: instance.exports.add is not a function`

**解决**: 确保在汇编器中正确导出了函数：

```rust
assembler.add_export("add", ExportKind::Function(add_func.index()));
```

## 性能测试

创建一个简单的性能测试：

```javascript
// 添加到 HTML 中的脚本
async function performanceTest() {
    if (!calculator) return;

    const iterations = 1000000;
    const start = performance.now();

    for (let i = 0; i < iterations; i++) {
        calculator.add(i, i + 1);
    }

    const end = performance.now();
    const duration = end - start;
    const opsPerSecond = (iterations / duration * 1000).toFixed(0);

    console.log(`性能测试结果:`);
    console.log(`- 执行次数: ${iterations.toLocaleString()}`);
    console.log(`- 总耗时: ${duration.toFixed(2)}ms`);
    console.log(`- 每秒操作数: ${opsPerSecond.toLocaleString()}`);
}
```

## 下一步

恭喜！您已经成功创建了第一个 WASM 模块。接下来可以：

1. 📖 阅读 [**基础概念**](./concepts.md) 了解 WASM 核心概念
2. 🏗️ 学习 [**模块结构**](./module-structure.md) 深入理解 WASM 格式
3. 🔧 探索 [**内存管理**](./memory-management.md) 处理复杂数据
4. 🌐 掌握 [**JavaScript 互操作**](./js-interop.md) 实现更复杂的交互

## 示例代码仓库

完整的示例代码可以在以下位置找到：

- [基础计算器示例](./examples/calculator/)
- [图像处理示例](./examples/image-processing/)
- [游戏引擎示例](./examples/game-engine/)

---

*遇到问题？查看 [故障排除指南](./debugging.md) 或在 [GitHub Issues](https://github.com/nyar-vm/gaia/issues) 中提问。*