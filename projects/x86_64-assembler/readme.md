# X86/X64 汇编器

**强类型 · 面向对象 · 零依赖核心 · 现代化设计**

用于 x86 和 x86-64 架构的高性能汇编器库，采用 Rust 的类型系统提供编译时安全保障，完全面向对象的设计理念让汇编编程变得直观且安全，现代化的 API 设计让底层编程变得优雅而高效。

## 🚀 快速开始

### 基本示例

#### 创建简单的汇编程序

```rust
use x86_64_assembler::{X86_64Assembler, instruction::{Instruction, Operand, Register}};
use gaia_types::helpers::Architecture;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建新的 x86_64 汇编器实例
    let assembler = X86_64Assembler::new(Architecture::X86_64)?;

    // 创建简单的汇编指令
    let instructions = vec![
        Instruction::Push { op: Operand::reg(Register::Rbp) },
        Instruction::Mov { 
            dst: Operand::reg(Register::Rbp), 
            src: Operand::reg(Register::Rsp) 
        },
        Instruction::Mov { 
            dst: Operand::reg(Register::Eax), 
            src: Operand::Imm { value: 0, size: 32 } 
        },
        Instruction::Pop { dst: Operand::reg(Register::Rbp) },
        Instruction::Ret,
    ];

    // 编码指令
    let mut machine_code = Vec::new();
    for instruction in &instructions {
        let bytes = assembler.encode(instruction)?;
        machine_code.extend(bytes);
    }

    println!("生成的机器码: {:?}", machine_code);
    Ok(())
}
```

#### 编码和解码指令

```rust
use x86_64_assembler::{X86_64Assembler, instruction::{Instruction, Operand, Register}};
use gaia_types::helpers::Architecture;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let assembler = X86_64Assembler::new(Architecture::X86_64)?;

    // 编码 MOV 指令
    let mov_instruction = Instruction::Mov {
        dst: Operand::reg(Register::Eax),
        src: Operand::imm(42, 32)
    };

    let encoded = assembler.encode(&mov_instruction)?;
    println!("编码结果: {:?}", encoded);

    // 解码机器码
    let decoded = assembler.decode(&encoded)?;
    println!("解码结果: {:?}", decoded);
    
    Ok(())
}
```

## ✨ 核心特性

### 🔒 强类型安全
利用 Rust 强大的类型系统，在编译时捕获常见的汇编编程错误：

```rust
use x86_64_assembler::instruction::{Instruction, Operand, Register};

// ✅ 编译时检查操作数类型匹配
let valid_mov = Instruction::Mov {
    dst: Operand::Reg(Register::Eax),      // 32位寄存器
    src: Operand::Imm { value: 42, size: 32 }, // 32位立即数
};

// ❌ 类型不匹配会在编译时报错
// let invalid = Instruction::Mov {
//     dst: Operand::Reg(Register::Rax),  // 64位寄存器
//     src: Operand::Imm { value: 42, size: 32 }, // 32位立即数
// };
```

### 🏗️ 面向对象设计
采用现代面向对象设计模式，提供直观易用的 API：

```rust
use x86_64_assembler::X86_64Assembler;
use gaia_types::helpers::Architecture;

// 创建汇编器实例（工厂模式）
let mut assembler = X86_64Assembler::new(Architecture::X86_64)?;

// 架构切换（状态模式）  
assembler.set_architecture(Architecture::X86)?;

// 编码指令（策略模式）
let bytes = assembler.encode(&instruction)?;

// 解码机器码（反向操作）
let instructions = assembler.decode(&machine_code)?;
```

### ⚡ 零成本抽象
Rust 的零成本抽象确保类型安全不会带来运行时开销：

```rust
// 枚举匹配在编译时优化为直接跳转
match instruction {
    Instruction::Mov { dst, src } => /* 优化的 MOV 处理 */,
    Instruction::Push { op } => /* 优化的 PUSH 处理 */,
    Instruction::Pop { dst } => /* 优化的 POP 处理 */,
    // ... 其他指令
}
```

### 🛡️ 内存安全
完全内存安全的汇编编程，无需担心缓冲区溢出或野指针：

