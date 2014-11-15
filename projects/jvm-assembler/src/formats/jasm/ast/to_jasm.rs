use crate::{formats::jasm::ast::*, program::*};
use gaia_types::{GaiaDiagnostics, GaiaError, Result};

pub struct JvmToJasmConverter {
    constant_pool: JvmConstantPool,
}

impl JvmToJasmConverter {
    pub fn new() -> Self {
        Self { constant_pool: JvmConstantPool::new() }
    }

    pub fn convert(&mut self, program: JvmProgram) -> GaiaDiagnostics<JasmRoot> {
        let constant_pool = program.constant_pool.clone();
        self.constant_pool = constant_pool;
        match self.convert_class(program) {
            Ok(jasm_class) => GaiaDiagnostics::success(JasmRoot { class: jasm_class }),
            Err(error) => GaiaDiagnostics::failure(error),
        }
    }

    fn convert_class(&mut self, program: JvmProgram) -> Result<JasmClass> {
        let mut jasm_class = JasmClass {
            modifiers: program.access_flags.to_modifiers(),
            name: program.name,
            version: Some(format!("{}:{}", program.version.major, program.version.minor)),
            methods: Vec::new(),
            fields: Vec::new(),
            source_file: program.source_file,
        };

        for jvm_field in program.fields {
            let jasm_field = self.convert_field(jvm_field)?;
            jasm_class.fields.push(jasm_field);
        }

        for jvm_method in program.methods {
            let jasm_method = self.convert_method(jvm_method)?;
            jasm_class.methods.push(jasm_method);
        }

        Ok(jasm_class)
    }

    fn convert_method(&mut self, method: JvmMethod) -> Result<JasmMethod> {
        let modifiers = method.access_flags.to_modifiers();
        let name_and_descriptor = format!("{}:{}", method.name, method.descriptor);
        let stack_size = Some(method.max_stack as u32);
        let locals_count = Some(method.max_locals as u32);

        let mut instructions = Vec::new();
        for jvm_instruction in method.instructions {
            let jasm_instruction = self.convert_instruction(jvm_instruction)?;
            instructions.push(jasm_instruction);
        }

        Ok(JasmMethod { modifiers, name_and_descriptor, stack_size, locals_count, instructions })
    }

    fn convert_field(&mut self, field: JvmField) -> Result<JasmField> {
        let modifiers = field.access_flags.to_modifiers();
        let name_and_descriptor = format!("{}:{}", field.name, field.descriptor);
        Ok(JasmField { modifiers, name_and_descriptor })
    }

    fn convert_instruction(&mut self, instruction: JvmInstruction) -> Result<JasmInstruction> {
        match instruction {
            JvmInstruction::Simple { opcode } => Ok(JasmInstruction::Simple(opcode.to_string())),
            JvmInstruction::WithImmediate { opcode, value } => {
                Ok(JasmInstruction::WithArgument { instruction: opcode.to_string(), argument: value.to_string() })
            }
            JvmInstruction::WithConstantPool { opcode, symbol } => {
                let resolved_argument = self.resolve_constant_pool_symbol(&symbol)?;
                Ok(JasmInstruction::WithArgument { instruction: opcode.to_string(), argument: resolved_argument })
            }
            JvmInstruction::MethodCall { opcode, class_name, method_name, descriptor } => {
                let method_ref = format!("{}.\"{}\":\"{}\"", class_name, method_name, descriptor);
                Ok(JasmInstruction::MethodCall { instruction: opcode.to_string(), method_ref })
            }
            JvmInstruction::FieldAccess { opcode, class_name, field_name, descriptor } => {
                let field_ref = format!("{}.{}:\"{}\"", class_name, field_name, descriptor);
                Ok(JasmInstruction::FieldAccess { instruction: opcode.to_string(), field_ref })
            }
            _ => Err(GaiaError::custom_error(format!("Unsupported JVM instruction for JASM conversion: {:?}", instruction))),
        }
    }

    fn resolve_constant_pool_symbol(&self, symbol: &str) -> Result<String> {
        if let Some(index) = self.constant_pool.symbol_table.get(symbol) {
            if let Some(entry) = self.constant_pool.entries.get(*index as usize) {
                match entry {
                    JvmConstantPoolEntry::Utf8 { value } => Ok(format!("\"{}\"", value)),
                    JvmConstantPoolEntry::Integer { value } => Ok(value.to_string()),
                    JvmConstantPoolEntry::Float { value } => Ok(value.to_string()),
                    JvmConstantPoolEntry::Long { value } => Ok(value.to_string()),
                    JvmConstantPoolEntry::Double { value } => Ok(value.to_string()),
                    JvmConstantPoolEntry::Class { name } => Ok(name.clone()),
                    JvmConstantPoolEntry::String { value } => Ok(format!("String \"{}\"", value)),
                    JvmConstantPoolEntry::Fieldref { class_name, name, descriptor } => {
                        Ok(format!("{}.{}:\"{}\"", class_name, name, descriptor))
                    }
                    JvmConstantPoolEntry::Methodref { class_name, name, descriptor } => {
                        Ok(format!("{}.\"{}\":\"{}\"", class_name, name, descriptor))
                    }
                    JvmConstantPoolEntry::InterfaceMethodref { class_name, name, descriptor } => {
                        Ok(format!("{}.\"{}\":\"{}\"", class_name, name, descriptor))
                    }
                    _ => Err(GaiaError::custom_error(format!(
                        "Unsupported constant pool entry type for JASM conversion: {:?}",
                        entry
                    ))),
                }
            }
            else {
                Err(GaiaError::custom_error(format!("Constant pool entry not found for index: {}", index)))
            }
        }
        else {
            Err(GaiaError::custom_error(format!("Symbol not found in constant pool: {}", symbol)))
        }
    }
}

impl Default for JvmToJasmConverter {
    fn default() -> Self {
        Self::new()
    }
}

pub fn convert_jvm_to_jasm(program: JvmProgram) -> GaiaDiagnostics<JasmRoot> {
    let mut converter = JvmToJasmConverter::new();
    converter.convert(program)
}
