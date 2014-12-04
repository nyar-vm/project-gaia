#![doc = include_str!("readme.md")]

use crate::program::{JvmAccessFlags, JvmVersion};

#[derive(Clone, Debug)]
pub struct ClassInfo {
    pub magic: u32,
    pub version: JvmVersion,
    pub access_flags: JvmAccessFlags,
    pub this_class: String,
    pub super_class: Option<String>,
}

impl Default for ClassInfo {
    fn default() -> Self {
        Self {
            magic: 0xCAFEBABE,
            version: JvmVersion { major: 52, minor: 0 },
            access_flags: JvmAccessFlags::default(),
            this_class: "DefaultClass".to_string(),
            super_class: Some("java/lang/Object".to_string()),
        }
    }
}