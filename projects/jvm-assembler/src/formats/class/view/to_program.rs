use crate::{
    formats::class::view::ClassView,
    program::{JvmConstantPool, JvmProgram},
};
use gaia_types::{GaiaDiagnostics, Result};

impl ClassView {
    pub fn to_program(self) -> GaiaDiagnostics<JvmProgram> {
        let mut converter = Class2Program {};
        match converter.convert(self) {
            Ok(program) => GaiaDiagnostics::success(program),
            Err(error) => GaiaDiagnostics::failure(error),
        }
    }
}

struct Class2Program {}

impl Class2Program {
    fn convert(&mut self, class_view: ClassView) -> Result<JvmProgram> {
        let mut constant_pool = JvmConstantPool::new();
        for entry in class_view.constant_pool {
            constant_pool.add_entry(entry);
        }

        Ok(JvmProgram {
            version: class_view.version,
            constant_pool,
            access_flags: class_view.access_flags,
            name: class_view.this_class,
            super_class: class_view.super_class,
            interfaces: class_view.interfaces,
            fields: class_view.fields,
            methods: class_view.methods,
            attributes: class_view.attributes,
            source_file: None, // SourceFile 属性需要单独处理
        })
    }
}
