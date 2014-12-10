use serde::{Deserialize, Serialize};

/// SASS 寄存器
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SassReg {
    R(u8),
    UR(u8),
    PR(u8),
}

impl std::fmt::Display for SassReg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SassReg::R(n) => write!(f, "R{}", n),
            SassReg::UR(n) => write!(f, "UR{}", n),
            SassReg::PR(n) => write!(f, "P{}", n),
        }
    }
}

/// SASS 指令集 (Maxwell/Pascal/Ampere/Hopper 基础)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SassInstruction {
    /// 浮点加法: FADD dst, src0, src1
    FAdd { dst: SassReg, src0: SassReg, src1: SassReg },
    /// 浮点乘法: FMUL dst, src0, src1
    FMul { dst: SassReg, src0: SassReg, src1: SassReg },
    /// 张量核乘累加: IMMA dst, src0, src1, src2
    Imma { dst: SassReg, src0: SassReg, src1: SassReg, src2: SassReg },
    /// 内存加载: LDG.E dst, [addr]
    Ldg { dst: SassReg, addr: SassReg },
    /// 内存存储: STG.E [addr], src
    Stg { addr: SassReg, src: SassReg },
    /// 控制流: EXIT
    Exit,
    /// 控制流: NOP
    Nop,
}
