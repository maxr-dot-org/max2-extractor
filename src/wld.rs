use std::error;
use std::fs::{File, create_dir_all};
use std::io::{Error, ErrorKind, Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::str;
use std::vec::Vec;
use image::{ImageBuffer, Rgb, RgbImage};

use super::utils::buf_to_le_u32;

const WLD_FILE_HEADER: &str = "WLD";
const INVALID_HEADER_ERROR: &str = "Opened file is not a valid WLD package";
const SECTOR_EDGE: i64 = 32;

pub fn extract_wld(
    wld_file: &mut PathBuf, path: &mut PathBuf
) -> Result<bool, Box<dyn error::Error>> {
    // Create dst dir named after file
    let mut path = path.to_path_buf();
    let dst_dirname = wld_file.file_stem().unwrap();
    path.push(dst_dirname);
    if !path.is_dir() {
        create_dir_all(&path)?;
    }

    // Open wld file
    let mut wld_file = File::open(wld_file)?;
    check_wld_file_header(&mut wld_file)?;

    // Skip next two bytes
    wld_file.seek(SeekFrom::Current(2))?;

    // Next four bytes is map width x height
    let mut header = [0;4];
    wld_file.read(&mut header)?;
    let width = buf_to_le_u32(&header[0..2])?;
    let height = buf_to_le_u32(&header[2..4])?;

    // Calculate data length
    let length = (width * height) as usize;

    // Next length bytes is minimap
    let mut minimap = vec![0u8; length];
    wld_file.read(&mut minimap)?;

    // Next length * 2 bytes is map data chunk order
    let mut chunk_order: Vec<u32> = Vec::new();
    // Zerofill chunk order
    for _ in 0..length {
        chunk_order.push(0);
    }
    // Read chunk ordering from file
    for i in 0..length {
        let mut chunk = [0;2];
        wld_file.read(&mut chunk)?;
        let chunk = buf_to_le_u32(&chunk)? as usize;
        if chunk < length {
            chunk_order[chunk] = i as u32;
        }
    }

    // Heightmap is not using palette, so it can be rendered in place
    let heightmap_len = ((width + 1) * (height + 1) * 2) as i64;
    if !render_heightmap(width, height, &mut wld_file, &path)? {
        // If we didn't read heightmap, skip its data
        wld_file.seek(SeekFrom::Current(heightmap_len))?;
    }

    // Height map is followed by 2 bytes of grid length
    // It always equals width * height
    let mut grid_length = [0;2];
    wld_file.read(&mut grid_length)?;
    let grid_length = buf_to_le_u32(&grid_length)? as i64;

    // Calculate map data length
    let map_length = grid_length * SECTOR_EDGE * SECTOR_EDGE;
    // And skip it (for now)
    wld_file.seek(SeekFrom::Current(map_length))?;

    // Map data is followed by palette data (3 * 256 bytes)
    let palette_length = 256 * 3;
    let mut palette = vec![0u8; palette_length];
    wld_file.read(&mut palette)?;

    // Render palette and minimap
    render_palette(&palette, &path)?;
    render_minimap(width, height, &minimap, &palette, &path)?;

    // Seek back to map data, and render it
    wld_file.seek(SeekFrom::Current(palette_length as i64 * -1))?;
    wld_file.seek(SeekFrom::Current(map_length * -1))?;
    if !render_map(width, height, &chunk_order, &mut wld_file, &palette, &path)? {
        // If we didn't read the map, skip its data
        wld_file.seek(SeekFrom::Current(map_length))?;
    }

    // After rendering map data, skip palette and render sector types
    wld_file.seek(SeekFrom::Current(palette_length as i64))?;
    render_sector_types(width, height, &chunk_order, &mut wld_file, &path)?;

    Ok(true)
}

fn check_wld_file_header(
    wld_file: &mut File
) -> Result<(), Box<dyn error::Error>> {
    // First 4 bytes should be "WLD" string
    let mut buffer = [0; 3];
    wld_file.read(&mut buffer)?;

    let header = str::from_utf8(&buffer)?;
    if header != WLD_FILE_HEADER {
        let err = Error::new(ErrorKind::InvalidData, INVALID_HEADER_ERROR);
        return Err(Box::new(err));
    }

    Ok(())
}

fn render_heightmap(
    width: u32, height: u32, wld_file: &mut File, path: &PathBuf
) -> Result<bool, Box<dyn error::Error>> {
    let mut path = path.to_path_buf();
    path.push("heightmap");
    path.set_extension("png");

    if path.is_file() {
        return Ok(false);
    }

    // Height map uses vertexes, so it adds extra data
    let width = width + 1;
    let height = height + 1;

    // Read image from wld file
    let mut img: RgbImage = ImageBuffer::new(width, height);
    for (_x, _y, pixel) in img.enumerate_pixels_mut() {
        // Read vertex height
        let mut vertex = [0;2];
        wld_file.read(&mut vertex)?;
        let mut vertex = buf_to_le_u32(&vertex)?;
        // Height can be bigger than 255, but we are limiting that
        // So we can use simple gradient presentation
        if vertex > 255 {
            vertex = 255;
        }
        let vertex = vertex as u8;
        *pixel = Rgb([vertex, vertex, vertex]);
    }

    img.save(path)?;

    Ok(true)
}

fn render_palette(
    palette: &Vec<u8>, path: &PathBuf
) -> Result<bool, Box<dyn error::Error>> {
    let mut path = path.to_path_buf();
    path.push("palette");
    path.set_extension("png");

    if path.is_file() {
        return Ok(false);
    }

    // Render palette as 16 x 16 pixels square
    let mut img: RgbImage = ImageBuffer::new(16, 16);
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let palette_color = (x + (y * 16)) as usize;
        let color = [
            palette[(palette_color * 3)],
            palette[(palette_color * 3) + 1],
            palette[(palette_color * 3) + 2]
        ];
        *pixel = Rgb(color);
    }

    img.save(path)?;

    Ok(true)
}

