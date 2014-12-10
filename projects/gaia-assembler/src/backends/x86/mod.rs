//! Native x86_64 backend compiler

use crate::{
    config::GaiaConfig,
    instruction::{CoreInstruction, DomainInstruction, GaiaInstruction},
    program::{GaiaConstant, GaiaFunction, GaiaModule},
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
        let mut external_call_positions = HashMap::new();

        // 1. Generate entry point (stub)
        // sub rsp, 32 (shadow space for Win64 calls)
        code.extend_from_slice(&[0x48, 0x83, 0xEC, 0x20]);

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
        for function in &program.functions {
            function_offsets.insert(function.name.clone(), code.len());
            self.generate_function(function, &mut code, &mut external_call_positions)?;
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

        let mut files = HashMap::new();
        files.insert("main.bin".to_string(), code.clone());

        let pe_bytes = self.create_pe_exe(&code, program, call_exit_pos, &external_call_positions)?;
        files.insert("main.exe".to_string(), pe_bytes);

        Ok(GeneratedFiles { files, diagnostics: vec![] })
    }
}

impl X86Backend {
    fn generate_function(
        &self,
        function: &GaiaFunction,
        code: &mut Vec<u8>,
        external_call_positions: &mut HashMap<String, Vec<usize>>,
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
        let locals_size = locals_count * 8;
        let shadow_space = 32;
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
                                _ => return Err(GaiaError::custom_error("Unsupported constant type for x86 backend")),
                            }
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
                        CoreInstruction::Alloca(_, _) => {
                            // Space already reserved in prologue
                        }
                        CoreInstruction::LoadLocal(index, _) => {
                            // mov rax, [rbp - offset]; push rax
                            let offset = 32 + (index + 1) * 8;
                            code.extend_from_slice(&[0x48, 0x8B, 0x85]); // mov rax, [rbp - imm32]
                            code.extend_from_slice(&(-(offset as i32)).to_le_bytes());
                            code.push(0x50); // push rax
                        }
                        CoreInstruction::StoreLocal(index, _) => {
                            // pop rax; mov [rbp - offset], rax
                            let offset = 32 + (index + 1) * 8;
                            code.push(0x58); // pop rax
                            code.extend_from_slice(&[0x48, 0x89, 0x85]); // mov [rbp - imm32], rax
                            code.extend_from_slice(&(-(offset as i32)).to_le_bytes());
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
                            // Gaia Call: arguments are on stack in order.
                            // So the last argument is at the top of the stack.

                            // 1. Pop arguments into registers
                            if *argc >= 1 {
                                code.push(0x59);
                            } // pop rcx (1st arg if only 1, or temporary)
                            if *argc >= 2 {
                                code.push(0x5A); // pop rdx (2nd arg)
                                                 // swap rcx, rdx to get order right if we popped them in reverse
                                                 // Actually, if stack is [arg1, arg2], pop rdx gives arg2, pop rcx gives arg1. Correct.
                            }
                            if *argc >= 3 {
                                code.push(0x41);
                                code.push(0x58); // pop r8
                                                 // Now we have: R8=arg3, RDX=arg2, RCX=arg1. Correct.
                            }
                            if *argc >= 4 {
                                code.push(0x41);
                                code.push(0x59); // pop r9
                            }
                            // argc > 4 would need more work (stack arguments)

                            // 2. Call the function
                            external_call_positions.entry(name.clone()).or_default().push(code.len());
                            // call [rip + offset] (IAT style)
                            code.extend_from_slice(&[0xFF, 0x15, 0x00, 0x00, 0x00, 0x00]);

                            // 3. Push return value (rax) back to stack
                            code.push(0x50); // push rax
                        }
                        _ => return Err(GaiaError::custom_error(format!("Unsupported core instruction: {:?}", core_inst))),
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
                    _ => return Err(GaiaError::custom_error(format!("Unsupported instruction tier for x86: {:?}", inst))),
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
    ) -> Result<Vec<u8>> {
        let mut imports = pe_assembler::types::ImportTable::new();

        // Group imports by library
        let mut lib_imports: HashMap<String, Vec<String>> = HashMap::new();
        for imp in &program.imports {
            lib_imports.entry(imp.library.clone()).or_default().push(imp.symbol.clone());
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
        pe_program.header.optional_header.size_of_image = 0x1000 + text_size_aligned + idata_size_aligned;
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
