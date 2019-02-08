use std::io::{Cursor, Result};
use byteorder::{LittleEndian, ReadBytesExt};

pub fn buf_to_le_i32(buf: &[u8]) -> Result<i32> {
    if buf.len() == 2 {
        let as_i16 = Cursor::new(buf).read_i16::<LittleEndian>()?;
        return Ok(as_i16 as i32);
    } else {
        return Cursor::new(buf).read_i32::<LittleEndian>();
    }
}

pub fn buf_to_le_u32(buf: &[u8]) -> Result<u32> {
    if buf.len() == 2 {
        let as_u16 = Cursor::new(buf).read_u16::<LittleEndian>()?;
        return Ok(as_u16 as u32);
    } else {
        return Cursor::new(buf).read_u32::<LittleEndian>();
    }
}

pub fn buf_to_le_u64(buf: &[u8]) -> Result<u64> {
    let as_u32 = buf_to_le_u32(buf)?;
    Ok(as_u32 as u64)
}