
use std::io::Write;
use std::io::Error as IoError;
use super::{Parsable, Streamable, ParseError};

// RFC 5234, B.1  Core Rules
//const CR: u8 = 0x0D;     //   CR             =  %x0D      ; carriage return
//const LF: u8 = 0x0A;     //   LF             =  %x0A      ; linefeed
const SP: u8 = 0x20;     //   SP             =  %x20
const HTAB: u8 = 0x09;   //   HTAB           =  %x09      ; horizontal tab
//const DQUOTE: u8 = 0x22; //   DQUOTE         =  %x22      ; " (Double Quote)

// RFC 5234, B.1  Core Rules
// VCHAR           =  %x21-7E   ; visible (printing) characters)
#[inline]
pub fn is_vchar(c: u8) -> bool { c>=0x21 && c<=0x7E }
def_cclass!(VChar, is_vchar);

// RFC 5234, B.1  Core Rules  WSP            =  SP / HTAB ; white space
#[inline]
pub fn is_wsp(c: u8) -> bool { c==SP || c==HTAB }
def_cclass!(WSP, is_wsp);

// RFC 5234, B.1  Core Rules  CHAR           =  %x01-7F ; any 7-bit US-ASCII character,
//                                                      ;  excluding NUL
#[inline]
pub fn is_ascii(c: u8) -> bool { c>=1 && c<=127 }
def_cclass!(ASCII, is_ascii);

// RFC 5234, B.1  Core Rules  DIGIT          =  %x30-39   ; 0-9
#[inline]
pub fn is_digit(c: u8) -> bool { c>=0x30 && c<=0x39 }
def_cclass!(Digit, is_digit);

// RFC 5234, B.1  Core Rules  ALPHA          = %x41-5A / %x61-7A   ; A-Z / a-z
#[inline]
pub fn is_alpha(c: u8) -> bool { (c>=0x41 && c<=0x5A) || (c>=0x61 && c<=0x7A) }
def_cclass!(Alpha, is_alpha);

// 3.2.1
// quoted-pair     =   ("\" (VCHAR / WSP)) / obs-qp
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuotedPair(pub u8);
impl Parsable for QuotedPair {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        let mut pos: usize = 0;
        if pos >= input.len() { return Err(ParseError::Eof); }
        if pos + 1 >= input.len() { return Err(ParseError::NotFound); }
        if input[pos]!=b'\\' { return Err(ParseError::NotFound); }
        if is_vchar(input[pos + 1]) || is_wsp(input[pos + 1]) {
            pos += 2;
            let qp = QuotedPair(input[pos - 1]);
            return Ok((qp, &input[pos..]));
        }
        Err(ParseError::NotFound)
    }
}
impl Streamable for QuotedPair {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        Ok(try!(w.write(b"\\"))
           + try!(w.write(&[self.0])))
    }
}
