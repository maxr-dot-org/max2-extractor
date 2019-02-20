use std::env::current_dir;
use std::error::Error;
use std::fs::{File, create_dir_all};
use std::io;
use std::iter::Iterator;
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::vec::Vec;

mod directory;
mod palette;
mod resfile;
mod utils;

use directory::{Directory, get_directory};
use palette::render_palette;
use resfile::open_res_file;
use utils::buf_to_le_u32;

fn main() -> Result<(), Box<dyn Error>> {
    let mut max2_res = match open_res_file("MAX2.RES") {
        Ok(file) => file,
        Err(error) => {
            panic!("Failed to open MAX2.RES: {:?}", error)
        },
    };

    let mut max2_caf = match open_res_file("MAX2.CAF") {
        Ok(file) => file,
        Err(error) => {
            panic!("Failed to open MAX2.CAF: {:?}", error)
        },
    };

    let dst_path = match get_dst_path() {
        Ok(dst_path) => dst_path,
        Err(error) => {
            panic!("Failed to find current directory: {:?}", error)
        },
    };

    match create_dir_all(&dst_path) {
        Ok(_) => (),
        Err(error) => {
            panic!("Failed to create \"extracted\" directory: {:?}", error)
        },
    };

    match extract_max2_res(&dst_path, &mut max2_res) {
        Ok(_) => (),
        Err(error) => {
            panic!("Failed to extract MAX2.RES: {:?}", error)
        },
    };

    match extract_max2_caf(&dst_path, &mut max2_caf) {
        Ok(_) => (),
        Err(error) => {
            panic!("Failed to extract MAX2.CAF: {:?}", error)
        },
    };

    Ok(())
}

fn get_dst_path() -> Result<PathBuf, io::Error> {
    let mut path = current_dir()?;
    path.push("extracted");
    Ok(path)
}

fn extract_max2_res(
    dst_path: &PathBuf, res_file: &mut File
) -> Result<(), Box<dyn Error>> {
    println!("Extracting MAX2.RES...");

    let directory = get_directory(res_file)?;
    let palettes = get_palettes(res_file, &directory)?;

    extract_max2_res_palettes(dst_path, &palettes)?;

    Ok(())
}

fn extract_max2_res_palettes(
    dst_path: &PathBuf, palettes: &Vec<[u8; 768]>
) -> Result<(), Box<dyn Error>> {
    println!("Extracting {} palettes...", palettes.len());

    let mut dst_path = dst_path.to_path_buf();
    dst_path.push("palette");
    create_dir_all(&dst_path)?;

    for (i, &palette) in palettes.into_iter().enumerate() {
        let mut palette_path = dst_path.to_path_buf();
        palette_path.push(i.to_string().as_str());
        palette_path.set_extension("PNG");
        render_palette(&palette_path, &palette)?;
        println!("Extracted palette #{}", i);
    }

    Ok(())
}

fn extract_max2_caf(
    dst_path: &PathBuf, res_file: &mut File
) -> Result<(), Box<dyn Error>> {
    println!("Extracting MAX2.CAF...");

    let directory = get_directory(res_file)?;
    println!("{} {}", directory.offset, directory.length);

    Ok(())
}

pub fn get_palettes(
    res_file: &mut File, directory: &Directory
) -> Result<Vec<[u8; 768]>, Box<dyn Error>> {
    // Palettes start right after directory's header
    let palettes_offset = directory.offset + directory.length;
    res_file.seek(SeekFrom::Start(palettes_offset))?;

    // Palettes list starts from 2 bytes with palettes count
    let mut palettes_count = [0; 2];
    res_file.read(&mut palettes_count)?;
    let mut palettes_count = buf_to_le_u32(&palettes_count)? as usize;

    // Every palette is 3 * 256 bytes
    let mut palettes: Vec<[u8; 768]> = Vec::new();
    while palettes.len() < palettes_count {
        let mut palette = [0; 768];
        res_file.read(&mut palette)?;
        palettes.push(palette);
    }

    Ok(palettes)
}
