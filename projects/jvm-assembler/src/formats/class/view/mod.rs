use crate::program::{JvmAccessFlags, JvmVersion};

#[derive(Clone, Debug)]
pub struct ClassInfo {
    pub magic: u32,
    pub version: JvmVersion,
    pub access_flags: JvmAccessFlags,
    pub this_class: String,
    pub super_class: Option<String>,
}
