mod headers;
mod metadata;
mod program;
mod utils;

use crate::{
    formats::dll::DllReadConfig,
    program::{ClrHeader, ClrProgram, ClrVersion, DotNetAssemblyInfo, MetadataHeader, StreamHeader},
};
use gaia_types::{GaiaError, SourceLocation};
use pe_assembler::{
    helpers::PeReader,
    types::{PeHeader, PeProgram, SectionHeader},
};
use std::io::{Read, Seek};

#[derive(Debug)]
pub struct DllReader<'config, R> {
    /// Configuration options
    options: &'config DllReadConfig,
    reader: pe_assembler::formats::dll::reader::DllReader<R>,
    /// Parsed CLR header information
    clr_header: Option<ClrHeader>,
    /// Parsed metadata header information
    metadata_header: Option<MetadataHeader>,
    /// Metadata stream header list (lazy loaded)
    stream_headers: Option<Vec<StreamHeader>>,
    /// Extracted basic assembly information (lazy loaded)
    assembly_info: Option<DotNetAssemblyInfo>,
    /// Fully parsed CLR program (lazy loaded)
    clr_program: Option<ClrProgram>,
}

impl<'config, R: Read + Seek> PeReader<R> for DllReader<'config, R> {
    fn get_viewer(&mut self) -> &mut R {
        self.reader.get_viewer()
    }

    fn add_diagnostics(&mut self, error: impl Into<GaiaError>) {
        self.reader.add_diagnostics(error)
    }

    fn get_section_headers(&mut self) -> Result<&[SectionHeader], GaiaError> {
        self.reader.get_section_headers()
    }

    fn get_pe_header(&mut self) -> Result<&PeHeader, GaiaError> {
        self.reader.get_pe_header()
    }

    fn get_program(&mut self) -> Result<&PeProgram, GaiaError> {
        self.reader.get_program()
    }
}

impl<'config, R> DllReader<'config, R> {
    /// Constructs a .NET reader (DLL) using a generic PE reader.
    ///
    /// Note: This is a lazy constructor and will not immediately execute the parsing workflow.
    pub fn new(reader: R, options: &'config DllReadConfig) -> Self {
        Self {
            reader: pe_assembler::formats::dll::reader::DllReader::new(reader),
            clr_header: None,
            metadata_header: None,
            stream_headers: None,
            assembly_info: None,
            clr_program: None,
            options,
        }
    }
}

impl<'config, R> DllReader<'config, R>
where
    R: Read + Seek,
{
    /// Lazily reads basic assembly information.
    ///
    /// Only reads basic identification information of the assembly without parsing the full type system.
    /// Suitable for scenarios where you need to quickly get assembly name, version, etc.
    ///
    /// # Returns
    /// * `Ok(DotNetAssemblyInfo)` - Basic assembly information.
    /// * `Err(GaiaError)` - Error during the reading process.
    pub fn get_assembly_info(&mut self) -> Result<DotNetAssemblyInfo, GaiaError> {
        if self.assembly_info.is_none() {
            self.ensure_assembly_info_parsed()?;
        }

        self.assembly_info
            .as_ref()
            .cloned()
            .ok_or_else(|| GaiaError::syntax_error("程序集信息未解析".to_string(), SourceLocation::default()))
    }

    /// Parses as a full CLR program.
    ///
    /// Parses the entire .NET assembly, including all types, methods, fields, and other information.
    /// This is a heavyweight operation that consumes significant memory and time.
    ///
    /// # Returns
    /// * `Ok(ClrProgram)` - Full representation of the CLR program.
    /// * `Err(GaiaError)` - Error during the parsing process.
    pub fn to_clr_program(&mut self) -> Result<ClrProgram, GaiaError> {
        if let Some(ref program) = self.clr_program {
            return Ok(program.clone());
        }

        // 执行完整解析
        let program = self.parse_full_program()?;
        self.clr_program = Some(program.clone());
        Ok(program)
    }

    /// Validates the integrity of the assembly.
    ///
    /// Checks if the parsed .NET assembly contains all required components:
    /// - CLR Header: Contains runtime information.
    /// - Metadata Header: Describes the type system.
    /// - Metadata Streams: Contains the actual metadata.
    ///
    /// # Returns
    /// * `Ok(Vec<String>)` - List of warning messages; an empty list indicates validation success.
    /// * `Err(GaiaError)` - Error during the validation process.
    pub fn validate_assembly(&mut self) -> Result<Vec<String>, GaiaError> {
        let mut warnings = Vec::new();

        // 确保基本信息已解析
        self.ensure_assembly_info_parsed()?;

        // 验证 CLR 头 - 必需的核心头信息
        if self.clr_header.is_none() {
            warnings.push("缺少 CLR 头".to_string());
        }

        // 验证元数据头 - 描述类型系统的元数据
        if self.metadata_header.is_none() {
            warnings.push("缺少元数据头".to_string());
        }

        // 验证流头 - 包含实际的元数据流
        if self.stream_headers.as_ref().map_or(true, |h| h.is_empty()) {
            warnings.push("缺少元数据流".to_string());
        }

        Ok(warnings)
    }

    /// Gets a summary of the assembly information.
    ///
    /// Returns the basic information of the assembly in a friendly format, suitable for display or logging.
    /// If the assembly information is unavailable, it returns a corresponding error message.
    ///
    /// # Returns
    /// * `String` - Formatted assembly information containing name, version, culture, public key token, and runtime version.
    pub fn get_assembly_summary(&mut self) -> String {
        match self.get_assembly_info() {
            Ok(info) => {
                format!(
                    "Assembly: {}\nVersion: {}\nCulture: {}\nPublic Key Token: {}\nRuntime Version: {}",
                    info.name,
                    info.version,
                    info.culture.as_deref().unwrap_or("neutral"),
                    info.public_key_token.as_deref().unwrap_or("null"),
                    info.runtime_version.as_deref().unwrap_or("unknown")
                )
            }
            Err(_) => "Unable to get assembly information".to_string(),
        }
    }

    /// Ensures that the assembly information is parsed (lazy loading helper method).
    fn ensure_assembly_info_parsed(&mut self) -> Result<(), GaiaError> {
        if self.assembly_info.is_some() {
            return Ok(());
        }

        // Execute the parsing workflow on demand
        self.parse_clr_header_lazy()?;
        self.parse_metadata_lazy()?;
        self.extract_assembly_info()?;

        Ok(())
    }

    /// Parses the CLR header.
    fn parse_clr_header_lazy(&mut self) -> Result<(), GaiaError> {
        eprintln!("Starting to parse CLR header");
        self.clr_header = self.find_and_read_clr_header()?;
        eprintln!("find_and_read_clr_header returned: {:?}", self.clr_header.is_some());
        if self.clr_header.is_none() {
            eprintln!("CLR header is empty, returning error");
            return Err(GaiaError::syntax_error("Missing CLR header", SourceLocation::default()));
        }
        eprintln!("CLR header parsed successfully");
        Ok(())
    }

    /// Parses the metadata.
    fn parse_metadata_lazy(&mut self) -> Result<(), GaiaError> {
        if let Some(ref clr_header) = self.clr_header {
            // Convert metadata RVA to file offset
            let pe_program = self.reader.get_program()?.clone();
            let metadata_offset = utils::rva_to_file_offset(&pe_program, clr_header.metadata_rva)?;
            // Read metadata header
            self.metadata_header = Some(self.read_metadata_header(metadata_offset)?);
            // Read stream header information
            self.stream_headers = Some(self.read_stream_headers(metadata_offset)?);
        }

        Ok(())
    }
}
