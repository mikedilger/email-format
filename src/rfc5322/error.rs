
use std::error::Error as StdError;
use std::fmt;
use std::io::Error as IoError;

pub enum ParseError {
    Eof(&'static str),
    NotFound(&'static str),
    Expected(Vec<u8>),
    ExpectedType(&'static str),
    Io(IoError),
    InvalidBodyChar(u8),
    LineTooLong(usize),
    TrailingInput(&'static str, usize),
    InternalError,
    Parse(&'static str, Box<ParseError>),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error>
    {
        match *self {
            ParseError::Eof(ref field) => write!(f, "End of File while looking for \"{}\"", field),
            ParseError::NotFound(ref token) => write!(f, "\"{}\" Not Found", token),
            ParseError::Expected(ref bytes) => write!(f, "Expectation Failed. Expected \"{:?}\"", bytes),
            ParseError::ExpectedType(ref t) => write!(f, "Expectation Failed. Expected {}", t),
            ParseError::Io(ref e) => write!(f, "I/O Error: {}", e),
            ParseError::InvalidBodyChar(ref c) => write!(f, "Invalid Body Character: {} is not 7-bit ASCII", c),
            ParseError::LineTooLong(ref l) => write!(f, "Line {} is too long", l),
            ParseError::TrailingInput(ref field, ref c) => write!(f, "Trailing input at byte {} in {}", c, field),
            ParseError::InternalError => write!(f, "Internal error"),
            ParseError::Parse(ref field, ref inner) => write!(f, "Unable to parse {}: {}", field, inner),
        }
    }
}

impl fmt::Debug for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error>
    {
        <Self as fmt::Display>::fmt(self, f)
    }
}

impl StdError for ParseError { }
