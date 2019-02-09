use std::fs::{File};
use std::io::{Read, Result, Seek, SeekFrom, Write};
use std::path::PathBuf;
use crate::assets::Asset;

pub fn extract_txt_asset(res_file: &mut File, dst_dir: &PathBuf, asset: &Asset) -> Result<bool> {
    let path = asset_path(dst_dir, asset);
    if path.is_file() {
        return Ok(false) // Skip file
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

fn asset_path(dst_dir: &PathBuf, asset: &Asset) -> PathBuf {
    let mut path = dst_dir.to_path_buf();
    let file_name = format!("{}_{}", asset.type_, asset.name);
    path.push(&file_name);
    path.set_extension("TXT");
    path
}