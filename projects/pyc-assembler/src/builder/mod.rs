#![doc = include_str!("readme.md")]

use crate::{
    instructions::PythonInstruction,
    program::{PycHeader, PythonCodeObject, PythonObject, PythonProgram},
};

/// PycProgram 的构建器
#[derive(Debug)]
pub struct PythonBuilder {
    _instructions: Vec<PythonInstruction>,
    constants: Vec<PythonObject>,
    names: Vec<String>,
}

impl PythonBuilder {
    /// 创建一个新的 PythonBuilder 实例。
    pub fn new() -> Self {
        Self { _instructions: Vec::new(), constants: Vec::new(), names: Vec::new() }
    }

    /// 添加打印字符串指令：print("...")
    pub fn print_str(mut self, s: &str) -> Self {
        let const_value = PythonObject::Str(s.to_string());
        let _const_index = self.constants.len() as u8;
        self.constants.push(const_value);

        let print_name = "print".to_string();
        let _print_index = if let Some(idx) = self.names.iter().position(|n| n == &print_name) {
            idx as u8
        }
        else {
            self.names.push(print_name);
            (self.names.len() - 1) as u8
        };
        self
    }

    /// 构建 PythonProgram。
    pub fn build(self, header: PycHeader) -> PythonProgram {
        PythonProgram {
            header,
            code_object: PythonCodeObject {
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
                co_code: vec![],
                co_consts: self.constants,
                upvalue_n: 0,
            },
            version: Default::default(),
        }
    }
}
