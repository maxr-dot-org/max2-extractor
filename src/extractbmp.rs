use std::fs::{File};
use std::io::{Read, Result, Seek, SeekFrom};
use std::path::PathBuf;
use image::{ImageBuffer, Rgb, RgbImage};
use crate::assets::Asset;
use crate::decompress::decompress_data;
use crate::utils::buf_to_le_u32;

// Asset header: 4 bytes + 2 bytes + 2 bytes + 3*256 bytes of palette
const UNKNOWN_LEN: usize = 4;
const PALETTE_LEN: usize = 3 * 256;
const HEADER_LEN: usize = 2 + 2 + PALETTE_LEN;

pub fn extract_bmp_asset(res_file: &mut File, dst_dir: &PathBuf, asset: &Asset) -> Result<bool> {
    let path = asset_path(dst_dir, asset);
    if path.is_file() {
        return Ok(false) // Skip file
    }

    // Jump to asset start + 4 bytes of trash
    res_file.seek(SeekFrom::Start(asset.offset + (UNKNOWN_LEN as u64)))?;

    // Read 2 + 2 + (3 * 256) bytes of asset header
    let mut header = [0;HEADER_LEN];
    res_file.read(&mut header)?;

    // First two bytes is image width
    let width = buf_to_le_u32(&header[0..2]).unwrap();
    // Next two bytes is image height
    let height = buf_to_le_u32(&header[2..4]).unwrap();
    // Next 3*256 bytes is image palette
    let palette = &header[4..(PALETTE_LEN + 4)];

    // Decompress image data
    let data_len = (asset.length as usize) - HEADER_LEN - UNKNOWN_LEN;
    let image_data = decompress_data(res_file, data_len);

    // Create image
    let mut img: RgbImage = ImageBuffer::new(width, height);

    // Iterate over the coordinates and pixels of the image
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let src_pixel = (x + (y * width)) as usize;
        let palette_color = (image_data[src_pixel] as usize) * 3;
        let red = palette[palette_color];
        let green = palette[palette_color + 1];
        let blue = palette[palette_color + 2];
        *pixel = Rgb([red, green, blue]);
    }

    // Save image file
    img.save(path)?;

    Ok(true)
}

fn asset_path(dst_dir: &PathBuf, asset: &Asset) -> PathBuf {
    let mut path = dst_dir.to_path_buf();
    path.push(&asset.name);
    path.set_extension("PNG");  // PNG takes less space and is lossless too
    path
}