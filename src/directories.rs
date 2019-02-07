use std::fs::File;
use std::io::{Read, Result, Seek, SeekFrom};
use std::vec::Vec;
use crate::utils::{buf_to_le_u64};

pub struct Directory {
    pub offset: u64,
    pub length: u64,
}

pub fn read_directories(f: &mut File, d: &mut Vec<Directory>) -> Result<()> {
    let mut done = false;
    while !done {
        done = read_directory(f, d)?;
    }
    Ok(())
}

fn read_directory(f: &mut File, d: &mut Vec<Directory>) -> Result<bool> {
    // Directory header is two 4 byte long unsigned little endian integers
    // First number is offset where data starts
    let mut offset = [0; 4];
    let read_len = f.read(&mut offset)?;
    if read_len != 4 {
        return Ok(false)
    }

    // Second number is length of the data
    let mut length = [0; 4];
    f.read(&mut length)?;

    let offset = buf_to_le_u64(&offset).unwrap();
    let length = buf_to_le_u64(&length).unwrap();
    let directory = Directory { offset, length };
    d.push(directory);

    // Hop to directory end
    let jump_to = offset + length;
    f.seek(SeekFrom::Start(jump_to))?;

    // Keep reading directories
    Ok(true)
}