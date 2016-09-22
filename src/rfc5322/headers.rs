
use std::io::Write;
use std::io::Error as IoError;
use std::ascii::AsciiExt;
use super::{Parsable, ParseError, Streamable};
use super::types::{DateTime, MailboxList, Mailbox, AddressList, CFWS, MsgId,
                   Unstructured, Phrase};

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

// 3.6.2
// reply-to        =   "Reply-To:" address-list CRLF
#[derive(Debug, Clone, PartialEq)]
pub struct ReplyTo(pub AddressList);
impl Parsable for ReplyTo {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof); }
        let mut rem = input;
        req_name!(rem, b"reply-to:", input);
        if let Ok(x) = parse!(AddressList, rem) {
            req_crlf!(rem, input);
            return Ok((ReplyTo(x), rem));
        }
        Err(ParseError::NotFound)
    }
}
impl Streamable for ReplyTo {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        Ok(try!(w.write(b"Reply-To: "))
           + try!(self.0.stream(w))
           + try!(w.write(b"\r\n")))
    }
}

// 3.6.3
// to              =   "To:" address-list CRLF
#[derive(Debug, Clone, PartialEq)]
pub struct To(pub AddressList);
impl Parsable for To {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof); }
        let mut rem = input;
        req_name!(rem, b"to:", input);
        if let Ok(x) = parse!(AddressList, rem) {
            req_crlf!(rem, input);
            return Ok((To(x), rem));
        }
        Err(ParseError::NotFound)
    }
}
impl Streamable for To {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        Ok(try!(w.write(b"To: "))
           + try!(self.0.stream(w))
           + try!(w.write(b"\r\n")))
    }
}

// 3.6.3
// cc              =   "Cc:" address-list CRLF
#[derive(Debug, Clone, PartialEq)]
pub struct Cc(pub AddressList);
impl Parsable for Cc {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof); }
        let mut rem = input;
        req_name!(rem, b"cc:", input);
        if let Ok(x) = parse!(AddressList, rem) {
            req_crlf!(rem, input);
            return Ok((Cc(x), rem));
        }
        Err(ParseError::NotFound)
    }
}
impl Streamable for Cc {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        Ok(try!(w.write(b"Cc: "))
           + try!(self.0.stream(w))
           + try!(w.write(b"\r\n")))
    }
}

// 3.6.3
// bcc             =   "Bcc:" [address-list / CFWS] CRLF
#[derive(Debug, Clone, PartialEq)]
pub enum Bcc {
    AddressList(AddressList),
    CFWS(CFWS),
    Empty
}
impl Parsable for Bcc {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof); }
        let mut rem = input;
        req_name!(rem, b"bcc:", input);
        if let Ok(x) = parse!(AddressList, rem) {
            req_crlf!(rem, input);
            return Ok((Bcc::AddressList(x), rem));
        }
        if let Ok(x) = parse!(CFWS, rem) {
            req_crlf!(rem, input);
            return Ok((Bcc::CFWS(x), rem));
        }
        req_crlf!(rem, input);
        return Ok((Bcc::Empty, rem));
    }
}
impl Streamable for Bcc {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        let mut count: usize = 0;
        count += try!(w.write(b"Bcc: "));
        count += match *self {
            Bcc::AddressList(ref al) => try!(al.stream(w)),
            Bcc::CFWS(ref cfws) => try!(cfws.stream(w)),
            Bcc::Empty => 0,
        };
        count += try!(w.write(b"\r\n"));
        Ok(count)
    }
}

// 3.6.4
// message-id      =   "Message-ID:" msg-id CRLF
#[derive(Debug, Clone, PartialEq)]
pub struct MessageId(pub MsgId);
impl Parsable for MessageId {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof); }
        let mut rem = input;
        req_name!(rem, b"message-id:", input);
        if let Ok(x) = parse!(MsgId, rem) {
            req_crlf!(rem, input);
            return Ok((MessageId(x), rem));
        }
        Err(ParseError::NotFound)
    }
}
impl Streamable for MessageId {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        Ok(try!(w.write(b"Message-ID: "))
           + try!(self.0.stream(w))
           + try!(w.write(b"\r\n")))
    }
}

// 3.6.4
// in-reply-to     =   "In-Reply-To:" 1*msg-id CRLF
#[derive(Debug, Clone, PartialEq)]
pub struct InReplyTo(pub Vec<MsgId>);
impl Parsable for InReplyTo {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof); }
        let mut rem = input;
        let mut contents: Vec<MsgId> = Vec::new();
        req_name!(rem, b"in-reply-to:", input);
        while let Ok(x) = parse!(MsgId, rem) {
            contents.push(x);
        }
        if contents.len() == 0 {
            return Err(ParseError::NotFound);
        }
        req_crlf!(rem, input);
        Ok((InReplyTo(contents), rem))
    }
}
impl Streamable for InReplyTo {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        let mut count: usize = 0;
        count += try!(w.write(b"In-Reply-To: "));
        for msgid in &self.0 {
            count += try!(msgid.stream(w))
        }
        count += try!(w.write(b"\r\n"));
        Ok(count)
    }
}

// 3.6.4
// references      =   "References:" 1*msg-id CRLF
#[derive(Debug, Clone, PartialEq)]
pub struct References(pub Vec<MsgId>);
impl Parsable for References {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof); }
        let mut rem = input;
        let mut contents: Vec<MsgId> = Vec::new();
        req_name!(rem, b"references:", input);
        while let Ok(x) = parse!(MsgId, rem) {
            contents.push(x);
        }
        if contents.len() == 0 {
            return Err(ParseError::NotFound);
        }
        req_crlf!(rem, input);
        Ok((References(contents), rem))
    }
}
impl Streamable for References {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        let mut count: usize = 0;
        count += try!(w.write(b"References: "));
        for msgid in &self.0 {
            count += try!(msgid.stream(w))
        }
        count += try!(w.write(b"\r\n"));
        Ok(count)
    }
}

