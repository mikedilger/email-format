
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
            ParseError::Eof(ref field) => write!(f, "{} while looking for \"{}\"",
                                                 self.description(), field),
            ParseError::NotFound(ref token) => write!(f, "\"{}\" {}",
                                                      token, self.description()),
            ParseError::Expected(ref bytes) => write!(f, "{}. Expected \"{:?}\"",
                                                      self.description(), bytes),
            ParseError::ExpectedType(ref t) => write!(f, "{}. Expected {}",
                                                      self.description(), t),
            ParseError::Io(ref e) => write!(f, "{}: {}",
                                            self.description(), e),
            ParseError::InvalidBodyChar(ref c) => write!(f, "{}: {} is not 7-bit ASCII",
                                                         self.description(), c),
            ParseError::LineTooLong(ref l) => write!(f, "Line {} is too long", l),
            ParseError::TrailingInput(ref field, ref c) => write!(
                f, "Trailing input at byte {} in {}", c, field),
            ParseError::Parse(ref field, ref inner) => write!(
                f, "{} {}: {}", self.description(), field, inner),
            _ => write!(f, "{}", self.description()),
        }
    }
}

impl fmt::Debug for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error>
    {
        <Self as fmt::Display>::fmt(self, f)
    }
}

impl StdError for ParseError {
    fn description(&self) -> &str
    {
        match *self {
            ParseError::Eof(_) => "End of File",
            ParseError::NotFound(_) => "Not Found",
            ParseError::Expected(_) => "Expectation Failed",
            ParseError::ExpectedType(_) => "Expectation Failed",
            ParseError::Io(_) => "I/O Error",
            ParseError::InvalidBodyChar(_) => "Invalid Body Character",
            ParseError::LineTooLong(_) => "Line too long",
            ParseError::TrailingInput(_,_) => "Trailing input",
            ParseError::InternalError => "Internal error",
            ParseError::Parse(_,_) => "Unable to parse",
        }
    }

    fn cause(&self) -> Option<&StdError>
    {
        match *self {
            ParseError::Io(ref e) => Some(e),
            ParseError::Parse(_, ref e) => Some(e),
            _ => None,
        }
    }
}