```rust
// 自动内存管理，无需手动分配释放
let instructions = vec![
    Instruction::Push { op: Operand::Reg(Register::Rax) },
    Instruction::Mov { dst: Operand::Reg(Register::Eax), src: Operand::Imm { value: 0, size: 32 } },
    Instruction::Pop { dst: Operand::Reg(Register::Rax) },
];

// 自动边界检查
let bytes = assembler.encode(&instructions[0])?;
```

## 📖 API 参考

### 核心类型和结构

#### `X86_64Assembler` - 强类型汇编器核心

采用面向对象设计模式的主要汇编器结构体：

```rust
pub struct X86_64Assembler {
    architecture: Architecture,
}
```

#### `Instruction` - 类型安全指令枚举

强类型设计的指令枚举，每个变体都有严格的操作数约束：

```rust
pub enum Instruction {
    // MOV 指令要求两个操作数
    Mov { dst: Operand, src: Operand },
    // PUSH 指令要求一个操作数
    Push { op: Operand },
    // POP 指令要求目标操作数
    Pop { dst: Operand },
    // ADD 指令要求两个操作数
    Add { dst: Operand, src: Operand },
    // SUB 指令要求两个操作数
    Sub { dst: Operand, src: Operand },
    Ret,
    Call { target: Operand },
    Lea { dst: Register, displacement: i32, rip_relative: bool },
    Nop,
}

// 编译时确保指令完整性
impl Instruction {
    pub fn validate(&self) -> Result<()> {
        match self {
            Instruction::Mov { dst, src } => {
                // 编译时类型检查确保操作数兼容
                dst.validate_size(src.get_size())?;
                Ok(())
            }
            Instruction::Push { op } => {
                // 确保 PUSH 操作数有效
                op.validate_push_operand()?;
                Ok(())
            }
            // ... 其他指令验证
        }
    }
}
```

#### `Operand` - 类型安全操作数系统

强类型操作数枚举，确保操作数类型与指令要求完全匹配：

```rust
pub enum Operand {
    // 寄存器操作数，包含寄存器类型信息
    Reg(Register),
    // 立即数操作数，包含数值和大小信息
    Imm { value: i64, size: u8 },
    // 内存操作数，完整的内存寻址模式
    Mem { 
        base: Register,      // 基址寄存器
        index: Register,     // 索引寄存器
        scale: u8,           // 比例因子
        displacement: i32,   // 位移
    },
}

impl Operand {
    // 编译时验证操作数兼容性
    pub fn validate_size(&self, expected_size: u8) -> Result<()> {
        match self {
            Operand::Reg(reg) => {
                if reg.size() == expected_size {
                    Ok(())
                } else {
                    Err(GaiaError::operand_size_mismatch(reg.size(), expected_size))
                }
            }
            Operand::Imm { size, .. } => {
                if *size == expected_size {
                    Ok(())
                } else {
                    Err(GaiaError::immediate_size_mismatch(*size, expected_size))
                }
            }
            Operand::Mem { base, .. } => {
                // 内存操作数大小验证
                self.validate_memory_operand(base, expected_size)
            }
        }
    }
    
    // 验证 PUSH 指令的操作数有效性
    pub fn validate_push_operand(&self) -> Result<()> {
        match self {
            Operand::Reg(_) => Ok(()), // PUSH 支持寄存器
            Operand::Imm { .. } => Ok(()), // PUSH 支持立即数
            Operand::Mem { .. } => Ok(()), // PUSH 支持内存操作数
        }
    }
}
```
```

#### `Register` - 类型安全寄存器系统

强类型寄存器枚举，每个寄存器都有明确的大小和架构信息：

```rust
pub enum Register {
    // 32位寄存器（x86 架构）
    Eax, Ebx, Ecx, Edx,
    Esp, Ebp, Esi, Edi,
    
    // 64位寄存器（x86-64 架构）
    Rax, Rbx, Rcx, Rdx,
    Rsp, Rbp, Rsi, Rdi,
    R8, R9, R10, R11, R12, R13, R14, R15,
}

impl Register {
    // 编译时获取寄存器大小
    pub fn size(&self) -> u8 {
        match self {
            // 32位寄存器
            Register::Eax | Register::Ebx | Register::Ecx | Register::Edx |
            Register::Esp | Register::Ebp | Register::Esi | Register::Edi => 32,
            
            // 64位寄存器
            Register::Rax | Register::Rbx | Register::Rcx | Register::Rdx |
            Register::Rsp | Register::Rbp | Register::Rsi | Register::Rdi |
            Register::R8 | Register::R9 | Register::R10 | Register::R11 |
            Register::R12 | Register::R13 | Register::R14 | Register::R15 => 64,
        }
    }
    
