//! MSIL AST 到 CLR Program 转换器
//!
//! 这个模块负责将解析后的 MSIL 抽象语法树转换为 CLR Program 高层语义信息结构。

use crate::{
    formats::msil::ast::{MsilClass, MsilInstruction, MsilMethod, MsilMethodBody, MsilRoot, MsilStatement},
    program::{
        ClrAccessFlags, ClrExternalAssembly, ClrInstruction, ClrLocalVariable, ClrMethod, ClrMethodImplFlags, ClrModule,
        ClrOpcode, ClrParameter, ClrProgram, ClrType, ClrTypeReference, ClrVersion,
    },
};
use gaia_types::GaiaDiagnostics;

/// MSIL AST 到 CLR Program 转换器
#[derive(Debug, Clone)]
pub struct MsilToClrConverter {
    /// 当前程序集名称
    current_assembly: Option<String>,
    /// 当前模块名称
    current_module: Option<String>,
}

impl MsilToClrConverter {
    /// 创建新的转换器
    pub fn new() -> Self {
        Self { current_assembly: None, current_module: None }
    }

    /// 转换 MSIL 根节点到 CLR 程序
    pub fn convert(&mut self, msil_root: MsilRoot) -> GaiaDiagnostics<ClrProgram> {
        let mut program = ClrProgram::new("DefaultAssembly");
        let mut diagnostics = GaiaDiagnostics::success(program.clone());

        for statement in msil_root.statements {
            match statement {
                MsilStatement::Assembly(name) => {
                    program.name = name;
                }
                MsilStatement::Module(name) => {
                    program.module = Some(ClrModule { name, mvid: None });
                }
                MsilStatement::Class(msil_class) => {
                    if let Some(clr_type) = self.convert_class(msil_class) {
                        program.types.push(clr_type);
                    }
                }
                MsilStatement::AssemblyExtern(name) => {
                    let external_assembly = ClrExternalAssembly {
                        name,
                        version: ClrVersion::default(),
                        public_key_token: None,
                        culture: None,
                        hash_algorithm: None,
                    };
                    program.external_assemblies.push(external_assembly);
                }
            }
        }

        diagnostics.result = Ok(program);
        diagnostics
    }

    /// 转换类定义
    fn convert_class(&self, msil_class: MsilClass) -> Option<ClrType> {
        let namespace = self.extract_namespace(&msil_class.name);
        let simple_name = if let Some(ns) = &namespace {
            msil_class.name.strip_prefix(&format!("{}.", ns)).unwrap_or(&msil_class.name).to_string()
        }
        else {
            msil_class.name.clone()
        };

        let mut clr_type = ClrType {
            name: simple_name,
            namespace,
            access_flags: self.convert_class_modifiers(&msil_class.modifiers),
            base_type: msil_class.extends,
            interfaces: Vec::new(),
            fields: Vec::new(),
            methods: Vec::new(),
            properties: Vec::new(),
            events: Vec::new(),
            nested_types: Vec::new(),
            attributes: Vec::new(),
        };

        // 转换方法
        for msil_method in msil_class.methods {
            if let Some(clr_method) = self.convert_method(msil_method) {
                clr_type.methods.push(clr_method);
            }
        }

        Some(clr_type)
    }

    /// 转换方法定义
    fn convert_method(&self, msil_method: MsilMethod) -> Option<ClrMethod> {
        let return_type = ClrTypeReference {
            name: msil_method.return_type,
            namespace: None,
            assembly: None,
            is_value_type: false,
            is_reference_type: true,
            generic_parameters: Vec::new(),
        };

        let mut clr_method = ClrMethod {
            name: msil_method.name,
            return_type,
            parameters: Vec::new(),
            access_flags: self.convert_method_modifiers(&msil_method.modifiers),
            impl_flags: self.convert_method_impl_flags(&msil_method.modifiers),
            instructions: Vec::new(),
            max_stack: 8, // 默认值
            locals: Vec::new(),
            exception_handlers: Vec::new(),
            attributes: Vec::new(),
            is_entry_point: false,
        };

        // 转换参数
        for msil_param in msil_method.parameters {
            let parameter = ClrParameter {
                name: msil_param.name.unwrap_or_else(|| "param".to_string()),
                parameter_type: ClrTypeReference {
                    name: msil_param.param_type,
                    namespace: None,
                    assembly: None,
                    is_value_type: false,
                    is_reference_type: true,
                    generic_parameters: Vec::new(),
                },
                is_in: false,
                is_out: false,
                is_optional: false,
                default_value: None,
                attributes: Vec::new(),
            };
            clr_method.parameters.push(parameter);
        }

        // 转换方法体
        if let Some(body) = msil_method.body {
            self.convert_method_body(&mut clr_method, body);
        }

        Some(clr_method)
    }

