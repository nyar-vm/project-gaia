use crate::{formats::class::view::ClassView, program::JvmProgram};
use gaia_types::{GaiaDiagnostics, Result};

impl JvmProgram {
    pub fn to_class_view(self) -> GaiaDiagnostics<ClassView> {
        let mut converter = Program2Class {};
        match converter.convert(self) {
            Ok(class_view) => GaiaDiagnostics::success(class_view),
            Err(error) => GaiaDiagnostics::failure(error),
        }
    }
}

struct Program2Class {}

impl Program2Class {
    fn convert(&mut self, program: JvmProgram) -> Result<ClassView> {
        let mut constant_pool_entries = Vec::new();
        for entry in program.constant_pool.entries {
            constant_pool_entries.push(entry);
        }

        Ok(ClassView {
            magic: 0xCAFEBABE,
            version: program.version,
            constant_pool: constant_pool_entries,
            access_flags: program.access_flags,
            this_class: program.name,
            super_class: program.super_class,
            interfaces: program.interfaces,
            fields: program.fields,
            methods: program.methods,
            attributes: program.attributes,
        })
    }
}
