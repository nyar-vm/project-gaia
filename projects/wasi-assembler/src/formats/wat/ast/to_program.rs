use crate::{
    formats::wat::ast::*,
    program::{WasiProgram, *},
};
use gaia_types::GaiaDiagnostics;
use std::collections::HashMap;

impl WatRoot {
    pub fn to_program(self) -> GaiaDiagnostics<WasiProgram> {
        let mut converter = Wat2Wasi::new();
        match converter.convert(self) {
            Ok(program) => GaiaDiagnostics::success(program),
            Err(error) => GaiaDiagnostics::failure(error),
        }
    }
}

struct Wat2Wasi {
    /// 符号表，用于名称到索引的映射
    symbol_table: HashMap<String, u32>,
    /// 当前函数索引计数器
    function_index: u32,
    /// 当前类型索引计数器
    type_index: u32,
    /// 当前内存索引计数器
    memory_index: u32,
    /// 当前表索引计数器
    table_index: u32,
    /// 当前全局变量索引计数器
    global_index: u32,
}

impl Wat2Wasi {
    fn new() -> Self {
        Self {
            symbol_table: HashMap::new(),
            function_index: 0,
            type_index: 0,
            memory_index: 0,
            table_index: 0,
            global_index: 0,
        }
    }

    fn convert(&mut self, root: WatRoot) -> Result<WasiProgram, gaia_types::GaiaError> {
        todo!()
    }
}