fn render_minimap(
    width: u32,
    height: u32,
    minimap: &Vec<u8>,
    palette: &Vec<u8>,
    path: &PathBuf
) -> Result<bool, Box<dyn error::Error>> {
    let mut path = path.to_path_buf();
    path.push("minimap");
    path.set_extension("png");

    if path.is_file() {
        return Ok(false);
    }

    let mut img: RgbImage = ImageBuffer::new(width, height);
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let color = (x + (y * width)) as usize;
        let color = minimap[color] as usize;
        let color = [
            palette[(color * 3)],
            palette[(color * 3) + 1],
            palette[(color * 3) + 2]
        ];
        *pixel = Rgb(color);
    }

    img.save(path)?;

    Ok(true)
}

fn render_map(
    width: u32,
    height: u32,
    chunk_order: &Vec<u32>,
    wld_file: &mut File,
    palette: &Vec<u8>,
    path: &PathBuf
) -> Result<bool, Box<dyn error::Error>> {
    let mut path = path.to_path_buf();
    path.push("full");
    path.set_extension("png");

    if path.is_file() {
        return Ok(false);
    }

    let sector = SECTOR_EDGE as u32;
    let sector_length = (sector * sector) as usize;
    let width_px = width * sector;
    let height_px = height * sector;

    let mut img: RgbImage = ImageBuffer::new(width_px, height_px);

    for chunk in chunk_order {
        // Calculate chunk coords
        let chunk_y = chunk / height;
        let chunk_x = chunk % width;

        // Read chunk data
        let mut data = vec![0u8; sector_length];
        wld_file.read(&mut data)?;

        // Place them on map
        for y in 0..sector {
            let y_abs = y + (chunk_y * sector);
            for x in 0..sector {
                let x_abs = x + (chunk_x * sector);

                let offset = (x + (y * sector)) as usize;
                let color = data[offset] as usize;
                let color = [
                    palette[(color * 3)],
                    palette[(color * 3) + 1],
                    palette[(color * 3) + 2]
                ];

                let pixel = img.get_pixel_mut(x_abs as u32, y_abs as u32);
                *pixel = Rgb(color);
            }
        }
    }

    img.save(path)?;

    Ok(true)
}

const TYPE_GRASS: u8 = 0;
const TYPE_WATER: u8 = 1;
const TYPE_SHORE: u8 = 2;
const TYPE_BLOCK: u8 = 3;
const TYPE_SLOWER: u8 = 4;
const TYPE_SLOWST: u8 = 5;

fn render_sector_types(
    width: u32,
    height: u32,
    chunk_order: &Vec<u32>,
    wld_file: &mut File,
    path: &PathBuf
) -> Result<bool, Box<dyn error::Error>> {
    let mut path = path.to_path_buf();
    path.push("typemap");
    path.set_extension("png");

    if path.is_file() {
        return Ok(false);
    }

    let length = (width * height) as usize;
    let width_half = (width / 2) as u32;
    let height_half = (height / 2) as u32;
    let quarter = (width_half * height_half) as usize;
    
    let mut img: RgbImage = ImageBuffer::new(width, height);
    // Typemap is split in four quarters
    for y_quarter in 0..2 {
        for x_quarter in 0..2 {
            let mut data = vec![0u8; quarter];
            wld_file.read(&mut data)?;

            for y in 0..height_half {
                for x in 0..width_half {
                    let mut x = x;
                    let mut y = y;

                    let src = (x + (y * height_half)) as usize;
                    let type_ = data[src];

                    if x_quarter == 1 { x += width_half; }
                    if y_quarter == 1 { y += height_half; }

                    let pixel = img.get_pixel_mut(x, y);

                    match type_ {
                        TYPE_GRASS => *pixel = Rgb([49, 97, 8]),
                        TYPE_WATER => *pixel = Rgb([24, 73, 107]),
                        TYPE_SHORE => *pixel = Rgb([82, 158, 206]),
                        TYPE_BLOCK => *pixel = Rgb([206, 32, 0]),
                        TYPE_SLOWER => *pixel = Rgb([189, 186, 16]),
                        TYPE_SLOWST => *pixel = Rgb([231, 142, 24]),
                        _ => println!("type: {}", type_),
                    }
                }
            }
        }   
    }

    img.save(path)?;

    Ok(true)
}