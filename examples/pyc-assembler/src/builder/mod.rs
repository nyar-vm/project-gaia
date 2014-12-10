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
        Self { _instructions: vec![PythonInstruction::RESUME], constants: vec![PythonObject::None], names: Vec::new() }
    }

    /// 添加打印字符串指令：print("...")
    pub fn print_str(mut self, s: &str) -> Self {
        let const_value = PythonObject::Str(s.to_string());
        let const_index = self.constants.len() as u32;
        self.constants.push(const_value);

        let print_name = "print".to_string();
        let name_index = if let Some(idx) = self.names.iter().position(|n| n == &print_name) {
            idx as u32
        }
        else {
            self.names.push(print_name);
            (self.names.len() - 1) as u32
        };

        // Python 3.11+ 打印序列
        self._instructions.push(PythonInstruction::PUSH_NULL);
        self._instructions.push(PythonInstruction::LOAD_GLOBAL(name_index << 1)); // 3.11+ LOAD_GLOBAL arg is (index << 1) | push_null
        self._instructions.push(PythonInstruction::LOAD_CONST(const_index));
        self._instructions.push(PythonInstruction::CALL(1));
        self._instructions.push(PythonInstruction::POP_TOP);
        self
    }

    /// 构建 PythonProgram。
    pub fn build(mut self, header: PycHeader) -> PythonProgram {
        use crate::program::PythonVersion;

        // 结束指令
        self._instructions.push(PythonInstruction::RETURN_CONST(0)); // 返回 None (constants[0])

        let version = PythonVersion::from_magic(header.magic);
        PythonProgram {
            header,
            code_object: PythonCodeObject {
                name: "<module>".to_string(),
                qualname: "<module>".to_string(),
                source_name: "<string>".to_string(),
                first_line: 1,
                last_line: 1,
                co_argcount: 0,
                co_posonlyargcount: 0,
                co_kwonlyargcount: 0,
                co_nlocals: 0,
                co_stacksize: 10,
                co_flags: 0x40, // CO_NOFREE
                co_code: self._instructions,
                co_consts: self.constants,
                co_names: self.names,
                co_localsplusnames: vec![],
                co_localspluskinds: vec![],
                co_linetable: vec![0, 1], // Simple linetable
                co_exceptiontable: vec![],
            },
            version,
        }
    }
}
