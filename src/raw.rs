use std::error::Error;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;

use super::directory::Asset;

pub fn extract_raw(
    res_file: &mut File, asset: &Asset, path: &mut PathBuf
) -> Result<bool, Box<dyn Error>> {
    // Add filename to path
    path.push(&asset.name);

    // If file already exists skip extraction
    if path.is_file() {
        return Ok(false);
    }

    // Jump to asset start, then read its length
    res_file.seek(SeekFrom::Start(asset.offset))?;
    let mut data = vec![0u8; asset.length as usize];
    res_file.read(&mut data)?;

    // Write file data
    let mut output = File::create(path)?;
    output.write_all(&data.as_mut_slice())?;

    Ok(true)
}