use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Cursor;

pub struct Reader<'a> {
    data: Cursor<&'a [u8]>,
}

impl<'a> Reader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Reader {
            data: Cursor::new(data),
        }
    }

    /// Get remaining bytes
    #[inline]
    pub fn remaining(&self) -> usize {
        self.data.get_ref().len() - self.data.position() as usize
    }

    /// Reads a single u8, does not do bounds checking.
    #[inline]
    pub fn read_uint8(&mut self) -> u8 {
        self.data.read_u8().unwrap()
    }

    /// Reads a single u16, does not do bounds checking.
    #[inline]
    pub fn read_uint16(&mut self) -> u16 {
        self.data.read_u16::<LittleEndian>().unwrap()
    }

    /// Reads a single u32, does not do bounds checking.
    #[inline]
    pub fn read_uint32(&mut self) -> u32 {
        self.data.read_u32::<LittleEndian>().unwrap()
    }

    /// Reads a single i8, does not do bounds checking.
    #[inline]
    pub fn read_int8(&mut self) -> i8 {
        self.data.read_i8().unwrap()
    }

    /// Reads a single i16, does not do bounds checking.
    #[inline]
    pub fn read_int16(&mut self) -> i16 {
        self.data.read_i16::<LittleEndian>().unwrap()
    }

    /// Reads a single i32, does not do bounds checking.
    #[inline]
    pub fn read_int32(&mut self) -> i32 {
        self.data.read_i32::<LittleEndian>().unwrap()
    }

    /// Reads a single f32, does not do bounds checking.
    #[inline]
    pub fn read_float(&mut self) -> f32 {
        self.data.read_f32::<LittleEndian>().unwrap()
    }

    /// Reads a slice of `len`, does not do bounds checking
    #[inline]
    pub fn read_slice(&'a mut self, len: usize) -> &'a [u8] {
        // consume `len` bytes by returning them as a string slice
        let pos = self.data.position() as usize;
        self.data.set_position((pos + len) as u64);
        &self.data.get_ref()[pos..pos + len]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str;

    #[test]
    fn read_u8() {
        let buf = 100u8.to_le_bytes();
        let mut reader = Reader::new(&buf);
        assert_eq!(reader.read_uint8(), 100u8);
    }

    #[test]
    fn read_u16() {
        let buf = 10000u16.to_le_bytes();
        let mut reader = Reader::new(&buf);
        assert_eq!(reader.read_uint16(), 10000u16);
    }

    #[test]
    fn read_u32() {
        let buf = 1_000_000_000u32.to_le_bytes();
        let mut reader = Reader::new(&buf);
        assert_eq!(reader.read_uint32(), 1_000_000_000u32);
    }

    #[test]
    fn read_i8() {
        let buf = 100i8.to_le_bytes();
        let mut reader = Reader::new(&buf);
        assert_eq!(reader.read_int8(), 100i8);
    }

    #[test]
    fn read_i16() {
        let buf = 10000i16.to_le_bytes();
        let mut reader = Reader::new(&buf);
        assert_eq!(reader.read_int16(), 10000i16);
    }

    #[test]
    fn read_i32() {
        let buf = 1_000_000_000i32.to_le_bytes();
        let mut reader = Reader::new(&buf);
        assert_eq!(reader.read_int32(), 1_000_000_000i32);
    }

    #[test]
    fn read_f32() {
        let buf = 10.5f32.to_le_bytes();
        let mut reader = Reader::new(&buf);
        // the bytes have to be exactly the same
        assert_eq!(reader.read_float().to_le_bytes(), 10.5f32.to_le_bytes());
    }

    #[test]
    fn read_string() {
        let string = "testing";
        let buf = string.as_bytes();
        let mut reader = Reader::new(&buf);
        assert_eq!(
            str::from_utf8(reader.read_slice(buf.len())).unwrap(),
            "testing"
        );
    }
}
