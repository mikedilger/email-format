
use std::error::Error as StdError;
use std::fmt;

#[derive(PartialEq)]
pub enum ParseError {
    Eof,
    NotFound,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error>
    {
        match *self {
            _ => write!(f, "{}", self.description()),
        }
    }
}

impl fmt::Debug for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error>
    {
        match *self {
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
        }
    }

    fn cause(&self) -> Option<&StdError>
    {
        match *self {
            ParseError::Eof => None,
            ParseError::NotFound => None,
        }
    }
}
