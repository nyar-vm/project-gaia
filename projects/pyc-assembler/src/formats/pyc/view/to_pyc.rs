use crate::{formats::pyc::view::PycView, program::PythonProgram};
use gaia_types::GaiaDiagnostics;

impl PythonProgram {
    /// 将 PycView 转换为 Luac 格式。
    pub fn to_luac(self) -> GaiaDiagnostics<PycView> {
        todo!()
    }
}

// struct Program2Luac {}
