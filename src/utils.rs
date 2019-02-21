use std::error::Error;
use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};

pub fn buf_to_le_i32(buf: &[u8]) -> Result<i32, Box<dyn Error>> {
    if buf.len() == 2 {
        let value = Cursor::new(buf).read_i16::<LittleEndian>()?;
        return Ok(value as i32);
    }
    
    Ok(Cursor::new(buf).read_i32::<LittleEndian>()?)
}

pub fn buf_to_le_u32(buf: &[u8]) -> Result<u32, Box<dyn Error>> {
    if buf.len() == 2 {
        let value = Cursor::new(buf).read_u16::<LittleEndian>()?;
        return Ok(value as u32);
    }
        
    Ok(Cursor::new(buf).read_u32::<LittleEndian>()?)
}

pub fn buf_to_le_u64(buf: &[u8]) -> Result<u64, Box<dyn Error>> {
    let value = buf_to_le_u32(buf)?;
    Ok(value as u64)
}