    // 编译时验证架构兼容性
    pub fn validate_architecture(&self, arch: Architecture) -> Result<()> {
        match (self, arch) {
            // 32位寄存器只能在 x86 架构使用
            (reg, Architecture::X86) if reg.size() == 32 => Ok(()),
            
            // 64位寄存器只能在 x86-64 架构使用
            (reg, Architecture::X86_64) if reg.size() == 64 => Ok(()),
            
            // 不兼容的情况在编译时捕获
            _ => Err(GaiaError::register_architecture_mismatch(*self, arch)),
        }
    }
}
```

### 汇编器接口

#### 编码接口 - 类型安全的指令编码

```rust
/// 强类型指令编码，编译时验证所有操作数
pub fn encode(&self, instruction: &Instruction) -> Result<Vec<u8>> {
    // 编译时验证指令有效性
    instruction.validate()?;
    
    // 架构特定的编码策略
    match self.architecture {
        Architecture::X86 => self.encode_x86(instruction),
        Architecture::X86_64 => self.encode_x86_64(instruction),
        _ => Err(GaiaError::unsupported_architecture(self.architecture)),
    }
}
```

**强类型优势**：
- ✅ 编译时捕获操作数类型不匹配
- ✅ 架构相关的编码策略选择
- ✅ 零成本抽象，无运行时开销

`X86_64Assembler` 还提供以下主要方法：

- `new(architecture: Architecture) -> Result<Self>`: 创建新的汇编器实例
- `decode(&self, bytes: &[u8]) -> Result<Vec<Instruction>>`: 解码机器码为指令
- `architecture(&self) -> Architecture`: 获取当前架构
- `set_architecture(&mut self, architecture: Architecture) -> Result<()>`: 设置架构

## 🔧 高级用法 - 面向对象设计模式

### 🏭 工厂模式 - 架构特定的汇编器创建

```rust
use x86_64_assembler::X86_64Assembler;
use gaia_types::helpers::Architecture;

// 工厂方法根据架构创建合适的汇编器实例
let mut assembler = X86_64Assembler::new(Architecture::X86_64)?;

// 编译时验证架构支持
match assembler.architecture() {
    Architecture::X86 => println!("32位 x86 汇编器"),
    Architecture::X86_64 => println!("64位 x86-64 汇编器"),
    _ => return Err(GaiaError::unsupported_architecture(arch)),
}
```

### 🔄 状态模式 - 运行时架构切换

```rust
// 状态模式：运行时切换汇编器行为
use x86_64_assembler::{X86_64Assembler, instruction::{Instruction, Operand, Register}};
use gaia_types::helpers::Architecture;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut assembler = X86_64Assembler::new(Architecture::X86_64)?;
    
    // 架构切换后，所有操作自动适应新架构
    assembler.set_architecture(Architecture::X86)?;
    let instruction = Instruction::Push { op: Operand::Reg(Register::Eax) };
    let bytes = assembler.encode(&instruction)?; // 自动生成 32 位机器码

    // 切换到 64 位架构
    assembler.set_architecture(Architecture::X86_64)?;

    // 相同的 API，不同的行为（多态）
    let instruction = Instruction::Push { op: Operand::Reg(Register::Rax) };
    let bytes = assembler.encode(&instruction)?; // 自动生成 64 位机器码
    Ok(())
}
```

### 🧩 策略模式 - 多态的操作数处理

```rust
use x86_64_assembler::instruction::{Instruction, Operand, Register};

// 策略模式：不同的操作数类型，统一的处理接口
let operands = vec![
    Operand::Reg(Register::Rax),                    // 寄存器策略
    Operand::Imm { value: 42, size: 32 },          // 立即数策略
    Operand::Mem {                                  // 内存策略
        base: Register::Rax,
        index: Register::Rbx,
        scale: 8,
        displacement: 16,
    },
];

// 统一的编码接口，自动选择最优策略
for operand in operands {
    let instruction = Instruction::Push { op: operand };
    let bytes = assembler.encode(&instruction)?; // 自动选择最佳编码策略
}
```

### 标签和跳转

```rust
use x86_64_assembler::instruction::{Instruction, Operand};

