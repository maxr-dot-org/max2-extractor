use std::env::current_dir;
use std::fs::File;
use std::io::{Read, Result, Seek, SeekFrom};
use std::path::PathBuf;
use std::str;
use std::vec::Vec;
use max2_extractor::assets::{Asset, read_assets};
use max2_extractor::directories::{Directory, read_directories};

const FILE_NAME: &str = "MAX2";
const FILE_EXT: &str = "RES";
const FILE_HEADER: &str = "RES0";

fn main() -> Result<()> {
    let max2_res = max2_res_path();

    if !max2_res.is_file() {
        let max2_res_str = max2_res.to_string_lossy().into_owned();
        let error_message = format!("Could not find: {}", max2_res_str);
        panic!(error_message);
    }

    let mut f = File::open(max2_res).expect("Could not open MAX2.RES");
    check_file_header(&mut f)?;

    // Jump to 6th byte where first assets directory starts
    f.seek(SeekFrom::Start(6))?;

    let mut directories: Vec<Directory> = Vec::new();
    read_directories(&mut f, &mut directories)?;

    let mut assets: Vec<Asset> = Vec::new();    
    read_assets(&mut f, &directories, &mut assets)?;

    Ok(())
}

fn max2_res_path() -> PathBuf {
    let mut path = current_dir().expect("Failed to find CHDIR");
    path.push(&FILE_NAME);
    path.set_extension(&FILE_EXT);
    path
}

fn check_file_header(f: &mut File) -> Result<()> {
    // First 4 bytes should be "RES0" string
    let mut buffer = [0; 4];
    f.read(&mut buffer)?;

    let header = str::from_utf8(&buffer).expect("Could not read header");
    assert_eq!(header, FILE_HEADER, "Unrecognized file type");

    Ok(())
}
