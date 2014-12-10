#![doc = include_str!("readme.md")]

use serde::{Deserialize, Serialize};

/// 寄存器枚举，表示x86_64架构中的所有寄存器
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Register {
    /// 低 8 位累加器寄存器 (AL)
    AL,
    /// 低 8 位计数器寄存器 (CL)
    CL,
    /// 低 8 位数据寄存器 (DL)
    DL,
    /// 低 8 位基址寄存器 (BL)
    BL,
    /// 高 8 位累加器寄存器 (AH)
    AH,
    /// 高 8 位计数器寄存器 (CH)
    CH,
    /// 高 8 位数据寄存器 (DH)
    DH,
    /// 高 8 位基址寄存器 (BH)
    BH,
    /// 16 位累加器寄存器 (AX)
    AX,
    /// 16 位计数器寄存器 (CX)
    CX,
    /// 16 位数据寄存器 (DX)
    DX,
    /// 16 位基址寄存器 (BX)
    BX,
    /// 16 位栈指针寄存器 (SP)
    SP,
    /// 16 位基址指针寄存器 (BP)
    BP,
    /// 16 位源变址寄存器 (SI)
    SI,
    /// 16 位目的变址寄存器 (DI)
    DI,
    /// 32 位扩展累加器寄存器 (EAX)
    EAX,
    /// 32 位扩展计数器寄存器 (ECX)
    ECX,
    /// 32 位扩展数据寄存器 (EDX)
    EDX,
    /// 32 位扩展基址寄存器 (EBX)
    EBX,
    /// 32 位扩展栈指针寄存器 (ESP)
    ESP,
    /// 32 位扩展基址指针寄存器 (EBP)
    EBP,
    /// 32 位扩展源变址寄存器 (ESI)
    ESI,
    /// 32 位扩展目的变址寄存器 (EDI)
    EDI,
    /// 64 位扩展累加器寄存器 (RAX)
    RAX,
    /// 64 位扩展计数器寄存器 (RCX)
    RCX,
    /// 64 位扩展数据寄存器 (RDX)
    RDX,
    /// 64 位扩展基址寄存器 (RBX)
    RBX,
    /// 64 位扩展栈指针寄存器 (RSP)
    RSP,
    /// 64 位扩展基址指针寄存器 (RBP)
    RBP,
    /// 64 位扩展源变址寄存器 (RSI)
    RSI,
    /// 64 位扩展目的变址寄存器 (RDI)
    RDI,
    /// 64 位扩展寄存器 R8
    R8,
    /// 64 位扩展寄存器 R9
    R9,
    /// 64 位扩展寄存器 R10
    R10,
    /// 64 位扩展寄存器 R11
    R11,
    /// 64 位扩展寄存器 R12
    R12,
    /// 64 位扩展寄存器 R13
    R13,
    /// 64 位扩展寄存器 R14
    R14,
    /// 64 位扩展寄存器 R15
    R15,
    /// 8 位扩展寄存器 R8 低字节
    R8B,
    /// 8 位扩展寄存器 R9 低字节
    R9B,
    /// 8 位扩展寄存器 R10 低字节
    R10B,
    /// 8 位扩展寄存器 R11 低字节
    R11B,
    /// 8 位扩展寄存器 R12 低字节
    R12B,
    /// 8 位扩展寄存器 R13 低字节
    R13B,
    /// 8 位扩展寄存器 R14 低字节
    R14B,
    /// 8 位扩展寄存器 R15 低字节
    R15B,
    /// 16 位扩展寄存器 R8
    R8W,
    /// 16 位扩展寄存器 R9
    R9W,
    /// 16 位扩展寄存器 R10
    R10W,
    /// 16 位扩展寄存器 R11
    R11W,
    /// 16 位扩展寄存器 R12
    R12W,
    /// 16 位扩展寄存器 R13
    R13W,
    /// 16 位扩展寄存器 R14
    R14W,
    /// 16 位扩展寄存器 R15
    R15W,
    /// 32 位扩展寄存器 R8
    R8D,
    /// 32 位扩展寄存器 R9
    R9D,
    /// 32 位扩展寄存器 R10
    R10D,
    /// 32 位扩展寄存器 R11
    R11D,
    /// 32 位扩展寄存器 R12
    R12D,
    /// 32 位扩展寄存器 R13
    R13D,
    /// 32 位扩展寄存器 R14
    R14D,
    /// 32 位扩展寄存器 R15
    R15D,
    /// SSE 寄存器 XMM0
    XMM0,
    /// SSE 寄存器 XMM1
    XMM1,
    /// SSE 寄存器 XMM2
    XMM2,
    /// SSE 寄存器 XMM3
    XMM3,
    /// SSE 寄存器 XMM4
    XMM4,
    /// SSE 寄存器 XMM5
    XMM5,
    /// SSE 寄存器 XMM6
    XMM6,
    /// SSE 寄存器 XMM7
    XMM7,
}

