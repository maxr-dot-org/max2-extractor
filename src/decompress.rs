use std::fs::{File};
use std::io::{Read, Write};
use std::vec::Vec;
use crate::utils::buf_to_le_i32;

pub fn decompress_data(res_file: &mut File, data_len: usize) -> Vec<u8> {
    let mut data = vec![0u8; data_len];
    res_file.read(&mut data).unwrap();

    let mut position: usize = 0;
    let mut unpacked_data: Vec<u8> = Vec::new();

    while position < data_len {
        // Every compressed chunk starts with 2-bytes word
        let sword = buf_to_le_i32(&data[position..position+2]).unwrap();
        let start = position + 2;
        // If sword is positive, its number of uncompressed bytes
        if sword > 0 {
            let end = start + (sword as usize);
            unpacked_data.write(&data[start..end]).unwrap();
            position = end;
        } else {
            let mut repeat = sword.abs();
            let end = start + 1;
            while repeat > 0 {
                unpacked_data.write(&data[start..end]).unwrap();
                repeat -= 1;
            }
            position = end;
        }
    }

    unpacked_data
}