use std::fs::File;
use std::io::{Result};
use std::vec::Vec;
use crate::directories::Directory;

pub struct Asset {
    pub offset: u32,
    pub length: u32,
}

pub fn read_assets(f: &mut File, directories: &Vec<Directory>, a: &mut Vec<Asset>) -> Result<()> {
    for directory in directories {
        read_directory_assets(f, directory, a)?;
    }

    Ok(())
}

fn read_directory_assets(f: &mut File, directory: &Directory, a: &mut Vec<Asset>) -> Result<()> {
    let mut offset = directory.offset;
    let end = directory.offset + directory.length;
    while offset < end {
        offset = end;
    }
    Ok(())
}