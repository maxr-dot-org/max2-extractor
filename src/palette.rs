use std::error::Error;
use std::path::PathBuf;
use image::{ImageBuffer, Rgb, RgbImage};

pub fn render_palette(
    dst: &PathBuf, palette: &[u8; 768]
) -> Result<bool, Box<dyn Error>> {
    if dst.is_file() {
        return Ok(false) // Skip file
    }

    // Create 16x16 image (256 pixels total)
    let mut img: RgbImage = ImageBuffer::new(16, 16);
    // Iterate over the coordinates and pixels of the image
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let palette_color = (x + (y * 16)) as usize;
        let color = [
            palette[(palette_color * 3)],
            palette[(palette_color * 3) + 1],
            palette[(palette_color * 3) + 2]
        ];
        *pixel = Rgb(color);
    }
    // Save image file
    img.save(dst)?;

    Ok(true)
}