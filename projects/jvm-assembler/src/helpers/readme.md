# 辅助工具模块

提供 JVM 汇编器使用的各种辅助工具和实用函数。

## 概述

该模块包含 JVM 汇编器使用的各种辅助工具、实用函数和通用数据结构，提供：

- **字节操作**: 安全的字节序列处理工具
- **字符串处理**: JVM 特定的字符串转换和验证
- **数值转换**: 各种数值类型的字节表示转换
- **验证工具**: JVM 规范符合性验证
- **调试支持**: 调试和日志记录工具
- **性能优化**: 高效的算法和数据结构

## 核心功能

### 字节操作工具

```rust
/// 安全的字节读取工具（概念性展示）
pub struct ByteReader {
    data: Vec<u8>,
    position: usize,
}

impl ByteReader {
    /// 创建新的字节读取器
    pub fn new(data: Vec<u8>) -> Self {
        ByteReader { data, position: 0 }
    }
    
    /// 读取 u8 值
    pub fn read_u8(&mut self) -> Result<u8, GaiaError> {
        if self.position >= self.data.len() {
            return Err(GaiaError::custom_error("Buffer underflow".to_string()));
        }
        let value = self.data[self.position];
        self.position += 1;
        Ok(value)
    }
    
    /// 读取 u16 值（大端序）
    pub fn read_u16(&mut self) -> Result<u16, GaiaError> {
        if self.position + 2 > self.data.len() {
            return Err(GaiaError::custom_error("Buffer underflow".to_string()));
        }
        let value = ((self.data[self.position] as u16) << 8) | (self.data[self.position + 1] as u16);
        self.position += 2;
        Ok(value)
    }
    
    /// 读取 u32 值（大端序）
    pub fn read_u32(&mut self) -> Result<u32, GaiaError> {
        if self.position + 4 > self.data.len() {
            return Err(GaiaError::custom_error("Buffer underflow".to_string()));
        }
        let value = ((self.data[self.position] as u32) << 24) |
                   ((self.data[self.position + 1] as u32) << 16) |
                   ((self.data[self.position + 2] as u32) << 8) |
                   (self.data[self.position + 3] as u32);
        self.position += 4;
        Ok(value)
    }
    
    /// 读取 i32 值（大端序）
    pub fn read_i32(&mut self) -> Result<i32, GaiaError> {
        let value = self.read_u32()?;
        Ok(value as i32)
    }
    
    /// 读取指定长度的字节数组
    pub fn read_bytes(&mut self, length: usize) -> Result<Vec<u8>, GaiaError> {
        if self.position + length > self.data.len() {
            return Err(GaiaError::custom_error("Buffer underflow".to_string()));
        }
        let bytes = self.data[self.position..self.position + length].to_vec();
        self.position += length;
        Ok(bytes)
    }
    
    /// 检查是否还有剩余字节
    pub fn has_remaining(&self) -> bool {
        self.position < self.data.len()
    }
}
```

### 字节写入工具

```rust
/// 高效的字节写入工具（概念性展示）
pub struct ByteWriter {
    buffer: Vec<u8>,
}

impl ByteWriter {
    /// 创建新的字节写入器
    pub fn new() -> Self {
        ByteWriter { buffer: Vec::new() }
    }
    
    /// 写入 u8 值
    pub fn write_u8(&mut self, value: u8) {
        self.buffer.push(value);
    }
    
    /// 写入 u16 值（大端序）
    pub fn write_u16(&mut self, value: u16) {
        self.buffer.push((value >> 8) as u8);
        self.buffer.push(value as u8);
    }
    
    /// 写入 u32 值（大端序）
    pub fn write_u32(&mut self, value: u32) {
        self.buffer.push((value >> 24) as u8);
        self.buffer.push((value >> 16) as u8);
        self.buffer.push((value >> 8) as u8);
        self.buffer.push(value as u8);
    }
    
    /// 写入 i32 值（大端序）
    pub fn write_i32(&mut self, value: i32) {
        self.write_u32(value as u32);
    }
    
    /// 写入字节数组
    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.buffer.extend_from_slice(bytes);
    }
    
    /// 获取写入的字节
    pub fn into_bytes(self) -> Vec<u8> {
        self.buffer
    }
}
```

