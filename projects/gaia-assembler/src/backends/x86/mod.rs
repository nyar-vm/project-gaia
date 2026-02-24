//! Native x86_64 backend compiler

use crate::{
    config::GaiaConfig,
    instruction::{CoreInstruction, DomainInstruction, GaiaInstruction, ManagedInstruction},
    program::{GaiaConstant, GaiaFunction, GaiaModule},
    types::GaiaType,
    Backend, GeneratedFiles,
};
use gaia_types::{
    helpers::{AbiCompatible, ApiCompatible, Architecture, CompilationTarget},
    GaiaError, Result,
};
use std::collections::HashMap;

/// Native x86_64 Backend implementation
#[derive(Default)]
pub struct X86Backend {}

impl Backend for X86Backend {
    fn name(&self) -> &'static str {
        "Native x86_64"
    }

    fn primary_target(&self) -> CompilationTarget {
        CompilationTarget { build: Architecture::X86_64, host: AbiCompatible::PE, target: ApiCompatible::MicrosoftVisualC }
    }

    fn match_score(&self, target: &CompilationTarget) -> f32 {
        if target.build == Architecture::X86_64 && target.host == AbiCompatible::PE {
            if target.target == ApiCompatible::MicrosoftVisualC {
                return 100.0; // Perfect match for native x86_64 on Windows
            }
            return 80.0;
        }
        0.0
    }

    fn generate(&self, program: &GaiaModule, _config: &GaiaConfig) -> Result<GeneratedFiles> {
        let mut code = Vec::new();
        let mut external_call_positions: HashMap<String, Vec<usize>> = HashMap::new();
        let mut string_patches = Vec::new(); // (position of imm32, string offset in rdata)

        // 0. Pre-pass: Collect strings
        let mut string_table = HashMap::new();
        let mut rdata_content = Vec::new();
        let mut next_string_offset = 0;

        for function in &program.functions {
            for block in &function.blocks {
                for inst in &block.instructions {
                    match inst {
                        GaiaInstruction::Core(CoreInstruction::PushConstant(GaiaConstant::String(s)))
                        | GaiaInstruction::Core(CoreInstruction::New(s))
                        | GaiaInstruction::Core(CoreInstruction::StoreField(_, s))
                        | GaiaInstruction::Core(CoreInstruction::LoadField(_, s))
                        | GaiaInstruction::Managed(ManagedInstruction::CallMethod { method: s, .. }) => {
                            if !string_table.contains_key(s) {
                                string_table.insert(s.clone(), next_string_offset);
                                rdata_content.extend_from_slice(s.as_bytes());
                                rdata_content.push(0); // null terminator
                                next_string_offset += s.len() + 1;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        // 1. Generate entry point (stub)
        // sub rsp, 32 (shadow space for Win64 calls)
        code.extend_from_slice(&[0x48, 0x83, 0xEC, 0x20]);

        // Initialize runtime
        let symbol = "nyar_init_runtime".to_string();
        let pos = code.len();
        code.extend_from_slice(&[0xFF, 0x15, 0x00, 0x00, 0x00, 0x00]); // call [rip+offset]
        external_call_positions.entry(symbol).or_default().push(pos);

        // call <main>
        let call_main_pos = code.len();
        code.extend_from_slice(&[0xE8, 0x00, 0x00, 0x00, 0x00]);

        // mov rcx, rax (return value as exit code)
        code.extend_from_slice(&[0x48, 0x89, 0xC1]);

        // call [rip + <iat_offset>] (ExitProcess)
        // Note: ExitProcess is usually required for EXEs, but we'll try to find it in imports
        let call_exit_pos = code.len();
        code.extend_from_slice(&[0xFF, 0x15, 0x00, 0x00, 0x00, 0x00]);

        // 2. Generate functions
        let mut function_offsets = HashMap::new();
        let mut internal_functions = std::collections::HashSet::new();
        for function in &program.functions {
            internal_functions.insert(function.name.clone());
        }

        let mut internal_call_positions = HashMap::new();

        for function in &program.functions {
            function_offsets.insert(function.name.clone(), code.len());
            self.generate_function(
                function,
                &internal_functions,
                &mut code,
                &mut external_call_positions,
                &mut internal_call_positions,
                &string_table,
                &mut string_patches,
            )?;
        }

        // 3. Patch main call
        let main_name = if function_offsets.contains_key("main") {
            "main"
        }
        else {
            program.functions.first().map(|f| f.name.as_str()).unwrap_or("")
        };

        if !main_name.is_empty() {
            let main_offset = function_offsets[main_name];
            let relative_offset = (main_offset as i32) - (call_main_pos as i32 + 5);
            code[call_main_pos + 1..call_main_pos + 5].copy_from_slice(&relative_offset.to_le_bytes());
        }

        // 4. Patch internal calls
        for (name, positions) in internal_call_positions {
            if let Some(&func_offset) = function_offsets.get(&name) {
                for pos in positions {
                    let relative_offset = (func_offset as i32) - (pos as i32 + 5);
                    code[pos + 1..pos + 5].copy_from_slice(&relative_offset.to_le_bytes());
                }
            }
        }

        let mut files = HashMap::new();
        files.insert("main.bin".to_string(), code.clone());

        let pe_bytes =
            self.create_pe_exe(&code, program, call_exit_pos, &external_call_positions, &rdata_content, &string_patches)?;
        files.insert("main.exe".to_string(), pe_bytes);

        Ok(GeneratedFiles { files, diagnostics: vec![] })
    }
}

impl X86Backend {
    fn generate_function(
        &self,
        function: &GaiaFunction,
        internal_functions: &std::collections::HashSet<String>,
        code: &mut Vec<u8>,
        external_call_positions: &mut HashMap<String, Vec<usize>>,
        internal_call_positions: &mut HashMap<String, Vec<usize>>,
        string_table: &HashMap<String, usize>,
        string_patches: &mut Vec<(usize, usize)>,
    ) -> Result<()> {
        let mut labels = HashMap::new();
        let mut jump_patches = Vec::new();
        let _function_start = code.len();

        // Function prologue
        code.push(0x55); // push rbp
        code.extend_from_slice(&[0x48, 0x89, 0xE5]); // mov rbp, rsp

        // Calculate stack space for locals and shadow space (32 bytes for Win64)
        // Ensure 16-byte alignment
        let locals_count = function
            .blocks
            .iter()
            .flat_map(|b| &b.instructions)
            .filter(|i| matches!(i, GaiaInstruction::Core(CoreInstruction::Alloca(_, _))))
            .count();
        let has_managed_calls = function
            .blocks
            .iter()
            .flat_map(|b| &b.instructions)
            .any(|i| matches!(i, GaiaInstruction::Managed(ManagedInstruction::CallMethod { .. })));

        let locals_size = locals_count * 8;
        let shadow_space = if has_managed_calls { 64 } else { 32 };
        let total_stack_size = (locals_size + shadow_space + 15) & !15;

        if total_stack_size > 0 {
            if total_stack_size <= 127 {
                code.extend_from_slice(&[0x48, 0x83, 0xEC]); // sub rsp, imm8
                code.push(total_stack_size as u8);
            }
            else {
                code.extend_from_slice(&[0x48, 0x81, 0xEC]); // sub rsp, imm32
                code.extend_from_slice(&(total_stack_size as i32).to_le_bytes());
            }
        }

        // Save arguments to shadow space for later use (e.g. by ManagedInstructions)
        // [rbp + 16] = rcx, [rbp + 24] = rdx, [rbp + 32] = r8, [rbp + 40] = r9
        code.extend_from_slice(&[0x48, 0x89, 0x4D, 0x10]);
        code.extend_from_slice(&[0x48, 0x89, 0x55, 0x18]);
        code.extend_from_slice(&[0x4C, 0x89, 0x45, 0x20]);
        code.extend_from_slice(&[0x4C, 0x89, 0x4D, 0x28]);

        for block in &function.blocks {
            labels.insert(block.label.clone(), code.len());

            for inst in &block.instructions {
                match inst {
                    GaiaInstruction::Core(core_inst) => match core_inst {
                        CoreInstruction::PushConstant(constant) => {
                            match constant {
                                GaiaConstant::I8(v) => {
                                    code.push(0x6A); // push (imm8)
                                    code.push(*v as u8);
                                }
                                GaiaConstant::I16(v) => {
                                    code.push(0x68); // push (imm32, sign-extended)
                                    code.extend_from_slice(&(*v as i32).to_le_bytes());
                                }
                                GaiaConstant::I32(v) => {
                                    code.push(0x68); // push (imm32)
                                    code.extend_from_slice(&v.to_le_bytes());
                                }
                                GaiaConstant::I64(v) => {
                                    // mov rax, imm64; push rax
                                    code.extend_from_slice(&[0x48, 0xB8]);
                                    code.extend_from_slice(&v.to_le_bytes());
                                    code.push(0x50); // push rax
                                }
                                GaiaConstant::String(s) => {
                                    // lea rax, [rip + offset]
                                    code.extend_from_slice(&[0x48, 0x8D, 0x05]);
                                    let str_offset = *string_table.get(s).unwrap();
                                    string_patches.push((code.len(), str_offset));
                                    code.extend_from_slice(&[0, 0, 0, 0]);
                                    code.push(0x50); // push rax
                                }
                                _ => return Err(GaiaError::custom_error("Unsupported constant type for x86 backend")),
                            }
                        }
                        CoreInstruction::Pop => {
                            code.push(0x58); // pop rax
                        }
                        CoreInstruction::Dup => {
                            // mov rax, [rsp]; push rax
                            code.extend_from_slice(&[0x48, 0x8B, 0x04, 0x24]);
                            code.push(0x50);
                        }
                        CoreInstruction::Add(_) => {
                            // pop rbx; pop rax; add rax, rbx; push rax
                            code.push(0x5B); // pop rbx
                            code.push(0x58); // pop rax
                            code.extend_from_slice(&[0x48, 0x01, 0xD8]); // add rax, rbx
                            code.push(0x50); // push rax
                        }
                        CoreInstruction::Sub(_) => {
                            // pop rbx; pop rax; sub rax, rbx; push rax
                            code.push(0x5B); // pop rbx
                            code.push(0x58); // pop rax
                            code.extend_from_slice(&[0x48, 0x29, 0xD8]); // sub rax, rbx
                            code.push(0x50); // push rax
                        }
                        CoreInstruction::Mul(_) => {
                            // pop rbx; pop rax; imul rax, rbx; push rax
                            code.push(0x5B); // pop rbx
                            code.push(0x58); // pop rax
                            code.extend_from_slice(&[0x48, 0x0F, 0xAF, 0xC3]); // imul rax, rbx
                            code.push(0x50); // push rax
                        }
                        CoreInstruction::Div(_) => {
                            // pop rbx; pop rax; cqo; idiv rbx; push rax
                            code.push(0x5B); // pop rbx
                            code.push(0x58); // pop rax
                            code.extend_from_slice(&[0x48, 0x99]); // cqo
                            code.extend_from_slice(&[0x48, 0xF7, 0xFB]); // idiv rbx
                            code.push(0x50); // push rax
                        }
                        CoreInstruction::Rem(_) => {
                            // pop rbx; pop rax; cqo; idiv rbx; push rdx
                            code.push(0x5B); // pop rbx
                            code.push(0x58); // pop rax
                            code.extend_from_slice(&[0x48, 0x99]); // cqo
                            code.extend_from_slice(&[0x48, 0xF7, 0xFB]); // idiv rbx
                            code.push(0x52); // push rdx
                        }
                        CoreInstruction::Neg(_) => {
                            // pop rax; neg rax; push rax
                            code.push(0x58); // pop rax
                            code.extend_from_slice(&[0x48, 0xF7, 0xD8]); // neg rax
                            code.push(0x50); // push rax
                        }
                        CoreInstruction::Not(ty) => {
                            // pop rax; not rax; push rax
                            // If boolean, xor rax, 1
                            code.push(0x58); // pop rax
                            match ty {
                                GaiaType::Bool => {
                                    code.extend_from_slice(&[0x48, 0x83, 0xF0, 0x01]);
                                    // xor rax, 1
                                }
                                _ => {
                                    code.extend_from_slice(&[0x48, 0xF7, 0xD0]);
                                    // not rax
                                }
                            }
                            code.push(0x50); // push rax
                        }
                        CoreInstruction::Shl(_) => {
                            // pop rcx; pop rax; shl rax, cl; push rax
                            code.push(0x59); // pop rcx
                            code.push(0x58); // pop rax
                            code.extend_from_slice(&[0x48, 0xD3, 0xE0]); // shl rax, cl
                            code.push(0x50); // push rax
                        }
                        CoreInstruction::Shr(_) => {
                            // pop rcx; pop rax; shr rax, cl; push rax
                            code.push(0x59); // pop rcx
                            code.push(0x58); // pop rax
                            code.extend_from_slice(&[0x48, 0xD3, 0xE8]); // shr rax, cl
                            code.push(0x50); // push rax
                        }
                        CoreInstruction::And(_) => {
                            // pop rbx; pop rax; and rax, rbx; push rax
                            code.push(0x5B); // pop rbx
                            code.push(0x58); // pop rax
                            code.extend_from_slice(&[0x48, 0x21, 0xD8]); // and rax, rbx
                            code.push(0x50); // push rax
                        }
                        CoreInstruction::Or(_) => {
                            // pop rbx; pop rax; or rax, rbx; push rax
                            code.push(0x5B); // pop rbx
                            code.push(0x58); // pop rax
                            code.extend_from_slice(&[0x48, 0x09, 0xD8]); // or rax, rbx
                            code.push(0x50); // push rax
                        }
                        CoreInstruction::Xor(_) => {
                            // pop rbx; pop rax; xor rax, rbx; push rax
                            code.push(0x5B); // pop rbx
                            code.push(0x58); // pop rax
                            code.extend_from_slice(&[0x48, 0x31, 0xD8]); // xor rax, rbx
                            code.push(0x50); // push rax
                        }
                        CoreInstruction::LoadArg(idx, _ty) => {
                            match idx {
                                0 => code.push(0x51),                       // push rcx
                                1 => code.push(0x52),                       // push rdx
                                2 => code.extend_from_slice(&[0x41, 0x50]), // push r8
                                3 => code.extend_from_slice(&[0x41, 0x51]), // push r9
                                _ => {
                                    // Load from stack [rbp + 16 + 32 + (idx-4)*8]
                                    let offset = 48 + (idx - 4) * 8;
                                    code.extend_from_slice(&[0x48, 0x8B, 0x45]);
                                    code.push(offset as u8);
                                    code.push(0x50); // push rax
                                }
                            }
                        }
                        CoreInstruction::StoreLocal(idx, _ty) => {
                            // pop rax
                            code.push(0x58);
                            // mov [rbp - (idx+1)*8], rax
                            let offset = (idx + 1) * 8;
                            code.extend_from_slice(&[0x48, 0x89, 0x45]);
                            code.push((-(offset as i32)) as u8);
                        }
                        CoreInstruction::LoadLocal(idx, _ty) => {
                            // mov rax, [rbp - (idx+1)*8]
                            let offset = (idx + 1) * 8;
                            code.extend_from_slice(&[0x48, 0x8B, 0x45]);
                            code.push((-(offset as i32)) as u8);
                            // push rax
                            code.push(0x50);
                        }
                        CoreInstruction::Alloca(_, _) => {
                            // Handled in prologue, no-op here
                        }
                        CoreInstruction::Label(name) => {
                            labels.insert(name.clone(), code.len());
                        }
                        CoreInstruction::Br(target) => {
                            code.push(0xE9); // jmp rel32
                            jump_patches.push((code.len(), target.clone()));
                            code.extend_from_slice(&[0, 0, 0, 0]);
                        }
                        CoreInstruction::BrTrue(target) => {
                            // pop rax; test rax, rax; jnz rel32
                            code.push(0x58); // pop rax
                            code.extend_from_slice(&[0x48, 0x85, 0xC0]); // test rax, rax
                            code.extend_from_slice(&[0x0F, 0x85]); // jnz rel32
                            jump_patches.push((code.len(), target.clone()));
                            code.extend_from_slice(&[0, 0, 0, 0]);
                        }
                        CoreInstruction::BrFalse(target) => {
                            // pop rax; test rax, rax; jz rel32
                            code.push(0x58); // pop rax
                            code.extend_from_slice(&[0x48, 0x85, 0xC0]); // test rax, rax
                            code.extend_from_slice(&[0x0F, 0x84]); // jz rel32
                            jump_patches.push((code.len(), target.clone()));
                            code.extend_from_slice(&[0, 0, 0, 0]);
                        }
                        CoreInstruction::Cmp(cond, _) => {
                            use crate::instruction::CmpCondition;
                            // pop rbx; pop rax; cmp rax, rbx; set<cond> al; movzx rax, al; push rax
                            code.push(0x5B); // pop rbx
                            code.push(0x58); // pop rax
                            code.extend_from_slice(&[0x48, 0x39, 0xD8]); // cmp rax, rbx
                            match cond {
                                CmpCondition::Eq => code.extend_from_slice(&[0x0F, 0x94, 0xC0]), // sete al
                                CmpCondition::Ne => code.extend_from_slice(&[0x0F, 0x95, 0xC0]), // setne al
                                CmpCondition::Lt => code.extend_from_slice(&[0x0F, 0x9C, 0xC0]), // setl al
                                CmpCondition::Le => code.extend_from_slice(&[0x0F, 0x9E, 0xC0]), // setle al
                                CmpCondition::Gt => code.extend_from_slice(&[0x0F, 0x9F, 0xC0]), // setg al
                                CmpCondition::Ge => code.extend_from_slice(&[0x0F, 0x9D, 0xC0]), // setge al
                            }
                            code.extend_from_slice(&[0x48, 0x0F, 0xB6, 0xC0]); // movzx rax, al
                            code.push(0x50); // push rax
                        }
                        CoreInstruction::Call(name, argc) => {
                            // Win64 ABI: RCX, RDX, R8, R9
                            // Pop arguments in reverse order (last arg first)
                            if *argc >= 4 {
                                code.extend_from_slice(&[0x41, 0x59]); // pop r9
                            }
                            if *argc >= 3 {
                                code.extend_from_slice(&[0x41, 0x58]); // pop r8
                            }
                            if *argc >= 2 {
                                code.push(0x5A); // pop rdx
                            }
                            if *argc >= 1 {
                                code.push(0x59); // pop rcx
                            }

                            if internal_functions.contains(name) {
                                // call rel32 (internal)
                                let pos = code.len();
                                code.extend_from_slice(&[0xE8, 0x00, 0x00, 0x00, 0x00]);
                                internal_call_positions.entry(name.clone()).or_default().push(pos);
                            }
                            else {
                                // call [rip + offset] (external)
                                let pos = code.len();
                                code.extend_from_slice(&[0xFF, 0x15, 0x00, 0x00, 0x00, 0x00]);
                                external_call_positions.entry(name.clone()).or_default().push(pos);
                            }

                            code.push(0x50); // push rax
                        }
                        CoreInstruction::CallIndirect(argc) => {
                            // Stack: [..., func_ptr, arg1, arg2, ...]
                            // Pops: args in reverse, then func_ptr

                            if *argc >= 4 {
                                code.extend_from_slice(&[0x41, 0x59]); // pop r9
                            }
                            if *argc >= 3 {
                                code.extend_from_slice(&[0x41, 0x58]); // pop r8
                            }
                            if *argc >= 2 {
                                code.push(0x5A); // pop rdx
                            }
                            if *argc >= 1 {
                                code.push(0x59); // pop rcx
                            }

                            // pop rax (func_ptr)
                            code.push(0x58);
                            // call rax
                            code.extend_from_slice(&[0xFF, 0xD0]);
                            // push result
                            code.push(0x50);
                        }
                        CoreInstruction::New(ty_name) => {
                            // call nyar_new_object(type_name)
                            // 1. Load string address into RCX (1st arg)
                            code.extend_from_slice(&[0x48, 0x8D, 0x0D]); // lea rcx, [rip + offset]
                            let str_offset = *string_table.get(ty_name).unwrap();
                            let pos = code.len();
                            string_patches.push((pos, str_offset));
                            code.extend_from_slice(&[0, 0, 0, 0]);

                            // 2. Call runtime
                            let symbol = "nyar_new_object".to_string();
                            code.extend_from_slice(&[0x48, 0x83, 0xEC, 0x20]); // sub rsp, 32
                            let pos = code.len();
                            code.extend_from_slice(&[0xFF, 0x15, 0x00, 0x00, 0x00, 0x00]); // call [rip+offset]
                            external_call_positions.entry(symbol).or_default().push(pos);
                            code.extend_from_slice(&[0x48, 0x83, 0xC4, 0x20]); // add rsp, 32

                            code.push(0x50); // push rax (obj ptr)
                        }
                        CoreInstruction::StoreField(_ty, field) => {
                            // Stack: [..., obj, value] (value is top)
                            // Pops: value(r8), obj(rcx)
                            // Arg2: field_name (rdx)

                            code.extend_from_slice(&[0x41, 0x58]); // pop r8 (value)
                            code.push(0x59); // pop rcx (obj)

                            // Load field name string into RDX
                            code.extend_from_slice(&[0x48, 0x8D, 0x15]); // lea rdx, [rip + offset]
                            let str_offset = *string_table.get(field).unwrap();
                            let pos = code.len();
                            string_patches.push((pos, str_offset));
                            code.extend_from_slice(&[0, 0, 0, 0]);

                            let symbol = "nyar_object_set".to_string();
                            code.extend_from_slice(&[0x48, 0x83, 0xEC, 0x20]);
                            let pos = code.len();
                            code.extend_from_slice(&[0xFF, 0x15, 0x00, 0x00, 0x00, 0x00]);
                            external_call_positions.entry(symbol).or_default().push(pos);
                            code.extend_from_slice(&[0x48, 0x83, 0xC4, 0x20]);

                            // StoreField returns void, no push
                        }
                        CoreInstruction::LoadField(_ty, field) => {
                            // Stack: [..., obj]
                            // Pops: obj(rcx)
                            // Arg2: key(rdx)

                            code.push(0x59); // pop rcx

                            // Load field name string into RDX
                            code.extend_from_slice(&[0x48, 0x8D, 0x15]); // lea rdx, [rip + offset]
                            let str_offset = *string_table.get(field).unwrap();
                            let pos = code.len();
                            string_patches.push((pos, str_offset));
                            code.extend_from_slice(&[0, 0, 0, 0]);

                            let symbol = "nyar_object_get".to_string();
                            code.extend_from_slice(&[0x48, 0x83, 0xEC, 0x20]);
                            let pos = code.len();
                            code.extend_from_slice(&[0xFF, 0x15, 0x00, 0x00, 0x00, 0x00]);
                            external_call_positions.entry(symbol).or_default().push(pos);
                            code.extend_from_slice(&[0x48, 0x83, 0xC4, 0x20]);

                            code.push(0x50); // push rax
                        }
                        CoreInstruction::NewArray(_, len_on_stack) => {
                            if *len_on_stack {
                                code.push(0x59); // pop rcx (length)
                            }
                            else {
                                // Assume 0 length if not on stack? Or error?
                                // For now, just zero out rcx
                                code.extend_from_slice(&[0x48, 0x31, 0xC9]); // xor rcx, rcx
                            }

                            // call nyar_new_array(length)
                            // We need to implement this symbol in runtime or link it
                            let symbol = "nyar_new_array".to_string();

                            // Prepare call
                            code.extend_from_slice(&[0x48, 0x83, 0xEC, 0x20]); // sub rsp, 32

                            let pos = code.len();
                            code.extend_from_slice(&[0xFF, 0x15, 0x00, 0x00, 0x00, 0x00]); // call [rip+offset]
                            external_call_positions.entry(symbol).or_default().push(pos);

                            code.extend_from_slice(&[0x48, 0x83, 0xC4, 0x20]); // add rsp, 32
                            code.push(0x50); // push rax (array ptr)
                        }
                        CoreInstruction::StoreElement(_) => {
                            // Stack: [..., array, index, value]
                            // Pops: value(r8), index(rdx), array(rcx)

                            code.extend_from_slice(&[0x41, 0x58]); // pop r8 (value)
                            code.push(0x5A); // pop rdx (index)
                            code.push(0x59); // pop rcx (array)

                            let symbol = "nyar_array_set".to_string();

                            code.extend_from_slice(&[0x48, 0x83, 0xEC, 0x20]); // sub rsp, 32

                            let pos = code.len();
                            code.extend_from_slice(&[0xFF, 0x15, 0x00, 0x00, 0x00, 0x00]); // call [rip+offset]
                            external_call_positions.entry(symbol).or_default().push(pos);

                            code.extend_from_slice(&[0x48, 0x83, 0xC4, 0x20]); // add rsp, 32
                        }
                        CoreInstruction::LoadElement(_) => {
                            // Stack: [..., array, index]
                            // Pops: index(rdx), array(rcx)

                            code.push(0x5A); // pop rdx (index)
                            code.push(0x59); // pop rcx (array)

                            let symbol = "nyar_array_get".to_string();

                            code.extend_from_slice(&[0x48, 0x83, 0xEC, 0x20]); // sub rsp, 32

                            let pos = code.len();
                            code.extend_from_slice(&[0xFF, 0x15, 0x00, 0x00, 0x00, 0x00]); // call [rip+offset]
                            external_call_positions.entry(symbol).or_default().push(pos);

                            code.extend_from_slice(&[0x48, 0x83, 0xC4, 0x20]); // add rsp, 32
                            code.push(0x50); // push rax
                        }
                        CoreInstruction::ArrayLength => {
                            // Stack: [..., array]
                            // Pops: array(rcx)

                            code.push(0x59); // pop rcx

                            let symbol = "nyar_array_len".to_string();

                            code.extend_from_slice(&[0x48, 0x83, 0xEC, 0x20]); // sub rsp, 32

                            let pos = code.len();
                            code.extend_from_slice(&[0xFF, 0x15, 0x00, 0x00, 0x00, 0x00]); // call [rip+offset]
                            external_call_positions.entry(symbol).or_default().push(pos);

                            code.extend_from_slice(&[0x48, 0x83, 0xC4, 0x20]); // add rsp, 32
                            code.push(0x50); // push rax
                        }
                        CoreInstruction::ArrayPush => {
                            // Stack: [..., array, value]
                            // Pops: value(rdx), array(rcx)

                            code.push(0x5A); // pop rdx (value)
                            code.push(0x59); // pop rcx (array)

                            let symbol = "nyar_array_push".to_string();

                            code.extend_from_slice(&[0x48, 0x83, 0xEC, 0x20]); // sub rsp, 32

                            let pos = code.len();
                            code.extend_from_slice(&[0xFF, 0x15, 0x00, 0x00, 0x00, 0x00]); // call [rip+offset]
                            external_call_positions.entry(symbol).or_default().push(pos);

                            code.extend_from_slice(&[0x48, 0x83, 0xC4, 0x20]); // add rsp, 32

                            // Push result (void -> 0 or new length?)
                            // JS push returns new length.
                            // nyar_array_push returns void.
                            // For now, push 0 (undefined/void)
                            code.push(0x31);
                            code.push(0xC0); // xor eax, eax
                            code.push(0x50); // push rax
                        }
                        _ => return Err(GaiaError::custom_error(format!("Unsupported core instruction: {:?}", core_inst))),
                    },
                    GaiaInstruction::Managed(managed_inst) => match managed_inst {
                        ManagedInstruction::CallMethod { method, signature, call_site_id, .. } => {
                            let argc = signature.params.len() as u32;

                            // 0. Allocate space for the call (shadow space + 5th/6th args)
                            // 32 (shadow) + 8 (method_name) + 8 (argc) = 48.
                            // 48 is 16-byte aligned.
                            code.extend_from_slice(&[0x48, 0x83, 0xEC, 0x30]);

                            // 1. Load vm (rcx) from [rbp + 16]
                            code.extend_from_slice(&[0x48, 0x8B, 0x4D, 0x10]);

                            // 2. Load ic (rdx) from [rbp + 24]
                            code.extend_from_slice(&[0x48, 0x8B, 0x55, 0x18]);

                            // 3. Load call_site_id (r8)
                            code.extend_from_slice(&[0x49, 0xC7, 0xC0]);
                            if let Some(id) = call_site_id {
                                code.extend_from_slice(&id.to_le_bytes());
                            }
                            else {
                                code.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]);
                            }

                            // 4. Load receiver (r9) from [rsp + 48 + argc * 8]
                            // Adjusting offset because stack was already sub 48.
                            // Arguments were pushed onto stack BEFORE this call.
                            let receiver_offset = 48 + argc * 8;
                            code.extend_from_slice(&[0x4C, 0x8B, 0x8C, 0x24]);
                            code.extend_from_slice(&receiver_offset.to_le_bytes());

                            // 5. Load method_name pointer into [rsp + 32]
                            code.extend_from_slice(&[0x48, 0x8D, 0x05]);
                            let str_offset = *string_table.get(method).unwrap();
                            let pos = code.len();
                            string_patches.push((pos, str_offset));
                            code.extend_from_slice(&[0, 0, 0, 0]);
                            code.extend_from_slice(&[0x48, 0x89, 0x44, 0x24, 0x20]);

                            // 6. Load argc into [rsp + 40]
                            code.extend_from_slice(&[0x48, 0xC7, 0x44, 0x24, 0x28]);
                            code.extend_from_slice(&argc.to_le_bytes());

                            // 7. Call nyar_managed_call_method
                            let symbol = "nyar_managed_call_method".to_string();
                            let pos = code.len();
                            code.extend_from_slice(&[0xFF, 0x15, 0x00, 0x00, 0x00, 0x00]);
                            external_call_positions.entry(symbol).or_default().push(pos);

                            // 8. Clean up call space
                            code.extend_from_slice(&[0x48, 0x83, 0xC4, 0x30]);

                            // 9. Clean up stack (pop args and receiver)
                            let total_to_pop = (argc + 1) * 8;
                            code.extend_from_slice(&[0x48, 0x81, 0xC4]);
                            code.extend_from_slice(&total_to_pop.to_le_bytes());

                            // 10. Push result
                            code.push(0x50);
                        }
                        _ => {
                            return Err(GaiaError::custom_error(format!("Unsupported managed instruction: {:?}", managed_inst)))
                        }
                    },
                    GaiaInstruction::Domain(domain_inst) => match domain_inst {
                        DomainInstruction::Neural(node) => {
                            // 调用 matmul (专用加速路径)
                            if let gaia_types::neural::NeuralNode::MatMul(_) = node {
                                let symbol = "gaia_matmul".to_string();
                                external_call_positions.entry(symbol).or_default().push(code.len());
                                // 这里插入一个占位符，后续在 PE 生成时打补丁到导入表或静态库
                                code.extend_from_slice(&[0xFF, 0x15, 0x00, 0x00, 0x00, 0x00]);
                            }
                        }
                        _ => return Err(GaiaError::custom_error(format!("Unsupported domain instruction: {:?}", domain_inst))),
                    },
                }
            }

            // Handle terminator
            match &block.terminator {
                crate::program::GaiaTerminator::Jump(target) => {
                    code.push(0xE9); // jmp rel32
                    jump_patches.push((code.len(), target.clone()));
                    code.extend_from_slice(&[0, 0, 0, 0]);
                }
                crate::program::GaiaTerminator::Branch { true_label, false_label } => {
                    // pop rax; test rax, rax; jnz true; jmp false
                    code.push(0x58); // pop rax
                    code.extend_from_slice(&[0x48, 0x85, 0xC0]); // test rax, rax
                    code.extend_from_slice(&[0x0F, 0x85]); // jnz rel32
                    jump_patches.push((code.len(), true_label.clone()));
                    code.extend_from_slice(&[0, 0, 0, 0]);

                    code.push(0xE9); // jmp rel32
                    jump_patches.push((code.len(), false_label.clone()));
                    code.extend_from_slice(&[0, 0, 0, 0]);
                }
                crate::program::GaiaTerminator::Return => {
                    // Function epilogue
                    code.extend_from_slice(&[0x48, 0x89, 0xEC]); // mov rsp, rbp
                    code.push(0x5D); // pop rbp
                    code.push(0xC3); // ret
                }
                _ => {}
            }
        }

        for (pos, name) in jump_patches {
            if let Some(&label_pos) = labels.get(&name) {
                let relative_offset = (label_pos as i32) - (pos as i32 + 4);
                code[pos..pos + 4].copy_from_slice(&relative_offset.to_le_bytes());
            }
        }

        Ok(())
    }

    fn create_pe_exe(
        &self,
        code: &[u8],
        program: &GaiaModule,
        call_exit_pos: usize,
        external_call_positions: &HashMap<String, Vec<usize>>,
        rdata_content: &Vec<u8>,
        string_patches: &Vec<(usize, usize)>,
    ) -> Result<Vec<u8>> {
        let mut imports = pe_assembler::types::ImportTable::new();

        // Group imports by library
        let mut lib_imports: HashMap<String, Vec<String>> = HashMap::new();
        for imp in &program.imports {
            lib_imports.entry(imp.library.clone()).or_default().push(imp.symbol.clone());
        }

        // Add implicit imports from external calls
        for symbol in external_call_positions.keys() {
            if symbol.starts_with("nyar_") {
                let entry = lib_imports.entry("nyar_runtime.dll".to_string()).or_default();
                if !entry.contains(symbol) {
                    entry.push(symbol.clone());
                }
            }
        }

        // Ensure kernel32.dll!ExitProcess is present if we are an EXE and it's not provided
        if !lib_imports.values().any(|funcs| funcs.contains(&"ExitProcess".to_string())) {
            lib_imports.entry("kernel32.dll".to_string()).or_default().push("ExitProcess".to_string());
        }

        for (lib, funcs) in lib_imports {
            imports.entries.push(pe_assembler::types::ImportEntry { dll_name: lib, functions: funcs });
        }

        let mut pe_program = pe_assembler::types::PeProgram::create_executable(code.to_vec()).with_imports(imports);

        // Ensure some critical fields are set correctly for native x64
        pe_program.header.optional_header.image_base = 0x400000;
        pe_program.header.optional_header.section_alignment = 0x1000;
        pe_program.header.optional_header.file_alignment = 0x200;
        pe_program.header.optional_header.major_operating_system_version = 6;
        pe_program.header.optional_header.minor_operating_system_version = 0;
        pe_program.header.optional_header.major_subsystem_version = 6;
        pe_program.header.optional_header.minor_subsystem_version = 0;

        // DYNAMIC_BASE | NX_COMPAT | NO_SEH | TERMINAL_SERVER_AWARE
        pe_program.header.optional_header.dll_characteristics = 0x8160;

        // Recalculate size of image and headers
        let text_size_aligned = (pe_program.sections[0].data.len() as u32 + 0xFFF) & !0xFFF;
        let idata_size_aligned =
            if pe_program.sections.len() > 1 { (pe_program.sections[1].virtual_size + 0xFFF) & !0xFFF } else { 0 };

        let rdata_rva = 0x1000 + text_size_aligned + idata_size_aligned;

        if !rdata_content.is_empty() {
            pe_program.sections.push(pe_assembler::types::PeSection {
                name: ".rdata".to_string(),
                characteristics: 0x40000040, // IMAGE_SCN_CNT_INITIALIZED_DATA | IMAGE_SCN_MEM_READ
                virtual_size: rdata_content.len() as u32,
                data: rdata_content.clone(),
                number_of_line_numbers: 0,
                number_of_relocations: 0,
                pointer_to_line_numbers: 0,
                pointer_to_relocations: 0,
                pointer_to_raw_data: 0,
                size_of_raw_data: 0,
                virtual_address: 0,
            });

            let code_data = &mut pe_program.sections[0].data;
            for &(pos, str_offset) in string_patches {
                let next_rip_rva = 0x1000 + (pos as u32) + 4; // pos points to start of imm32
                let target_rva = rdata_rva + str_offset as u32;
                let rel_offset = target_rva as i32 - next_rip_rva as i32;
                code_data[pos..pos + 4].copy_from_slice(&rel_offset.to_le_bytes());
            }
        }

        let rdata_size_aligned = if pe_program.sections.len() > (if idata_size_aligned > 0 { 2 } else { 1 }) {
            (pe_program.sections.last().unwrap().virtual_size + 0xFFF) & !0xFFF
        }
        else {
            0
        };

        pe_program.header.optional_header.size_of_image = 0x1000 + text_size_aligned + idata_size_aligned + rdata_size_aligned;
        pe_program.header.optional_header.size_of_headers = 0x200;

        // Patch calls to imported functions
        if pe_program.sections.len() > 1 {
            let iat_rva = pe_program.header.optional_header.data_directories[12].virtual_address;
            let code_data = &mut pe_program.sections[0].data;

            // Find ExitProcess in IAT
            let mut exit_process_iat_rva = 0;
            let mut current_iat_offset = 0;

            for entry in &pe_program.imports.entries {
                for (_i, func) in entry.functions.iter().enumerate() {
                    let rva = iat_rva + current_iat_offset;

                    if func == "ExitProcess" {
                        exit_process_iat_rva = rva;
                    }

                    if let Some(positions) = external_call_positions.get(func) {
                        for &pos in positions {
                            let next_rip_rva = 0x1000 + pos as u32 + 6;
                            let relative_offset = (rva as i32) - (next_rip_rva as i32);
                            code_data[pos + 2..pos + 6].copy_from_slice(&relative_offset.to_le_bytes());
                        }
                    }

                    current_iat_offset += 8; // x64 IAT entry size
                }
                current_iat_offset += 8; // Null terminator for DLL
            }

            // Patch the hardcoded ExitProcess call in entry point
            if exit_process_iat_rva != 0 {
                let next_rip_rva_exit = 0x1000 + call_exit_pos as u32 + 6;
                let relative_offset_exit = (exit_process_iat_rva as i32) - (next_rip_rva_exit as i32);
                code_data[call_exit_pos + 2..call_exit_pos + 6].copy_from_slice(&relative_offset_exit.to_le_bytes());
            }
        }

        let mut buffer = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut buffer);
        let mut writer = pe_assembler::formats::exe::writer::ExeWriter::new(&mut cursor);
        use pe_assembler::helpers::PeWriter;
        writer.write_program(&pe_program).map_err(|e| GaiaError::custom_error(format!("PE write error: {}", e)))?;
        Ok(buffer)
    }
}
