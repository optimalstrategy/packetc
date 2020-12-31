#![allow(clippy::new_without_default)]
use byteorder::{LittleEndian, WriteBytesExt};

pub struct Writer {
    data: Vec<u8>,
}

impl Writer {
    pub fn new() -> Writer {
        Writer { data: Vec::new() }
    }

    pub fn with_capacity(capacity: usize) -> Writer {
        Writer {
            data: Vec::with_capacity(capacity),
        }
    }

    /// Creates a Writer which uses an existing buffer as its storage
    pub fn with_buffer(buffer: Vec<u8>) -> Writer {
        Writer { data: buffer }
    }

    /// Writes a single u8
    #[inline]
    pub fn write_uint8(&mut self, value: u8) {
        self.data.write_u8(value).unwrap();
    }

    /// Writes a single u16
    #[inline]
    pub fn write_uint16(&mut self, value: u16) {
        self.data.write_u16::<LittleEndian>(value).unwrap();
    }

    /// Writes a single u32
    #[inline]
    pub fn write_uint32(&mut self, value: u32) {
        self.data.write_u32::<LittleEndian>(value).unwrap();
    }

    /// Writes a single i8
    #[inline]
    pub fn write_int8(&mut self, value: i8) {
        self.data.write_i8(value).unwrap();
    }

    /// Writes a single i16
    #[inline]
    pub fn write_int16(&mut self, value: i16) {
        self.data.write_i16::<LittleEndian>(value).unwrap();
    }

    /// Writes a single i32
    #[inline]
    pub fn write_int32(&mut self, value: i32) {
        self.data.write_i32::<LittleEndian>(value).unwrap();
    }

    /// Writes a single f32
    #[inline]
    pub fn write_float(&mut self, value: f32) {
        self.data.write_f32::<LittleEndian>(value).unwrap();
    }

    /// Writes a slice
    #[inline]
    pub fn write_slice(&mut self, value: &[u8]) {
        self.data.extend_from_slice(value);
    }

    /// Returns the internal buffer, replacing it with an empty one
    pub fn finish(mut self) -> Vec<u8> {
        std::mem::take(&mut self.data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_u8() {
        let mut writer = Writer::with_capacity(1);
        writer.write_uint8(100u8);
        let buf = writer.finish();
        assert_eq!(buf.as_slice(), 100u8.to_le_bytes());
    }

    #[test]
    fn write_u16() {
        let mut writer = Writer::with_capacity(2);
        writer.write_uint16(10000u16);
        let buf = writer.finish();
        assert_eq!(buf.as_slice(), 10000u16.to_le_bytes());
    }

    #[test]
    fn write_u32() {
        let mut writer = Writer::with_capacity(4);
        writer.write_uint32(1_000_000_000u32);
        let buf = writer.finish();
        assert_eq!(buf.as_slice(), 1_000_000_000u32.to_le_bytes());
    }

    #[test]
    fn write_i8() {
        let mut writer = Writer::with_capacity(1);
        writer.write_int8(100i8);
        let buf = writer.finish();
        assert_eq!(buf.as_slice(), 100i8.to_le_bytes());
    }

    #[test]
    fn write_i16() {
        let mut writer = Writer::with_capacity(2);
        writer.write_int16(10000i16);
        let buf = writer.finish();
        assert_eq!(buf.as_slice(), 10000i16.to_le_bytes());
    }

    #[test]
    fn write_i32() {
        let mut writer = Writer::with_capacity(4);
        writer.write_int32(1_000_000_000i32);
        let buf = writer.finish();
        assert_eq!(buf.as_slice(), 1_000_000_000i32.to_le_bytes());
    }

    #[test]
    fn write_f32() {
        let mut writer = Writer::with_capacity(4);
        writer.write_float(10.5f32);
        let buf = writer.finish();
        assert_eq!(buf.as_slice(), 10.5f32.to_le_bytes());
    }

    #[test]
    fn write_string() {
        let mut writer = Writer::with_capacity(4);
        writer.write_slice("testing".as_bytes());
        let buf = writer.finish();
        assert_eq!(std::str::from_utf8(buf.as_slice()).unwrap(), "testing");
    }
}
