use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::vec::Vec;

// Transparent pixel color
pub const TRANSPARENT: [u8; 4] = [247, 0, 247, 255];
// Offset on which first palette begins in MAX2.RES
const OFFSET: u64 = 36829628;
// Total number of palettes in game
const TOTAL_PALETTES: usize = 207;

pub fn read_palettes(res_file: &mut File, palettes: &mut Vec<[u8; 768]>) {
    // Jump to palettes start
    res_file.seek(SeekFrom::Start(OFFSET)).unwrap();
    // Palette
    let mut loaded_palettes: usize = 0;
    while loaded_palettes < TOTAL_PALETTES {
        let mut palette = [0; 768];
        res_file.read(&mut palette).unwrap();
        palettes.push(palette);
        loaded_palettes += 1;
    }
}