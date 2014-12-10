use crate::{
    formats::dll::DllReadConfig,
    program::{
        ClrAccessFlags, ClrHeader, ClrMethod, ClrProgram, ClrType, ClrTypeReference, ClrVersion, DotNetAssemblyInfo,
        MetadataHeader, StreamHeader,
    },
};
use byteorder::{LittleEndian, ReadBytesExt};
use gaia_types::{GaiaDiagnostics, GaiaError, SourceLocation};
use pe_assembler::{
    helpers::PeReader,
    types::{PeHeader, PeProgram, SectionHeader},
};
use std::io::{Read, Seek, SeekFrom};
use url::Url;

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
    /// Reads a .NET assembly from a file.
    ///
    /// This method reads and parses a .NET assembly file with the following steps:
    /// 1. Read the entire file into memory.
    /// 2. Create a PE view to access the PE structure.
    /// 3. Create a reader instance.
    /// 4. Execute the parsing workflow.
    ///
    /// # Arguments
    /// * `file_path` - Path to the .NET assembly file.
    ///
    /// # Returns
    /// * `Ok(DotNetReader)` - Successfully parsed reader.
    /// * `Err(GaiaError)` - Error during reading or parsing.
    // Convenient constructors are provided in a specialized impl.
    /// Checks if a file is a .NET assembly.
    ///
    /// A fast check method that doesn't require full parsing, only checks the PE data directory:
    /// - Read the PE file and create a view.
    /// - Check if the 15th data directory (index 14) is a CLR runtime header.
    /// - If the directory exists and is valid, it's a .NET assembly.
    ///
    /// # Arguments
    /// * `file_path` - Path to the PE file to check.
    ///
    /// # Returns
    /// * `Ok(true)` - Is a .NET assembly.
    /// * `Ok(false)` - Is not a .NET assembly.
    /// * `Err(GaiaError)` - Error during the check process.
    // Convenient checks are provided in a specialized impl.

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
        self.parse_clr_header()?;
        self.parse_metadata()?;
        self.extract_assembly_info()?;

        Ok(())
    }

    /// Parses the CLR header.
    ///
    /// This is the first step of the parsing process, responsible for locating and reading the CLR header information.
    /// The CLR header contains core information required by the .NET runtime, such as metadata location, runtime version, etc.
    ///
    /// # Returns
    /// * `Ok(())` - Successfully parsed.
    /// * `Err(GaiaError)` - Error during the parsing process.
    fn parse_clr_header(&mut self) -> Result<(), GaiaError> {
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
    ///
    /// This is the second step of the parsing process, executed after the CLR header is successfully parsed:
    /// 1. Use the metadata_rva from the CLR header to locate the metadata position.
    /// 2. Read the metadata header to get basic information about the metadata.
    /// 3. Read all stream headers to understand the organization of the metadata.
    ///
    /// # Returns
    /// * `Ok(())` - Successfully parsed (does not error even if there is no CLR header).
    /// * `Err(GaiaError)` - Error during the parsing process.
    fn parse_metadata(&mut self) -> Result<(), GaiaError> {
        if let Some(ref clr_header) = self.clr_header {
            // Convert metadata RVA to file offset
            let metadata_offset = self.rva_to_file_offset(clr_header.metadata_rva)?;
            // Read metadata header
            self.metadata_header = Some(self.read_metadata_header(metadata_offset)?);
            // Read stream header information
            self.stream_headers = Some(self.read_stream_headers(metadata_offset)?);
        }

        Ok(())
    }

    /// Extracts assembly information.
    ///
    /// This is the third step of the parsing process, responsible for extracting assembly-level information from the metadata.
    /// This information includes assembly name, version, culture, public key token, etc., used for identification and version control.
    ///
    /// # Returns
    /// * `Ok(())` - Successfully extracted.
    /// * `Err(GaiaError)` - Error during the extraction process.
    fn extract_assembly_info(&mut self) -> Result<(), GaiaError> {
        // Depends on parsed CLR header and metadata stream headers
        let clr_header = match &self.clr_header {
            Some(h) => *h,
            None => return Ok(()),
        };
        let metadata_offset = self.rva_to_file_offset(clr_header.metadata_rva)?;

        // Find #~ and #Strings streams
        let mut tables_stream: Option<StreamHeader> = None;
        let mut strings_stream: Option<StreamHeader> = None;
        if let Some(ref stream_headers) = self.stream_headers {
            for sh in stream_headers {
                match sh.name.as_str() {
                    "#~" => tables_stream = Some(sh.clone()),
                    "#Strings" => strings_stream = Some(sh.clone()),
                    _ => {}
                }
            }
        }
        if tables_stream.is_none() || strings_stream.is_none() {
            return Ok(());
        }
        let tables_stream = tables_stream.unwrap();
        let strings_stream = strings_stream.unwrap();

        let tables_start = metadata_offset + tables_stream.offset;
        let strings_start = metadata_offset + strings_stream.offset;

        // Read compressed metadata table header
        let mut cur = self.reader.get_viewer();
        cur.seek(SeekFrom::Start(tables_start as u64))
            .map_err(|e| GaiaError::io_error(e, Url::parse("memory://table").unwrap()))?;

        let _reserved =
            cur.read_u32::<LittleEndian>().map_err(|e| GaiaError::io_error(e, Url::parse("memory://tables_hdr").unwrap()))?;
        let _major = cur.read_u8().map_err(|e| GaiaError::io_error(e, Url::parse("memory://tables_hdr").unwrap()))?;
        let _minor = cur.read_u8().map_err(|e| GaiaError::io_error(e, Url::parse("memory://tables_hdr").unwrap()))?;
        let heap_sizes = cur.read_u8().map_err(|e| GaiaError::io_error(e, Url::parse("memory://tables_hdr").unwrap()))?;
        let _reserved2 = cur.read_u8().map_err(|e| GaiaError::io_error(e, Url::parse("memory://tables_hdr").unwrap()))?;
        let valid_mask =
            cur.read_u64::<LittleEndian>().map_err(|e| GaiaError::io_error(e, Url::parse("memory://tables_hdr").unwrap()))?;
        let _sorted_mask =
            cur.read_u64::<LittleEndian>().map_err(|e| GaiaError::io_error(e, Url::parse("memory://tables_hdr").unwrap()))?;

        // Heap index sizes
        let str_idx_sz = if (heap_sizes & 0x01) != 0 { 4 } else { 2 };
        let guid_idx_sz = if (heap_sizes & 0x02) != 0 { 4 } else { 2 };
        let blob_idx_sz = if (heap_sizes & 0x04) != 0 { 4 } else { 2 };
        let strings_size = strings_stream.size;

        eprintln!("Heap size flags: 0x{:02x}", heap_sizes);
        eprintln!("String index size: {}, GUID index size: {}, Blob index size: {}", str_idx_sz, guid_idx_sz, blob_idx_sz);
        eprintln!("String stream size: {}", strings_size);
        eprintln!("Valid table mask: 0x{:016x}", valid_mask);

        // Read row counts
        let mut row_counts: [u32; 64] = [0; 64];
        for tid in 0..64u8 {
            if (valid_mask >> tid) & 1 == 1 {
                row_counts[tid as usize] = cur
                    .read_u32::<LittleEndian>()
                    .map_err(|e| GaiaError::io_error(e, Url::parse("memory://tables_rows").unwrap()))?;
                eprintln!("Table 0x{:02x} row count: {}", tid, row_counts[tid as usize]);
            }
        }

        // Calculate relevant coded index sizes
        fn coded_size(rows: &[u32; 64], tags: &[u8]) -> u32 {
            let max_rows = tags.iter().map(|&t| rows[t as usize]).max().unwrap_or(0);
            let tag_bits = (tags.len() as f32).log2().ceil() as u32;
            if (max_rows << tag_bits) < (1 << 16) {
                2
            }
            else {
                4
            }
        }
        let type_def_or_ref_sz = coded_size(&row_counts, &[0x02, 0x01, 0x18]);
        let resolution_scope_sz = coded_size(&row_counts, &[0x00, 0x01, 0x17, 0x23]);

        // Common table row sizes
        let module_row_size = 2 + str_idx_sz + guid_idx_sz + guid_idx_sz + guid_idx_sz;
        let type_def_row_size = 4
            + str_idx_sz
            + str_idx_sz
            + type_def_or_ref_sz
            + (if row_counts[0x04] < (1 << 16) { 2 } else { 4 })
            + (if row_counts[0x06] < (1 << 16) { 2 } else { 4 });
        let methoddef_row_size = 4 + 2 + 2 + str_idx_sz + blob_idx_sz + (if row_counts[0x07] < (1 << 16) { 2 } else { 4 });
        let typeref_row_size = resolution_scope_sz + str_idx_sz + str_idx_sz;
        let assembly_row_size = 4 + 2 + 2 + 2 + 2 + 4 + blob_idx_sz + str_idx_sz + str_idx_sz;

        // Data area starting position
        let tables_data_start =
            cur.stream_position().map_err(|e| GaiaError::io_error(e, Url::parse("memory://tables_data").unwrap()))? as u32;
        eprintln!("Table data area start: 0x{:x}", tables_data_start);
        // Table start offset mapping
        let mut table_start: [Option<u32>; 64] = [None; 64];
        let mut table_row_size: [u32; 64] = [0; 64];
        let mut running = tables_data_start;
        for tid in 0..64u8 {
            if (valid_mask >> tid) & 1 == 1 {
                let rows = row_counts[tid as usize];
                let row_size = match tid {
                    0x00 => module_row_size,
                    0x01 => typeref_row_size,
                    0x02 => type_def_row_size,
                    0x06 => methoddef_row_size,
                    0x20 => assembly_row_size, // Assembly table is 0x20 per ECMA-335
                    _ => 0,
                } as u32;
                table_start[tid as usize] = Some(running);
                table_row_size[tid as usize] = row_size;
                eprintln!(
                    "Table 0x{:02x}: start=0x{:x}, rows={}, row_size={}, total_size={}",
                    tid,
                    running,
                    rows,
                    row_size,
                    rows * row_size
                );
                running += rows * row_size;
            }
        }

        // Parse assembly name and version
        let mut name = String::from("Unknown");
        let mut version = ClrVersion { major: 0, minor: 0, build: 0, revision: 0 };

        // First try reading from Assembly table (0x20)
        if let Some(asm_start) = table_start[0x20] {
            if row_counts[0x20] > 0 {
                let mut c = self.reader.get_viewer();
                c.seek(SeekFrom::Start(asm_start as u64))
                    .map_err(|e| GaiaError::io_error(e, Url::parse("memory://asm").unwrap()))?;

                // Read Assembly table first row - according to ECMA-335 specification
                let hash_alg =
                    c.read_u32::<LittleEndian>().map_err(|e| GaiaError::io_error(e, Url::parse("memory://asm").unwrap()))?;
                version.major =
                    c.read_u16::<LittleEndian>().map_err(|e| GaiaError::io_error(e, Url::parse("memory://asm").unwrap()))?;
                version.minor =
                    c.read_u16::<LittleEndian>().map_err(|e| GaiaError::io_error(e, Url::parse("memory://asm").unwrap()))?;
                version.build =
                    c.read_u16::<LittleEndian>().map_err(|e| GaiaError::io_error(e, Url::parse("memory://asm").unwrap()))?;
                version.revision =
                    c.read_u16::<LittleEndian>().map_err(|e| GaiaError::io_error(e, Url::parse("memory://asm").unwrap()))?;
                let flags =
                    c.read_u32::<LittleEndian>().map_err(|e| GaiaError::io_error(e, Url::parse("memory://asm").unwrap()))?;
                let pk_idx = read_heap_index(&mut c, blob_idx_sz)?;
                let name_idx = read_heap_index(&mut c, str_idx_sz)?;
                let culture_idx = read_heap_index(&mut c, str_idx_sz)?;

                // Validate if read data is reasonable
                if hash_alg > 0x10000 || version.major > 100 || name_idx > strings_size {
                    eprintln!("Warning: Assembly table data abnormal, table offset calculation might be wrong");
                    eprintln!(
                        "HashAlg: 0x{:x}, Version: {}.{}.{}.{}, NameIdx: {}",
                        hash_alg, version.major, version.minor, version.build, version.revision, name_idx
                    );
                }
                else {
                    let n = self.read_string_from_strings_heap(strings_start, strings_size, name_idx)?;
                    if !n.is_empty() {
                        name = n;
                    }
                }
            }
        }
        else if let Some(mod_start) = table_start[0x00] {
            // Module table
            if row_counts[0x00] > 0 {
                let mut c = self.reader.get_viewer();
                c.seek(SeekFrom::Start(mod_start as u64))
                    .map_err(|e| GaiaError::io_error(e, Url::parse("memory://mod").unwrap()))?;
                let _generation =
                    c.read_u16::<LittleEndian>().map_err(|e| GaiaError::io_error(e, Url::parse("memory://mod").unwrap()))?;
                let name_idx = read_heap_index(&mut c, str_idx_sz)?;
                let _mvid_idx = read_heap_index(&mut c, guid_idx_sz)?;
                let _enc_id_idx = read_heap_index(&mut c, guid_idx_sz)?;
                let _enc_base_id_idx = read_heap_index(&mut c, guid_idx_sz)?;
                let n = self.read_string_from_strings_heap(strings_start, strings_size, name_idx)?;
                if !n.is_empty() {
                    name = n;
                }
            }
        }

        // Runtime version string
        let runtime_version = self.metadata_header.as_ref().map(|h| h.version_string.clone());

        // Save information
        self.assembly_info = Some(DotNetAssemblyInfo {
            name,
            version: format!("{}.{}.{}.{}", version.major, version.minor, version.build, version.revision),
            culture: None,
            public_key_token: None,
            runtime_version,
        });

        Ok(())
    }

    /// Parses the full CLR program.
    ///
    /// Executes full assembly parsing, including all types, methods, fields, and other information.
    /// This is a heavyweight operation that parses the entire metadata table structure.
    ///
    /// # Returns
    /// * `Ok(ClrProgram)` - Full representation of the CLR program.
    /// * `Err(GaiaError)` - Error during the parsing process.
    fn parse_full_program(&mut self) -> Result<ClrProgram, GaiaError> {
        // Defensive check: requires parsed CLR header and metadata header
        let metadata_rva = self
            .clr_header
            .as_ref()
            .ok_or_else(|| GaiaError::syntax_error("Missing CLR header".to_string(), SourceLocation::default()))?
            .metadata_rva;
        let _version_string = self
            .metadata_header
            .as_ref()
            .ok_or_else(|| GaiaError::syntax_error("Missing metadata header".to_string(), SourceLocation::default()))?
            .version_string
            .clone();

        // Calculate metadata starting file offset
        let metadata_base = self.rva_to_file_offset(metadata_rva)?;

        // Find key streams: #~ (or #-) and #Strings
        let mut tables_stream: Option<StreamHeader> = None;
        let mut strings_stream: Option<StreamHeader> = None;
        if let Some(ref stream_headers) = self.stream_headers {
            for sh in stream_headers {
                match sh.name.as_str() {
                    "#~" | "#-" => tables_stream = Some(sh.clone()),
                    "#Strings" => strings_stream = Some(sh.clone()),
                    _ => {}
                }
            }
        }

        let tables_stream = tables_stream
            .ok_or_else(|| GaiaError::syntax_error("Missing metadata table stream (#~/#-)".to_string(), SourceLocation::default()))?;
        let strings_stream = strings_stream
            .ok_or_else(|| GaiaError::syntax_error("Missing string stream (#Strings)".to_string(), SourceLocation::default()))?;

        // Convenience: treat file as a cursor
        let mut cur = self.reader.get_viewer();
        // Absolute file offsets for the start of the table stream and string stream
        let tables_start = metadata_base + tables_stream.offset;
        let strings_start = metadata_base + strings_stream.offset;

        // Read the table header (compressed metadata format, ECMA-335 II.24.2.6)
        cur.seek(SeekFrom::Start(tables_start as u64))
            .map_err(|e| GaiaError::io_error(e, Url::parse("memory://tables").unwrap()))?;
        let _reserved =
            cur.read_u32::<LittleEndian>().map_err(|e| GaiaError::io_error(e, Url::parse("memory://tables").unwrap()))?;
        let _major = cur.read_u8().map_err(|e| GaiaError::io_error(e, Url::parse("memory://tables").unwrap()))?;
        let _minor = cur.read_u8().map_err(|e| GaiaError::io_error(e, Url::parse("memory://tables").unwrap()))?;
        let heap_sizes = cur.read_u8().map_err(|e| GaiaError::io_error(e, Url::parse("memory://tables").unwrap()))?;
        let _reserved2 = cur.read_u8().map_err(|e| GaiaError::io_error(e, Url::parse("memory://tables").unwrap()))?;
        let valid_mask =
            cur.read_u64::<LittleEndian>().map_err(|e| GaiaError::io_error(e, Url::parse("memory://tables").unwrap()))?;
        let _sorted_mask =
            cur.read_u64::<LittleEndian>().map_err(|e| GaiaError::io_error(e, Url::parse("memory://tables").unwrap()))?;

        // Read the number of rows for existing tables
        let mut row_counts: [u32; 64] = [0; 64];
        for tid in 0..64u8 {
            if (valid_mask >> tid) & 1 == 1 {
                let cnt = cur
                    .read_u32::<LittleEndian>()
                    .map_err(|e| GaiaError::io_error(e, Url::parse("memory://tables").unwrap()))?;
                row_counts[tid as usize] = cnt;
            }
        }

        // Calculate heap index sizes
        let str_idx_sz = if (heap_sizes & 0x01) != 0 { 4 } else { 2 };
        let guid_idx_sz = if (heap_sizes & 0x02) != 0 { 4 } else { 2 };
        let blob_idx_sz = if (heap_sizes & 0x04) != 0 { 4 } else { 2 };
        let _ = guid_idx_sz; // Currently unused, avoid warning

        // Data area starting position (current cursor position)
        let tables_data_start =
            cur.stream_position().map_err(|e| GaiaError::io_error(e, Url::parse("memory://tables_data").unwrap()))? as u32;

        // Calculate simple index size (to a specified table)
        let mut simple_index_size = |table_id: u8| -> u32 {
            let rows = row_counts[table_id as usize];
            if rows < (1 << 16) {
                2
            }
            else {
                4
            }
        };

        // Calculate MethodDef row size
        let param_index_sz = simple_index_size(0x07); // Param
        let methoddef_row_size = 4 /*RVA*/ + 2 /*ImplFlags*/ + 2 /*Flags*/ + str_idx_sz + blob_idx_sz + param_index_sz;

        // Calculate row sizes of some preceding tables to accumulate offset to MethodDef
        let field_row_size = 2 /*Flags*/ + str_idx_sz /*Name*/ + blob_idx_sz /*Signature*/;
        let fieldptr_row_size = simple_index_size(0x04);
        let methodptr_row_size = simple_index_size(0x06);
        // TypeRef row size: ResolutionScope (coded index) + Name (String) + Namespace (String)
        // ResolutionScope can point to: Module(0x00), ModuleRef(0x1A/0x17), AssemblyRef(0x20), TypeRef(0x01)
        let rs_candidates = [0x00u8, 0x17u8, 0x20u8, 0x01u8];
        let mut max_rs_rows = 0u32;
        for &t in &rs_candidates {
            max_rs_rows = max_rs_rows.max(row_counts[t as usize]);
        }
        let rs_tag_bits = 2u32;
        let resolution_scope_sz = if max_rs_rows < (1 << (16 - rs_tag_bits)) { 2 } else { 4 };
        let typeref_row_size = resolution_scope_sz + str_idx_sz + str_idx_sz;
        // TypeDef row size: Flags(u32) + Name(String) + Namespace(String) + Extends(TypeDefOrRef) + FieldList(simple index to Field) + MethodList(simple index to MethodDef)
        // TypeDefOrRef coded index candidates: TypeDef(0x02), TypeRef(0x01), TypeSpec(0x1B/0x18)
        let tdr_candidates = [0x02u8, 0x01u8, 0x18u8];
        let mut max_tdr_rows = 0u32;
        for &t in &tdr_candidates {
            max_tdr_rows = max_tdr_rows.max(row_counts[t as usize]);
        }
        let tdr_tag_bits = 2u32;
        let type_def_or_ref_sz = if max_tdr_rows < (1 << (16 - tdr_tag_bits)) { 2 } else { 4 };
        let type_def_row_size =
            4 /*Flags*/ + str_idx_sz + str_idx_sz + type_def_or_ref_sz + simple_index_size(0x04) + simple_index_size(0x06);
        // Module row size: Generation(u16) + Name(String) + Mvid(Guid) + EncId(Guid) + EncBaseId(Guid)
        let module_row_size = 2 + str_idx_sz + guid_idx_sz + guid_idx_sz + guid_idx_sz;

        // Calculate start offsets and row size mapping for common tables
        let mut table_start: [Option<u32>; 64] = [None; 64];
        let mut table_row_size: [u32; 64] = [0; 64];
        let mut running = tables_data_start;
        for tid in 0..64u8 {
            if (valid_mask >> tid) & 1 == 1 {
                let rows = row_counts[tid as usize];
                let row_size = match tid {
                    0x00 => module_row_size,
                    0x01 => typeref_row_size,
                    0x02 => type_def_row_size,
                    0x03 => fieldptr_row_size,
                    0x04 => field_row_size,
                    0x05 => methodptr_row_size,
                    0x06 => methoddef_row_size,
                    0x07 => 2 /*Flags*/ + str_idx_sz + blob_idx_sz, // Param
                    0x08 => simple_index_size(0x02) + simple_index_size(0x01), // InterfaceImpl
                    0x09 => resolution_scope_sz + str_idx_sz + blob_idx_sz, // MemberRef
                    0x0A => 2 /*Type*/ + blob_idx_sz,               // Constant
                    0x0B => simple_index_size(0x02) + simple_index_size(0x0A) + simple_index_size(0x0C), /* CustomAttribute (rough) */
                    0x0C => simple_index_size(0x04) + simple_index_size(0x07),                           // FieldMarshal
                    0x0D => 2 + blob_idx_sz,                                                             // DeclSecurity
                    0x0E => 2 + 4 + 4,                                                                   // ClassLayout
                    0x0F => simple_index_size(0x04) + 4,                                                 // FieldLayout
                    0x10 => blob_idx_sz,                                                                 // StandAloneSig
                    0x11 => simple_index_size(0x02) + simple_index_size(0x12),                           // EventMap
                    0x12 => 2 + str_idx_sz + simple_index_size(0x10),                                    // Event
                    0x13 => simple_index_size(0x02) + simple_index_size(0x14),                           // PropertyMap
                    0x14 => 2 + str_idx_sz + blob_idx_sz,                                                // Property
                    0x15 => 2 + simple_index_size(0x06) + simple_index_size(0x14),                       // MethodSemantics
                    0x16 => simple_index_size(0x02) + simple_index_size(0x06) + simple_index_size(0x01), // MethodImpl
                    0x20 => 4 + 2 + 2 + 2 + 2 + 4 + blob_idx_sz + str_idx_sz + str_idx_sz,               // Assembly
                    0x21 => 4 + 4,                                                                       // AssemblyProcessor
                    0x22 => 4 + 4 + 4,                                                                   // AssemblyOS
                    0x23 => 2 + 2 + 2 + 2 + 4 + blob_idx_sz + str_idx_sz + str_idx_sz + blob_idx_sz,     // AssemblyRef
                    _ => 0,
                } as u32;
                table_start[tid as usize] = Some(running);
                table_row_size[tid as usize] = row_size;
                running += rows * row_size;
            }
        }
        let methoddef_offset = table_start[0x06].unwrap_or(tables_data_start);

        // Construct program object
        let mut program = ClrProgram::new("UnknownAssembly");
        program.version = ClrVersion { major: 1, minor: 0, build: 0, revision: 0 };
        program.access_flags =
            ClrAccessFlags { is_public: true, is_private: false, is_security_transparent: false, is_retargetable: false };

        // Try to fill name and version from Assembly table, otherwise use Module name
        if let Some(asm_start) = table_start[0x20] {
            // Assembly table exists (0x20)
            let asm_rows = row_counts[0x20];
            if asm_rows > 0 {
                let asm0 = asm_start; // First row offset
                let mut c2 = self.reader.get_viewer();
                c2.seek(SeekFrom::Start(asm0 as u64))
                    .map_err(|e| GaiaError::io_error(e, Url::parse("memory://asm").unwrap()))?;
                let _hash_alg =
                    c2.read_u32::<LittleEndian>().map_err(|e| GaiaError::io_error(e, Url::parse("memory://asm").unwrap()))?;
                let ver_major =
                    c2.read_u16::<LittleEndian>().map_err(|e| GaiaError::io_error(e, Url::parse("memory://asm").unwrap()))?;
                let ver_minor =
                    c2.read_u16::<LittleEndian>().map_err(|e| GaiaError::io_error(e, Url::parse("memory://asm").unwrap()))?;
                let ver_build =
                    c2.read_u16::<LittleEndian>().map_err(|e| GaiaError::io_error(e, Url::parse("memory://asm").unwrap()))?;
                let ver_rev =
                    c2.read_u16::<LittleEndian>().map_err(|e| GaiaError::io_error(e, Url::parse("memory://asm").unwrap()))?;
                let _flags =
                    c2.read_u32::<LittleEndian>().map_err(|e| GaiaError::io_error(e, Url::parse("memory://asm").unwrap()))?;
                let _pk_idx = read_heap_index(&mut c2, blob_idx_sz)?;
                let name_idx = read_heap_index(&mut c2, str_idx_sz)?;
                let culture_idx = read_heap_index(&mut c2, str_idx_sz)?;
                let _hash_idx = read_heap_index(&mut c2, blob_idx_sz)?;

                let name = self.read_string_from_strings_heap(strings_start, strings_stream.size, name_idx)?;
                let culture = if culture_idx != 0 {
                    Some(self.read_string_from_strings_heap(strings_start, strings_stream.size, culture_idx)?)
                }
                else {
                    None
                };

                if !name.is_empty() {
                    program.name = name;
                }
                program.version = ClrVersion { major: ver_major, minor: ver_minor, build: ver_build, revision: ver_rev };
            }
        }
        else if let Some(module_start) = table_start[0x00] {
            // Use name when Module table exists
            let mut cm = self.reader.get_viewer();
            cm.seek(SeekFrom::Start(module_start as u64))
                .map_err(|e| GaiaError::io_error(e, Url::parse("memory://module").unwrap()))?;
            let _generation =
                cm.read_u16::<LittleEndian>().map_err(|e| GaiaError::io_error(e, Url::parse("memory://module").unwrap()))?;
            let name_idx = read_heap_index(&mut cm, str_idx_sz)?;
            let _mvid_idx = read_heap_index(&mut cm, guid_idx_sz)?;
            let _encid = read_heap_index(&mut cm, guid_idx_sz)?;
            let _encbase = read_heap_index(&mut cm, guid_idx_sz)?;
            let mod_name = self.read_string_from_strings_heap(strings_start, strings_stream.size, name_idx)?;
            if !mod_name.is_empty() {
                program.name = mod_name;
            }
        }

        // Read TypeDef table
        if let Some(typedef_start) = table_start[0x02] {
            for i in 0..row_counts[0x02] {
                let mut ct = self.reader.get_viewer();
                ct.seek(SeekFrom::Start((typedef_start + i * type_def_row_size) as u64))
                    .map_err(|e| GaiaError::io_error(e, Url::parse("memory://typedef").unwrap()))?;
                let flags = ct
                    .read_u32::<LittleEndian>()
                    .map_err(|e| GaiaError::io_error(e, Url::parse("memory://typedef").unwrap()))?;
                let name_idx = read_heap_index(&mut ct, str_idx_sz)?;
                let ns_idx = read_heap_index(&mut ct, str_idx_sz)?;
                let _extends_idx = read_type_def_or_ref_index(&mut ct, type_def_or_ref_sz)?;
                let _field_list_idx = read_heap_index(&mut ct, (if row_counts[0x04] < (1 << 16) { 2 } else { 4 }))?;
                let _method_list_idx = read_heap_index(&mut ct, (if row_counts[0x06] < (1 << 16) { 2 } else { 4 }))?;

                let type_name = self.read_string_from_strings_heap(strings_start, strings_stream.size, name_idx)?;
                let namespace = if ns_idx != 0 {
                    Some(self.read_string_from_strings_heap(strings_start, strings_stream.size, ns_idx)?)
                }
                else {
                    None
                };

                if !type_name.is_empty() {
                    let mdef = ClrMethod::new(
                        "DefaultMethod".to_string(),
                        ClrTypeReference {
                            name: "Void".to_string(),
                            namespace: Some("System".to_string()),
                            assembly: Some("mscorlib".to_string()),
                            is_value_type: true,
                            is_reference_type: false,
                            generic_parameters: Vec::new(),
                        },
                    );
                    let mut clr_type = ClrType::new(type_name, namespace);
                    clr_type.access_flags.is_public = true;
                    clr_type.add_method(mdef);
                }
            }
        }

        // Read MethodDef table
        if let Some(methoddef_start) = table_start[0x06] {
            for i in 0..row_counts[0x06] {
                let mut c3 = self.reader.get_viewer();
                c3.seek(SeekFrom::Start((methoddef_start + i * methoddef_row_size) as u64))
                    .map_err(|e| GaiaError::io_error(e, Url::parse("memory://methoddef").unwrap()))?;
                let _rva = c3
                    .read_u32::<LittleEndian>()
                    .map_err(|e| GaiaError::io_error(e, Url::parse("memory://methoddef").unwrap()))?;
                let _impl_flags = c3
                    .read_u16::<LittleEndian>()
                    .map_err(|e| GaiaError::io_error(e, Url::parse("memory://methoddef").unwrap()))?;
                let _flags = c3
                    .read_u16::<LittleEndian>()
                    .map_err(|e| GaiaError::io_error(e, Url::parse("memory://methoddef").unwrap()))?;
                let name_idx = read_heap_index(&mut c3, str_idx_sz)?;
                let _sig_idx = read_heap_index(&mut c3, blob_idx_sz)?;
                let _param_list_idx = read_heap_index(&mut c3, (if row_counts[0x07] < (1 << 16) { 2 } else { 4 }))?;

                let method_name = self.read_string_from_strings_heap(strings_start, strings_stream.size, name_idx)?;

                if !method_name.is_empty() {
                    let _mdef = ClrMethod::new(
                        method_name,
                        ClrTypeReference {
                            name: "Void".to_string(),
                            namespace: Some("System".to_string()),
                            assembly: Some("mscorlib".to_string()),
                            is_value_type: true,
                            is_reference_type: false,
                            generic_parameters: Vec::new(),
                        },
                    );
                    // TODO: Add the method to the corresponding type
                }
            }
        }

        // Read Field table
        if let Some(field_start) = table_start[0x04] {
            for i in 0..row_counts[0x04] {
                let mut c4 = self.reader.get_viewer();
                c4.seek(SeekFrom::Start((field_start + i * field_row_size) as u64))
                    .map_err(|e| GaiaError::io_error(e, Url::parse("memory://field").unwrap()))?;
                let _flags =
                    c4.read_u16::<LittleEndian>().map_err(|e| GaiaError::io_error(e, Url::parse("memory://field").unwrap()))?;
                let name_idx = read_heap_index(&mut c4, str_idx_sz)?;
                let _sig_idx = read_heap_index(&mut c4, blob_idx_sz)?;

                let name = self.read_string_from_strings_heap(strings_start, strings_stream.size, name_idx)?;

                if !name.is_empty() {
                    let _mdef = ClrMethod::new(
                        "DefaultMethod".to_string(),
                        ClrTypeReference {
                            name: "Void".to_string(),
                            namespace: Some("System".to_string()),
                            assembly: Some("mscorlib".to_string()),
                            is_value_type: true,
                            is_reference_type: false,
                            generic_parameters: Vec::new(),
                        },
                    );
                    let mut _clr_type = ClrType::new(name, None);
                    _clr_type.access_flags.is_public = true;
                    _clr_type.add_method(_mdef);
                }
            }
        }

        // Parse external assemblies: strictly based on the AssemblyRef table
        let mut external_assemblies: Vec<crate::program::ClrExternalAssembly> = Vec::new();
        // AssemblyRef table
        if let Some(asmref_start) = table_start[0x23] {
            // Calculate offset to AssemblyRef table
            let assemblyref_rows = row_counts[0x23];

            // Parse AssemblyRef row: Version(4x u16) + Flags(u32) + PublicKeyOrToken(Blob) + Name(String) + Culture(String) + HashValue(Blob)
            let row_size = table_row_size[0x23];
            for i in 0..assemblyref_rows {
                let row_off = asmref_start + i * row_size;
                let mut c4 = self.reader.get_viewer();
                c4.seek(SeekFrom::Start(row_off as u64))
                    .map_err(|e| GaiaError::io_error(e, Url::parse("memory://asmref").unwrap()))?;
                let ver_major = c4
                    .read_u16::<LittleEndian>()
                    .map_err(|e| GaiaError::io_error(e, Url::parse("memory://asmref").unwrap()))?;
                let ver_minor = c4
                    .read_u16::<LittleEndian>()
                    .map_err(|e| GaiaError::io_error(e, Url::parse("memory://asmref").unwrap()))?;
                let ver_build = c4
                    .read_u16::<LittleEndian>()
                    .map_err(|e| GaiaError::io_error(e, Url::parse("memory://asmref").unwrap()))?;
                let ver_rev = c4
                    .read_u16::<LittleEndian>()
                    .map_err(|e| GaiaError::io_error(e, Url::parse("memory://asmref").unwrap()))?;
                let _flags = c4
                    .read_u32::<LittleEndian>()
                    .map_err(|e| GaiaError::io_error(e, Url::parse("memory://asmref").unwrap()))?;
                let _pkt_idx = read_heap_index(&mut c4, blob_idx_sz)?;
                let name_idx = read_heap_index(&mut c4, str_idx_sz)?;
                let culture_idx = read_heap_index(&mut c4, str_idx_sz)?;
                let _hash_idx = read_heap_index(&mut c4, blob_idx_sz)?;
                let name = self.read_string_from_strings_heap(strings_start, strings_stream.size, name_idx)?;
                if !name.is_empty() {
                    external_assemblies.push(crate::program::ClrExternalAssembly {
                        name,
                        version: ClrVersion { major: ver_major, minor: ver_minor, build: ver_build, revision: ver_rev },
                        public_key_token: None,
                        culture: None,
                        hash_algorithm: None,
                    });
                }
            }
        }

        // If AssemblyRef not found or empty, try extracting common reference names from #Strings as fallback (only added if they actually appear)
        if external_assemblies.is_empty() {
            let cfg = &self.options.assembly_ref_fallback_names;
            let heap = self.read_strings_heap_data(strings_start, strings_stream.size)?;
            for name in cfg.iter() {
                if find_subslice(&heap, name.as_bytes()) {
                    external_assemblies.push(crate::program::ClrExternalAssembly {
                        name: name.to_string(),
                        version: ClrVersion { major: 0, minor: 0, build: 0, revision: 0 },
                        public_key_token: None,
                        culture: None,
                        hash_algorithm: None,
                    });
                }
            }
        }

        for ea in external_assemblies {
            program.add_external_assembly(ea);
        }

        // Set runtime version string (metadata header version string) as info source
        let _ = _version_string.as_str();

        Ok(program)
    }

    /// Finds and reads the CLR header.
    ///
    /// This method searches for the CLR header within the PE file. The CLR header contains:
    /// - Size and version information
    /// - Metadata location (RVA and size)
    /// - Entry point token
    /// - Various flags and configurations
    ///
    /// # Returns
    /// * `Ok(Some(ClrHeader))` - Successfully found and read the CLR header
    /// * `Ok(None)` - CLR header not found (not a .NET assembly)
    fn find_and_read_clr_header(&mut self) -> Result<Option<ClrHeader>, GaiaError> {
        eprintln!("Starting to find CLR header");

        // Get PE program to access data directories
        let pe_program = self.reader.get_program()?.clone();
        eprintln!("Successfully got PE program");

        // Check number of data directories
        let data_dirs_count = pe_program.header.optional_header.data_directories.len();
        eprintln!("Data directories count: {}", data_dirs_count);

        // Check if CLR data directory exists (index 14 is the CLR Runtime Header)
        if let Some(clr_dir) = pe_program.header.optional_header.data_directories.get(14) {
            eprintln!("Found CLR data directory - RVA: 0x{:x}, Size: {}", clr_dir.virtual_address, clr_dir.size);

            if clr_dir.virtual_address == 0 || clr_dir.size == 0 {
                eprintln!("CLR data directory is empty");
                return Ok(None);
            }

            // Convert RVA to file offset
            let file_offset = self.rva_to_file_offset(clr_dir.virtual_address)?;
            eprintln!("CLR header file offset: 0x{:x}", file_offset);

            // Read CLR header
            let mut cursor = self.reader.get_viewer();
            cursor
                .seek(SeekFrom::Start(file_offset as u64))
                .map_err(|e| GaiaError::io_error(e, Url::parse("memory://clr_header").unwrap()))?;

            let cb = cursor
                .read_u32::<LittleEndian>()
                .map_err(|e| GaiaError::io_error(e, Url::parse("memory://clr_header").unwrap()))?;
            let major_runtime_version = cursor
                .read_u16::<LittleEndian>()
                .map_err(|e| GaiaError::io_error(e, Url::parse("memory://clr_header").unwrap()))?;
            let minor_runtime_version = cursor
                .read_u16::<LittleEndian>()
                .map_err(|e| GaiaError::io_error(e, Url::parse("memory://clr_header").unwrap()))?;
            let metadata_rva = cursor
                .read_u32::<LittleEndian>()
                .map_err(|e| GaiaError::io_error(e, Url::parse("memory://clr_header").unwrap()))?;
            let metadata_size = cursor
                .read_u32::<LittleEndian>()
                .map_err(|e| GaiaError::io_error(e, Url::parse("memory://clr_header").unwrap()))?;
            let flags = cursor
                .read_u32::<LittleEndian>()
                .map_err(|e| GaiaError::io_error(e, Url::parse("memory://clr_header").unwrap()))?;

            eprintln!(
                "Successfully read CLR header - cb: {}, Version: {}.{}, Metadata RVA: 0x{:x}, Size: {}",
                cb, major_runtime_version, minor_runtime_version, metadata_rva, metadata_size
            );

            Ok(Some(ClrHeader { cb, major_runtime_version, minor_runtime_version, metadata_rva, metadata_size, flags }))
        }
        else {
            eprintln!("CLR data directory (index 14) not found");
            Ok(None)
        }
    }

    /// Reads the metadata header.
    ///
    /// This method reads the metadata header from the specified file offset.
    /// The metadata header contains basic information about the metadata structure:
    /// - Signature: Magic number representing .NET metadata (0x424A5342)
    /// - Major and minor versions
    /// - Reserved fields
    /// - Version string length and content
    /// - Flags and number of streams
    ///
    /// # Arguments
    /// * `offset` - File offset where the metadata header starts
    ///
    /// # Returns
    /// * `Ok(MetadataHeader)` - Successfully read the metadata header
    /// * `Err(GaiaError)` - Error during the reading process
    fn read_metadata_header(&mut self, offset: u32) -> Result<MetadataHeader, GaiaError> {
        let mut cursor = self.reader.get_viewer();
        cursor
            .seek(SeekFrom::Start(offset as u64))
            .map_err(|e| GaiaError::io_error(e, Url::parse("memory://metadata_header").unwrap()))?;

        // Read fixed-length header fields
        let signature = cursor
            .read_u32::<LittleEndian>()
            .map_err(|e| GaiaError::io_error(e, Url::parse("memory://metadata_header").unwrap()))?;
        let major_version = cursor
            .read_u16::<LittleEndian>()
            .map_err(|e| GaiaError::io_error(e, Url::parse("memory://metadata_header").unwrap()))?;
        let minor_version = cursor
            .read_u16::<LittleEndian>()
            .map_err(|e| GaiaError::io_error(e, Url::parse("memory://metadata_header").unwrap()))?;
        let reserved = cursor
            .read_u32::<LittleEndian>()
            .map_err(|e| GaiaError::io_error(e, Url::parse("memory://metadata_header").unwrap()))?;
        let version_length = cursor
            .read_u32::<LittleEndian>()
            .map_err(|e| GaiaError::io_error(e, Url::parse("memory://metadata_header").unwrap()))?;

        // Read version string (variable length)
        let mut version_bytes = vec![0u8; version_length as usize];
        cursor
            .read_exact(&mut version_bytes)
            .map_err(|e| GaiaError::io_error(e, Url::parse("memory://metadata_header").unwrap()))?;
        let version_string = String::from_utf8_lossy(&version_bytes).trim_end_matches('\0').to_string();

        // Read remaining fixed-length fields
        let flags = cursor
            .read_u16::<LittleEndian>()
            .map_err(|e| GaiaError::io_error(e, Url::parse("memory://metadata_header").unwrap()))?;
        let streams = cursor
            .read_u16::<LittleEndian>()
            .map_err(|e| GaiaError::io_error(e, Url::parse("memory://metadata_header").unwrap()))?;

        // Create metadata header structure
        Ok(MetadataHeader { signature, major_version, minor_version, reserved, version_length, version_string, flags, streams })
    }

    /// Reads stream header information.
    ///
    /// Reads all stream headers starting from the position after the metadata header.
    /// Stream headers immediately follow the variable-length version string in the metadata header.
    ///
    /// Stream header structure (per stream):
    /// - offset: Offset of the stream within the metadata (4 bytes)
    /// - size: Size of the stream (4 bytes)
    /// - name: Name of the stream (null-terminated string, aligned to 4-byte boundary)
    ///
    /// Common stream names:
    /// - "#Strings": String heap, containing various names
    /// - "#US": User strings, containing string literals
    /// - "#GUID": GUID heap, containing GUID values
    /// - "#Blob": Blob heap, containing binary data
    /// - "#~": Compressed metadata table stream
    /// - "#-": Uncompressed metadata table stream
    ///
    /// # Arguments
    /// * `metadata_offset` - Starting file offset of the metadata header
    ///
    /// # Returns
    /// * `Ok(Vec<StreamHeader>)` - Successfully read list of stream headers
    /// * `Err(GaiaError)` - Error during the reading process
    fn read_stream_headers(&mut self, metadata_offset: u32) -> Result<Vec<StreamHeader>, GaiaError> {
        let mut stream_headers = Vec::new();

        if let Some(ref metadata_header) = self.metadata_header {
            let mut cursor = self.reader.get_viewer();
            // Calculate starting position of stream headers: skip fixed part of metadata header (20 bytes) and version string
            let stream_start_offset = metadata_offset + 20 + metadata_header.version_length;
            cursor
                .seek(SeekFrom::Start(stream_start_offset as u64))
                .map_err(|e| GaiaError::io_error(e, Url::parse("memory://stream_headers").unwrap()))?;

            // Read header information for each stream
            for _ in 0..metadata_header.streams {
                let offset = cursor
                    .read_u32::<LittleEndian>()
                    .map_err(|e| GaiaError::io_error(e, Url::parse("memory://stream_headers").unwrap()))?;
                let size = cursor
                    .read_u32::<LittleEndian>()
                    .map_err(|e| GaiaError::io_error(e, Url::parse("memory://stream_headers").unwrap()))?;

                // Read stream name (null-terminated string)
                let mut name_bytes = Vec::new();
                loop {
                    let byte =
                        cursor.read_u8().map_err(|e| GaiaError::io_error(e, Url::parse("memory://stream_headers").unwrap()))?;
                    if byte == 0 {
                        break;
                    }
                    name_bytes.push(byte);
                }
                let name = String::from_utf8_lossy(&name_bytes).to_string();

                // Align to 4-byte boundary
                let current_pos = cursor
                    .stream_position()
                    .map_err(|e| GaiaError::io_error(e, Url::parse("memory://stream_headers").unwrap()))?;
                let aligned_pos = (current_pos + 3) & !3;
                cursor
                    .seek(SeekFrom::Start(aligned_pos))
                    .map_err(|e| GaiaError::io_error(e, Url::parse("memory://stream_headers").unwrap()))?;

                stream_headers.push(StreamHeader { offset, size, name });
            }
        }

        Ok(stream_headers)
    }

    /// Reads raw data of the strings heap.
    fn read_strings_heap_data(&mut self, strings_start: u32, strings_size: u32) -> Result<Vec<u8>, GaiaError> {
        let mut reader = self.reader.get_viewer();
        reader
            .seek(SeekFrom::Start(strings_start as u64))
            .map_err(|e| GaiaError::io_error(e, Url::parse("memory://strings_heap").unwrap()))?;

        let mut buffer = vec![0u8; strings_size as usize];
        reader.read_exact(&mut buffer).map_err(|e| GaiaError::io_error(e, Url::parse("memory://strings_heap").unwrap()))?;

        Ok(buffer)
    }

    /// Helper method to read a string from the strings heap.
    fn read_string_from_strings_heap(
        &mut self,
        strings_start: u32,
        strings_size: u32,
        index: u32,
    ) -> Result<String, GaiaError> {
        eprintln!("Reading string - Start: {}, Size: {}, Index: {}", strings_start, strings_size, index);

        if index == 0 {
            return Ok(String::new());
        }

        let base = strings_start + index;
        let end = strings_start + strings_size;

        eprintln!("Calculating position - base: {}, end: {}", base, end);

        if base >= end {
            eprintln!("Index out of bounds - base {} >= end {}", base, end);
            return Err(GaiaError::syntax_error(format!("String index {} exceeds heap range", index), SourceLocation::default()));
        }

        // Seek to string position
        let viewer = self.reader.get_viewer();
        viewer
            .seek(SeekFrom::Start(base as u64))
            .map_err(|e| GaiaError::io_error(e, Url::parse("memory://strings_heap").unwrap()))?;

        // Read null-terminated string
        let mut bytes = Vec::new();
        loop {
            let byte = viewer.read_u8().map_err(|e| GaiaError::io_error(e, Url::parse("memory://strings_heap").unwrap()))?;
            if byte == 0 {
                break;
            }
            bytes.push(byte);
        }

        let result = String::from_utf8_lossy(&bytes).to_string();
        eprintln!("Successfully read string: '{}'", result);
        Ok(result)
    }

    /// Converts RVA (Relative Virtual Address) to file offset.
    ///
    /// RVA (Relative Virtual Address) is an important concept in PE files:
    /// - RVA is the offset relative to the image base.
    /// - File offset is the physical position relative to the beginning of the file.
    ///
    /// Conversion process:
    /// 1. Look for the section containing the target RVA in the PE section table.
    /// 2. Calculate the relative offset of the RVA within the section.
    /// 3. Add the relative offset to the section's file offset to get the final file offset.
    ///
    /// # Arguments
    /// * `rva` - Relative virtual address to convert
    ///
    /// # Returns
    /// * `Ok(u32)` - Successfully converted file offset
    /// * `Err(GaiaError)` - Error when no section contains the RVA
    ///
    /// # Example
    /// ```
    /// let file_offset = reader.rva_to_file_offset(0x2000)?;
    /// ```
    fn rva_to_file_offset(&mut self, rva: u32) -> Result<u32, GaiaError> {
        // Need to read full PE program to access section information
        let pe_program = self.reader.get_program()?.clone();

        // Look for the section containing this RVA in the section table
        for section in &pe_program.sections {
            let section_start = section.virtual_address;
            let section_end = section_start + section.virtual_size;

            // Check if RVA is within the address range of this section
            if rva >= section_start && rva < section_end {
                // Calculate relative offset within the section
                let offset_in_section = rva - section_start;
                // Return file offset = section file offset + relative offset
                return Ok(section.pointer_to_raw_data + offset_in_section);
            }
        }

        // Section containing the RVA not found
        Err(GaiaError::syntax_error(format!("Unable to convert RVA 0x{:x} to file offset", rva), SourceLocation::default()))
    }
}

