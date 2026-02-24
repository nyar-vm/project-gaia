use gaia_types::{GaiaError, SourceLocation};
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Read, Seek, SeekFrom};
use url::Url;
use pe_assembler::types::PeProgram;

/// Reads a type def or ref index (2 or 4 bytes depending on size).
pub fn read_type_def_or_ref_index<R: Read>(cursor: &mut R, idx_size: u32) -> Result<u32, GaiaError> {
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

/// Reads a heap index (2 or 4 bytes depending on size).
pub fn read_heap_index<R: Read>(cursor: &mut R, idx_size: u32) -> Result<u32, GaiaError> {
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

/// Simple check if a subslice exists.
pub fn find_subslice(haystack: &[u8], needle: &[u8]) -> bool {
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

/// Converts RVA (Relative Virtual Address) to file offset.
pub fn rva_to_file_offset(pe_program: &PeProgram, rva: u32) -> Result<u32, GaiaError> {
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
