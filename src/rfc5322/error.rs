
use std::error::Error as StdError;
use std::fmt;

#[derive(PartialEq)]
pub enum ParseError {
    Eof,
    NotFound,
    Expected(Vec<u8>),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error>
    {
        match *self {
            ParseError::Expected(ref bytes) => write!(f, "{}. Expected {:?}",
                                                      self.description(), bytes),
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
        }
    }

    fn cause(&self) -> Option<&StdError>
    {
        match *self {
            ParseError::Eof => None,
            ParseError::NotFound => None,
            ParseError::Expected(_) => None,
        }
    }
}
