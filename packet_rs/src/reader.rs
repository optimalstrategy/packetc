use super::Error;
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Cursor;

macro_rules! try_read {
    ($it:expr) => {
        match $it {
            Ok(v) => Ok(v),
            Err(_) => Err(Error::OutOfBounds),
        }
    };
}

pub struct Reader<'a> {
    data: Cursor<&'a [u8]>,
}

impl<'a> Reader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Reader {
            data: Cursor::new(data),
        }
    }

    /// Reads a single u8
    #[inline]
    pub fn read_uint8(&mut self) -> Result<u8, Error> {
        try_read!(self.data.read_u8())
    }

    /// Reads a single u16
    #[inline]
    pub fn read_uint16(&mut self) -> Result<u16, Error> {
        try_read!(self.data.read_u16::<LittleEndian>())
    }

    /// Reads a single u32
    #[inline]
    pub fn read_uint32(&mut self) -> Result<u32, Error> {
        try_read!(self.data.read_u32::<LittleEndian>())
    }

    /// Reads a single i8
    #[inline]
    pub fn read_int8(&mut self) -> Result<i8, Error> {
        try_read!(self.data.read_i8())
    }

    /// Reads a single i16
    #[inline]
    pub fn read_int16(&mut self) -> Result<i16, Error> {
        try_read!(self.data.read_i16::<LittleEndian>())
    }

    /// Reads a single i32
    #[inline]
    pub fn read_int32(&mut self) -> Result<i32, Error> {
        try_read!(self.data.read_i32::<LittleEndian>())
    }

    /// Reads a single f32
    #[inline]
    pub fn read_float(&mut self) -> Result<f32, Error> {
        try_read!(self.data.read_f32::<LittleEndian>())
    }

    /// Reads a string of `len` bytes
    #[inline]
    pub fn read_string(&mut self, len: usize) -> Result<String, Error> {
        // consume `len` bytes by returning them as a string slice
        let pos = self.data.position() as usize;
        self.data.set_position((pos + len) as u64);
        match String::from_utf8(self.data.get_ref()[pos..pos + len].to_owned()) {
            Ok(v) => Ok(v),
            Err(_) => Err(Error::InvalidUtf8),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_u8() {
        let buf = 100u8.to_le_bytes();
        let mut reader = Reader::new(&buf);
        assert_eq!(reader.read_uint8().unwrap(), 100u8);
    }

    #[test]
    fn read_u16() {
        let buf = 10000u16.to_le_bytes();
        let mut reader = Reader::new(&buf);
        assert_eq!(reader.read_uint16().unwrap(), 10000u16);
    }

    #[test]
    fn read_u32() {
        let buf = 1_000_000_000u32.to_le_bytes();
        let mut reader = Reader::new(&buf);
        assert_eq!(reader.read_uint32().unwrap(), 1_000_000_000u32);
    }

    #[test]
    fn read_i8() {
        let buf = 100i8.to_le_bytes();
        let mut reader = Reader::new(&buf);
        assert_eq!(reader.read_int8().unwrap(), 100i8);
    }

    #[test]
    fn read_i16() {
        let buf = 10000i16.to_le_bytes();
        let mut reader = Reader::new(&buf);
        assert_eq!(reader.read_int16().unwrap(), 10000i16);
    }

    #[test]
    fn read_i32() {
        let buf = 1_000_000_000i32.to_le_bytes();
        let mut reader = Reader::new(&buf);
        assert_eq!(reader.read_int32().unwrap(), 1_000_000_000i32);
    }

    #[test]
    fn read_f32() {
        let buf = 10.5f32.to_le_bytes();
        let mut reader = Reader::new(&buf);
        // the bytes have to be exactly the same
        assert_eq!(reader.read_float().unwrap().to_le_bytes(), 10.5f32.to_le_bytes());
    }

    #[test]
    fn read_string() {
        let string = "testing";

        let buf = string.as_bytes();
        let mut reader = Reader::new(&buf);
        assert_eq!(reader.read_string(buf.len()).unwrap(), "testing");
    }
}
