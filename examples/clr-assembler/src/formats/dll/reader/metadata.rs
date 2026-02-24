use crate::program::{ClrVersion, DotNetAssemblyInfo, StreamHeader};
use gaia_types::{GaiaError, SourceLocation};
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Read, Seek, SeekFrom};
use url::Url;
use super::DllReader;
use super::utils::{read_heap_index, rva_to_file_offset};

impl<'config, R: Read + Seek> DllReader<'config, R> {
    /// Extracts assembly information.
    pub(super) fn extract_assembly_info(&mut self) -> Result<(), GaiaError> {
        // Depends on parsed CLR header and metadata stream headers
        let clr_header = match &self.clr_header {
            Some(h) => *h,
            None => return Ok(()),
        };
        let pe_program = self.reader.get_program()?.clone();
        let metadata_offset = rva_to_file_offset(&pe_program, clr_header.metadata_rva)?;

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
                let _flags =
                    c.read_u32::<LittleEndian>().map_err(|e| GaiaError::io_error(e, Url::parse("memory://asm").unwrap()))?;
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

    /// Reads raw data of the strings heap.
    pub(super) fn read_strings_heap_data(&mut self, strings_start: u32, strings_size: u32) -> Result<Vec<u8>, GaiaError> {
        let mut reader = self.reader.get_viewer();
        reader
            .seek(SeekFrom::Start(strings_start as u64))
            .map_err(|e| GaiaError::io_error(e, Url::parse("memory://strings_heap").unwrap()))?;

        let mut buffer = vec![0u8; strings_size as usize];
        reader.read_exact(&mut buffer).map_err(|e| GaiaError::io_error(e, Url::parse("memory://strings_heap").unwrap()))?;

        Ok(buffer)
    }

    /// Helper method to read a string from the strings heap.
    pub(super) fn read_string_from_strings_heap(
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
            return Err(GaiaError::syntax_error(
                format!("String index {} exceeds heap range", index),
                SourceLocation::default(),
            ));
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
}
