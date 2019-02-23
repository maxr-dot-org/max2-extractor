use std::env::current_dir;
use std::error;
use std::fs::File;
use std::io::{Error, ErrorKind, Read};
use std::path::PathBuf;
use std::str;

const RES_FILE_HEADER: &str = "RES0";
const INVALID_HEADER_ERROR: &str = "Opened file is not a valid RES package";

pub fn open_res_file(file_name: &str) -> Result<File, Box<dyn error::Error>> {
    let path = get_file_path(file_name)?;
    let mut file = File::open(path)?;
    check_res_file_header(&mut file)?;
    Ok(file)
}

fn get_file_path(file_name: &str) -> Result<PathBuf, Error> {
    let mut path = current_dir()?;
    path.push(file_name);
    Ok(path)
}

fn check_res_file_header(
    res_file: &mut File
) -> Result<(), Box<dyn error::Error>> {
    // First 4 bytes should be "RES0" string
    let mut buffer = [0; 4];
    res_file.read(&mut buffer)?;

    let header = str::from_utf8(&buffer)?;
    if header != RES_FILE_HEADER {
        let err = Error::new(ErrorKind::InvalidData, INVALID_HEADER_ERROR);
        return Err(Box::new(err));
    }

    Ok(())
}
