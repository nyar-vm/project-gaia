use crate::types::*;

/// High-level, intuitive view of PE file structure
/// Provides reorganized and simplified access to PE information
#[derive(Debug, Clone)]
pub struct PeView {
    /// Basic file information
    pub file_info: FileInfo,
    /// PE headers summary
    pub headers: HeaderSummary,
    /// Section overview
    pub sections: Vec<SectionView>,
    /// Import summary
    pub imports: ImportSummary,
    /// Export summary  
    pub exports: ExportSummary,
    /// Security and characteristics
    pub security: SecurityInfo,
}

/// Basic file information
#[derive(Debug, Clone)]
pub struct FileInfo {
    /// File size in bytes
    pub size: usize,
    /// Target architecture
    pub architecture: String,
    /// Application type (GUI, Console, etc.)
    pub application_type: String,
    /// Entry point address
    pub entry_point: String,
    /// Preferred load address
    pub image_base: String,
    /// Time stamp
    pub timestamp: String,
}

/// Simplified header information
#[derive(Debug, Clone)]
pub struct HeaderSummary {
    /// DOS header signature check
    pub dos_signature_valid: bool,
    /// PE header signature check
    pub pe_signature_valid: bool,
    /// Number of sections
    pub section_count: u16,
    /// Size of optional header
    pub optional_header_size: u16,
    /// File characteristics
    pub characteristics: Vec<String>,
}

/// Section view with simplified information
#[derive(Debug, Clone)]
pub struct SectionView {
    /// Section name
    pub name: String,
    /// Virtual address
    pub virtual_address: String,
    /// Virtual size
    pub virtual_size: String,
    /// Raw data size
    pub raw_size: String,
    /// Section characteristics
    pub characteristics: Vec<String>,
    /// Whether section contains code
    pub is_executable: bool,
    /// Whether section contains initialized data
    pub is_initialized_data: bool,
    /// Whether section contains uninitialized data
    pub is_uninitialized_data: bool,
}

/// Import summary
#[derive(Debug, Clone)]
pub struct ImportSummary {
    /// Number of imported DLLs
    pub dll_count: usize,
    /// Number of imported functions
    pub function_count: usize,
    /// List of imported DLLs
    pub dlls: Vec<String>,
}

/// Export summary
#[derive(Debug, Clone)]
pub struct ExportSummary {
    /// Number of exported functions
    pub function_count: usize,
    /// Export DLL name
    pub dll_name: Option<String>,
    /// List of exported function names
    pub functions: Vec<String>,
}

/// Security and characteristics information
#[derive(Debug, Clone)]
pub struct SecurityInfo {
    /// ASLR support
    pub aslr_enabled: bool,
    /// DEP/NX support
    pub dep_enabled: bool,
    /// Code integrity
    pub code_integrity: bool,
    /// High entropy VA
    pub high_entropy_va: bool,
    /// Dynamic base
    pub dynamic_base: bool,
}

impl PeView {
    /// Create a new PeView from a PeProgram
    pub fn from_program(program: &PeProgram) -> Self {
        let file_info = FileInfo {
            size: 0, // TODO: Calculate from program data
            architecture: match program.header.coff_header.machine {
                0x014c => "x86".to_string(),
                0x8664 => "x64".to_string(),
                _ => "Unknown".to_string(),
            },
            application_type: match program.header.optional_header.subsystem {
                SubsystemType::Console => "Console Application".to_string(),
                SubsystemType::Windows => "Windows Application".to_string(),
                SubsystemType::Native => "Native Application".to_string(),
                _ => "Unknown".to_string(),
            },
            entry_point: format!("0x{:08X}", program.header.optional_header.address_of_entry_point),
            image_base: format!("0x{:016X}", program.header.optional_header.image_base),
            timestamp: format!("{}", program.header.coff_header.time_date_stamp),
        };

        let headers = HeaderSummary {
            dos_signature_valid: program.header.dos_header.e_magic == 0x5A4D,
            pe_signature_valid: program.header.nt_header.signature == 0x00004550,
            section_count: program.header.coff_header.number_of_sections,
            optional_header_size: program.header.coff_header.size_of_optional_header,
            characteristics: Self::parse_file_characteristics(program.header.coff_header.characteristics),
        };

        let sections = program
            .sections
            .iter()
            .map(|section| SectionView {
                name: section.name.clone(),
                virtual_address: format!("0x{:08X}", section.virtual_address),
                virtual_size: format!("0x{:08X}", section.virtual_size),
                raw_size: format!("0x{:08X}", section.size_of_raw_data),
                characteristics: Self::parse_section_characteristics(section.characteristics),
                is_executable: (section.characteristics & 0x20000000) != 0,
                is_initialized_data: (section.characteristics & 0x40000000) != 0,
                is_uninitialized_data: (section.characteristics & 0x80000000) != 0,
            })
            .collect();

        // TODO: Implement import/export parsing
        let imports = ImportSummary { dll_count: 0, function_count: 0, dlls: Vec::new() };

        let exports = ExportSummary { function_count: 0, dll_name: None, functions: Vec::new() };

        let security = SecurityInfo {
            aslr_enabled: (program.header.optional_header.dll_characteristics & 0x0040) != 0,
            dep_enabled: (program.header.optional_header.dll_characteristics & 0x0100) != 0,
            code_integrity: (program.header.optional_header.dll_characteristics & 0x0080) != 0,
            high_entropy_va: (program.header.optional_header.dll_characteristics & 0x0020) != 0,
            dynamic_base: (program.header.optional_header.dll_characteristics & 0x0040) != 0,
        };

        PeView { file_info, headers, sections, imports, exports, security }
    }

