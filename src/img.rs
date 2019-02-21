use std::error::Error;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::vec::Vec;
use image::{ImageBuffer, Rgba, RgbaImage};

use super::directory::Asset;
use super::utils::buf_to_le_u32;

// Asset header: 10 bytes
const HEADER_LEN: usize = 10;

pub fn extract_img(
    res_file: &mut File,
    palettes: &Vec<[u8; 768]>,
    asset: &Asset, 
    path: &mut PathBuf
) -> Result<bool, Box<dyn Error>> {
    // Add filename to path
    path.push(&asset.name);
    path.set_extension("PNG");

    // If file already exists skip extraction
    if path.is_file() {
        return Ok(false);
    }

    // Jump to asset start
    res_file.seek(SeekFrom::Start(asset.offset))?;

    // Read 10 bytes of asset header
    let mut header = [0;HEADER_LEN];
    res_file.read(&mut header)?;

    // First two bytes is image width
    let width = buf_to_le_u32(&header[0..2])?;
    // Next two bytes is image height
    let height = buf_to_le_u32(&header[2..4])?;
    // Next 4 bytes is origin pixel coords
    let _origin_x = buf_to_le_u32(&header[4..6])?;
    let _origin_y = buf_to_le_u32(&header[6..8])?;
    // Last two pixels is palette ID
    let palette_id = buf_to_le_u32(&header[8..10])? as usize;

    // Read image data
    let data_len = (asset.length as usize) - HEADER_LEN;
    let mut image_data = vec![0u8;data_len];
    res_file.read(&mut image_data)?;

    // Create output image
    let mut img: RgbaImage = ImageBuffer::new(width, height);
    // Find palette in palettes
    let palette = &palettes[palette_id];
    // First color of palette may be transparency pixel
    // We bias to this interpretation
    let transparency = &palette[0..3];

    // Iterate over the coordinates and pixels of the image
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let src_pixel = (x + (y * width)) as usize;
        let palette_color = image_data[src_pixel] as usize;
        let color = [
            palette[(palette_color * 3)],
            palette[(palette_color * 3) + 1],
            palette[(palette_color * 3) + 2],
            255
        ];

        if &color[0..3] == transparency {
            *pixel = Rgba([0, 0, 0, 0]);
        } else {
            *pixel = Rgba(color);
        }
    }

    // Save image file
    img.save(path)?;

    Ok(true)
}