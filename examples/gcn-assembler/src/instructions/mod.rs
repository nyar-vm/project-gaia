use serde::{Deserialize, Serialize};

/// GCN 寄存器
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GcnReg {
    VGPR(u8),
    SGPR(u8),
    AGPR(u8),
}

impl std::fmt::Display for GcnReg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GcnReg::VGPR(n) => write!(f, "v{}", n),
            GcnReg::SGPR(n) => write!(f, "s{}", n),
            GcnReg::AGPR(n) => write!(f, "a{}", n),
        }
    }
}

/// GCN 指令集 (CDNA/RDNA 基础)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GcnInstruction {
    /// 算术运算: v_add_f32 dst, src0, src1
    VAddF32 { dst: GcnReg, src0: GcnReg, src1: GcnReg },
    /// 算术运算: v_mul_f32 dst, src0, src1
    VMulF32 { dst: GcnReg, src0: GcnReg, src1: GcnReg },
    /// 算术运算: v_dot2_f32_f16 dst, src0, src1
    VDot2F32F16 { dst: GcnReg, src0: GcnReg, src1: GcnReg },
    /// 内存操作: global_load_dword dst, addr, off
    GlobalLoadDword { dst: GcnReg, addr: GcnReg, offset: u16 },
    /// 内存操作: global_store_dword addr, src, off
    GlobalStoreDword { addr: GcnReg, src: GcnReg, offset: u16 },
    /// 控制流: s_endpgm
    SEndPgm,
    /// 控制流: s_nop
    SNop(u16),
}
