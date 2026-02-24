use crate::program::{
    ClrAccessFlags, ClrMethod, ClrProgram, ClrType, ClrTypeReference, ClrVersion, StreamHeader,
};
use gaia_types::{GaiaError, SourceLocation};
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Read, Seek, SeekFrom};
use url::Url;
use super::DllReader;
use super::utils::{find_subslice, read_heap_index, read_type_def_or_ref_index, rva_to_file_offset};

impl<'config, R: Read + Seek> DllReader<'config, R> {
    /// Parses the full CLR program.
    pub(super) fn parse_full_program(&mut self) -> Result<ClrProgram, GaiaError> {
        // Defensive check: requires parsed CLR header and metadata header
        let clr_header = self
            .clr_header
            .as_ref()
            .ok_or_else(|| GaiaError::syntax_error("Missing CLR header".to_string(), SourceLocation::default()))?;
        let metadata_rva = clr_header.metadata_rva;
        let version_string = self
            .metadata_header
            .as_ref()
            .ok_or_else(|| GaiaError::syntax_error("Missing metadata header".to_string(), SourceLocation::default()))?
            .version_string
            .clone();

        // Calculate metadata starting file offset
        let pe_program = self.reader.get_program()?.clone();
        let metadata_base = rva_to_file_offset(&pe_program, metadata_rva)?;

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
            GaiaError::syntax_error("Missing metadata table stream (#~/#-)".to_string(), SourceLocation::default())
        })?;
        let strings_stream = strings_stream.ok_or_else(|| {
            GaiaError::syntax_error("Missing string stream (#Strings)".to_string(), SourceLocation::default())
        })?;

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
                let _culture = if culture_idx != 0 {
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
                let _flags = ct
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
        let _ = version_string.as_str();

        Ok(program)
    }
}
