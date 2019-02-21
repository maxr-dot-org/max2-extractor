use std::error;
use std::fs::File;
use std::io::{Error, ErrorKind, Read, Seek, SeekFrom};
use std::str;
use std::vec::Vec;

use super::utils::{buf_to_le_u32, buf_to_le_u64};

// Directory header starts at 6 byte in the file
const HEADER_OFFSET: u64 = 6;
// Header read error
const INVALID_HEADER_ERROR: &str = "Failed to read directory header";

pub struct Asset {
    pub offset: u64,
    pub length: u64,
    pub type_: u32,
    pub name: String,
}

pub struct Directory {
    pub offset: u64,
    pub length: u64,
    pub assets: Vec<Asset>,
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
    // Use those to read directory's assets
    let assets = get_directory_assets(res_file, offset, length)?;

    Ok(Directory { offset, length, assets })
}

fn get_directory_assets(
    res_file: &mut File, offset: u64, length: u64
) -> Result<Vec<Asset>, Box<dyn error::Error>> {
    // Seek to directory offset
    res_file.seek(SeekFrom::Start(offset))?;
    // Read dictionary length
    let mut headers = vec![0u8; length as usize];
    res_file.read(&mut headers)?;

    // Create empty assets list
    let mut assets: Vec<Asset> = Vec::new();
    // Keep extracting data from assets headers
    while !headers.is_empty() {
        assets.push(get_asset_from_headers(&mut headers)?);
    }

    Ok(assets)
}

fn get_asset_from_headers(
    headers: &mut Vec<u8>
) -> Result<Asset, Box<dyn error::Error>> {
    // First 13 bytes of asset header are constant
    let header: Vec<_> = headers.drain(..13).collect();
    // Unpack asset header into individual parts
    // First four bytes is asset offset
    let offset = buf_to_le_u64(&header[0..4]).unwrap();
    // Second four bytes is asset data length
    let length = buf_to_le_u64(&header[4..8]).unwrap();
    // Last four bytes is asset type
    let type_ = buf_to_le_u32(&header[8..12]).unwrap();
    // Final byte is asset's name length
    let name_len = header[12] as usize;

    // Read asset name
    let name: Vec<_> = headers.drain(..name_len).collect();
    let name = str::from_utf8(&name)?;
    let name = String::from(name);

    Ok(Asset { offset, length, type_, name })
}