use std::io::{Cursor, Result};
use byteorder::{LittleEndian, ReadBytesExt};

pub fn buf_to_le_u32(buf: &[u8]) -> Result<u32> {
    return Cursor::new(buf).read_u32::<LittleEndian>();
}