// 3.6.5
// subject         =   "Subject:" unstructured CRLF
#[derive(Debug, Clone, PartialEq)]
pub struct Subject(pub Unstructured);
impl Parsable for Subject {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof); }
        let mut rem = input;
        req_name!(rem, b"subject:", input);
        if let Ok(x) = parse!(Unstructured, rem) {
            req_crlf!(rem, input);
            return Ok((Subject(x), rem));
        }
        Err(ParseError::NotFound)
    }
}
impl Streamable for Subject {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        Ok(try!(w.write(b"Subject: "))
           + try!(self.0.stream(w))
           + try!(w.write(b"\r\n")))
    }
}

// 3.6.5
// comments        =   "Comments:" unstructured CRLF
#[derive(Debug, Clone, PartialEq)]
pub struct Comments(pub Unstructured);
impl Parsable for Comments {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof); }
        let mut rem = input;
        req_name!(rem, b"comments:", input);
        if let Ok(x) = parse!(Unstructured, rem) {
            req_crlf!(rem, input);
            return Ok((Comments(x), rem));
        }
        Err(ParseError::NotFound)
    }
}
impl Streamable for Comments {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        Ok(try!(w.write(b"Comments: "))
           + try!(self.0.stream(w))
           + try!(w.write(b"\r\n")))
    }
}

// 3.6.5
// keywords        =   "Keywords:" phrase *("," phrase) CRLF
#[derive(Debug, Clone, PartialEq)]
pub struct Keywords(pub Vec<Phrase>);
impl Parsable for Keywords {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof); }
        let mut rem = input;
        req_name!(rem, b"keywords:", input);
        let mut output: Vec<Phrase> = Vec::new();
        while let Ok(x) = parse!(Phrase, rem) {
            output.push(x);
        }
        if output.len()==0 {
            return Err(ParseError::NotFound);
        }
        req_crlf!(rem, input);
        Ok((Keywords(output), rem))
    }
}
impl Streamable for Keywords {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        let mut count: usize = 0;
        count += try!(w.write(b"Keywords: "));
        let mut virgin = true;
        for phrase in &self.0 {
            if ! virgin {
                count += try!(w.write(b","));
            }
            count += try!(phrase.stream(w));
            virgin = false
        }
        count += try!(w.write(b"\r\n"));
        Ok(count)
    }
}

// 3.6.6
// resent-date     =   "Resent-Date:" date-time CRLF
#[derive(Debug, Clone, PartialEq)]
pub struct ResentDate(pub DateTime);
impl Parsable for ResentDate {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof); }
        let mut rem = input;
        req_name!(rem, b"resent-date:", input);
        if let Ok(dt) = parse!(DateTime, rem) {
            req_crlf!(rem, input);
            Ok((ResentDate(dt), rem))
        } else {
            Err(ParseError::NotFound)
        }
    }
}
impl Streamable for ResentDate {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        Ok(try!(w.write(b"Resent-Date:"))
           + try!(self.0.stream(w))
           + try!(w.write(b"\r\n")))
    }
}

// 3.6.6
// resent-from     =   "Resent-From:" mailbox-list CRLF
#[derive(Debug, Clone, PartialEq)]
pub struct ResentFrom(pub MailboxList);
impl Parsable for ResentFrom {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof); }
        let mut rem = input;
        req_name!(rem, b"resent-from:", input);
        if let Ok(mbl) = parse!(MailboxList, rem) {
            req_crlf!(rem, input);
            return Ok((ResentFrom(mbl), rem));
        }
        Err(ParseError::NotFound)
    }
}
impl Streamable for ResentFrom {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        Ok(try!(w.write(b"Resent-From: "))
           + try!(self.0.stream(w))
           + try!(w.write(b"\r\n")))
    }
}

// 3.6.6
// resent-sender   =   "Resent-Sender:" mailbox CRLF
#[derive(Debug, Clone, PartialEq)]
pub struct ResentSender(pub Mailbox);
impl Parsable for ResentSender {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof); }
        let mut rem = input;
        req_name!(rem, b"resent-sender:", input);
        if let Ok(mb) = parse!(Mailbox, rem) {
            req_crlf!(rem, input);
            return Ok((ResentSender(mb), rem));
        }
        Err(ParseError::NotFound)
    }
}
impl Streamable for ResentSender {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        Ok(try!(w.write(b"Resent-Sender: "))
           + try!(self.0.stream(w))
           + try!(w.write(b"\r\n")))
    }
}

// 3.6.6
// resent-to       =   "Resent-To:" address-list CRLF
#[derive(Debug, Clone, PartialEq)]
pub struct ResentTo(pub AddressList);
impl Parsable for ResentTo {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof); }
        let mut rem = input;
        req_name!(rem, b"resent-to:", input);
        if let Ok(x) = parse!(AddressList, rem) {
            req_crlf!(rem, input);
            return Ok((ResentTo(x), rem));
        }
        Err(ParseError::NotFound)
    }
}
impl Streamable for ResentTo {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        Ok(try!(w.write(b"Resent-To: "))
           + try!(self.0.stream(w))
           + try!(w.write(b"\r\n")))
    }
}
