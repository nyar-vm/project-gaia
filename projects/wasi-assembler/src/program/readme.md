# WASM 程序高层次抽象

这个模块定义了 WASM 程序的高层次表示，用于在 WAT AST 和二进制 WASM 之间提供中间抽象层。
支持 WebAssembly Component Model 和传统的核心模块。

## 核心概念

### `WasiProgram`

主要的程序结构，可以表示两种类型的 WebAssembly 程序：
- **核心模块** (`WasiProgramType::CoreModule`): 传统的 WebAssembly 模块
- **组件** (`WasiProgramType::Component`): WebAssembly Component Model 组件

### 程序组件

程序包含以下主要组件：
- **函数类型**: 定义函数的签名（参数和返回值类型）
- **函数**: 实际的函数实现
- **导入**: 从外部模块导入的函数、内存等
- **导出**: 向外部暴露的函数、内存等
- **内存**: 线性内存定义
- **表**: 函数表定义
- **全局变量**: 全局变量定义
- **自定义段**: 自定义数据段

### 组件模型支持

对于组件类型，还支持：
- **组件项目**: 类型定义、别名、实例等
- **核心模块**: 嵌套的核心模块
- **实例**: 组件或模块实例
- **别名**: 名称别名定义

## 使用示例

### 创建简单的核心模块

```rust,no_run
use wasi_assembler::program::{
    WasiProgram, WasiProgramType, WasiFunctionType, WasiFunction,
    WasiExport, WasiExportType, WasiInstruction, WasiType
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建核心模块程序
    let mut program = WasiProgram::new(WasiProgramType::CoreModule);
    
    // 添加函数类型 (i32, i32) -> i32
    program.function_types.push(WasiFunctionType {
        params: vec![WasiType::I32, WasiType::I32],
        results: vec![WasiType::I32],
    });
    
    // 添加函数实现
    program.functions.push(WasiFunction {
        name: Some("add".to_string()),
        function_type: 0, // 使用第一个函数类型
        locals: vec![],   // 没有局部变量
        body: vec![
            WasiInstruction::LocalGet(0),  // 获取第一个参数
            WasiInstruction::LocalGet(1),  // 获取第二个参数
            WasiInstruction::I32Add,       // i32 加法
        ],
    });
    
    // 导出函数
    program.exports.push(WasiExport {
        name: "add".to_string(),
        export_type: WasiExportType::Function,
        index: 0, // 第一个函数
    });
    
    // 生成 WASM 字节码
    let wasm_bytes = program.to_wasm()?;
    Ok(())
}
```

### 创建组件

```rust,no_run
use wasi_assembler::program::{
    WasiProgram, WasiProgramType, WasiComponentItem,
    WasiTypeDefinition, WasiType, WasiInstance, WasiInstanceType
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建组件程序
    let mut program = WasiProgram::new(WasiProgramType::Component);
    
    // 添加接口类型
    program.component_items.push(WasiComponentItem::Type(
        WasiTypeDefinition {
            name: Some("calculator-interface".to_string()),
            index: 0,
            type_content: WasiType::Interface("calculator".to_string()),
        }
    ));
    
    // 添加实例
    program.instances.push(WasiInstance {
        name: Some("calc".to_string()),
        index: 0,
        instantiate_target: "calculator-interface".to_string(),
        args: vec![],
        instance_type: WasiInstanceType::Component,
    });
    
    // 生成组件字节码
    let component_bytes = program.to_wasm()?;
    Ok(())
}
```

## 符号表管理

程序包含一个符号表，用于管理名称到索引的映射：

```rust,no_run
use wasi_assembler::program::{WasiProgram, WasiProgramType, WasiSymbol};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut program = WasiProgram::new(WasiProgramType::CoreModule);
    
    // 符号表自动维护名称到索引的映射
    program.symbol_table.insert("my_function".to_string(), WasiSymbol::Function(0));
    program.symbol_table.insert("my_memory".to_string(), WasiSymbol::Memory(0));
    
    // 通过名称查找符号
    if let Some(WasiSymbol::Function(idx)) = program.symbol_table.get("my_function") {
        println!("函数 my_function 的索引是: {}", idx);
    }
    Ok(())
}
```