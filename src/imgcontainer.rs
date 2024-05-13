use std::error::Error;
use std::fs::{File, create_dir_all};
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::vec::Vec;
use image::{ImageBuffer, Rgba, RgbaImage};

use super::directory::Asset;
use super::utils::{buf_to_le_u32, buf_to_le_u64};

pub fn extract_img_container(
    res_file: &mut File,
    palettes: &Vec<[u8; 768]>,
    asset: &Asset,
    path: &mut PathBuf
) -> Result<bool, Box<dyn Error>> {
    // Add filename to path
    path.push(&asset.name);
    // Create directory for asset images
    if !path.is_dir() {
        create_dir_all(&path)?;
    }

    // Jump to asset start
    res_file.seek(SeekFrom::Start(asset.offset))?;

    // First two bytes of asset is number of images
    let mut images_count = [0;2];
    res_file.read(&mut images_count)?;
    let images_count = buf_to_le_u32(&images_count)? as usize;

    // Next two bytes of asset is palette id
    let mut palette = [0;2];
    res_file.read(&mut palette)?;
    let palette = buf_to_le_u32(&palette)? as usize;
    let palette = palettes[palette];

    // Palette id is followed by list of image offsets
    // each offset is written with 4 bytes
    let mut images_offsets: Vec<u64> = Vec::new();
    while images_offsets.len() < images_count {
        let mut image_offset = [0;4];
        res_file.read(&mut image_offset)?;
        let image_offset = asset.offset + buf_to_le_u64(&image_offset)?;
        images_offsets.push(image_offset);
    }

    // Extract every image
    for (i, image_offset) in images_offsets.into_iter().enumerate() {
        // Create final image path
        let mut img_path = path.to_path_buf();
        img_path.push(i.to_string());
        img_path.set_extension("PNG");
        // If file doesnt exist, extract it
        if !img_path.is_file() {
            extract_img_from_container(
                res_file, palette, asset.offset, image_offset, img_path
            )?;
        }
    }

    Ok(true)
}

fn extract_img_from_container(
    res_file: &mut File,
    palette: [u8; 768],
    asset_offset: u64,
    img_offset: u64,
    path: PathBuf
) -> Result<bool, Box<dyn Error>> {
    // Jump to image start
    res_file.seek(SeekFrom::Start(img_offset))?;

    // Read 8 bytes image header
    let mut header = [0;8];
    res_file.read(&mut header)?;

    // Deconstruct header into data
    let width = buf_to_le_u32(&header[0..2])?;
    let height = buf_to_le_u32(&header[2..4])?;
    let _center_x = buf_to_le_u32(&header[4..6])?;
    let _center_y = buf_to_le_u32(&header[6..8])?;

    // File is split into number of rows, each of varying length
    // Read rows offsets list
    let mut offsets: Vec<u64> = Vec::new();
    while offsets.len() < (height as usize) {
        let mut offset = [0;4];
        res_file.read(&mut offset)?;
        let offset = asset_offset + buf_to_le_u64(&offset)?;
        offsets.push(offset);
    }

    // Create output image
    let mut img: RgbaImage = ImageBuffer::new(width, height);
    
    // Draw image row after row
    for (y, offset) in offsets.into_iter().enumerate() {
        // Jump to row start in image
        res_file.seek(SeekFrom::Start(offset))?;
        let mut x: u32 = 0;
        // Render row
        while x < width {
            // Row is split into chunks of varying length
            // Chunk header is always two bytes long
            let mut header = [0;2];
            res_file.read(&mut header)?;
            // First byte is number of transparent pixels before color pixels
            let margin = header[0] as u32;
            // If margin is 255, row end is reached
            if margin == 255 {
                break;
            }

            // Second byte is number of color pixels
            let data_len = header[1] as usize;
            // Skip transparent pixels
            x += margin;
            // Read color pixels
            let mut colors = vec![0u8;data_len];
            res_file.read(&mut colors)?;
            for color in colors {
                let color = color as usize;
                let pixel = img.get_pixel_mut(x as u32, y as u32);
                *pixel = Rgba([
                    palette[color * 3],
                    palette[(color * 3) + 1],
                    palette[(color * 3) + 2],
                    255
                ]);
                x += 1;
            }
        }
    }
    
    // Save image file
    img.save(path)?;

    Ok(true)
}