use std::env::current_dir;
use std::fs::{File, create_dir_all};
use std::io::{Read, Result, Seek, SeekFrom};
use std::path::PathBuf;
use std::str;
use std::vec::Vec;
use max2_extractor::assets::{Asset, find_assets};
use max2_extractor::directories::{Directory, find_directories};
use max2_extractor::extractbmp::extract_bmp_asset;
use max2_extractor::extractimg::extract_img_asset;
use max2_extractor::extractraw::extract_raw_asset;
use max2_extractor::extracttxt::extract_txt_asset;
use max2_extractor::palette::read_palettes;

const FILE_HEADER: &str = "RES0";
const RES_DIRNAME: &str = "res";
const CAF_DIRNAME: &str = "caf";

const ASSET_BMP: u32 = 1;
const ASSET_STR: u32 = 4;
const ASSET_IMG: u32 = 5;
const ASSET_TXT: u32 = 7;
const ASSET_WAV: u32 = 8;

fn main() -> Result<()> {
    extract_res().expect("Failed to extract MAX2.RES");
    extract_caf().expect("Failed to extract MAX2.CAF");
    Ok(())
}

fn extract_res() -> Result<()> {
    let res_path = max2_res_path();
    let mut assets: Vec<Asset> = Vec::new();
    let mut res_file = res0_file(res_path).unwrap();
    res0_assets(&mut res_file, &mut assets)?;

    // Create output directory
    let dst_dir = res_dst_path();
    if !dst_dir.is_dir() {
        create_dir_all(dst_dir.as_path())?
    }

    // Extract assets
    extract_res_assets(&mut res_file, &dst_dir, &assets).unwrap();

    Ok(())
}

fn extract_caf() -> Result<()> {
    let res_path = max2_caf_path();
    let mut assets: Vec<Asset> = Vec::new();
    let mut res_file = res0_file(res_path).unwrap();
    res0_assets(&mut res_file, &mut assets)?;

    // Create output directory
    let dst_dir = caf_dst_path();
    if !dst_dir.is_dir() {
        create_dir_all(dst_dir.as_path())?
    }

    // Extract assets
    extract_caf_assets(&mut res_file, &dst_dir, &assets).unwrap();

    Ok(())
}

fn max2_res_path() -> PathBuf {
    let mut path = current_dir().expect("Failed to find CHDIR");
    path.push("MAX2");
    path.set_extension("RES");
    path
}

fn res_dst_path() -> PathBuf {
    let mut path = current_dir().expect("Failed to find CHDIR");
    path.push(&RES_DIRNAME);
    path
}

fn max2_caf_path() -> PathBuf {
    let mut path = current_dir().expect("Failed to find CHDIR");
    path.push("MAX2");
    path.set_extension("CAF");
    path
}

fn caf_dst_path() -> PathBuf {
    let mut path = current_dir().expect("Failed to find CHDIR");
    path.push(&CAF_DIRNAME);
    path
}

fn res0_file(path: PathBuf) -> Result<File> {
    if !path.is_file() {
        let path_str = path.to_string_lossy().into_owned();
        let error_message = format!("Could not find: {}", path_str);
        panic!(error_message);
    }

    let path_str = path.to_string_lossy().into_owned();
    let error_message = format!("Could not find: {}", path_str);
    let mut res_file = File::open(path).expect(&error_message);
    check_file_header(&mut res_file).unwrap();

    Ok(res_file)
}

fn res0_assets(res_file: &mut File, assets: &mut Vec<Asset>) -> Result<()> {
    // Jump to 6th byte where assets directory starts
    res_file.seek(SeekFrom::Start(6)).unwrap();

    // Find directories
    let mut directories: Vec<Directory> = Vec::new();
    find_directories(res_file, &mut directories)?;

    // Find assets
    find_assets(res_file, &directories, assets)?;

    Ok(())
}

fn extract_res_assets(res_file: &mut File, dst_dir: &PathBuf, assets: &Vec<Asset>) -> Result<()> {
    let mut palettes: Vec<[u8; 768]> = Vec::new();
    read_palettes(res_file, &mut palettes);

    for asset in assets {
        if asset.type_ == ASSET_BMP {
            if extract_bmp_asset(res_file, &dst_dir, &asset)? {
                println!("Extracted: {}.PNG", asset.name);
            }
        } else if asset.type_ == ASSET_IMG {
            if extract_img_asset(res_file, &dst_dir, &asset, &palettes)? {
                println!("Extracted: {}.PNG", asset.name);
            }
        } else if asset.type_ == ASSET_STR || asset.type_ == ASSET_TXT {
            if extract_txt_asset(res_file, &dst_dir, &asset)? {
                println!("Extracted: {}.TXT", asset.name);
            }
        } else {
            if extract_raw_asset(res_file, &dst_dir, &asset)? {
                println!("Extracted: {}", asset.name);
            } else {
                println!("Skipped: {}", asset.name);
            }
        }
    }

    Ok(())
}

fn extract_caf_assets(res_file: &mut File, dst_dir: &PathBuf, assets: &Vec<Asset>) -> Result<()> {
    for asset in assets {if asset.type_ == ASSET_STR || asset.type_ == ASSET_TXT {
            if extract_txt_asset(res_file, &dst_dir, &asset)? {
                println!("Extracted: {}.TXT", asset.name);
            }
        }  else if asset.type_ == ASSET_WAV {
            // TODO: extract WAVs
            if extract_raw_asset(res_file, &dst_dir, &asset)? {
                println!("Extracted: {}", asset.name);
            } else {
                println!("Skipped: {}", asset.name);
            }
        } else {
            if extract_raw_asset(res_file, &dst_dir, &asset)? {
                println!("Extracted: {}", asset.name);
            } else {
                println!("Skipped: {}", asset.name);
            }
        }
    }

    Ok(())
}

fn check_file_header(res_file: &mut File) -> Result<()> {
    // First 4 bytes should be "RES0" string
    let mut buffer = [0; 4];
    res_file.read(&mut buffer)?;

    let header = str::from_utf8(&buffer).expect("Could not read header");
    assert_eq!(header, FILE_HEADER, "Unrecognized file type");

    Ok(())
}
