use crate::program::LLvmProgram;
use oak_llvm_ir::ast::{LLirBlock, LLirFunction, LLirGlobal, LLirInstruction, LLirItem, LLirParameter};

/// LLVM 程序构建器
pub struct LLvmProgramBuilder {
    program: LLvmProgram,
}

impl LLvmProgramBuilder {
    pub fn new() -> Self {
        Self {
            program: LLvmProgram::new(),
        }
    }

    pub fn add_function<F>(mut self, name: impl Into<String>, return_type: impl Into<String>, f: F) -> Self
    where
        F: FnOnce(LLvmFunctionBuilder) -> LLvmFunctionBuilder,
    {
        let builder = LLvmFunctionBuilder::new(name.into(), return_type.into());
        let function = f(builder).build();
        self.program.root.items.push(LLirItem::Function(function));
        self
    }

    pub fn add_global(
        mut self,
        name: impl Into<String>,
        ty: impl Into<String>,
        value: impl Into<String>,
        is_constant: bool,
    ) -> Self {
        self.program.root.items.push(LLirItem::Global(LLirGlobal {
            name: name.into(),
            ty: ty.into(),
            value: value.into(),
            is_constant,
        }));
        self
    }

    pub fn build(self) -> LLvmProgram {
        self.program
    }
}

/// LLVM 函数构建器
pub struct LLvmFunctionBuilder {
    function: LLirFunction,
}

impl LLvmFunctionBuilder {
    pub fn new(name: String, return_type: String) -> Self {
        Self {
            function: LLirFunction {
                name,
                return_type,
                parameters: Vec::new(),
                blocks: Vec::new(),
                span: (0..0).into(),
            },
        }
    }

    pub fn add_parameter(mut self, name: impl Into<String>, ty: impl Into<String>) -> Self {
        self.function.parameters.push(LLirParameter {
            name: name.into(),
            ty: ty.into(),
        });
        self
    }

    pub fn add_block<F>(mut self, label: impl Into<String>, f: F) -> Self
    where
        F: FnOnce(LLvmBlockBuilder) -> LLvmBlockBuilder,
    {
        let builder = LLvmBlockBuilder::new(Some(label.into()));
        let block = f(builder).build();
        self.function.blocks.push(block);
        self
    }

    pub fn build(self) -> LLirFunction {
        self.function
    }
}

/// LLVM 基本块构建器
pub struct LLvmBlockBuilder {
    block: LLirBlock,
}

impl LLvmBlockBuilder {
    pub fn new(label: Option<String>) -> Self {
        Self {
            block: LLirBlock {
                label,
                instructions: Vec::new(),
            },
        }
    }

    pub fn add_instruction(
        mut self,
        opcode: impl Into<String>,
        operands: Vec<impl Into<String>>,
        result: Option<impl Into<String>>,
    ) -> Self {
        self.block.instructions.push(LLirInstruction {
            opcode: opcode.into(),
            operands: operands.into_iter().map(|o| o.into()).collect(),
            result: result.map(|r| r.into()),
        });
        self
    }

    /// 常用指令的便捷方法
    pub fn ret(self, ty: impl Into<String>, val: impl Into<String>) -> Self {
        let ty = ty.into();
        let val = val.into();
        self.add_instruction("ret", vec![format!("{} {}", ty, val)], None::<String>)
    }

    pub fn add(
        self,
        result: impl Into<String>,
        ty: impl Into<String>,
        lhs: impl Into<String>,
        rhs: impl Into<String>,
    ) -> Self {
        let ty = ty.into();
        let lhs = lhs.into();
        let rhs = rhs.into();
        self.add_instruction("add", vec![format!("{} {}, {}", ty, lhs, rhs)], Some(result))
    }

    pub fn sub(
        self,
        result: impl Into<String>,
        ty: impl Into<String>,
        lhs: impl Into<String>,
        rhs: impl Into<String>,
    ) -> Self {
        let ty = ty.into();
        let lhs = lhs.into();
        let rhs = rhs.into();
        self.add_instruction("sub", vec![format!("{} {}, {}", ty, lhs, rhs)], Some(result))
    }

    pub fn mul(
        self,
        result: impl Into<String>,
        ty: impl Into<String>,
        lhs: impl Into<String>,
        rhs: impl Into<String>,
    ) -> Self {
        let ty = ty.into();
        let lhs = lhs.into();
        let rhs = rhs.into();
        self.add_instruction("mul", vec![format!("{} {}, {}", ty, lhs, rhs)], Some(result))
    }

    pub fn call(
        self,
        result: Option<impl Into<String>>,
        ty: impl Into<String>,
        name: impl Into<String>,
        args: Vec<impl Into<String>>,
    ) -> Self {
        let ty = ty.into();
        let name = name.into();
        let mut operands = vec![ty, name];
        operands.extend(args.into_iter().map(|a| a.into()));
        self.add_instruction("call", operands, result)
    }

    pub fn build(self) -> LLirBlock {
        self.block
    }
}
