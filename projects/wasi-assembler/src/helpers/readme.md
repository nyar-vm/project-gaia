# 辅助工具模块

这个模块提供了 WASI 汇编器使用的各种辅助功能和工具函数，包括：
- **工具函数**: 常用的工具函数
- **类型转换**: 各种类型之间的转换函数
- **验证函数**: 输入验证和检查函数
- **常量定义**: 常用的常量和枚举值

## 主要功能

### 工具函数

提供各种实用的工具函数：
- 字符串处理和验证
- 数值转换和编码
- 错误处理和格式化
- 调试和日志功能

### 类型转换

支持各种类型之间的转换：
- WebAssembly 类型与 Rust 类型的转换
- 文本格式与二进制格式的转换
- 高层次结构与低层次表示的转换

### 验证函数

提供输入验证和检查功能：
- 语法验证
- 语义检查
- 类型验证
- 边界检查

## 使用示例

```rust
use wasi_assembler::helpers::*;

// 验证函数名称
if is_valid_function_name("my_function") {
    println!("函数名称有效");
}

// 转换 WebAssembly 类型
let wasm_type = rust_type_to_wasm_type::<i32>();
println!("Rust i32 对应的 WASM 类型: {:?}", wasm_type);

// 编码 LEB128 数值
let value = 12345u32;
let encoded = encode_leb128(value);
println!("LEB128 编码结果: {:?}", encoded);

// 验证模块结构
let module = create_sample_module();
if validate_module_structure(&module) {
    println!("模块结构有效");
}
```

## 扩展性

这个模块设计为可扩展的，可以轻松添加新的辅助函数：

```rust
use wasi_assembler::helpers::*;

// 添加自定义验证函数
pub fn validate_custom_section(name: &str, data: &[u8]) -> bool {
    // 自定义验证逻辑
    !name.is_empty() && !data.is_empty() && data.len() <= 65535
}

// 使用现有工具函数
let name = "my_custom_section";
let data = vec![1, 2, 3, 4, 5];
if validate_custom_section(name, &data) {
    println!("自定义段有效");
}
```