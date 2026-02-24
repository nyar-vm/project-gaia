use crate::program::{ClrHeader, MetadataHeader};
use byteorder::{LittleEndian, ReadBytesExt};
use gaia_types::{GaiaError, SourceLocation};
use pe_assembler::types::PeReader;
use std::io::{Read, Seek, SeekFrom};
use url::Url;
use super::DllReader;

impl<'config, R: Read + Seek> DllReader<'config, R> {
    /// Parses the CLR header.
    pub(crate) fn parse_clr_header(&mut self) -> Result<(), GaiaError> {
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

    /// Finds and reads the CLR header from the PE data directory.
    pub(crate) fn find_and_read_clr_header(&mut self) -> Result<Option<ClrHeader>, GaiaError> {
        // PE header's data directory 14 is the CLR Runtime Header
        let pe_header = self.get_pe_header()?.clone();
        if pe_header.data_directories.len() < 15 {
            return Ok(None);
        }

        let clr_dir = pe_header.data_directories[14];
        if clr_dir.virtual_address == 0 || clr_dir.size == 0 {
            return Ok(None);
        }

        // Convert RVA to file offset
        let file_offset = self.rva_to_file_offset(clr_dir.virtual_address)?;
        let mut cursor = self.get_viewer();
        cursor.seek(SeekFrom::Start(file_offset as u64)).map_err(|e| {
            GaiaError::io_error(e, Url::parse("memory://clr_header").unwrap())
        })?;

        // Read CLR header (fixed 72 bytes)
        let cb = cursor.read_u32::<LittleEndian>().map_err(|e| {
            GaiaError::io_error(e, Url::parse("memory://clr_header").unwrap())
        })?;
        let major_runtime_version = cursor.read_u16::<LittleEndian>().map_err(|e| {
            GaiaError::io_error(e, Url::parse("memory://clr_header").unwrap())
        })?;
        let minor_runtime_version = cursor.read_u16::<LittleEndian>().map_err(|e| {
            GaiaError::io_error(e, Url::parse("memory://clr_header").unwrap())
        })?;
        let metadata_rva = cursor.read_u32::<LittleEndian>().map_err(|e| {
            GaiaError::io_error(e, Url::parse("memory://clr_header").unwrap())
        })?;
        let metadata_size = cursor.read_u32::<LittleEndian>().map_err(|e| {
            GaiaError::io_error(e, Url::parse("memory://clr_header").unwrap())
        })?;
        let flags = cursor.read_u32::<LittleEndian>().map_err(|e| {
            GaiaError::io_error(e, Url::parse("memory://clr_header").unwrap())
        })?;
        let entry_point_token = cursor.read_u32::<LittleEndian>().map_err(|e| {
            GaiaError::io_error(e, Url::parse("memory://clr_header").unwrap())
        })?;

        Ok(Some(ClrHeader {
            cb,
            major_runtime_version,
            minor_runtime_version,
            metadata_rva,
            metadata_size,
            flags,
            entry_point_token,
        }))
    }

    /// Reads the metadata header.
    pub(crate) fn read_metadata_header(&mut self, offset: u32) -> Result<MetadataHeader, GaiaError> {
        let mut cursor = self.get_viewer();
        cursor.seek(SeekFrom::Start(offset as u64)).map_err(|e| {
            GaiaError::io_error(e, Url::parse("memory://metadata_header").unwrap())
        })?;

        // Fixed-length fields
        let signature = cursor.read_u32::<LittleEndian>().map_err(|e| {
            GaiaError::io_error(e, Url::parse("memory://metadata_header").unwrap())
        })?;
        let major_version = cursor.read_u16::<LittleEndian>().map_err(|e| {
            GaiaError::io_error(e, Url::parse("memory://metadata_header").unwrap())
        })?;
        let minor_version = cursor.read_u16::<LittleEndian>().map_err(|e| {
            GaiaError::io_error(e, Url::parse("memory://metadata_header").unwrap())
        })?;
        let reserved = cursor.read_u32::<LittleEndian>().map_err(|e| {
            GaiaError::io_error(e, Url::parse("memory://metadata_header").unwrap())
        })?;
        let version_length = cursor.read_u32::<LittleEndian>().map_err(|e| {
            GaiaError::io_error(e, Url::parse("memory://metadata_header").unwrap())
        })?;

        // Read version string (variable length)
        let mut version_bytes = vec![0u8; version_length as usize];
        cursor.read_exact(&mut version_bytes).map_err(|e| {
            GaiaError::io_error(e, Url::parse("memory://metadata_header").unwrap())
        })?;
        let version_string = String::from_utf8_lossy(&version_bytes)
            .trim_end_matches('\0')
            .to_string();

        // Read remaining fixed-length fields
        let flags = cursor.read_u16::<LittleEndian>().map_err(|e| {
            GaiaError::io_error(e, Url::parse("memory://metadata_header").unwrap())
        })?;
        let streams = cursor.read_u16::<LittleEndian>().map_err(|e| {
            GaiaError::io_error(e, Url::parse("memory://metadata_header").unwrap())
        })?;

        Ok(MetadataHeader {
            signature,
            major_version,
            minor_version,
            reserved,
            version_length,
            version_string,
            flags,
            streams,
        })
    }
}
