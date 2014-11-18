use crate::{formats::wasm::view::WasmView, program::WasiProgram};
use gaia_types::{GaiaDiagnostics, GaiaError};

impl WasiProgram {
    pub fn to_wasm_view(self) -> GaiaDiagnostics<WasmView> {
        let mut state = Program2WasmView { program: self, view: WasmView::default(), errors: vec![] };
        match state.convert() {
            Ok(_) => GaiaDiagnostics { result: Ok(state.view), diagnostics: state.errors },
            Err(e) => GaiaDiagnostics { result: Err(e), diagnostics: state.errors },
        }
    }
}

struct Program2WasmView {
    program: WasiProgram,
    view: WasmView,
    errors: Vec<GaiaError>,
}

impl Program2WasmView {
    fn convert(&mut self) -> Result<(), GaiaError> {
        todo!()
    }
}
