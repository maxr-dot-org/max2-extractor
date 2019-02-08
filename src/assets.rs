use std::fs::File;
use std::io::{Read, Result, Seek, SeekFrom};
use std::str;
use std::vec::Vec;
use crate::directories::Directory;
use crate::utils::{buf_to_le_u32, buf_to_le_u64};

pub struct Asset {
    pub offset: u64,
    pub length: u64,
    pub type_: u32,
    pub name: String,
}

pub fn find_assets(res_file: &mut File, directories: &Vec<Directory>, assets: &mut Vec<Asset>) -> Result<()> {
    for directory in directories {
        find_directory_assets(res_file, directory, assets)?;
    }
    Ok(())
}

fn find_directory_assets(res_file: &mut File, directory: &Directory, assets: &mut Vec<Asset>) -> Result<()> {
    let mut offset = directory.offset;
    let end = directory.offset + directory.length;
    while offset > 0 && offset < end {
        offset = read_asset_header(offset, res_file, assets)?;
    }
    Ok(())
}

fn read_asset_header(start: u64, res_file: &mut File, assets: &mut Vec<Asset>) -> Result<u64> {
    res_file.seek(SeekFrom::Start(start))?;

    // Constant part of asset header is 13 bytes
    let mut header = [0; 13];
    let read_len = res_file.read(&mut header)?;
    if read_len != 13 {
        return Ok(0)
    }

    // First four bytes is asset offset
    let offset = buf_to_le_u64(&header[0..4]).unwrap();
    // Second four bytes is asset data length
    let length = buf_to_le_u64(&header[4..8]).unwrap();
    // Last four bytes is asset type
    let type_ = buf_to_le_u32(&header[8..12]).unwrap();
    // Final byte is asset name length
    let name_len = header[12] as u64;

    // Next name_len bytes is name
    let mut name = vec![0u8; name_len as usize];
    res_file.read(&mut name)?;

    let name_str = str::from_utf8(&name).unwrap();
    let name = String::from(name_str);

    let asset = Asset { offset, length, type_, name };
    assets.push(asset);

    let header_len: u64 = name_len + 13;
    Ok(start + header_len)
}