use std::env::current_dir;
use std::fs::File;
use std::io::{Read, Result, Seek, SeekFrom};
use std::path::PathBuf;
use std::str;
use std::vec::Vec;
use max2_extractor::assets::{Asset, find_assets};
use max2_extractor::directories::{Directory, find_directories};

const FILE_NAME: &str = "MAX2";
const FILE_EXT: &str = "RES";
const FILE_HEADER: &str = "RES0";

const ASSET_IMG: u32 = 1;

fn main() -> Result<()> {
    let max2_res = max2_res_path();

    if !max2_res.is_file() {
        let max2_res_str = max2_res.to_string_lossy().into_owned();
        let error_message = format!("Could not find: {}", max2_res_str);
        panic!(error_message);
    }

    let mut res_file = File::open(max2_res).expect("Could not open MAX2.RES");
    check_file_header(&mut res_file)?;

    // Jump to 6th byte where assets directory starts
    res_file.seek(SeekFrom::Start(6))?;

    // Find directories
    let mut directories: Vec<Directory> = Vec::new();
    find_directories(&mut res_file, &mut directories)?;

    // Find assets
    let mut assets: Vec<Asset> = Vec::new();    
    find_assets(&mut res_file, &directories, &mut assets)?;

    // Extract assets
    for asset in assets {
        if asset.type_ == ASSET_IMG {
            println!("{} (img)", asset.name);
        } else {
            println!("{} ({})", asset.name, asset.type_);
        }
    }

    Ok(())
}

fn max2_res_path() -> PathBuf {
    let mut path = current_dir().expect("Failed to find CHDIR");
    path.push(&FILE_NAME);
    path.set_extension(&FILE_EXT);
    path
}

fn check_file_header(res_file: &mut File) -> Result<()> {
    // First 4 bytes should be "RES0" string
    let mut buffer = [0; 4];
    res_file.read(&mut buffer)?;

    let header = str::from_utf8(&buffer).expect("Could not read header");
    assert_eq!(header, FILE_HEADER, "Unrecognized file type");

    Ok(())
}
