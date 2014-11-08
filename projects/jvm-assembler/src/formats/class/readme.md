# Class 文件格式

Java 类文件格式的读取器和写入器，支持标准 JVM 类文件结构的解析和生成。

## 概述

Class 文件格式是 Java 虚拟机执行的标准二进制格式。该模块提供：

- **类文件读取**: 从二进制数据解析类文件结构
- **类文件写入**: 将类文件结构序列化为二进制数据
- **完整支持**: 支持所有标准类文件组件
- **验证机制**: 内置格式验证和错误检查
- **性能优化**: 高效处理大型类文件

## 类文件结构

Java 类文件遵循严格的二进制格式：

```
ClassFile {
    u4             magic;                    // 魔数: 0xCAFEBABE
    u2             minor_version;            // 次版本号
    u2             major_version;            // 主版本号
    u2             constant_pool_count;      // 常量池计数
    cp_info        constant_pool[constant_pool_count-1];  // 常量池
    u2             access_flags;               // 访问标志
    u2             this_class;                // 当前类索引
    u2             super_class;               // 超类索引
    u2             interfaces_count;          // 接口计数
    u2             interfaces[interfaces_count];          // 接口数组
    u2             fields_count;              // 字段计数
    field_info     fields[fields_count];      // 字段数组
    u2             methods_count;             // 方法计数
    method_info    methods[methods_count];     // 方法数组
    u2             attributes_count;          // 属性计数
    attribute_info attributes[attributes_count];          // 属性数组
}
```

## 支持的版本

| Java 版本 | 主版本号 | 次版本号 | 支持状态 |
|-----------|----------|----------|----------|
| Java 1.1  | 45       | 3        | ✅ 支持 |
| Java 1.2  | 46       | 0        | ✅ 支持 |
| Java 1.3  | 47       | 0        | ✅ 支持 |
| Java 1.4  | 48       | 0        | ✅ 支持 |
| Java 5    | 49       | 0        | ✅ 支持 |
| Java 6    | 50       | 0        | ✅ 支持 |
| Java 7    | 51       | 0        | ✅ 支持 |
| Java 8    | 52       | 0        | ✅ 支持 |
| Java 9    | 53       | 0        | ✅ 支持 |
| Java 10   | 54       | 0        | ✅ 支持 |
| Java 11   | 55       | 0        | ✅ 支持 |
| Java 12   | 56       | 0        | ✅ 支持 |
| Java 13   | 57       | 0        | ✅ 支持 |
| Java 14   | 58       | 0        | ✅ 支持 |
| Java 15   | 59       | 0        | ✅ 支持 |
| Java 16   | 60       | 0        | ✅ 支持 |
| Java 17   | 61       | 0        | ✅ 支持 |
| Java 18   | 62       | 0        | ✅ 支持 |
| Java 19   | 63       | 0        | ✅ 支持 |
| Java 20   | 64       | 0        | ✅ 支持 |
| Java 21   | 65       | 0        | ✅ 支持 |

## 常量池类型

支持所有标准常量池条目类型：

### 基本类型
- **CONSTANT_Utf8**: UTF-8 编码的字符串
- **CONSTANT_Integer**: 32位整数
- **CONSTANT_Float**: 32位浮点数
- **CONSTANT_Long**: 64位长整数
- **CONSTANT_Double**: 64位双精度浮点数

### 引用类型
- **CONSTANT_Class**: 类或接口的符号引用
- **CONSTANT_String**: 字符串对象的符号引用
- **CONSTANT_Fieldref**: 字段的符号引用
- **CONSTANT_Methodref**: 类方法的符号引用
- **CONSTANT_InterfaceMethodref**: 接口方法的符号引用
- **CONSTANT_NameAndType**: 字段或方法的名称和类型描述符

### 动态类型
- **CONSTANT_MethodHandle**: 方法句柄
- **CONSTANT_MethodType**: 方法类型
- **CONSTANT_Dynamic**: 动态计算常量
- **CONSTANT_InvokeDynamic**: 动态方法调用

## 属性支持

### 标准属性
- **Code**: 方法的字节码和异常表
- **ConstantValue**: 常量字段的常量值
- **SourceFile**: 源文件名
- **LineNumberTable**: 字节码到源代码行号的映射
- **LocalVariableTable**: 局部变量的调试信息
- **LocalVariableTypeTable**: 泛型局部变量的调试信息

### 高级属性
- **StackMapTable**: 栈映射帧（用于验证）
- **Exceptions**: 方法抛出的异常类型
- **InnerClasses**: 内部类信息
- **EnclosingMethod**: 匿名类的封闭方法
- **Signature**: 泛型签名信息
- **RuntimeVisibleAnnotations**: 运行时可见注解
- **RuntimeInvisibleAnnotations**: 运行时不可见注解

## 使用示例

### 读取类文件

```rust
use jvm_assembler::formats::class::reader::ClassReader;
use std::fs;
use std::io::Cursor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 读取类文件
    let bytes = fs::read("Example.class")?;
    
    // 解析类文件
    let cursor = Cursor::new(bytes);
    let mut reader = ClassReader::new(cursor);
    let result = reader.read();
    
    match result {
        Ok(program) => {
            // 访问类信息
            println!("Class name: {}", program.name);
            println!("Super class: {:?}", program.super_class);
            println!("Methods: {}", program.methods.len());
        }
        Err(error) => {
            return Err(error.into());
        }
    }
    
    Ok(())
}
```

### 写入类文件

```rust
use jvm_assembler::formats::class::writer::ClassWriter;
use jvm_assembler::program::{JvmProgram, JvmMethod, JvmField};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建 JVM 程序
    let mut program = JvmProgram::new("Example".to_string());
    
    // 添加方法和字段
    program.add_method(JvmMethod::new("main".to_string(), "([Ljava/lang/String;)V".to_string()));
    program.add_field(JvmField::new("count".to_string(), "I".to_string()));
    
    // 写入类文件
    let buffer = Vec::new();
    let writer = ClassWriter::new(buffer);
    let result = writer.write(program);
    
    match result {
        Ok(buffer) => {
            // 保存到文件
            std::fs::write("Example.class", buffer)?;
        }
        Err(error) => {
            return Err(error.into());
        }
    }
    
    Ok(())
}
```

## 错误处理

类文件读取和写入过程中可能遇到的错误：

### 读取错误
- **魔数错误**: 无效的类文件魔数
- **版本不支持**: 不支持的 Java 版本
- **格式错误**: 损坏的类文件格式
- **常量池错误**: 无效的常量池引用
- **验证错误**: 字节码验证失败

### 写入错误
- **数据过大**: 超出类文件格式限制
- **引用错误**: 无效的交叉引用
- **编码错误**: 字符串编码问题

所有错误都通过 [`GaiaError`](../../gaia_types/struct.GaiaError.html) 类型返回，提供详细的错误信息。

## 性能特性

- **内存效率**: 流式处理大型类文件
- **快速解析**: 优化的二进制解析算法
- **零拷贝**: 尽可能减少内存分配
- **缓存友好**: 优化的数据结构设计

## 相关模块

- [`reader`](reader/index.html) - 类文件读取器
- [`writer`](writer/index.html) - 类文件写入器
- [`program`](../../program/index.html) - JVM 程序高级抽象