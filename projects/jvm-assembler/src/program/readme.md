# JVM 程序抽象

JVM 程序的高级抽象表示，提供对 Java 类文件结构的程序化访问和操作。

## 概述

该模块定义了 JVM 程序的高级表示，作为 JASM AST 和二进制 Class 文件之间的中间抽象层。它提供了：

- **类型安全**: Rust 类型系统保证的程序结构
- **内存安全**: 自动内存管理和生命周期管理
- **高性能**: 零成本抽象和优化数据结构
- **完整性**: 支持所有 JVM 概念和特性
- **可扩展性**: 易于扩展和自定义的程序结构

## 核心类型

### JvmProgram

JVM 程序的顶级表示，包含完整的类定义信息：

```rust
pub struct JvmProgram {
    pub name: String,                    // 程序名称（类名）
    pub access_flags: JvmAccessFlags,      // 访问标志
    pub super_class: Option<String>,     // 超类名称
    pub interfaces: Vec<String>,           // 实现的接口列表
    pub fields: Vec<JvmField>,             // 字段列表
    pub methods: Vec<JvmMethod>,           // 方法列表
    pub attributes: Vec<JvmAttribute>,     // 属性列表
    pub constant_pool: JvmConstantPool,    // 常量池
    pub version: JvmVersion,               // 版本信息
    pub source_file: Option<String>,       // 源文件信息
}
```

### JvmMethod

JVM 方法的完整表示：

```rust
pub struct JvmMethod {
    pub name: String,                      // 方法名称
    pub descriptor: String,                // 方法描述符
    pub access_flags: JvmAccessFlags,     // 访问标志
    pub instructions: Vec<JvmInstruction>, // 指令列表
    pub max_stack: u16,                   // 最大栈深度
    pub max_locals: u16,                // 最大局部变量数
    pub exception_table: Vec<JvmExceptionHandler>, // 异常表
    pub attributes: Vec<JvmAttribute>,     // 属性列表
}
```

### JvmField

JVM 字段的表示：

```rust
pub struct JvmField {
    pub name: String,                      // 字段名称
    pub descriptor: String,                // 字段描述符
    pub access_flags: JvmAccessFlags,     // 访问标志
    pub constant_value: Option<JvmConstantPoolEntry>, // 常量值
    pub attributes: Vec<JvmAttribute>,     // 属性列表
}
```

## 指令系统

### JvmInstruction

JVM 字节码指令的统一表示：

```rust
pub enum JvmInstruction {
    Simple { opcode: JvmOpcode },                                    // 无操作数指令
    WithImmediate { opcode: JvmOpcode, value: i32 },              // 带立即数的指令
    WithLocalVar { opcode: JvmOpcode, index: u16 },               // 带局部变量索引的指令
    WithConstantPool { opcode: JvmOpcode, symbol: String },       // 带常量池引用的指令
    MethodCall { opcode: JvmOpcode, class_name: String, method_name: String, descriptor: String },
    FieldAccess { opcode: JvmOpcode, class_name: String, field_name: String, descriptor: String },
    Branch { opcode: JvmOpcode, target: String },                // 跳转指令
    TypeCast { opcode: JvmOpcode, target_type: String },           // 类型转换指令
}
```

### JvmOpcode

完整的 JVM 操作码枚举，包含所有标准字节码指令：

#### 常量加载指令
- `Nop`, `AconstNull`
- `IconstM1`, `Iconst0`, `Iconst1`, `Iconst2`, `Iconst3`, `Iconst4`, `Iconst5`
- `Lconst0`, `Lconst1`, `Fconst0`, `Fconst1`, `Fconst2`, `Dconst0`, `Dconst1`
- `Bipush`, `Sipush`, `Ldc`, `LdcW`, `Ldc2W`

#### 局部变量操作
- `Iload`, `Lload`, `Fload`, `Dload`, `Aload`
- `Istore`, `Lstore`, `Fstore`, `Dstore`, `Astore`
- `Iinc`

#### 栈操作指令
- `Pop`, `Pop2`, `Dup`, `DupX1`, `DupX2`, `Dup2`, `Dup2X1`, `Dup2X2`, `Swap`

#### 算术运算
- `Iadd`, `Ladd`, `Fadd`, `Dadd`
- `Isub`, `Lsub`, `Fsub`, `Dsub`
- `Imul`, `Lmul`, `Fmul`, `Dmul`
- `Idiv`, `Ldiv`, `Fdiv`, `Ddiv`
- `Irem`, `Lrem`, `Frem`, `Drem`
- `Ineg`, `Lneg`, `Fneg`, `Dneg`

#### 位运算
- `Ishl`, `Lshl`, `Ishr`, `Lshr`, `Iushr`, `Lushr`
- `Iand`, `Land`, `Ior`, `Lor`, `Ixor`, `Lxor`

#### 比较和跳转
- `Lcmp`, `Fcmpl`, `Fcmpg`, `Dcmpl`, `Dcmpg`
- `Ifeq`, `Ifne`, `Iflt`, `Ifge`, `Ifgt`, `Ifle`
- `IfIcmpeq`, `IfIcmpne`, `IfIcmplt`, `IfIcmpge`, `IfIcmpgt`, `IfIcmple`
- `IfAcmpeq`, `IfAcmpne`, `Goto`, `Jsr`, `Ret`

#### 方法调用
- `Invokevirtual`, `Invokespecial`, `Invokestatic`, `Invokeinterface`, `Invokedynamic`

#### 对象操作
- `New`, `Newarray`, `Anewarray`, `Arraylength`
- `Athrow`, `Checkcast`, `Instanceof`
- `Monitorenter`, `Monitorexit`

