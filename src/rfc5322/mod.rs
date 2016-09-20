// Format validated types representing lexical tokens defined in
// RFC 5322 (as well as some referred from RFC 5234)
// in order to support SMTP (RFC 5321)

// Macro for defining sequences of characters within a character class
macro_rules! def_cclass {
    ( $typ:ident, $test:ident) => {
        #[derive(Debug, Clone, PartialEq)]
        pub struct $typ(pub Vec<u8>);
        impl Parsable for $typ {
            fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
                let mut pos: usize = 0;
                let mut output: Vec<u8> = Vec::new();
                while pos < input.len() && $test(input[pos]) {
                    output.push(input[pos]);
                    pos += 1;
                }
                if output.len() > 0 {
                    Ok( ($typ(output), &input[pos..]) )
                }
                else {
                    if pos >= input.len() { Err( ParseError::Eof ) }
                    else { Err( ParseError::NotFound ) }
                }
            }
        }
        impl Streamable for $typ {
            fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
                Ok(try!(w.write(&self.0[..])))
            }
        }
    };
}

// Macro for assigning the returned remaining input of a parse function to an existing
// variable
macro_rules! parse {
    ($pth:ident, $rem:ident) => {
        {
            $pth::parse($rem).map(|(value, r)| { $rem = r; value })
        }
    };
}

macro_rules! req {
    ($rem:ident, $bytes:expr, $input:ident) => {
        let len: usize = $bytes.len();
        if $rem.len() < len {
            return Err(ParseError::Eof);
        }
        if &$rem[0..len] != $bytes {
            return Err(ParseError::Expected($bytes.to_vec()));
        }
        $rem = &$rem[len..];
    };
}

pub mod error;
pub use self::error::ParseError;
pub mod types;

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