    /// Parse file characteristics into human-readable strings
    fn parse_file_characteristics(characteristics: u16) -> Vec<String> {
        let mut result = Vec::new();

        if (characteristics & 0x0001) != 0 {
            result.push("Relocation info stripped".to_string());
        }
        if (characteristics & 0x0002) != 0 {
            result.push("Executable image".to_string());
        }
        if (characteristics & 0x0004) != 0 {
            result.push("Line numbers stripped".to_string());
        }
        if (characteristics & 0x0008) != 0 {
            result.push("Local symbols stripped".to_string());
        }
        if (characteristics & 0x0010) != 0 {
            result.push("Aggressively trim working set".to_string());
        }
        if (characteristics & 0x0020) != 0 {
            result.push("Large address aware".to_string());
        }
        if (characteristics & 0x0080) != 0 {
            result.push("Little endian".to_string());
        }
        if (characteristics & 0x0100) != 0 {
            result.push("32-bit machine".to_string());
        }
        if (characteristics & 0x0200) != 0 {
            result.push("Debug info stripped".to_string());
        }
        if (characteristics & 0x0400) != 0 {
            result.push("Removable run from swap".to_string());
        }
        if (characteristics & 0x0800) != 0 {
            result.push("Net run from swap".to_string());
        }
        if (characteristics & 0x1000) != 0 {
            result.push("System file".to_string());
        }
        if (characteristics & 0x2000) != 0 {
            result.push("DLL".to_string());
        }
        if (characteristics & 0x4000) != 0 {
            result.push("Up system only".to_string());
        }
        if (characteristics & 0x8000) != 0 {
            result.push("Big endian".to_string());
        }

        result
    }

    /// Parse section characteristics into human-readable strings
    fn parse_section_characteristics(characteristics: u32) -> Vec<String> {
        let mut result = Vec::new();

        if (characteristics & 0x00000020) != 0 {
            result.push("Contains code".to_string());
        }
        if (characteristics & 0x00000040) != 0 {
            result.push("Contains initialized data".to_string());
        }
        if (characteristics & 0x00000080) != 0 {
            result.push("Contains uninitialized data".to_string());
        }
        if (characteristics & 0x00000200) != 0 {
            result.push("Contains comments".to_string());
        }
        if (characteristics & 0x00000800) != 0 {
            result.push("Will not be paged".to_string());
        }
        if (characteristics & 0x02000000) != 0 {
            result.push("Can be discarded".to_string());
        }
        if (characteristics & 0x04000000) != 0 {
            result.push("Cannot be cached".to_string());
        }
        if (characteristics & 0x08000000) != 0 {
            result.push("Is not pageable".to_string());
        }
        if (characteristics & 0x10000000) != 0 {
            result.push("Can be shared".to_string());
        }
        if (characteristics & 0x20000000) != 0 {
            result.push("Is executable".to_string());
        }
        if (characteristics & 0x40000000) != 0 {
            result.push("Is readable".to_string());
        }
        if (characteristics & 0x80000000) != 0 {
            result.push("Is writable".to_string());
        }

        result
    }

    /// Get a summary string of the PE file
    pub fn get_summary(&self) -> String {
        format!(
            "PE File Summary:\n\
            Architecture: {}\n\
            Type: {}\n\
            Entry Point: {}\n\
            Image Base: {}\n\
            Sections: {}\n\
            Size: {} bytes",
            self.file_info.architecture,
            self.file_info.application_type,
            self.file_info.entry_point,
            self.file_info.image_base,
            self.headers.section_count,
            self.file_info.size
        )
    }

    /// Get detailed section information
    pub fn get_section_details(&self) -> String {
        let mut result = String::from("Sections:\n");

        for section in &self.sections {
            result.push_str(&format!(
                "  {}: VA={}, VSize={}, RawSize={}\n",
                section.name, section.virtual_address, section.virtual_size, section.raw_size
            ));
        }

        result
    }

    /// Get security features summary
    pub fn get_security_summary(&self) -> String {
        format!(
            "Security Features:\n\
            ASLR: {}\n\
            DEP/NX: {}\n\
            Code Integrity: {}\n\
            High Entropy VA: {}\n\
            Dynamic Base: {}",
            if self.security.aslr_enabled { "Enabled" } else { "Disabled" },
            if self.security.dep_enabled { "Enabled" } else { "Disabled" },
            if self.security.code_integrity { "Enabled" } else { "Disabled" },
            if self.security.high_entropy_va { "Enabled" } else { "Disabled" },
            if self.security.dynamic_base { "Enabled" } else { "Disabled" }
        )
    }
}

// ... existing code ...
