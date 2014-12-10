use crate::program::*;
use gaia_types::{GaiaDiagnostics, GaiaError, Result};
use oak_msil::ast as msil;

/// MSIL 到 CLR 程序的转换器
#[derive(Debug)]
pub struct MsilToClrConverter {
    /// 诊断信息
    pub diagnostics: Vec<GaiaError>,
}

impl MsilToClrConverter {
    /// 创建新的转换器
    pub fn new() -> Self {
        Self { diagnostics: Vec::new() }
    }

    /// 将 MSIL AST 转换为 CLR 程序
    pub fn convert(&mut self, root: msil::MsilRoot) -> GaiaDiagnostics<ClrProgram> {
        let mut name = "Unknown".to_string();
        let mut external_assemblies = Vec::new();
        let mut module = None;
        let mut types = Vec::new();

        for item in root.items {
            match item {
                msil::Item::Assembly(asm) => {
                    name = asm.name;
                }
                msil::Item::AssemblyExtern(ext_name) => {
                    external_assemblies.push(ClrExternalAssembly {
                        name: ext_name,
                        version: ClrVersion { major: 4, minor: 0, build: 0, revision: 0 },
                        public_key_token: None,
                        culture: None,
                        hash_algorithm: None,
                    });
                }
                msil::Item::Module(mod_name) => {
                    module = Some(ClrModule { name: mod_name, mvid: None });
                }
                msil::Item::Class(cls) => {
                    types.push(self.convert_class(cls));
                }
            }
        }

        let program = ClrProgram {
            name,
            version: ClrVersion { major: 1, minor: 0, build: 0, revision: 0 },
            access_flags: ClrAccessFlags {
                is_public: true,
                is_private: false,
                is_security_transparent: false,
                is_retargetable: false,
            },
            external_assemblies,
            module,
            types,
            global_methods: Vec::new(),
            global_fields: Vec::new(),
            attributes: Vec::new(),
            constant_pool: ClrConstantPool::new(),
            source_file: None,
        };

        GaiaDiagnostics::success(program)
    }

    fn convert_class(&mut self, cls: msil::Class) -> ClrType {
        let mut clr_type = ClrType::new(cls.name, None);
        for method in cls.methods {
            clr_type.add_method(self.convert_method(method));
        }
        clr_type
    }

    fn convert_method(&mut self, method: msil::Method) -> ClrMethod {
        let mut clr_method = ClrMethod::new(
            method.name,
            ClrTypeReference {
                name: "Void".to_string(),
                namespace: Some("System".to_string()),
                assembly: None,
                is_value_type: true,
                is_reference_type: false,
                generic_parameters: Vec::new(),
            },
        );
        for inst in method.instructions {
            clr_method.add_instruction(self.convert_instruction(inst));
        }
        clr_method
    }

    fn convert_instruction(&mut self, inst: msil::Instruction) -> ClrInstruction {
        // 简单的映射逻辑，实际实现需要更复杂的指令解析
        match inst {
            msil::Instruction::Simple(s) => {
                let opcode = match s.as_str() {
                    "ret" => ClrOpcode::Ret,
                    "ldarg.0" => ClrOpcode::Ldarg0,
                    _ => ClrOpcode::Nop,
                };
                ClrInstruction::Simple { opcode }
            }
            msil::Instruction::String(s) => ClrInstruction::WithString { opcode: ClrOpcode::Ldstr, value: s },
            msil::Instruction::Call(s) => ClrInstruction::WithMethod { opcode: ClrOpcode::Call, method_ref: s },
        }
    }
}
