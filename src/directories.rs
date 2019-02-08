use std::fs::File;
use std::io::{Read, Result, Seek, SeekFrom};
use std::vec::Vec;
use crate::utils::buf_to_le_u64;

pub struct Directory {
    pub offset: u64,
    pub length: u64,
}

pub fn find_directories(res_file: &mut File, directories: &mut Vec<Directory>) -> Result<()> {
    read_directory_header(res_file, directories)?;
    Ok(())
}

fn read_directory_header(res_file: &mut File, directories: &mut Vec<Directory>) -> Result<bool> {
    // Directory header is two 4 byte long unsigned little endian integers
    let mut header = [0; 8];
    let read_len = res_file.read(&mut header)?;
    if read_len != 8 {
        return Ok(true)
    }

    // First 4 bytes is offset of directory data
    let offset = buf_to_le_u64(&header[0..4]).unwrap();
    // Second number is length of the data
    let length = buf_to_le_u64(&header[4..8]).unwrap();

    let directory = Directory { offset, length };
    directories.push(directory);

    // Hop to directory end
    let jump_to = offset + length;
    res_file.seek(SeekFrom::Start(jump_to))?;

    // Keep reading directories
    Ok(false)
}