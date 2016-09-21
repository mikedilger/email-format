
use std::io::Write;
use std::io::Error as IoError;
use std::ascii::AsciiExt;
use super::{Parsable, ParseError, Streamable};
use super::types::{DateTime, MailboxList, Mailbox};

macro_rules! req_name {
    ($rem:ident, $str:expr, $input:ident) => {
        let len: usize = $str.len();
        if $rem.len() < len || &(&$rem[0..len]).to_ascii_lowercase()!=$str {
            return Err(ParseError::NotFound);
        }
        $rem = &$rem[len..];
    };
}

macro_rules! req_crlf {
    ($rem:ident, $input:ident) => {
        if &$rem[..2] != b"\r\n" {
            return Err(ParseError::NotFound);
        }
        $rem = &$rem[2..];
    }
}

// 3.6.1
// orig-date       =   "Date:" date-time CRLF
#[derive(Debug, Clone, PartialEq)]
pub struct OrigDate(pub DateTime);
impl Parsable for OrigDate {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof); }
        let mut rem = input;
        req_name!(rem, b"date:", input);
        if let Ok(dt) = parse!(DateTime, rem) {
            req_crlf!(rem, input);
            Ok((OrigDate(dt), rem))
        } else {
            Err(ParseError::NotFound)
        }
    }
}
impl Streamable for OrigDate {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        Ok(try!(w.write(b"Date:"))
           + try!(self.0.stream(w))
           + try!(w.write(b"\r\n")))
    }
}

// 3.6.2
// from            =   "From:" mailbox-list CRLF
#[derive(Debug, Clone, PartialEq)]
pub struct From(pub MailboxList);
impl Parsable for From {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof); }
        let mut rem = input;
        req_name!(rem, b"from:", input);
        if let Ok(mbl) = parse!(MailboxList, rem) {
            req_crlf!(rem, input);
            return Ok((From(mbl), rem));
        }
        Err(ParseError::NotFound)
    }
}
impl Streamable for From {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        Ok(try!(w.write(b"From: "))
           + try!(self.0.stream(w))
           + try!(w.write(b"\r\n")))
    }
}

// 3.6.2
// sender          =   "Sender:" mailbox CRLF
#[derive(Debug, Clone, PartialEq)]
pub struct Sender(pub Mailbox);
impl Parsable for Sender {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof); }
        let mut rem = input;
        req_name!(rem, b"sender:", input);
        if let Ok(mb) = parse!(Mailbox, rem) {
            req_crlf!(rem, input);
            return Ok((Sender(mb), rem));
        }
        Err(ParseError::NotFound)
    }
}
impl Streamable for Sender {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        Ok(try!(w.write(b"Sender: "))
           + try!(self.0.stream(w))
           + try!(w.write(b"\r\n")))
    }
}