### 字符串处理

```rust
/// JVM 字符串验证和转换工具（概念性展示）
pub struct JvmStringUtils;

impl JvmStringUtils {
    /// 验证类名格式
    pub fn validate_class_name(name: &str) -> Result<(), GaiaError> {
        if name.is_empty() {
            return Err(GaiaError::custom_error("Class name cannot be empty".to_string()));
        }
        // 简化的验证逻辑
        if name.contains("..") {
            return Err(GaiaError::custom_error("Invalid class name format".to_string()));
        }
        Ok(())
    }
    
    /// 验证方法名格式
    pub fn validate_method_name(name: &str) -> Result<(), GaiaError> {
        if name.is_empty() {
            return Err(GaiaError::custom_error("Method name cannot be empty".to_string()));
        }
        Ok(())
    }
    
    /// 验证字段名格式
    pub fn validate_field_name(name: &str) -> Result<(), GaiaError> {
        if name.is_empty() {
            return Err(GaiaError::custom_error("Field name cannot be empty".to_string()));
        }
        Ok(())
    }
    
    /// 验证描述符格式
    pub fn validate_descriptor(desc: &str) -> Result<(), GaiaError> {
        if desc.is_empty() {
            return Err(GaiaError::custom_error("Descriptor cannot be empty".to_string()));
        }
        Ok(())
    }
    
    /// 将内部类名转换为源类名
    pub fn internal_to_source_name(internal: &str) -> String {
        internal.replace('/', ".")
    }
    
    /// 将源类名转换为内部类名
    pub fn source_to_internal_name(source: &str) -> String {
        source.replace(".", "/")
    }
    
    /// 转义字符串中的特殊字符
    pub fn escape_string(s: &str) -> String {
        s.replace("\\", "\\\\").replace("\"", "\\\"")
    }
}
```

### 数值转换

```rust
/// 数值类型的字节表示转换（概念性展示）
pub struct NumericUtils;

impl NumericUtils {
    /// 将 f32 转换为 IEEE 754 位表示
    pub fn f32_to_bits(value: f32) -> u32 {
        value.to_bits()
    }
    
    /// 将 IEEE 754 位表示转换为 f32
    pub fn bits_to_f32(bits: u32) -> f32 {
        f32::from_bits(bits)
    }
    
    /// 将 f64 转换为 IEEE 754 位表示
    pub fn f64_to_bits(value: f64) -> u64 {
        value.to_bits()
    }
    
    /// 将 IEEE 754 位表示转换为 f64
    pub fn bits_to_f64(bits: u64) -> f64 {
        f64::from_bits(bits)
    }
    
    /// 安全地将 i64 转换为 i32
    pub fn safe_i64_to_i32(value: i64) -> Result<i32, GaiaError> {
        if value > i32::MAX as i64 || value < i32::MIN as i64 {
            return Err(GaiaError::custom_error("i64 value out of i32 range".to_string()));
        }
        Ok(value as i32)
    }
    
    /// 安全地将 u64 转换为 u32
    pub fn safe_u64_to_u32(value: u64) -> Result<u32, GaiaError> {
        if value > u32::MAX as u64 {
            return Err(GaiaError::custom_error("u64 value out of u32 range".to_string()));
        }
        Ok(value as u32)
    }
}
```

### 验证工具

