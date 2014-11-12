use crate::{formats::luac::view::LuacView, program::LuaProgram};
use gaia_types::GaiaDiagnostics;

impl LuaProgram {
    pub fn to_luac(self) -> GaiaDiagnostics<LuacView> {
        todo!()
    }
}

struct Program2Luac {}
