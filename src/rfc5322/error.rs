
use std::error::Error as StdError;
use std::fmt;
use std::io::Error as IoError;

pub enum ParseError {
    Eof,
    NotFound,
    Expected(Vec<u8>),
    ExpectedType(&'static str),
    Io(IoError),
    InvalidBodyChar(u8),
    LineTooLong(usize),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error>
    {
        match *self {
            ParseError::Expected(ref bytes) => write!(f, "{}. Expected {:?}",
                                                      self.description(), bytes),
            ParseError::ExpectedType(ref t) => write!(f, "{}. Expected {}",
                                                      self.description(), t),
            ParseError::Io(ref e) => write!(f, "{}: {}",
                                            self.description(), e),
            ParseError::InvalidBodyChar(ref c) => write!(f, "{}: {}",
                                                         self.description(), c),
            ParseError::LineTooLong(ref l) => write!(f, "Line {} is too long", l),
            _ => write!(f, "{}", self.description()),
        }
    }
}

impl fmt::Debug for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error>
    {
        match *self {
            ParseError::Expected(ref bytes) => write!(f, "{}. Expected {:?}",
                                                      self.description(), bytes),
            ParseError::ExpectedType(ref t) => write!(f, "{}. Expected {}",
                                                      self.description(), t),
            ParseError::Io(ref e) => write!(f, "{}: {:?}",
                                             self.description(), e),
            ParseError::InvalidBodyChar(ref c) => write!(f, "{}: {}",
                                                         self.description(), c),
            ParseError::LineTooLong(ref l) => write!(f, "Line {} is too long", l),
            _ => write!(f, "{}", self.description()),
        }
    }
}

impl StdError for ParseError {
    fn description(&self) -> &str
    {
        match *self {
            ParseError::Eof => "End of File",
            ParseError::NotFound => "Not Found",
            ParseError::Expected(_) => "Expectation Failed",
            ParseError::ExpectedType(_) => "Expectation Failed",
            ParseError::Io(_) => "I/O Error",
            ParseError::InvalidBodyChar(_) => "Invalid Body Character",
            ParseError::LineTooLong(_) => "Line too long",
        }
    }

    fn cause(&self) -> Option<&StdError>
    {
        match *self {
            ParseError::Eof => None,
            ParseError::NotFound => None,
            ParseError::Expected(_) => None,
            ParseError::ExpectedType(_) => None,
            ParseError::Io(ref e) => Some(e),
            ParseError::InvalidBodyChar(_) => None,
            ParseError::LineTooLong(_) => None,
        }
    }
}
