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
pub mod headers;

use std::io::Write;
use std::io::Error as IoError;
use self::headers::{Return, Received};
use self::headers::{ResentDate, ResentFrom, ResentSender, ResentTo, ResentCc, ResentBcc,
                    ResentMessageId};

pub trait Parsable: Sized {
    /// Parse the object off of the beginning of the `input`.  If found, returns Some object,
    /// and a slice containing the remainer of the input.
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError>;
}

pub trait Streamable {
    /// Serializes and sends the content out to `w`, returning the number of bytes written.
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError>;
}

// 3.6.7
// trace           =   [return]
//                     1*received
#[derive(Debug, Clone, PartialEq)]
pub struct Trace {
    return_path: Option<Return>,
    received: Vec<Received>
}
impl Parsable for Trace {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        let mut rem = input;
        let maybe_return = parse!(Return, rem).ok();
        let mut received: Vec<Received> = Vec::new();
        while let Ok(r) = parse!(Received, rem) {
            received.push(r);
        }
        if received.len() < 1 { return Err(ParseError::NotFound); }
        Ok((Trace {
            return_path: maybe_return,
            received: received,
        }, rem))
    }
}
impl Streamable for Trace {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        let mut count: usize = 0;
        if let Some(ref rp) = self.return_path {
            count += try!(rp.stream(w));
        }
        for r in &self.received {
            count += try!(r.stream(w));
        }
        Ok(count)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResentField {
    Date(ResentDate),
    From(ResentFrom),
    Sender(ResentSender),
    To(ResentTo),
    Cc(ResentCc),
    Bcc(ResentBcc),
    MessageId(ResentMessageId),
}
impl Parsable for ResentField {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        let mut rem = input;
        if let Ok(x) = parse!(ResentDate, rem) {
            return Ok((ResentField::Date(x), rem));
        }
        if let Ok(x) = parse!(ResentFrom, rem) {
            return Ok((ResentField::From(x), rem));
        }
        if let Ok(x) = parse!(ResentSender, rem) {
            return Ok((ResentField::Sender(x), rem));
        }
        if let Ok(x) = parse!(ResentTo, rem) {
            return Ok((ResentField::To(x), rem));
        }
        if let Ok(x) = parse!(ResentCc, rem) {
            return Ok((ResentField::Cc(x), rem));
        }
        if let Ok(x) = parse!(ResentBcc, rem) {
            return Ok((ResentField::Bcc(x), rem));
        }
        if let Ok(x) = parse!(ResentMessageId, rem) {
            return Ok((ResentField::MessageId(x), rem));
        }
        Err(ParseError::NotFound)
    }
}
impl Streamable for ResentField {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        match *self {
            ResentField::Date(ref x) => x.stream(w),
            ResentField::From(ref x) => x.stream(w),
            ResentField::Sender(ref x) => x.stream(w),
            ResentField::To(ref x) => x.stream(w),
            ResentField::Cc(ref x) => x.stream(w),
            ResentField::Bcc(ref x) => x.stream(w),
            ResentField::MessageId(ref x) => x.stream(w),
        }
    }
}
