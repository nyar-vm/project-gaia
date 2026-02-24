use crate::program::{
    ClrMethod, ClrProgram, ClrType, ClrTypeReference, ClrVersion, DotNetAssemblyInfo, StreamHeader,
};
use byteorder::{LittleEndian, ReadBytesExt};
use gaia_types::{GaiaError, SourceLocation};
use pe_assembler::types::PeReader;
use std::io::{Read, Seek, SeekFrom};
use url::Url;
use super::DllReader;
use super::utils::{read_heap_index, read_type_def_or_ref_index};

impl<'config, R: Read + Seek> DllReader<'config, R> {
    /// Extracts assembly information.
    pub(crate) fn extract_assembly_info(&mut self) -> Result<(), GaiaError> {
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
        let mut cur = self.get_viewer();
        cur.seek(SeekFrom::Start(tables_start as u64)).map_err(|e| {
            GaiaError::io_error(e, Url::parse("memory://table").unwrap())
        })?;

        let _reserved = cursor_read_u32(&mut cur, "tables_hdr")?;
        let _major = cursor_read_u8(&mut cur, "tables_hdr")?;
        let _minor = cursor_read_u8(&mut cur, "tables_hdr")?;
        let heap_sizes = cursor_read_u8(&mut cur, "tables_hdr")?;
        let _reserved2 = cursor_read_u8(&mut cur, "tables_hdr")?;
        let valid_mask = cursor_read_u64(&mut cur, "tables_hdr")?;
        let _sorted_mask = cursor_read_u64(&mut cur, "tables_hdr")?;

        // Heap index sizes
        let str_idx_sz = if (heap_sizes & 0x01) != 0 { 4 } else { 2 };
        let guid_idx_sz = if (heap_sizes & 0x02) != 0 { 4 } else { 2 };
        let blob_idx_sz = if (heap_sizes & 0x04) != 0 { 4 } else { 2 };
        let strings_size = strings_stream.size;

        eprintln!("Heap size flags: 0x{:02x}", heap_sizes);
        eprintln!(
            "String index size: {}, GUID index size: {}, Blob index size: {}",
            str_idx_sz, guid_idx_sz, blob_idx_sz
        );
        eprintln!("String stream size: {}", strings_size);
        eprintln!("Valid table mask: 0x{:016x}", valid_mask);

        // Read row counts
        let mut row_counts: [u32; 64] = [0; 64];
        for tid in 0..64u8 {
            if (valid_mask >> tid) & 1 == 1 {
                row_counts[tid as usize] = cursor_read_u32(&mut cur, "tables_rows")?;
                eprintln!("Table 0x{:02x} row count: {}", tid, row_counts[tid as usize]);
            }
        }

        // Calculate relevant coded index sizes
        fn coded_size(rows: &[u32; 64], tags: &[u8]) -> u32 {
            let max_rows = tags.iter().map(|&t| rows[t as usize]).max().unwrap_or(0);
            let tag_bits = (tags.len() as f32).log2().ceil() as u32;
            if (max_rows << tag_bits) < (1 << 16) {
                2
            } else {
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
        let tables_data_start = cur.stream_position().map_err(|e| {
            GaiaError::io_error(e, Url::parse("memory://tables_data").unwrap())
        })? as u32;
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
        let mut version = ClrVersion {
            major: 0,
            minor: 0,
            build: 0,
            revision: 0,
        };

        // First try reading from Assembly table (0x20)
        if let Some(asm_start) = table_start[0x20] {
            if row_counts[0x20] > 0 {
                let mut c = self.get_viewer();
                c.seek(SeekFrom::Start(asm_start as u64)).map_err(|e| {
                    GaiaError::io_error(e, Url::parse("memory://asm").unwrap())
                })?;

                // Read Assembly table first row - according to ECMA-335 specification
                let hash_alg = cursor_read_u32(&mut c, "asm")?;
                version.major = cursor_read_u16(&mut c, "asm")?;
                version.minor = cursor_read_u16(&mut c, "asm")?;
                version.build = cursor_read_u16(&mut c, "asm")?;
                version.revision = cursor_read_u16(&mut c, "asm")?;
                let _flags = cursor_read_u32(&mut c, "asm")?;
                let _pk_idx = read_heap_index(&mut c, blob_idx_sz)?;
                let name_idx = read_heap_index(&mut c, str_idx_sz)?;
                let _culture_idx = read_heap_index(&mut c, str_idx_sz)?;

                // Validate if read data is reasonable
                if hash_alg > 0x10000 || version.major > 100 || name_idx > strings_size {
                    eprintln!("Warning: Assembly table data abnormal, table offset calculation might be wrong");
                    eprintln!(
                        "HashAlg: 0x{:x}, Version: {}.{}.{}.{}, NameIdx: {}",
                        hash_alg, version.major, version.minor, version.build, version.revision, name_idx
                    );
                } else {
                    let n = self.read_string_from_strings_heap(strings_start, strings_size, name_idx)?;
                    if !n.is_empty() {
                        name = n;
                    }
                }
            }
        } else if let Some(mod_start) = table_start[0x00] {
            // Module table
            if row_counts[0x00] > 0 {
                let mut c = self.get_viewer();
                c.seek(SeekFrom::Start(mod_start as u64)).map_err(|e| {
                    GaiaError::io_error(e, Url::parse("memory://mod").unwrap())
                })?;
                let _generation = cursor_read_u16(&mut c, "mod")?;
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
            version: format!(
                "{}.{}.{}.{}",
                version.major, version.minor, version.build, version.revision
            ),
            culture: None,
            public_key_token: None,
            runtime_version,
        });

        Ok(())
    }

    /// Parses the full CLR program.
    pub(crate) fn parse_full_program(&mut self) -> Result<ClrProgram, GaiaError> {
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

        let tables_stream = tables_stream.ok_or_else(|| {
            GaiaError::syntax_error(
                "Missing metadata table stream (#~/#-)".to_string(),
                SourceLocation::default(),
            )
        })?;
        let strings_stream = strings_stream.ok_or_else(|| {
            GaiaError::syntax_error("Missing string stream (#Strings)".to_string(), SourceLocation::default())
        })?;

        // Convenience: treat file as a cursor
        let mut cur = self.get_viewer();
        // Absolute file offsets for the start of the table stream and string stream
        let tables_start = metadata_base + tables_stream.offset;
        let strings_start = metadata_base + strings_stream.offset;

        // Read the table header (compressed metadata format, ECMA-335 II.24.2.6)
        cur.seek(SeekFrom::Start(tables_start as u64)).map_err(|e| {
            GaiaError::io_error(e, Url::parse("memory://tables").unwrap())
        })?;
        let _reserved = cursor_read_u32(&mut cur, "tables")?;
        let _major = cursor_read_u8(&mut cur, "tables")?;
        let _minor = cursor_read_u8(&mut cur, "tables")?;
        let heap_sizes = cursor_read_u8(&mut cur, "tables")?;
        let _reserved2 = cursor_read_u8(&mut cur, "tables")?;
        let valid_mask = cursor_read_u64(&mut cur, "tables")?;
        let _sorted_mask = cursor_read_u64(&mut cur, "tables")?;

        // Read the number of rows for existing tables
        let mut row_counts: [u32; 64] = [0; 64];
        for tid in 0..64u8 {
            if (valid_mask >> tid) & 1 == 1 {
                row_counts[tid as usize] = cursor_read_u32(&mut cur, "tables")?;
            }
        }

        // Calculate heap index sizes
        let str_idx_sz = if (heap_sizes & 0x01) != 0 { 4 } else { 2 };
        let guid_idx_sz = if (heap_sizes & 0x02) != 0 { 4 } else { 2 };
        let blob_idx_sz = if (heap_sizes & 0x04) != 0 { 4 } else { 2 };
        let _ = guid_idx_sz; // Currently unused, avoid warning

        // Data area starting position (current cursor position)
        let tables_data_start = cur.stream_position().map_err(|e| {
            GaiaError::io_error(e, Url::parse("memory://tables_data").unwrap())
        })? as u32;

        // Calculate simple index size (to a specified table)
        let simple_index_size = |table_id: u8, counts: &[u32; 64]| -> u32 {
            let rows = counts[table_id as usize];
            if rows < (1 << 16) {
                2
            } else {
                4
            }
        };

        // Calculate MethodDef row size
        let param_index_sz = simple_index_size(0x07, &row_counts); // Param
        let methoddef_row_size = 4 /*RVA*/ + 2 /*ImplFlags*/ + 2 /*Flags*/ + str_idx_sz + blob_idx_sz + param_index_sz;

        // Calculate row sizes of some preceding tables to accumulate offset to MethodDef
        let field_row_size = 2 /*Flags*/ + str_idx_sz /*Name*/ + blob_idx_sz /*Signature*/;
        let fieldptr_row_size = simple_index_size(0x04, &row_counts);
        let methodptr_row_size = simple_index_size(0x06, &row_counts);
        // TypeRef row size: ResolutionScope (coded index) + Name (String) + Namespace (String)
        // ResolutionScope can point to: Module(0x00), ModuleRef(0x1A/0x17), AssemblyRef(0x20), TypeRef(0x01)
        let rs_candidates = [0x00u8, 0x17u8, 0x20u8, 0x01u8];
        let mut max_rs_rows = 0u32;
        for &t in &rs_candidates {
            max_rs_rows = max_rs_rows.max(row_counts[t as usize]);
        }
        let rs_tag_bits = 2u32;
        let resolution_scope_sz = if max_rs_rows < (1 << (16 - rs_tag_bits)) {
            2
        } else {
            4
        };
        let typeref_row_size = resolution_scope_sz + str_idx_sz + str_idx_sz;
        // TypeDef row size: Flags(u32) + Name(String) + Namespace(String) + Extends(TypeDefOrRef) + FieldList(simple index to Field) + MethodList(simple index to MethodDef)
        // TypeDefOrRef coded index candidates: TypeDef(0x02), TypeRef(0x01), TypeSpec(0x1B/0x18)
        let tdr_candidates = [0x02u8, 0x01u8, 0x18u8];
        let mut max_tdr_rows = 0u32;
        for &t in &tdr_candidates {
            max_tdr_rows = max_tdr_rows.max(row_counts[t as usize]);
        }
        let tdr_tag_bits = 2u32;
        let type_def_or_ref_sz = if max_tdr_rows < (1 << (16 - tdr_tag_bits)) {
            2
        } else {
            4
        };
        let type_def_row_size = 4 /*Flags*/
            + str_idx_sz
            + str_idx_sz
            + type_def_or_ref_sz
            + simple_index_size(0x04, &row_counts)
            + simple_index_size(0x06, &row_counts);
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
                    0x08 => simple_index_size(0x02, &row_counts) + simple_index_size(0x01, &row_counts), // InterfaceImpl
                    0x09 => resolution_scope_sz + str_idx_sz + blob_idx_sz, // MemberRef
                    0x0A => 2 /*Type*/ + blob_idx_sz,               // Constant
                    0x0B => {
                        simple_index_size(0x02, &row_counts)
                            + simple_index_size(0x0A, &row_counts)
                            + simple_index_size(0x0C, &row_counts)
                    } /* CustomAttribute (rough) */
                    0x0C => simple_index_size(0x04, &row_counts) + simple_index_size(0x07, &row_counts), // FieldMarshal
                    0x0D => 2 + blob_idx_sz,                                                             // DeclSecurity
                    0x0E => 2 + 4 + 4,                                                                   // ClassLayout
                    0x0F => simple_index_size(0x04, &row_counts) + 4,                                    // FieldLayout
                    0x10 => blob_idx_sz,                                                                 // StandAloneSig
                    0x11 => simple_index_size(0x02, &row_counts) + simple_index_size(0x12, &row_counts), // EventMap
                    0x12 => 2 + str_idx_sz + simple_index_size(0x10, &row_counts),                       // Event
                    0x13 => simple_index_size(0x02, &row_counts) + simple_index_size(0x14, &row_counts), // PropertyMap
                    0x14 => 2 + str_idx_sz + blob_idx_sz,                                                // Property
                    0x15 => 2 + simple_index_size(0x06, &row_counts) + simple_index_size(0x14, &row_counts), // MethodSemantics
                    0x16 => {
                        simple_index_size(0x02, &row_counts)
                            + simple_index_size(0x06, &row_counts)
                            + simple_index_size(0x01, &row_counts)
                    } // MethodImpl
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

        // Construct program object
        let mut program = ClrProgram::new("UnknownAssembly");
        program.version = ClrVersion {
            major: 1,
            minor: 0,
            build: 0,
            revision: 0,
        };
        program.access_flags = crate::program::ClrAccessFlags {
            is_public: true,
            is_private: false,
            is_security_transparent: false,
            is_retargetable: false,
        };

        // Try to fill name and version from Assembly table, otherwise use Module name
        if let Some(asm_start) = table_start[0x20] {
            // Assembly table exists (0x20)
            let asm_rows = row_counts[0x20];
            if asm_rows > 0 {
                let asm0 = asm_start; // First row offset
                let mut c2 = self.get_viewer();
                c2.seek(SeekFrom::Start(asm0 as u64)).map_err(|e| {
                    GaiaError::io_error(e, Url::parse("memory://asm").unwrap())
                })?;
                let _hash_alg = cursor_read_u32(&mut c2, "asm")?;
                let ver_major = cursor_read_u16(&mut c2, "asm")?;
                let ver_minor = cursor_read_u16(&mut c2, "asm")?;
                let ver_build = cursor_read_u16(&mut c2, "asm")?;
                let ver_rev = cursor_read_u16(&mut c2, "asm")?;
                let _flags = cursor_read_u32(&mut c2, "asm")?;
                let _pk_idx = read_heap_index(&mut c2, blob_idx_sz)?;
                let name_idx = read_heap_index(&mut c2, str_idx_sz)?;
                let culture_idx = read_heap_index(&mut c2, str_idx_sz)?;
                let _hash_idx = read_heap_index(&mut c2, blob_idx_sz)?;

                let name = self.read_string_from_strings_heap(strings_start, strings_stream.size, name_idx)?;
                let _culture = if culture_idx != 0 {
                    Some(self.read_string_from_strings_heap(strings_start, strings_stream.size, culture_idx)?)
                } else {
                    None
                };

                if !name.is_empty() {
                    program.name = name;
                }
                program.version = ClrVersion {
                    major: ver_major,
                    minor: ver_minor,
                    build: ver_build,
                    revision: ver_rev,
                };
            }
        } else if let Some(module_start) = table_start[0x00] {
            // Use name when Module table exists
            let mut cm = self.get_viewer();
            cm.seek(SeekFrom::Start(module_start as u64)).map_err(|e| {
                GaiaError::io_error(e, Url::parse("memory://module").unwrap())
            })?;
            let _generation = cursor_read_u16(&mut cm, "module")?;
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
                let mut ct = self.get_viewer();
                ct.seek(SeekFrom::Start((typedef_start + i * type_def_row_size) as u64))
                    .map_err(|e| GaiaError::io_error(e, Url::parse("memory://typedef").unwrap()))?;
                let _flags = cursor_read_u32(&mut ct, "typedef")?;
                let name_idx = read_heap_index(&mut ct, str_idx_sz)?;
                let ns_idx = read_heap_index(&mut ct, str_idx_sz)?;
                let _extends_idx = read_type_def_or_ref_index(&mut ct, type_def_or_ref_sz)?;
                let _field_list_idx =
                    read_heap_index(&mut ct, (if row_counts[0x04] < (1 << 16) { 2 } else { 4 }))?;
                let _method_list_idx =
                    read_heap_index(&mut ct, (if row_counts[0x06] < (1 << 16) { 2 } else { 4 }))?;

                let type_name = self.read_string_from_strings_heap(strings_start, strings_stream.size, name_idx)?;
                let namespace = if ns_idx != 0 {
                    Some(self.read_string_from_strings_heap(strings_start, strings_stream.size, ns_idx)?)
                } else {
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
                    program.add_type(clr_type);
                }
            }
        }

        // Read MethodDef table
        if let Some(methoddef_start) = table_start[0x06] {
            for i in 0..row_counts[0x06] {
                let mut c3 = self.get_viewer();
                c3.seek(SeekFrom::Start((methoddef_start + i * methoddef_row_size) as u64))
                    .map_err(|e| GaiaError::io_error(e, Url::parse("memory://methoddef").unwrap()))?;
                let _rva = cursor_read_u32(&mut c3, "methoddef")?;
                let _impl_flags = cursor_read_u16(&mut c3, "methoddef")?;
                let _flags = cursor_read_u16(&mut c3, "methoddef")?;
                let name_idx = read_heap_index(&mut c3, str_idx_sz)?;
                let _sig_idx = read_heap_index(&mut c3, blob_idx_sz)?;
                let _param_list_idx =
                    read_heap_index(&mut c3, (if row_counts[0x07] < (1 << 16) { 2 } else { 4 }))?;

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
                let mut c4 = self.get_viewer();
                c4.seek(SeekFrom::Start((field_start + i * field_row_size) as u64))
                    .map_err(|e| GaiaError::io_error(e, Url::parse("memory://field").unwrap()))?;
                let _flags = cursor_read_u16(&mut c4, "field")?;
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
                let mut c4 = self.get_viewer();
                c4.seek(SeekFrom::Start(row_off as u64))
                    .map_err(|e| GaiaError::io_error(e, Url::parse("memory://asmref").unwrap()))?;
                let ver_major = cursor_read_u16(&mut c4, "asmref")?;
                let ver_minor = cursor_read_u16(&mut c4, "asmref")?;
                let ver_build = cursor_read_u16(&mut c4, "asmref")?;
                let ver_rev = cursor_read_u16(&mut c4, "asmref")?;
                let _flags = cursor_read_u32(&mut c4, "asmref")?;
                let _pkt_idx = read_heap_index(&mut c4, blob_idx_sz)?;
                let name_idx = read_heap_index(&mut c4, str_idx_sz)?;
                let culture_idx = read_heap_index(&mut c4, str_idx_sz)?;
                let _hash_idx = read_heap_index(&mut c4, blob_idx_sz)?;
                let name = self.read_string_from_strings_heap(strings_start, strings_stream.size, name_idx)?;
                if !name.is_empty() {
                    external_assemblies.push(crate::program::ClrExternalAssembly {
                        name,
                        version: format!("{}.{}.{}.{}", ver_major, ver_minor, ver_build, ver_rev),
                        culture: if culture_idx != 0 {
                            Some(self.read_string_from_strings_heap(
                                strings_start,
                                strings_stream.size,
                                culture_idx,
                            )?)
                        } else {
                            None
                        },
                        public_key_token: None,
                    });
                }
            }
        }

        program.external_assemblies = external_assemblies;

        Ok(program)
    }
}

// Helper functions for cleaner code
fn cursor_read_u8<R: Read>(cursor: &mut R, context: &str) -> Result<u8, GaiaError> {
    cursor
        .read_u8()
        .map_err(|e| GaiaError::io_error(e, Url::parse(&format!("memory://{}", context)).unwrap()))
}

fn cursor_read_u16<R: Read>(cursor: &mut R, context: &str) -> Result<u16, GaiaError> {
    cursor
        .read_u16::<LittleEndian>()
        .map_err(|e| GaiaError::io_error(e, Url::parse(&format!("memory://{}", context)).unwrap()))
}

fn cursor_read_u32<R: Read>(cursor: &mut R, context: &str) -> Result<u32, GaiaError> {
    cursor
        .read_u32::<LittleEndian>()
        .map_err(|e| GaiaError::io_error(e, Url::parse(&format!("memory://{}", context)).unwrap()))
}

fn cursor_read_u64<R: Read>(cursor: &mut R, context: &str) -> Result<u64, GaiaError> {
    cursor
        .read_u64::<LittleEndian>()
        .map_err(|e| GaiaError::io_error(e, Url::parse(&format!("memory://{}", context)).unwrap()))
}
