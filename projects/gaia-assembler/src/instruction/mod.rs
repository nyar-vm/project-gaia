use crate::types::GaiaType;
use gaia_types::neural::NeuralNode;
use serde::{Deserialize, Serialize};

/// Gaia 指令系统 (分层架构)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GaiaInstruction {
    /// 核心底层指令 (Tier 0)
    Core(CoreInstruction),
    /// 托管运行时指令 (Tier 1)
    Managed(ManagedInstruction),
    /// 领域特定指令 (Tier 2)
    Domain(DomainInstruction),
}

/// Tier 0: 核心底层指令 (类 LLVM/汇编)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CoreInstruction {
    // --- 内存操作 ---
    /// 栈上分配空间 (类型, 数量)
    Alloca(GaiaType, usize),
    /// 从内存加载 (目标寄存器类型, 指针)
    Load(GaiaType),
    /// 存储到内存 (值类型)
    Store(GaiaType),
    /// 获取元素指针 (计算偏移量)
    Gep {
        base_type: GaiaType,
        indices: Vec<usize>,
    },

    // --- 算术运算 (操作数从栈获取) ---
    Add(GaiaType),
    Sub(GaiaType),
    Mul(GaiaType),
    Div(GaiaType),
    Rem(GaiaType),
    And(GaiaType),
    Or(GaiaType),
    Xor(GaiaType),
    Shl(GaiaType),
    Shr(GaiaType),
    Neg(GaiaType),
    Not(GaiaType),

    // --- 比较运算 ---
    Cmp(CmpCondition, GaiaType),

    // --- 类型转换 ---
    Cast {
        from: GaiaType,
        to: GaiaType,
        kind: CastKind,
    },

    // --- 栈管理 ---
    PushConstant(crate::program::GaiaConstant),
    Pop,
    Dup,

    // --- 局部变量与参数 (Tier 0 版本) ---
    /// 加载局部变量
    LoadLocal(u32, GaiaType),
    /// 存储局部变量
    StoreLocal(u32, GaiaType),
    /// 加载参数
    LoadArg(u32, GaiaType),
    /// 存储参数
    StoreArg(u32, GaiaType),

    // --- 控制流 ---
    /// 返回
    Ret,
    /// 无条件跳转
    Br(String),
    /// 真跳转
    BrTrue(String),
    /// 假跳转
    BrFalse(String),
    /// 标签
    Label(String),
    /// 调用函数 (函数名, 参数数量)
    Call(String, usize),
    /// 间接调用 (参数数量). 栈: [..., func_ptr, arg1, arg2, ...]
    CallIndirect(usize),

    // --- 对象与数组操作 ---
    /// 创建新对象 (类型名)
    New(String),
    /// 创建新数组 (元素类型, 长度是否在栈上)
    NewArray(GaiaType, bool),
    /// 加载字段 (对象类型, 字段名)
    LoadField(String, String),
    /// 存储字段 (对象类型, 字段名)
    StoreField(String, String),
    /// 加载数组元素
    LoadElement(GaiaType),
    /// 存储数组元素
    StoreElement(GaiaType),
    /// 获取数组长度
    ArrayLength,
    /// 数组推入元素 (数组, 值)
    ArrayPush,

    // --- WASM GC 扩展指令 ---
    /// 创建 GC 结构体 (类型名)
    StructNew(String),
    /// 获取 GC 结构体字段 (类型名, 字段索引)
    StructGet {
        struct_name: String,
        field_index: u32,
        is_signed: bool,
    },
    /// 设置 GC 结构体字段 (类型名, 字段索引)
    StructSet {
        struct_name: String,
        field_index: u32,
    },
    /// 创建 GC 数组 (类型名)
    ArrayNew(String),
    /// 获取 GC 数组元素 (类型名)
    ArrayGet {
        array_name: String,
        is_signed: bool,
    },
    /// 设置 GC 数组元素 (类型名)
    ArraySet(String),
}

/// 比较条件
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CmpCondition {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

/// 转换类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CastKind {
    Bitcast,
    Trunc,
    Zext,
    Sext,
    FpToUi,
    FpToSi,
    UiToFp,
    SiToFp,
}

/// Tier 1: 托管运行时指令 (类 JVM/CLR/Lua)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ManagedInstruction {
    /// 调用方法 (对象类型, 方法名, 签名, 是否虚调用, IC 调用点 ID)
    CallMethod {
        target: String,
        method: String,
        signature: crate::types::GaiaSignature,
        is_virtual: bool,
        call_site_id: Option<u32>,
    },
    /// 调用静态方法
    CallStatic { target: String, method: String, signature: crate::types::GaiaSignature },
    /// 装箱
    Box(GaiaType),
    /// 拆箱
    Unbox(GaiaType),
    /// 运行时类型检查
    InstanceOf(GaiaType),
    /// 类型转换
    CheckCast(GaiaType),
    /// 初始化对象 (参数数量)
    Initiate(usize),
    /// 终结对象
    Finalize,
}

/// Tier 2: 领域特定指令 (神经网络/张量/并行计算)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DomainInstruction {
    /// 神经网络算子
    Neural(NeuralNode),

    // --- 张量基础运算 ---
    /// 矩阵乘法 (A, B, C)
    MatMul { a_shape: Vec<usize>, b_shape: Vec<usize>, transpose_a: bool, transpose_b: bool },
    /// 卷积运算
    Conv2D { stride: [usize; 2], padding: [usize; 2], dilation: [usize; 2], groups: usize },
    /// 逐元素运算 (类型, 算子名)
    ElementWise(GaiaType, String),

    // --- 并行计算 ---
    /// 获取线程 ID (维度)
    GetThreadId(usize),
    /// 获取工作组大小 (维度)
    GetGroupSize(usize),
    /// 屏障同步
    Barrier,
}
