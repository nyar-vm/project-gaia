mod to_class;
mod to_program;

use crate::program::{JvmAccessFlags, JvmAttribute, JvmConstantPoolEntry, JvmField, JvmMethod, JvmVersion};

#[derive(Clone, Debug)]
pub struct ClassView {
    pub magic: u32,
    pub version: JvmVersion,
    pub constant_pool: Vec<JvmConstantPoolEntry>,
    pub access_flags: JvmAccessFlags,
    pub this_class: String,
    pub super_class: Option<String>,
    pub interfaces: Vec<String>,
    pub fields: Vec<JvmField>,
    pub methods: Vec<JvmMethod>,
    pub attributes: Vec<JvmAttribute>,
}
