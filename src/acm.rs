use std::error::Error;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;

use super::directory::Asset;
use super::utils::{buf_to_le_u32};

pub fn extract_acm(
    res_file: &mut File, asset: &Asset, path: &mut PathBuf
) -> Result<bool, Box<dyn Error>> {
    // Add filename to path
    path.push(&asset.name);
    path.set_extension("ACM");

    // If file already exists skip extraction
    if path.is_file() {
        return Ok(false);
    }

    // Jump to asset start
    res_file.seek(SeekFrom::Start(asset.offset))?;

    // First four bytes of asset is its length
    let mut length = [0;4];
    res_file.read(&mut length)?;
    let length = buf_to_le_u32(&length)? as u64;

    // Move to audio data start
    res_file.seek(SeekFrom::Start(asset.offset + asset.length - length))?;

    let mut data = vec![0u8; length as usize];
    res_file.read(&mut data)?;

    // Write file data
    let mut output = File::create(path)?;
    output.write_all(&data.as_mut_slice())?;

    Ok(true)
}