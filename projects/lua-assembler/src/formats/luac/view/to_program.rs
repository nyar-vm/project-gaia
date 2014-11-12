use crate::{formats::luac::view::LuacView, program::LuaProgram};
use gaia_types::GaiaDiagnostics;

impl LuacView {
    pub fn to_program(self) -> GaiaDiagnostics<LuaProgram> {
        todo!()
    }
}

struct Luac2Program {}