```rust
/// JVM 规范符合性验证（概念性展示）
pub struct JvmValidator;

impl JvmValidator {
    /// 验证访问标志组合
    pub fn validate_access_flags(flags: &JvmAccessFlags) -> Result<(), GaiaError> {
        // 简化的验证逻辑
        if flags.is_public && flags.is_private {
            return Err(GaiaError::custom_error("Cannot be both public and private".to_string()));
        }
        Ok(())
    }
    
    /// 验证方法签名
    pub fn validate_method_signature(name: &str, descriptor: &str) -> Result<(), GaiaError> {
        JvmStringUtils::validate_method_name(name)?;
        JvmStringUtils::validate_descriptor(descriptor)?;
        Ok(())
    }
    
    /// 验证字段签名
    pub fn validate_field_signature(name: &str, descriptor: &str) -> Result<(), GaiaError> {
        JvmStringUtils::validate_field_name(name)?;
        JvmStringUtils::validate_descriptor(descriptor)?;
        Ok(())
    }
    
    /// 验证类继承关系
    pub fn validate_class_hierarchy(program: &JvmProgram) -> Result<(), GaiaError> {
        // 简化的验证逻辑
        if program.access_flags.is_interface && !program.access_flags.is_abstract {
            return Err(GaiaError::custom_error("Interface must be abstract".to_string()));
        }
        Ok(())
    }
    
    /// 验证指令序列
    pub fn validate_instructions(instructions: &[JvmInstruction]) -> Result<(), GaiaError> {
        if instructions.is_empty() {
            return Err(GaiaError::custom_error("Instruction sequence cannot be empty".to_string()));
        }
        Ok(())
    }
}
```

## 使用示例

### 字节操作

```rust
use jvm_assembler::helpers::*;

fn main() -> Result<()> {
    // 创建字节读取器
    let data = vec![0xCA, 0xFE, 0xBA, 0xBE];
    let mut reader = ByteReader::new(data);
    
    // 读取魔数
    let magic = reader.read_u32()?;
    assert_eq!(magic, 0xCAFEBABE);
    
    Ok(())
}
```

### 字符串验证

```rust
use jvm_assembler::helpers::*;

fn main() -> Result<()> {
    // 验证类名
    JvmStringUtils::validate_class_name("java/lang/String")?;
    
    // 转换类名格式
    let internal = "java/lang/String";
    let source = JvmStringUtils::internal_to_source_name(internal);
    assert_eq!(source, "java.lang.String");
    
    Ok(())
}
```

### 数值转换

```rust
use jvm_assembler::helpers::*;

fn main() {
    // 浮点数转换
    let value = 3.14f32;
    let bits = NumericUtils::f32_to_bits(value);
    let restored = NumericUtils::bits_to_f32(bits);
    assert_eq!(value, restored);
    
    // 安全数值转换
    let big_value: i64 = 100;
    let small_value = NumericUtils::safe_i64_to_i32(big_value).unwrap();
    assert_eq!(small_value, 100);
}
```

### 验证工具

```rust
use jvm_assembler::helpers::*;
use jvm_assembler::program::*;

fn main() -> Result<()> {
    // 验证访问标志
    let mut flags = JvmAccessFlags::new();
    flags.is_public = true;
    flags.is_static = true;
    JvmValidator::validate_access_flags(&flags)?;
    
    // 验证方法签名
    JvmValidator::validate_method_signature("main", "([Ljava/lang/String;)V")?;
    
    Ok(())
}
```

## 性能特性

- **零拷贝**: 尽可能避免不必要的数据复制
- **内存池**: 智能内存管理，减少分配开销
- **缓存友好**: 数据结构设计优化 CPU 缓存利用率
- **并行安全**: 支持多线程环境下的安全操作
- **编译时优化**: 利用 Rust 的零成本抽象

## 错误处理

所有工具函数都使用 `Result<T, GaiaError>` 返回类型，提供详细的错误信息：

```rust
pub enum HelperError {
    InvalidUtf8 { position: usize },           // UTF-8 编码错误
    InvalidClassName { name: String },         // 无效类名
    InvalidDescriptor { desc: String },        // 无效描述符
    NumericOverflow { value: i64 },          // 数值溢出
    InvalidAccessFlags { flags: String },    // 无效访问标志
    BufferUnderflow { required: usize, available: usize }, // 缓冲区下溢
}
```

## 相关模块

- [`program`](../program/index.html) - JVM 程序抽象
- [`formats`](../formats/index.html) - 文件格式支持
- [`gaia_types`](../../gaia_types/index.html) - 错误处理和结果类型