    /// 转换方法体
    fn convert_method_body(&self, clr_method: &mut ClrMethod, msil_body: MsilMethodBody) {
        // 设置最大栈深度
        clr_method.max_stack = msil_body.maxstack.unwrap_or(8) as u16;

        // 转换局部变量
        for msil_local in msil_body.locals {
            let local_var = ClrLocalVariable {
                name: msil_local.name,
                variable_type: ClrTypeReference {
                    name: msil_local.local_type,
                    namespace: None,
                    assembly: None,
                    is_value_type: false,
                    is_reference_type: true,
                    generic_parameters: Vec::new(),
                },
                is_pinned: false,
            };
            clr_method.locals.push(local_var);
        }

        // 转换指令
        for msil_instruction in msil_body.instructions {
            if let Some(clr_instruction) = self.convert_instruction(msil_instruction) {
                clr_method.instructions.push(clr_instruction);
            }
        }
    }

    /// 转换指令
    fn convert_instruction(&self, msil_instruction: MsilInstruction) -> Option<ClrInstruction> {
        let opcode = match ClrOpcode::from_str(&msil_instruction.opcode) {
            Some(op) => op,
            None => return None, // 跳过未知指令
        };

        // 根据操作数类型创建相应的 ClrInstruction 变体
        let clr_instruction = if msil_instruction.operands.is_empty() {
            ClrInstruction::Simple { opcode }
        }
        else if msil_instruction.operands.len() == 1 {
            let operand = &msil_instruction.operands[0];
            // 尝试解析为不同类型的操作数
            if let Ok(value) = operand.parse::<i32>() {
                ClrInstruction::WithImmediate { opcode, value }
            }
            else if operand.starts_with("\"") && operand.ends_with("\"") {
                let string_value = operand[1..operand.len() - 1].to_string();
                ClrInstruction::WithString { opcode, value: string_value }
            }
            else {
                // 假设是方法或字段引用
                ClrInstruction::WithMethod { opcode, method_ref: operand.clone() }
            }
        }
        else {
            // 多个操作数，简化处理
            ClrInstruction::Simple { opcode }
        };

        Some(clr_instruction)
    }

    /// 提取命名空间
    fn extract_namespace(&self, full_name: &str) -> Option<String> {
        if let Some(last_dot) = full_name.rfind('.') {
            Some(full_name[..last_dot].to_string())
        }
        else {
            None
        }
    }

    /// 转换类修饰符
    fn convert_class_modifiers(&self, modifiers: &[String]) -> ClrAccessFlags {
        let mut flags = ClrAccessFlags::default();

        for modifier in modifiers {
            match modifier.as_str() {
                "public" => flags.is_public = true,
                "private" => flags.is_private = true,
                _ => {} // 忽略未知修饰符
            }
        }

        if !flags.is_public && !flags.is_private {
            flags.is_public = true; // 默认为公共
        }

        flags
    }

    /// 转换方法修饰符
    fn convert_method_modifiers(&self, modifiers: &[String]) -> ClrAccessFlags {
        let mut flags = ClrAccessFlags::default();

        for modifier in modifiers {
            match modifier.as_str() {
                "public" => flags.is_public = true,
                "private" => flags.is_private = true,
                _ => {} // 忽略未知修饰符
            }
        }

        if !flags.is_public && !flags.is_private {
            flags.is_public = true; // 默认为公共
        }

        flags
    }

    /// 转换方法实现标志
    fn convert_method_impl_flags(&self, modifiers: &[String]) -> ClrMethodImplFlags {
        let mut flags = ClrMethodImplFlags::default();
        flags.is_managed = true; // 默认值

        for modifier in modifiers {
            match modifier.as_str() {
                "cil" => {} // IL 代码，默认
                "managed" => flags.is_managed = true,
                "native" => flags.is_native = true,
                "runtime" => flags.is_runtime = true,
                _ => {} // 忽略未知修饰符
            }
        }

        flags
    }
}

impl Default for MsilToClrConverter {
    fn default() -> Self {
        Self::new()
    }
}