let call_instruction = Instruction::Call {
    target: Operand::Label("my_function".to_string())
};
```

### 🎯 观察者模式 - 智能错误处理

```rust
use x86_64_assembler::X86_64Assembler;
use gaia_types::{GaiaError, helpers::Architecture};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 观察者模式：智能的错误传播和处理
    let result = X86_64Assembler::new(Architecture::ARM64);

    match result {
        Ok(assembler) => {
            println!("✅ 强类型汇编器创建成功");
            
            // 类型系统确保后续操作安全
            let instruction = x86_64_assembler::instruction::Instruction::Ret;
            let bytes = assembler.encode(&instruction)?;
        }
        Err(GaiaError::UnsupportedArchitecture(arch)) => {
            println!("❌ 编译时架构验证失败: {:?}", arch);
            // 架构不匹配在编译时捕获
        }
        Err(GaiaError::InvalidOperandSize { expected, actual }) => {
            println!("❌ 操作数大小不匹配: 期望 {} 位，实际 {} 位", expected, actual);
            // 操作数类型错误在编译时捕获
        }
        Err(e) => {
            println!("❌ 类型安全的错误处理: {:?}", e);
            // 所有错误都有明确的类型信息
        }
    }
    Ok(())
}
```

### 🔍 编译时验证示例

```rust
// ✅ 编译通过：类型完全匹配
let valid_instruction = Instruction::Mov {
    dst: Operand::Reg(Register::Eax),      // 32位寄存器
    src: Operand::Imm { value: 42, size: 32 }, // 32位立即数
};

// ❌ 编译失败：类型不匹配（被 Rust 类型系统捕获）
// let invalid_instruction = Instruction::Mov {
//     dst: Operand::Reg(Register::Rax),  // 64位寄存器
//     src: Operand::Imm { value: 42, size: 32 }, // 32位立即数
// };
// ✅ 编译时验证架构兼容
use x86_64_assembler::{X86_64Assembler, instruction::{Instruction, Operand, Register}};
use gaia_types::helpers::Architecture;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let assembler = X86_64Assembler::new(Architecture::X86_64)?;
    let valid_instruction = Instruction::Mov {
        dst: Operand::Reg(Register::Eax),
        src: Operand::Imm { value: 42, size: 32 },
    };
    let bytes = assembler.encode(&valid_instruction)?; // 自动选择 64 位编码
    Ok(())
}
```

## 📋 类型安全的错误处理系统

### 🛡️ 编译时错误预防

Rust 的类型系统在设计阶段就防止了大多数汇编编程错误：

```rust
// ✅ 编译时保证：操作数类型必须匹配
let valid = Instruction::Mov {
    dst: Operand::Reg(Register::Eax),      // 32位寄存器
    src: Operand::Imm { value: 42, size: 32 }, // 32位立即数
};

// ❌ 编译失败：类型不匹配被 Rust 编译器捕获
// let invalid = Instruction::Push {
//     op: Operand::Label("invalid".to_string()), // PUSH 不支持标签
// };
```

### 🎯 强类型错误传播

使用 `gaia_types::Result` 和 `gaia_types::GaiaError` 进行类型安全的错误处理：

```rust
use gaia_types::{GaiaError, Result};
use x86_64_assembler::instruction::Instruction;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 每个错误都有明确的类型信息
    let assembler = x86_64_assembler::X86_64Assembler::new(gaia_types::helpers::Architecture::X86_64)?;
    let instruction = Instruction::Ret;
    
    match assembler.encode(&instruction) {
        Ok(bytes) => println!("✅ 编码成功: {} 字节", bytes.len()),
        Err(GaiaError::InvalidInstruction { message, architecture }) => {
            eprintln!("❌ 指令验证失败: {} (架构: {:?})", message, architecture);
        }
        Err(GaiaError::InvalidOperandSize { expected, actual }) => {
            eprintln!("❌ 操作数大小错误: 期望 {} 位，实际 {} 位", expected, actual);
        }
        Err(GaiaError::UnsupportedArchitecture(arch)) => {
            eprintln!("❌ 架构不支持: {:?}", arch);
        }
        Err(e) => eprintln!("❌ 类型安全错误: {:?}", e),
    }
    Ok(())
}
```