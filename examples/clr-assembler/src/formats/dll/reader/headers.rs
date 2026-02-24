use crate::program::{ClrHeader, MetadataHeader, StreamHeader};
use gaia_types::{GaiaError, SourceLocation};
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Read, Seek, SeekFrom};
use url::Url;
use super::DllReader;
use super::utils::rva_to_file_offset;

impl<'config, R: Read + Seek> DllReader<'config, R> {
    /// Finds and reads the CLR header.
    pub(super) fn find_and_read_clr_header(&mut self) -> Result<Option<ClrHeader>, GaiaError> {
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
            let file_offset = rva_to_file_offset(&pe_program, clr_dir.virtual_address)?;
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
    pub(super) fn read_metadata_header(&mut self, offset: u32) -> Result<MetadataHeader, GaiaError> {
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
    pub(super) fn read_stream_headers(&mut self, metadata_offset: u32) -> Result<Vec<StreamHeader>, GaiaError> {
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
}
