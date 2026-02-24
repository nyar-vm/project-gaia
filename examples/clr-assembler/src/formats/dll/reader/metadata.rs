use crate::program::StreamHeader;
use byteorder::{LittleEndian, ReadBytesExt};
use gaia_types::{GaiaError, SourceLocation};
use pe_assembler::types::PeReader;
use std::io::{Read, Seek, SeekFrom};
use url::Url;
use super::DllReader;

impl<'config, R: Read + Seek> DllReader<'config, R> {
    /// Parses the metadata.
    pub(crate) fn parse_metadata(&mut self) -> Result<(), GaiaError> {
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

    /// Reads stream header information.
    pub(crate) fn read_stream_headers(&mut self, metadata_offset: u32) -> Result<Vec<StreamHeader>, GaiaError> {
        let mut stream_headers = Vec::new();

        if let Some(ref metadata_header) = self.metadata_header {
            let mut cursor = self.get_viewer();
            // Calculate starting position of stream headers: skip fixed part of metadata header (20 bytes) and version string
            let stream_start_offset = metadata_offset + 20 + metadata_header.version_length;
            cursor.seek(SeekFrom::Start(stream_start_offset as u64)).map_err(|e| {
                GaiaError::io_error(e, Url::parse("memory://stream_headers").unwrap())
            })?;

            // Read header information for each stream
            for _ in 0..metadata_header.streams {
                let offset = cursor.read_u32::<LittleEndian>().map_err(|e| {
                    GaiaError::io_error(e, Url::parse("memory://stream_headers").unwrap())
                })?;
                let size = cursor.read_u32::<LittleEndian>().map_err(|e| {
                    GaiaError::io_error(e, Url::parse("memory://stream_headers").unwrap())
                })?;

                // Read stream name (null-terminated string)
                let mut name_bytes = Vec::new();
                loop {
                    let byte = cursor.read_u8().map_err(|e| {
                        GaiaError::io_error(e, Url::parse("memory://stream_headers").unwrap())
                    })?;
                    if byte == 0 {
                        break;
                    }
                    name_bytes.push(byte);
                }
                let name = String::from_utf8_lossy(&name_bytes).to_string();

                // Align to 4-byte boundary
                let current_pos = cursor.stream_position().map_err(|e| {
                    GaiaError::io_error(e, Url::parse("memory://stream_headers").unwrap())
                })?;
                let aligned_pos = (current_pos + 3) & !3;
                cursor.seek(SeekFrom::Start(aligned_pos)).map_err(|e| {
                    GaiaError::io_error(e, Url::parse("memory://stream_headers").unwrap())
                })?;

                stream_headers.push(StreamHeader { offset, size, name });
            }
        }

        Ok(stream_headers)
    }

    /// Reads raw data of the strings heap.
    pub(crate) fn read_strings_heap_data(&mut self, strings_start: u32, strings_size: u32) -> Result<Vec<u8>, GaiaError> {
        let mut reader = self.get_viewer();
        reader.seek(SeekFrom::Start(strings_start as u64)).map_err(|e| {
            GaiaError::io_error(e, Url::parse("memory://strings_heap").unwrap())
        })?;

        let mut buffer = vec![0u8; strings_size as usize];
        reader.read_exact(&mut buffer).map_err(|e| {
            GaiaError::io_error(e, Url::parse("memory://strings_heap").unwrap())
        })?;

        Ok(buffer)
    }

    /// Helper method to read a string from the strings heap.
    pub(crate) fn read_string_from_strings_heap(
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
        let viewer = self.get_viewer();
        viewer.seek(SeekFrom::Start(base as u64)).map_err(|e| {
            GaiaError::io_error(e, Url::parse("memory://strings_heap").unwrap())
        })?;

        // Read null-terminated string
        let mut bytes = Vec::new();
        loop {
            let byte = viewer.read_u8().map_err(|e| {
                GaiaError::io_error(e, Url::parse("memory://strings_heap").unwrap())
            })?;
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