/// Reads a heap index (2 or 4 bytes depending on size).
fn read_type_def_or_ref_index<R: Read>(cursor: &mut R, idx_size: u32) -> Result<u32, GaiaError> {
    if idx_size == 2 {
        cursor
            .read_u16::<LittleEndian>()
            .map(|v| v as u32)
            .map_err(|e| GaiaError::io_error(e, Url::parse("memory://type_def_or_ref_index").unwrap()))
    }
    else {
        cursor
            .read_u32::<LittleEndian>()
            .map_err(|e| GaiaError::io_error(e, Url::parse("memory://type_def_or_ref_index").unwrap()))
    }
}

fn read_heap_index<R: Read>(cursor: &mut R, idx_size: u32) -> Result<u32, GaiaError> {
    if idx_size == 2 {
        cursor
            .read_u16::<LittleEndian>()
            .map(|v| v as u32)
            .map_err(|e| GaiaError::io_error(e, Url::parse("memory://heap_index").unwrap()))
    }
    else if idx_size == 4 {
        cursor.read_u32::<LittleEndian>().map_err(|e| GaiaError::io_error(e, Url::parse("memory://heap_index").unwrap()))
    }
    else {
        Err(GaiaError::syntax_error("Illegal heap index size".to_string(), SourceLocation::default()))
    }
}

/// Reads a string from the #Strings heap (null-terminated UTF-8).
fn read_string_from_heap(pe_data: &[u8], strings_start: u32, strings_size: u32, index: u32) -> Result<String, GaiaError> {
    if index == 0 {
        return Ok(String::new());
    }
    let base = strings_start + index;
    let end = strings_start + strings_size;
    if base >= end || (base as usize) >= pe_data.len() {
        return Ok(String::new());
    }
    let mut i = base as usize;
    let mut bytes = Vec::new();
    while i < pe_data.len() && (i as u32) < end {
        let b = pe_data[i];
        if b == 0 {
            break;
        }
        bytes.push(b);
        i += 1;
    }
    Ok(String::from_utf8_lossy(&bytes).to_string())
}

/// Simple check if a subslice exists.
fn find_subslice(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() {
        return true;
    }
    if haystack.len() < needle.len() {
        return false;
    }
    let n = needle.len();
    for i in 0..=haystack.len() - n {
        if &haystack[i..i + n] == needle {
            return true;
        }
    }
    false
}
