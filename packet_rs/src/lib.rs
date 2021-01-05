extern crate byteorder;
#[macro_use]
extern crate thiserror;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Value {0} is not a valid {1} variant")]
    InvalidEnumValue(String, &'static str),
    #[error("Out of bounds")]
    OutOfBounds,
    #[error("Invalid UTF-8")]
    InvalidUtf8,
}

pub mod reader;
pub mod writer;

pub use reader::Reader;
pub use writer::Writer;
