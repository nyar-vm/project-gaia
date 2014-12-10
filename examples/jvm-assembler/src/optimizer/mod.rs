use crate::program::{JvmInstruction, JvmMethod, JvmProgram};

/// JVM 字节码优化器
pub struct JvmOptimizer;

impl JvmOptimizer {
    /// 优化整个程序
    pub fn optimize_program(program: &mut JvmProgram) {
        for method in &mut program.methods {
            Self::optimize_method(method);
        }
    }

    /// 优化单个方法
    pub fn optimize_method(method: &mut JvmMethod) {
        let mut optimized_instructions = Vec::with_capacity(method.instructions.len());

        for instruction in method.instructions.drain(..) {
            optimized_instructions.push(Self::optimize_instruction(instruction));
        }

        method.instructions = optimized_instructions;
    }

    /// 优化单条指令
    fn optimize_instruction(instruction: JvmInstruction) -> JvmInstruction {
        match instruction {
            // 优化加载常量指令
            JvmInstruction::Bipush { value } => match value {
                -1 => JvmInstruction::IconstM1,
                0 => JvmInstruction::Iconst0,
                1 => JvmInstruction::Iconst1,
                2 => JvmInstruction::Iconst2,
                3 => JvmInstruction::Iconst3,
                4 => JvmInstruction::Iconst4,
                5 => JvmInstruction::Iconst5,
                _ => JvmInstruction::Bipush { value },
            },
            JvmInstruction::Sipush { value } => match value {
                -1 => JvmInstruction::IconstM1,
                0 => JvmInstruction::Iconst0,
                1 => JvmInstruction::Iconst1,
                2 => JvmInstruction::Iconst2,
                3 => JvmInstruction::Iconst3,
                4 => JvmInstruction::Iconst4,
                5 => JvmInstruction::Iconst5,
                v if v >= -128 && v <= 127 => JvmInstruction::Bipush { value: v as i8 },
                _ => JvmInstruction::Sipush { value },
            },
            // 优化 load 指令
            JvmInstruction::Iload { index } => match index {
                0 => JvmInstruction::Iload0,
                1 => JvmInstruction::Iload1,
                2 => JvmInstruction::Iload2,
                3 => JvmInstruction::Iload3,
                _ => JvmInstruction::Iload { index },
            },
            JvmInstruction::Lload { index } => match index {
                0 => JvmInstruction::Lload0,
                1 => JvmInstruction::Lload1,
                2 => JvmInstruction::Lload2,
                3 => JvmInstruction::Lload3,
                _ => JvmInstruction::Lload { index },
            },
            JvmInstruction::Fload { index } => match index {
                0 => JvmInstruction::Fload0,
                1 => JvmInstruction::Fload1,
                2 => JvmInstruction::Fload2,
                3 => JvmInstruction::Fload3,
                _ => JvmInstruction::Fload { index },
            },
            JvmInstruction::Dload { index } => match index {
                0 => JvmInstruction::Dload0,
                1 => JvmInstruction::Dload1,
                2 => JvmInstruction::Dload2,
                3 => JvmInstruction::Dload3,
                _ => JvmInstruction::Dload { index },
            },
            JvmInstruction::Aload { index } => match index {
                0 => JvmInstruction::Aload0,
                1 => JvmInstruction::Aload1,
                2 => JvmInstruction::Aload2,
                3 => JvmInstruction::Aload3,
                _ => JvmInstruction::Aload { index },
            },
            // 优化 store 指令
            JvmInstruction::Istore { index } => match index {
                0 => JvmInstruction::Istore0,
                1 => JvmInstruction::Istore1,
                2 => JvmInstruction::Istore2,
                3 => JvmInstruction::Istore3,
                _ => JvmInstruction::Istore { index },
            },
            JvmInstruction::Lstore { index } => match index {
                0 => JvmInstruction::Lstore0,
                1 => JvmInstruction::Lstore1,
                2 => JvmInstruction::Lstore2,
                3 => JvmInstruction::Lstore3,
                _ => JvmInstruction::Lstore { index },
            },
            JvmInstruction::Fstore { index } => match index {
                0 => JvmInstruction::Fstore0,
                1 => JvmInstruction::Fstore1,
                2 => JvmInstruction::Fstore2,
                3 => JvmInstruction::Fstore3,
                _ => JvmInstruction::Fstore { index },
            },
            JvmInstruction::Dstore { index } => match index {
                0 => JvmInstruction::Dstore0,
                1 => JvmInstruction::Dstore1,
                2 => JvmInstruction::Dstore2,
                3 => JvmInstruction::Dstore3,
                _ => JvmInstruction::Dstore { index },
            },
            JvmInstruction::Astore { index } => match index {
                0 => JvmInstruction::Astore0,
                1 => JvmInstruction::Astore1,
                2 => JvmInstruction::Astore2,
                3 => JvmInstruction::Astore3,
                _ => JvmInstruction::Astore { index },
            },
            _ => instruction,
        }
    }
}