/// 操作数枚举，表示汇编指令中的各种操作数类型
#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    /// 寄存器操作数
    Reg(Register),
    /// 立即数操作数，包含值和大小
    Imm {
        /// 立即数的值
        value: i64,
        /// 立即数的大小（位数）
        size: u8,
    },
    /// 内存操作数，包含基址、索引、比例和位移
    Mem {
        /// 基址寄存器
        base: Option<Register>,
        /// 索引寄存器
        index: Option<Register>,
        /// 比例因子
        scale: u8,
        /// 位移量
        displacement: i32,
    },
    /// 标签操作数
    Label(String),
}

/// 指令枚举，表示x86_64汇编指令
#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    /// 数据传送指令：将源操作数传送到目标操作数
    Mov {
        /// 目标操作数
        dst: Operand,
        /// 源操作数
        src: Operand,
    },
    /// 压栈指令：将操作数压入栈顶
    Push {
        /// 要压栈的操作数
        op: Operand,
    },
    /// 出栈指令：将栈顶数据弹出到目标操作数
    Pop {
        /// 接收出栈数据的目标操作数
        dst: Operand,
    },
    /// 加法指令：将源操作数加到目标操作数
    Add {
        /// 目标操作数（被加数，结果存储位置）
        dst: Operand,
        /// 源操作数（加数）
        src: Operand,
    },
    /// 减法指令：从目标操作数中减去源操作数
    Sub {
        /// 目标操作数（被减数，结果存储位置）
        dst: Operand,
        /// 源操作数（减数）
        src: Operand,
    },
    /// 返回指令：从函数返回
    Ret,
    /// 调用指令：调用目标函数
    Call {
        /// 要调用的目标函数或地址
        target: Operand,
    },
    /// 加载有效地址指令：计算地址并加载到目标寄存器
    Lea {
        /// 目标寄存器（存储计算出的地址）
        dst: Register,
        /// 位移量
        displacement: i32,
        /// 是否使用 RIP 相对寻址
        rip_relative: bool,
    },
    /// 空操作指令：不执行任何操作
    /// NOP 指令
    Nop,
    /// 标签 (伪指令)
    Label(String),
    /// 比较指令
    Cmp { dst: Operand, src: Operand },
    /// 跳转指令
    Jmp { target: Operand },
    /// 如果不相等则跳转
    Jnz { target: Operand },
    /// 如果相等则跳转
    Jz { target: Operand },
    /// 递减指令
    Dec { op: Operand },
    /// 递增指令
    Inc { op: Operand },
    /// 标量单精度浮点数传送
    Movss { dst: Operand, src: Operand },
    /// 标量单精度浮点数加法
    Addss { dst: Operand, src: Operand },
    /// 标量单精度浮点数减法
    Subss { dst: Operand, src: Operand },
    /// 标量单精度浮点数乘法
    Mulss { dst: Operand, src: Operand },
    /// 标量单精度浮点数除法
    Divss { dst: Operand, src: Operand },
    /// 标量单精度浮点数最大值
    Maxss { dst: Operand, src: Operand },
    /// 位异或
    Xor { dst: Operand, src: Operand },
    /// 位异或 (用于清零 XMM)
    Xorps { dst: Operand, src: Operand },
    /// 整数乘法
    Imul { dst: Register, src: Operand },
    /// 无符号整数乘法
    Mul { src: Operand },
    /// 无符号整数除法
    Div { src: Operand },
    /// 有符号整数除法
    Idiv { src: Operand },
    /// 符号扩展 RAX 到 RDX:RAX (Convert Quadword to Octoword)
    Cqo,
}

impl Operand {
    /// 创建寄存器操作数
    pub fn reg(reg: Register) -> Self {
        Operand::Reg(reg)
    }
    /// 创建立即数操作数
    pub fn imm(value: i64, size: u8) -> Self {
        Operand::Imm { value, size }
    }
    /// 创建内存操作数
    pub fn mem(base: Option<Register>, index: Option<Register>, scale: u8, displacement: i32) -> Self {
        Operand::Mem { base, index, scale, displacement }
    }
    /// 创建标签操作数
    pub fn label(name: String) -> Self {
        Operand::Label(name)
    }
}

impl std::fmt::Display for Operand {
    /// 格式化显示操作数
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operand::Reg(reg) => write!(f, "{:?}", reg),
            Operand::Imm { value, .. } => write!(f, "0x{:x}", value),
            Operand::Mem { base, index, scale, displacement } => {
                write!(f, "[")?;
                if let Some(base) = base {
                    write!(f, "{:?}", base)?;
                }
                if let Some(index) = index {
                    if base.is_some() {
                        write!(f, " + ")?;
                    }
                    write!(f, "{:?}", index)?;
                    if *scale > 1 {
                        write!(f, " * {}", scale)?;
                    }
                }
                if *displacement != 0 {
                    if base.is_some() || index.is_some() {
                        if *displacement > 0 {
                            write!(f, " + 0x{:x}", displacement)?;
                        }
                        else {
                            write!(f, " - 0x{:x}", -displacement)?;
                        }
                    }
                    else {
                        write!(f, "0x{:x}", displacement)?;
                    }
                }
                write!(f, "]")?;
                Ok(())
            }
            Operand::Label(name) => write!(f, "{}", name),
        }
    }
}
