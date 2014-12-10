use crate::program::LuaVersion;

pub mod to_luac;
mod to_program;

#[derive(Copy, Clone, Debug)]
pub struct LuacView {
    pub(crate) magic_head: [u8; 4],
    pub(crate) lua_version: LuaVersion,
}

impl Default for LuacView {
    fn default() -> Self {
        Self { magic_head: [0; 4], lua_version: LuaVersion::Unknown }
    }
}
