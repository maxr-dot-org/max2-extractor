use std::error::Error;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::vec::Vec;
use image::{ImageBuffer, Rgb, RgbImage};

use super::directory::Asset;
use super::utils::{buf_to_le_i32, buf_to_le_u32};

// Asset header: 4 bytes + 2 bytes + 2 bytes + 3*256 bytes of palette
const UNKNOWN_LEN: usize = 4;
const PALETTE_LEN: usize = 3 * 256;
const HEADER_LEN: usize = 2 + 2 + PALETTE_LEN;

pub fn extract_img_with_palette(
    res_file: &mut File, asset: &Asset, path: &mut PathBuf
) -> Result<bool, Box<dyn Error>> {
    // Add filename to path
    path.push(&asset.name);
    path.set_extension("PNG");

    // If file already exists skip extraction
    if path.is_file() {
        return Ok(false);
    }

    // Jump asset offset in file
    // its offset from dict header + 4 bytes of trash
    let offset = asset.offset + (UNKNOWN_LEN as u64);
    res_file.seek(SeekFrom::Start(offset))?;

    // Read 2 + 2 + (3 * 256) bytes of asset header
    let mut header = [0;HEADER_LEN];
    res_file.read(&mut header)?;

    // First two bytes is image width
    let width = buf_to_le_u32(&header[0..2])?;
    // Next two bytes is image height
    let height = buf_to_le_u32(&header[2..4])?;
    // Next 3*256 bytes is image palette
    let palette = &header[4..(PALETTE_LEN + 4)];

    // Decompress image data
    let data_len = (asset.length as usize) - HEADER_LEN - UNKNOWN_LEN;
    let image_data = decompress_img_data(res_file, data_len)?;

    // Create image from data
    let mut img: RgbImage = ImageBuffer::new(width, height);
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let src_pixel = (x + (y * width)) as usize;
        let palette_color = (image_data[src_pixel] as usize) * 3;
        let red = palette[palette_color];
        let green = palette[palette_color + 1];
        let blue = palette[palette_color + 2];
        *pixel = Rgb([red, green, blue]);
    }

    img.save(path)?;

    Ok(true)
}

fn decompress_img_data(
    res_file: &mut File, data_len: usize
) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut data = vec![0u8; data_len];
    res_file.read(&mut data)?;

    let mut position: usize = 0;
    let mut unpacked_data: Vec<u8> = Vec::new();

    while position < data_len {
        // Every compressed chunk starts with 2-bytes word
        let sword = buf_to_le_i32(&data[position..position+2])?;
        let start = position + 2;
        // If sword is positive, its number of uncompressed bytes
        if sword > 0 {
            let end = start + (sword as usize);
            unpacked_data.write(&data[start..end])?;
            position = end;
        } else {
            let mut repeat = sword.abs();
            let end = start + 1;
            while repeat > 0 {
                unpacked_data.write(&data[start..end])?;
                repeat -= 1;
            }
            position = end;
        }
    }

    Ok(unpacked_data)
}