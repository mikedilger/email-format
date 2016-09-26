
pub mod error;
pub use self::error::ParseError;

use std::io::Write;
use std::io::Error as IoError;

pub trait Parsable: Sized {
    /// Parse the object off of the beginning of the `input`.  If found, returns Some object,
    /// and a slice containing the remainer of the input.
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError>;
}

pub trait Streamable {
    /// Serializes and sends the content out to `w`, returning the number of bytes written.
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError>;
}
