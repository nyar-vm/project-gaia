use byteorder::{LittleEndian, ReadBytesExt};
use gaia_types::{GaiaDiagnostics, GaiaError, SourceLocation};
use pe_assembler::types::PeReader;
use std::io::{Read, Seek};
use url::Url;
use super::DllReader;

impl<'config, R: Read + Seek> DllReader<'config, R> {
    /// Converts RVA (Relative Virtual Address) to file offset.
    pub(crate) fn rva_to_file_offset(&mut self, rva: u32) -> Result<u32, GaiaError> {
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
        Err(GaiaError::syntax_error(
            format!("Unable to convert RVA 0x{:x} to file offset", rva),
            SourceLocation::default(),
        ))
    }
}

/// Reads a heap index (2 or 4 bytes depending on size).
pub(crate) fn read_type_def_or_ref_index<R: Read>(cursor: &mut R, idx_size: u32) -> Result<u32, GaiaError> {
    if idx_size == 2 {
        cursor
            .read_u16::<LittleEndian>()
            .map(|v| v as u32)
            .map_err(|e| GaiaError::io_error(e, Url::parse("memory://type_def_or_ref_index").unwrap()))
    } else {
        cursor
            .read_u32::<LittleEndian>()
            .map_err(|e| GaiaError::io_error(e, Url::parse("memory://type_def_or_ref_index").unwrap()))
    }
}

pub(crate) fn read_heap_index<R: Read>(cursor: &mut R, idx_size: u32) -> Result<u32, GaiaError> {
    if idx_size == 2 {
        cursor
            .read_u16::<LittleEndian>()
            .map(|v| v as u32)
            .map_err(|e| GaiaError::io_error(e, Url::parse("memory://heap_index").unwrap()))
    } else if idx_size == 4 {
        cursor
            .read_u32::<LittleEndian>()
            .map_err(|e| GaiaError::io_error(e, Url::parse("memory://heap_index").unwrap()))
    } else {
        Err(GaiaError::syntax_error(
            "Illegal heap index size".to_string(),
            SourceLocation::default(),
        ))
    }
}

/// Reads a string from the #Strings heap (null-terminated UTF-8).
pub(crate) fn read_string_from_heap(
    pe_data: &[u8],
    strings_start: u32,
    strings_size: u32,
    index: u32,
) -> Result<String, GaiaError> {
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
pub(crate) fn find_subslice(haystack: &[u8], needle: &[u8]) -> bool {
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
