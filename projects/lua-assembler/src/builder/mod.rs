use crate::{
    instructions::LuacInstruction,
    program::{LuaObject, LuaProgram, LuacCodeObject, LuacHeader},
};

/// PycProgram 的构建器
#[derive(Debug)]
pub struct LuacBuilder {
    instructions: Vec<LuacInstruction>,
    constants: Vec<LuaObject>,
    names: Vec<String>,
}

impl LuacBuilder {
    pub fn new() -> Self {
        Self { instructions: Vec::new(), constants: Vec::new(), names: Vec::new() }
    }

    /// 添加打印字符串指令：print("...")
    pub fn print_str(mut self, s: &str) -> Self {
        let const_value = LuaObject::Str(s.to_string());
        let const_index = self.constants.len() as u8;
        self.constants.push(const_value);

        let print_name = "print".to_string();
        let print_index = if let Some(idx) = self.names.iter().position(|n| n == &print_name) {
            idx as u8
        }
        else {
            self.names.push(print_name);
            (self.names.len() - 1) as u8
        };

        self.instructions.push(LuacInstruction::LoadName(print_index));
        self.instructions.push(LuacInstruction::LoadConst(const_index));
        self.instructions.push(LuacInstruction::CallFunction(1));
        self.instructions.push(LuacInstruction::ReturnValue);
        self
    }

    pub fn build(self, header: LuacHeader) -> LuaProgram {
        let co_code_instructions: Vec<u32> = self.instructions.iter().flat_map(|instr| instr.to_bytecode()).collect();

        LuaProgram {
            header,
            code_object: LuacCodeObject {
                source_name: "<string>".to_string(),
                first_line: 1,
                last_line: 1,
                num_params: 0,
                is_vararg: 0,
                max_stack_size: 0,
                nested_functions: vec![],
                upvalues: vec![],
                local_vars: vec![],
                line_info: vec![],
                co_argcount: 0,
                co_nlocal: 0,
                co_stacks: 0,
                num_upval: 0,
                co_code: co_code_instructions,
                co_consts: self.constants,
                upvalue_n: 0,
            },
        }
    }
}
