use std::env::current_dir;
use std::error::Error;
use std::fs::{File, create_dir_all};
use std::io;
use std::iter::Iterator;
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::vec::Vec;
use glob::glob;

mod acm;
mod directory;
mod img;
mod imgcontainer;
mod imgmonocontainer;
mod imgwithpalette;
mod palette;
mod raw;
mod resfile;
mod text;
mod utils;
mod wld;

use acm::extract_acm;
use directory::{Asset, Directory, get_directory};
use img::extract_img;
use imgcontainer::extract_img_container;
use imgmonocontainer::extract_img_mono_container;
use imgwithpalette::extract_img_with_palette;
use palette::render_palette;
use raw::extract_raw;
use resfile::open_res_file;
use text::extract_txt;
use utils::buf_to_le_u32;
use wld::extract_wld;

// const ASSET_METADATA: u32 = 0; - Not used in M.A.X 2
const ASSET_IMG_WITH_PALETTE: u32 = 1;
const ASSET_IMG_CONTAINER: u32 = 2;
const ASSET_IMG_MONO_CONTAINER: u32 = 3;
const ASSET_STR: u32 = 4;
const ASSET_IMG: u32 = 5;
const ASSET_TXT: u32 = 7;
const ASSET_ACM: u32 = 8;

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

    match extract_wlds(&dst_path) {
        Ok(_) => (),
        Err(error) => {
            panic!("Failed to extract *.WLD: {:?}", error)
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

    let mut dst_path = dst_path.to_path_buf();
    dst_path.push("res");

    extract_max2_res_palettes(&dst_path, &palettes)?;
    extract_assets(&dst_path, res_file, &palettes, &directory.assets)?;

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
        if render_palette(&palette_path, &palette)? {
            println!("Extracted palette #{}", i);
        }
    }

    Ok(())
}

fn extract_max2_caf(
    dst_path: &PathBuf, res_file: &mut File
) -> Result<(), Box<dyn Error>> {
    println!("Extracting MAX2.CAF...");

    let directory = get_directory(res_file)?;

    let mut dst_path = dst_path.to_path_buf();
    dst_path.push("caf");

    for asset in directory.assets {
        // Assert that directory for type exists
        let mut dst_type_path = dst_path.to_path_buf();
        dst_type_path.push(asset.type_.to_string());
        create_dir_all(&dst_type_path)?;

        // Extract asset using type based algorithm
        match asset.type_ {
            ASSET_STR | ASSET_TXT => {
                if extract_txt(res_file, &asset, &mut dst_type_path)? {
                    println!("Extracted {}", asset.name)
                }
            },
            ASSET_ACM => {
                if extract_acm(res_file, &asset, &mut dst_type_path)? {
                    println!("Extracted {}", asset.name)
                }
            },
            _ => {
                if extract_raw(res_file, &asset, &mut dst_type_path)? {
                    println!("Extracted {}", asset.name)
                }
            },
        }
    }

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
    let palettes_count = buf_to_le_u32(&palettes_count)? as usize;

    // Every palette is 3 * 256 bytes
    let mut palettes: Vec<[u8; 768]> = Vec::new();
    while palettes.len() < palettes_count {
        let mut palette = [0; 768];
        res_file.read(&mut palette)?;
        palettes.push(palette);
    }

    Ok(palettes)
}

pub fn extract_assets(
    dst_path: &PathBuf,
    res_file: &mut File,
    palettes: &Vec<[u8; 768]>,
    assets: &Vec<Asset>
) -> Result<(), Box<dyn Error>> {
    for asset in assets {
        // Assert that directory for type exists
        let mut dst_type_path = dst_path.to_path_buf();
        dst_type_path.push(asset.type_.to_string());
        create_dir_all(&dst_type_path)?;

        // Extract asset using type based algorithm
        match asset.type_ {
            ASSET_IMG_WITH_PALETTE => {
                if extract_img_with_palette(res_file, &asset, &mut dst_type_path)? {
                    println!("Extracted {}", asset.name)
                }
            },
            ASSET_IMG_CONTAINER => {
                if extract_img_container(res_file, &palettes, &asset, &mut dst_type_path)? {
                    println!("Extracted {}", asset.name)
                }
            },
            ASSET_IMG_MONO_CONTAINER => {
                if extract_img_mono_container(res_file, &asset, &mut dst_type_path)? {
                    println!("Extracted {}", asset.name)
                }
            },
            ASSET_IMG => {
                if extract_img(res_file, &palettes, &asset, &mut dst_type_path)? {
                    println!("Extracted {}", asset.name)
                }
            },
            ASSET_STR | ASSET_TXT => {
                if extract_txt(res_file, &asset, &mut dst_type_path)? {
                    println!("Extracted {}", asset.name)
                }
            },
            _ => {
                if extract_raw(res_file, &asset, &mut dst_type_path)? {
                    println!("Extracted {}", asset.name)
                }
            },
        }
    }

    Ok(())
}

fn extract_wlds(dst_path: &PathBuf) -> Result<(), Box<dyn Error>> {
    // Assert that directory for type exists
    let mut dst_type_path = dst_path.to_path_buf();
    dst_type_path.push("wld");
    create_dir_all(&dst_type_path)?;

    // Iterate WLD files in chdir
    for wld_file in glob("*.WLD")? {
        let wld_file = wld_file?;
        let wld_file = wld_file.file_name();
        if wld_file.is_some() {
            let wld_file = wld_file.unwrap();
            let mut wld_path = current_dir()?;
            wld_path.push(wld_file);
            if wld_path.is_file() {
                println!("Extracting {}...", wld_file.to_string_lossy());
                extract_wld(&mut wld_path, &mut dst_type_path)?;
            }
        }
    }

    Ok(())
}