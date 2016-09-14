
use std::error::Error as StdError;
use std::fmt::{self, Display};

/// An error type for the `email-format` crate.
pub enum Error {
    /// Invalid Header Value
    InvalidHeaderValue,
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            _ => format!("{}", self.description()).fmt(f),
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!( f.write_str(&*self.description()) );
        if self.cause().is_some() {
            try!( write!(f, ": {:?}", self.cause().unwrap()) ); // recurse
        }
        Ok(())
    }
}

impl StdError for Error {
    fn description(&self) -> &str{
        match *self {
            Error::InvalidHeaderValue => "Invalid Header Value.",
        }
    }
}
