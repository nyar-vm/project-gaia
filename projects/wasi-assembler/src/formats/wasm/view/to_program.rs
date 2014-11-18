use crate::{formats::wasm::view::*, program::*};
use gaia_types::{GaiaDiagnostics, GaiaError};

impl WasmView {
    pub fn to_program(self) -> GaiaDiagnostics<WasiProgram> {
        let mut state = Wasm2Program { errors: vec![], view: self, program: WasiProgram::default() };
        match state.convert() {
            Ok(_) => GaiaDiagnostics { result: Ok(state.program), diagnostics: state.errors },
            Err(e) => GaiaDiagnostics { result: Err(e), diagnostics: state.errors },
        }
    }
}

struct Wasm2Program {
    errors: Vec<GaiaError>,
    view: WasmView,
    program: WasiProgram,
}

impl Wasm2Program {
    fn convert(&mut self) -> Result<(), gaia_types::GaiaError> {
        todo!()
    }
}