## 常量池管理

### JvmConstantPool

智能常量池管理，自动处理重复条目和符号解析：

```rust
pub struct JvmConstantPool {
    pub symbol_table: HashMap<String, u16>,  // 符号表，名称到索引的映射
    pub entries: Vec<JvmConstantPoolEntry>, // 常量池条目
}
```

### JvmConstantPoolEntry

支持所有 JVM 常量池条目类型：

```rust
pub enum JvmConstantPoolEntry {
    Utf8 { value: String },                                    // UTF-8 字符串
    Integer { value: i32 },                                    // 32位整数
    Float { value: f32 },                                      // 32位浮点数
    Long { value: i64 },                                       // 64位长整数
    Double { value: f64 },                                     // 64位双精度浮点数
    Class { name: String },                                    // 类引用
    String { value: String },                                // 字符串引用
    Fieldref { class_name: String, name: String, descriptor: String },  // 字段引用
    Methodref { class_name: String, name: String, descriptor: String }, // 方法引用
    InterfaceMethodref { class_name: String, name: String, descriptor: String }, // 接口方法引用
    NameAndType { name: String, descriptor: String },          // 名称和类型
}
```

## 访问控制

### JvmAccessFlags

完整的 JVM 访问标志支持：

```rust
pub struct JvmAccessFlags {
    pub is_public: bool,        // public
    pub is_private: bool,       // private
    pub is_protected: bool,     // protected
    pub is_static: bool,        // static
    pub is_final: bool,         // final
    pub is_super: bool,         // super（类专用）
    pub is_interface: bool,     // interface
    pub is_abstract: bool,      // abstract
    pub is_synthetic: bool,     // synthetic
    pub is_annotation: bool,    // annotation（类专用）
    pub is_enum: bool,          // enum（类专用）
}
```

## 异常处理

### JvmExceptionHandler

结构化的异常处理器表示：

```rust
pub struct JvmExceptionHandler {
    pub start_pc: u16,          // 起始 PC
    pub end_pc: u16,            // 结束 PC
    pub handler_pc: u16,        // 处理器 PC
    pub catch_type: Option<String>, // 异常类型（None 表示 finally）
}
```

## 属性系统

### JvmAttribute

支持所有标准类文件属性：

```rust
pub enum JvmAttribute {
    SourceFile { filename: String },                           // 源文件属性
    Code {
        max_stack: u16,
        max_locals: u16,
        code: Vec<u8>,
        exception_table: Vec<JvmExceptionHandler>,
        attributes: Vec<JvmAttribute>,
    },                                                        // 代码属性
    ConstantValue { value: JvmConstantPoolEntry },             // 常量值属性
    LineNumberTable { entries: Vec<(u16, u16)> },           // 行号表属性
    LocalVariableTable { entries: Vec<JvmLocalVariable> },    // 局部变量表属性
    Other { name: String, data: Vec<u8> },                  // 其他属性
}
```

## 使用示例

### 创建简单程序

```rust
use jvm_assembler::program::*;

fn main() -> Result<(), GaiaError> {
    // 创建新的 JVM 程序
    let mut program = JvmProgram::new("HelloWorld".to_string());
    
    // 设置访问标志
    program.access_flags.is_public = true;
    
    // 创建 main 方法
    let mut method = JvmMethod::new(
        "main".to_string(),
        "([Ljava/lang/String;)V".to_string()
    );
    method.access_flags.is_public = true;
    method.access_flags.is_static = true;
    
    // 添加指令
    method.add_instruction(JvmInstruction::Simple { opcode: JvmOpcode::Return });
    
    // 添加方法到程序
    program.add_method(method);
    
    // 验证程序
    program.validate()?;
    
    Ok(())
}
```

### 处理常量池

```rust
use jvm_assembler::program::*;

fn main() {
    let mut constant_pool = JvmConstantPool::new();
    
    // 添加字符串常量
    let string_index = constant_pool.add_entry(JvmConstantPoolEntry::Utf8 {
        value: "Hello, World!".to_string()
    });
    
    // 添加类引用
    let class_index = constant_pool.add_entry(JvmConstantPoolEntry::Class {
        name: "java/lang/String".to_string()
    });
    
    // 添加方法引用
    let method_index = constant_pool.add_entry(JvmConstantPoolEntry::Methodref {
        class_name: "java/io/PrintStream".to_string(),
        name: "println".to_string(),
        descriptor: "(Ljava/lang/String;)V".to_string()
    });
    
    println!("Added {} constants", constant_pool.entries.len());
}
```

## 验证和错误处理

所有程序结构都提供验证方法，确保符合 JVM 规范：

```rust
impl JvmProgram {
    pub fn validate(&self) -> Result<()> {
        // 验证类名不为空
        if self.name.is_empty() {
            return Err(GaiaError::custom_error("Class name cannot be empty".to_string()));
        }
        
        // 验证方法
        for method in &self.methods {
            method.validate()?;
        }
        
        // 验证字段
        for field in &self.fields {
            field.validate()?;
        }
        
        Ok(())
    }
}
```

## 性能优化

- **零成本抽象**: 编译时优化，运行时无额外开销
- **内存池**: 智能内存管理，减少分配开销
- **缓存友好**: 数据结构设计优化 CPU 缓存利用率
- **并行安全**: 支持多线程环境下的安全操作

## 相关模块

- [`formats`](../formats/index.html) - 文件格式支持（JASM、Class）
- [`helpers`](../helpers/index.html) - 辅助工具和实用函数
- [`gaia_types`](../../gaia_types/index.html) - 错误处理和结果类型