use std::error;
use std::fs::File;
use std::io::{Error, ErrorKind, Read, Seek, SeekFrom};

use super::utils::buf_to_le_u32;

// Directory header starts at 6 byte in the file
const HEADER_OFFSET: u64 = 6;
// Header read error
const INVALID_HEADER_ERROR: &str = "Failed to read directory header";

pub struct Directory {
    pub offset: u64,
    pub length: u64,
}

pub fn get_directory(
    res_file: &mut File
) -> Result<Directory, Box<dyn error::Error>> {
    // Seek to header offset
    res_file.seek(SeekFrom::Start(HEADER_OFFSET))?;

    // Directory header is two 4 byte long unsigned little endian integers
    let mut header = [0; 8];
    if res_file.read(&mut header)? != 8 {
        let err = Error::new(ErrorKind::InvalidData, INVALID_HEADER_ERROR);
        return Err(Box::new(err));
    }

    // First 4 bytes store directory's offset
    let offset = buf_to_le_u32(&header[0..4])? as u64;
    // Next 4 bytes store directory's length
    let length = buf_to_le_u32(&header[4..8])? as u64;

    Ok(Directory { offset, length })
}