use crate::{formats::wat::ast::WatRoot, program::WasiProgram};
use gaia_types::{GaiaDiagnostics, GaiaError};
use std::collections::HashMap;

impl WasiProgram {
    pub fn to_wat(self) -> GaiaDiagnostics<WatRoot> {
        let mut state =
            Wasi2Wat { wat: WatRoot::default(), index_to_name: Default::default(), name_counter: 0, errors: vec![] };
        match state.convert(self) {
            Ok(_) => GaiaDiagnostics { result: Ok(state.wat), diagnostics: state.errors },
            Err(e) => GaiaDiagnostics { result: Err(e), diagnostics: state.errors },
        }
    }
}

struct Wasi2Wat {
    wat: WatRoot,
    // 用于生成名称的映射
    index_to_name: HashMap<u32, String>,
    // 名称计数器
    name_counter: u32,
    errors: Vec<GaiaError>,
}

impl Wasi2Wat {
    fn convert(&mut self, program: WasiProgram) -> Result<(), GaiaError> {
        todo!()
    }
}
