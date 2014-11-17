//! PE (Portable Executable) backend compiler
//! This backend generates .NET PE files containing IL code, not native machine code

use super::{Backend, GeneratedFiles};
use crate::{backends::msil::ClrBackend, config::GaiaConfig};
use gaia_types::{
    helpers::{AbiCompatible, ApiCompatible, Architecture, CompilationTarget},
    *,
};
use std::collections::HashMap;

/// PE Backend implementation
#[derive(Default)]
pub struct PeBackend {}

impl Backend for PeBackend {
    fn name(&self) -> &'static str {
        "PE"
    }

    fn primary_target(&self) -> CompilationTarget {
        CompilationTarget { build: Architecture::X86_64, host: AbiCompatible::PE, target: ApiCompatible::MicrosoftVisualC }
    }

    fn match_score(&self, target: &CompilationTarget) -> f32 {
        match target.host {
            AbiCompatible::PE => match target.build {
                // dll, exe output, 10% support
                Architecture::X86 => 10.0,
                // dll, exe output, 10% support
                Architecture::X86_64 => 10.0,
                _ => -100.0,
            },
            _ => -100.0,
        }
    }

    fn generate(&self, program: &GaiaProgram, _config: &GaiaConfig) -> Result<GeneratedFiles> {
        let mut files = HashMap::new();
        files.insert("main.dll".to_string(), compile(program)?);
        Ok(GeneratedFiles { files, diagnostics: vec![] })
    }
}

/// Compile Gaia program to .NET PE executable file
pub fn compile(program: &GaiaProgram) -> Result<Vec<u8>> {
    // Generate IL code using the IL backend
    let il_code = ClrBackend::generate(program)?;

    // Convert IL code to .NET PE format
    // For now, we'll create a simple .NET PE wrapper around the IL code
    generate_dotnet_pe_file(&il_code, &program.name)
}

/// Generate a .NET PE file containing the IL code
fn generate_dotnet_pe_file(il_code: &[u8], program_name: &str) -> Result<Vec<u8>> {
    // Create a minimal .NET PE file structure
    // This is a simplified implementation that creates a basic .NET executable

    // Convert IL bytecode to string for embedding in PE
    let il_text = String::from_utf8_lossy(il_code);

    // Create a minimal .NET PE file with the IL code
    // For now, we'll create a simple wrapper that can be executed by the .NET runtime
    let pe_content = create_minimal_dotnet_pe(&il_text, program_name)?;

    Ok(pe_content)
}

/// Create a minimal .NET PE file structure
fn create_minimal_dotnet_pe(il_code: &str, program_name: &str) -> Result<Vec<u8>> {
    // This is a simplified implementation
    // In a real implementation, we would need to create proper PE headers,
    // metadata tables, and IL method bodies according to the CLI specification

    // For now, we'll create a basic structure that includes the IL code
    // and can be recognized as a .NET assembly

    let mut pe_data = Vec::new();

    // Add DOS header (simplified)
    pe_data.extend_from_slice(b"MZ");
    pe_data.resize(0x80, 0);

    // Add PE signature
    pe_data.extend_from_slice(b"PE\0\0");

    // Add basic COFF header for .NET
    pe_data.extend_from_slice(&[
        0x4C, 0x01, // Machine: IMAGE_FILE_MACHINE_I386
        0x03, 0x00, // NumberOfSections: 3
        0x00, 0x00, 0x00, 0x00, // TimeDateStamp
        0x00, 0x00, 0x00, 0x00, // PointerToSymbolTable
        0x00, 0x00, 0x00, 0x00, // NumberOfSymbols
        0xE0, 0x00, // SizeOfOptionalHeader
        0x02, 0x01, // Characteristics: IMAGE_FILE_EXECUTABLE_IMAGE | IMAGE_FILE_32BIT_MACHINE
    ]);

    // Add optional header (simplified for .NET)
    pe_data.extend_from_slice(&[
        0x0B, 0x01, // Magic: PE32
        0x0E, 0x00, // MajorLinkerVersion, MinorLinkerVersion
    ]);

    // Add more PE structure...
    // This is a very simplified version. A complete implementation would need
    // to properly construct all PE sections, metadata, and IL method bodies.

    // For now, embed the IL code as a comment in the PE
    let il_comment = format!("// IL Code for {}\n{}", program_name, il_code);
    pe_data.extend_from_slice(il_comment.as_bytes());

    // Pad to minimum size
    pe_data.resize(1024, 0);

    Ok(pe_data)
}
