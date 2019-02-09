use std::fs::{File};
use std::io::{Read, Result, Seek, SeekFrom};
use std::path::PathBuf;
use image::{ImageBuffer, Rgba, RgbaImage};
use crate::assets::Asset;
use crate::palette::PALETTE;
use crate::utils::buf_to_le_u32;

// Asset header: 2 bytes + 2 bytes + 6 bytes
const UNKNOWN_LEN: usize = 6;
const HEADER_LEN: usize = 2 + 2 + UNKNOWN_LEN;

pub fn extract_img_asset(res_file: &mut File, dst_dir: &PathBuf, asset: &Asset) -> Result<bool> {
    let path = asset_path(dst_dir, asset);
    if path.is_file() {
        return Ok(false) // Skip file
    }

    // Jump to asset start
    res_file.seek(SeekFrom::Start(asset.offset))?;

    // Read 2 + 2 + 6 bytes of asset header
    let mut header = [0;HEADER_LEN];
    res_file.read(&mut header)?;

    // First two bytes is image width
    let width = buf_to_le_u32(&header[0..2]).unwrap();
    // Next two bytes is image height
    let height = buf_to_le_u32(&header[2..4]).unwrap();

    // Read image data
    let data_len = (asset.length as usize) - HEADER_LEN;
    let mut image_data = vec![0u8;data_len];
    res_file.read(&mut image_data);

    // Create image
    let mut img: RgbaImage = ImageBuffer::new(width, height);

    // Iterate over the coordinates and pixels of the image
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let src_pixel = (x + (y * width)) as usize;
        let color = image_data[src_pixel] as usize;
        if color == 0 {
            *pixel = Rgba([0, 0, 0, 0]);
        } else {
            *pixel = Rgba([
                PALETTE[(color * 3)],
                PALETTE[(color * 3) + 1],
                PALETTE[(color * 3) + 2],
                255
            ]);
        }
    }

    // Save image file
    img.save(path)?;

    Ok(true)
}

fn asset_path(dst_dir: &PathBuf, asset: &Asset) -> PathBuf {
    let mut path = dst_dir.to_path_buf();
    path.push(&asset.name);
    path.set_extension("PNG");
    path
}