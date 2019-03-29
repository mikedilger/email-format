
use std::io::Write;
use std::io::Error as IoError;
use ::TryFrom;
use super::{Parsable, ParseError, Streamable};
use super::types::{DateTime, MailboxList, Mailbox, AddressList, CFWS, MsgId,
                   Unstructured, Phrase, ReceivedToken, Path, FieldName};

macro_rules! req_name {
    ($rem:ident, $str:expr) => {
        let len: usize = $str.as_bytes().len();
        if $rem.len() < len ||
            &(&$rem[0..len]).to_ascii_lowercase().as_slice() != &$str.as_bytes()
        {
            return Err(ParseError::NotFound($str));
        }
        $rem = &$rem[len..];
    };
}

macro_rules! req_crlf {
    ($rem:ident) => {
        if $rem.len() < 2 {
            return Err(ParseError::NotFound("CRLF"));
        }
        if &$rem[..2] != b"\r\n" {
            return Err(ParseError::NotFound("CRLF"));
        }
        $rem = &$rem[2..];
    }
}

macro_rules! impl_try_from {
    ($from:ident, $to:ident) => {
        impl<'a> TryFrom<&'a [u8]> for $to {
            type Error = ParseError;
            fn try_from(input: &'a [u8]) -> Result<$to, ParseError> {
                let (out,rem) = try!($from::parse(input));
                if rem.len() > 0 {
                    return Err(ParseError::TrailingInput("$to", input.len() - rem.len()));
                }
                Ok($to(out))
            }
        }
        impl<'a> TryFrom<&'a str> for $to {
            type Error = ParseError;
            fn try_from(input: &'a str) -> Result<$to, ParseError> {
                TryFrom::try_from(input.as_bytes())
            }
        }
        impl<'a> TryFrom<$from> for $to {
            type Error = ParseError;
            fn try_from(input: $from) -> Result<$to, ParseError> {
                Ok($to(input))
            }
        }
    }
}

// 3.6.1
// orig-date       =   "Date:" date-time CRLF
#[derive(Debug, Clone, PartialEq)]
pub struct OrigDate(pub DateTime);
impl Parsable for OrigDate {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof("Date")); }
        let mut rem = input;
        req_name!(rem, "date:");
        match parse!(DateTime, rem) {
            Ok(dt) => {
                req_crlf!(rem);
                Ok((OrigDate(dt), rem))
            },
            Err(e) => Err(ParseError::Parse("Date", Box::new(e)))
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
impl_try_from!(DateTime, OrigDate);
#[cfg(feature="time")]
impl<'a> TryFrom<&'a ::time::Tm> for OrigDate {
    type Error = ParseError;
    fn try_from(input: &'a ::time::Tm) -> Result<OrigDate, ParseError> {
        let s = match input.strftime("%a, %d %b %Y %T %z") {
            Ok(s) => format!("{}",s),
            Err(_) => return Err(ParseError::InternalError),
        };
        TryFrom::try_from(s.as_bytes())
    }
}
#[cfg(feature="chrono")]
impl<'a, Tz: ::chrono::TimeZone> TryFrom<&'a ::chrono::DateTime<Tz>> for OrigDate
    where Tz::Offset: ::std::fmt::Display
{
    type Error = ParseError;
    fn try_from(input: &'a ::chrono::DateTime<Tz>) -> Result<OrigDate, ParseError> {
        let s = input.format("%a, %d %b %Y %T %z").to_string();
        TryFrom::try_from(s.as_bytes())
    }
}
impl_display!(OrigDate);

// 3.6.2
// from            =   "From:" mailbox-list CRLF
#[derive(Debug, Clone, PartialEq)]
pub struct From(pub MailboxList);
impl Parsable for From {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof("From")); }
        let mut rem = input;
        req_name!(rem, "from:");
        match parse!(MailboxList, rem) {
            Ok(mbl) => {
                req_crlf!(rem);
                return Ok((From(mbl), rem));
            },
            Err(e) => Err(ParseError::Parse("From", Box::new(e)))
        }
    }
}
impl Streamable for From {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        Ok(try!(w.write(b"From:"))
           + try!(self.0.stream(w))
           + try!(w.write(b"\r\n")))
    }
}
impl_try_from!(MailboxList, From);
impl_display!(From);

// 3.6.2
// sender          =   "Sender:" mailbox CRLF
#[derive(Debug, Clone, PartialEq)]
pub struct Sender(pub Mailbox);
impl Parsable for Sender {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof("Sender")); }
        let mut rem = input;
        req_name!(rem, "sender:");
        match parse!(Mailbox, rem) {
            Ok(mb) => {
                req_crlf!(rem);
                return Ok((Sender(mb), rem));
            },
            Err(e) => Err(ParseError::Parse("Sender", Box::new(e)))
        }
    }
}
impl Streamable for Sender {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        Ok(try!(w.write(b"Sender:"))
           + try!(self.0.stream(w))
           + try!(w.write(b"\r\n")))
    }
}
impl_try_from!(Mailbox, Sender);
impl_display!(Sender);

// 3.6.2
// reply-to        =   "Reply-To:" address-list CRLF
#[derive(Debug, Clone, PartialEq)]
pub struct ReplyTo(pub AddressList);
impl Parsable for ReplyTo {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof("Reply-To")); }
        let mut rem = input;
        req_name!(rem, "reply-to:");
        match parse!(AddressList, rem) {
            Ok(x) => {
                req_crlf!(rem);
                return Ok((ReplyTo(x), rem));
            },
            Err(e) => Err(ParseError::Parse("Reply-To", Box::new(e)))
        }
    }
}
impl Streamable for ReplyTo {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        Ok(try!(w.write(b"Reply-To:"))
           + try!(self.0.stream(w))
           + try!(w.write(b"\r\n")))
    }
}
impl_try_from!(AddressList, ReplyTo);
impl_display!(ReplyTo);

// 3.6.3
// to              =   "To:" address-list CRLF
#[derive(Debug, Clone, PartialEq)]
pub struct To(pub AddressList);
impl Parsable for To {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof("To")); }
        let mut rem = input;
        req_name!(rem, "to:");
        match parse!(AddressList, rem) {
            Ok(x) => {
                req_crlf!(rem);
                return Ok((To(x), rem));
            },
            Err(e) => Err(ParseError::Parse("To", Box::new(e))),
        }
    }
}
impl Streamable for To {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        Ok(try!(w.write(b"To:"))
           + try!(self.0.stream(w))
           + try!(w.write(b"\r\n")))
    }
}
impl_try_from!(AddressList, To);
impl_display!(To);

// 3.6.3
// cc              =   "Cc:" address-list CRLF
#[derive(Debug, Clone, PartialEq)]
pub struct Cc(pub AddressList);
impl Parsable for Cc {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof("Cc")); }
        let mut rem = input;
        req_name!(rem, "cc:");
        match parse!(AddressList, rem) {
            Ok(x) => {
                req_crlf!(rem);
                return Ok((Cc(x), rem));
            },
            Err(e) => Err(ParseError::Parse("Cc", Box::new(e))),
        }
    }
}
impl Streamable for Cc {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        Ok(try!(w.write(b"Cc:"))
           + try!(self.0.stream(w))
           + try!(w.write(b"\r\n")))
    }
}
impl_try_from!(AddressList, Cc);
impl_display!(Cc);

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
        if input.len() == 0 { return Err(ParseError::Eof("Bcc")); }
        let mut rem = input;
        req_name!(rem, "bcc:");
        if let Ok(x) = parse!(AddressList, rem) {
            req_crlf!(rem);
            return Ok((Bcc::AddressList(x), rem));
        }
        if let Ok(x) = parse!(CFWS, rem) {
            req_crlf!(rem);
            return Ok((Bcc::CFWS(x), rem));
        }
        req_crlf!(rem);
        return Ok((Bcc::Empty, rem));
    }
}
impl Streamable for Bcc {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        let mut count: usize = 0;
        count += try!(w.write(b"Bcc:"));
        count += match *self {
            Bcc::AddressList(ref al) => try!(al.stream(w)),
            Bcc::CFWS(ref cfws) => try!(cfws.stream(w)),
            Bcc::Empty => 0,
        };
        count += try!(w.write(b"\r\n"));
        Ok(count)
    }
}
impl<'a> TryFrom<&'a [u8]> for Bcc {
    type Error = ParseError;
    fn try_from(input: &'a [u8]) -> Result<Bcc, ParseError> {
        let (out,rem) = try!(AddressList::parse(input));
        if rem.len() > 0 {
            return Err(ParseError::TrailingInput("Bcc", input.len() - rem.len()));
        }
        Ok(Bcc::AddressList(out))
    }
}
impl<'a> TryFrom<&'a str> for Bcc {
    type Error = ParseError;
    fn try_from(input: &'a str) -> Result<Bcc, ParseError> {
        TryFrom::try_from(input.as_bytes())
    }
}
impl<'a> TryFrom<AddressList> for Bcc {
    type Error = ParseError;
    fn try_from(input: AddressList) -> Result<Bcc, ParseError> {
        Ok(Bcc::AddressList(input))
    }
}
impl_display!(Bcc);

// 3.6.4
// message-id      =   "Message-ID:" msg-id CRLF
#[derive(Debug, Clone, PartialEq)]
pub struct MessageId(pub MsgId);
impl Parsable for MessageId {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof("MessageId")); }
        let mut rem = input;
        req_name!(rem, "message-id:");
        match parse!(MsgId, rem) {
            Ok(x) => {
                req_crlf!(rem);
                return Ok((MessageId(x), rem));
            },
            Err(e) => Err(ParseError::Parse("Message-Id", Box::new(e))),
        }
    }
}
impl Streamable for MessageId {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        Ok(try!(w.write(b"Message-ID:"))
           + try!(self.0.stream(w))
           + try!(w.write(b"\r\n")))
    }
}
impl_try_from!(MsgId, MessageId);
impl_display!(MessageId);

// 3.6.4
// in-reply-to     =   "In-Reply-To:" 1*msg-id CRLF
#[derive(Debug, Clone, PartialEq)]
pub struct InReplyTo(pub Vec<MsgId>);
impl Parsable for InReplyTo {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof("InReplyTo")); }
        let mut rem = input;
        let mut contents: Vec<MsgId> = Vec::new();
        req_name!(rem, "in-reply-to:");
        let err;
        loop {
            match parse!(MsgId, rem) {
                Ok(x) => contents.push(x),
                Err(e) => { err = e; break; }
            }
        }
        if contents.len() == 0 {
            return Err(ParseError::Parse("In-Reply-To", Box::new(err)));
        }
        req_crlf!(rem);
        Ok((InReplyTo(contents), rem))
    }
}
impl Streamable for InReplyTo {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        let mut count: usize = 0;
        count += try!(w.write(b"In-Reply-To:"));
        for msgid in &self.0 {
            count += try!(msgid.stream(w))
        }
        count += try!(w.write(b"\r\n"));
        Ok(count)
    }
}
impl<'a> TryFrom<&'a [u8]> for InReplyTo {
    type Error = ParseError;
    fn try_from(input: &'a [u8]) -> Result<InReplyTo, ParseError> {
        let mut msgids: Vec<MsgId> = Vec::new();
        let mut rem = input;
        while let Ok(x) = parse!(MsgId, rem) {
            msgids.push(x);
        }
        if rem.len() > 0 {
            Err(ParseError::TrailingInput("In-Reply-To", input.len() - rem.len()))
        } else {
            Ok(InReplyTo(msgids))
        }
    }
}
impl<'a> TryFrom<&'a str> for InReplyTo {
    type Error = ParseError;
    fn try_from(input: &'a str) -> Result<InReplyTo, ParseError> {
        TryFrom::try_from(input.as_bytes())
    }
}
impl<'a> TryFrom<Vec<MsgId>> for InReplyTo {
    type Error = ParseError;
    fn try_from(input: Vec<MsgId>) -> Result<InReplyTo, ParseError> {
        Ok(InReplyTo(input))
    }
}
impl_display!(InReplyTo);

// 3.6.4
// references      =   "References:" 1*msg-id CRLF
#[derive(Debug, Clone, PartialEq)]
pub struct References(pub Vec<MsgId>);
impl Parsable for References {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof("References")); }
        let mut rem = input;
        let mut contents: Vec<MsgId> = Vec::new();
        req_name!(rem, "references:");
        let err;
        loop {
            match parse!(MsgId, rem) {
                Ok(x) => contents.push(x),
                Err(e) => { err = e; break }
            }
        }
        if contents.len() == 0 {
            return Err(ParseError::Parse("References", Box::new(err)));
        }
        req_crlf!(rem);
        Ok((References(contents), rem))
    }
}
impl Streamable for References {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        let mut count: usize = 0;
        count += try!(w.write(b"References:"));
        for msgid in &self.0 {
            count += try!(msgid.stream(w))
        }
        count += try!(w.write(b"\r\n"));
        Ok(count)
    }
}
impl<'a> TryFrom<&'a [u8]> for References {
    type Error = ParseError;
    fn try_from(input: &'a [u8]) -> Result<References, ParseError> {
        let mut msgids: Vec<MsgId> = Vec::new();
        let mut rem = input;
        while let Ok(x) = parse!(MsgId, rem) {
            msgids.push(x);
        }
        if rem.len() > 0 {
            Err(ParseError::TrailingInput("References", input.len() - rem.len()))
        } else {
            Ok(References(msgids))
        }
    }
}
impl<'a> TryFrom<&'a str> for References {
    type Error = ParseError;
    fn try_from(input: &'a str) -> Result<References, ParseError> {
        TryFrom::try_from(input.as_bytes())
    }
}
impl<'a> TryFrom<Vec<MsgId>> for References {
    type Error = ParseError;
    fn try_from(input: Vec<MsgId>) -> Result<References, ParseError> {
        Ok(References(input))
    }
}
impl_display!(References);

// 3.6.5
// subject         =   "Subject:" unstructured CRLF
#[derive(Debug, Clone, PartialEq)]
pub struct Subject(pub Unstructured);
impl Parsable for Subject {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof("Subject")); }
        let mut rem = input;
        req_name!(rem, "subject:");
        match parse!(Unstructured, rem) {
            Ok(x) => {
                req_crlf!(rem);
                return Ok((Subject(x), rem));
            },
            Err(e) => Err(ParseError::Parse("Subject", Box::new(e))),
        }
    }
}
impl Streamable for Subject {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        Ok(try!(w.write(b"Subject:"))
           + try!(self.0.stream(w))
           + try!(w.write(b"\r\n")))
    }
}
impl_try_from!(Unstructured, Subject);
impl_display!(Subject);

// 3.6.5
// comments        =   "Comments:" unstructured CRLF
#[derive(Debug, Clone, PartialEq)]
pub struct Comments(pub Unstructured);
impl Parsable for Comments {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof("Comments")); }
        let mut rem = input;
        req_name!(rem, "comments:");
        match parse!(Unstructured, rem) {
            Ok(x) => {
                req_crlf!(rem);
                return Ok((Comments(x), rem));
            },
            Err(e) => Err(ParseError::Parse("Comments", Box::new(e))),
        }
    }
}
impl Streamable for Comments {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        Ok(try!(w.write(b"Comments:"))
           + try!(self.0.stream(w))
           + try!(w.write(b"\r\n")))
    }
}
impl_try_from!(Unstructured, Comments);
impl_display!(Comments);

// 3.6.5
// keywords        =   "Keywords:" phrase *("," phrase) CRLF
#[derive(Debug, Clone, PartialEq)]
pub struct Keywords(pub Vec<Phrase>);
impl Parsable for Keywords {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof("Keywords")); }
        let mut rem = input;
        req_name!(rem, "keywords:");
        let mut output: Vec<Phrase> = Vec::new();
        let err;
        loop {
            match parse!(Phrase, rem) {
                Ok(x) => output.push(x),
                Err(e) => { err = e; break; }
            }
        }
        if output.len()==0 {
            return Err(ParseError::Parse("Keywords", Box::new(err)));
        }
        req_crlf!(rem);
        Ok((Keywords(output), rem))
    }
}
impl Streamable for Keywords {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        let mut count: usize = 0;
        count += try!(w.write(b"Keywords:"));
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
impl<'a> TryFrom<&'a [u8]> for Keywords {
    type Error = ParseError;
    fn try_from(input: &'a [u8]) -> Result<Keywords, ParseError> {
        let mut msgids: Vec<Phrase> = Vec::new();
        let mut rem = input;
        while let Ok(x) = parse!(Phrase, rem) {
            msgids.push(x);
        }
        if rem.len() > 0 {
            Err(ParseError::TrailingInput("Keywords", input.len() - rem.len()))
        } else {
            Ok(Keywords(msgids))
        }
    }
}
impl<'a> TryFrom<&'a str> for Keywords {
    type Error = ParseError;
    fn try_from(input: &'a str) -> Result<Keywords, ParseError> {
        TryFrom::try_from(input.as_bytes())
    }
}
impl<'a> TryFrom<Vec<Phrase>> for Keywords {
    type Error = ParseError;
    fn try_from(input: Vec<Phrase>) -> Result<Keywords, ParseError> {
        Ok(Keywords(input))
    }
}
impl_display!(Keywords);

// 3.6.6
// resent-date     =   "Resent-Date:" date-time CRLF
#[derive(Debug, Clone, PartialEq)]
pub struct ResentDate(pub DateTime);
impl Parsable for ResentDate {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof("Resent-Date")); }
        let mut rem = input;
        req_name!(rem, "resent-date:");
        match parse!(DateTime, rem) {
            Ok(dt) => {
                req_crlf!(rem);
                Ok((ResentDate(dt), rem))
            },
            Err(e) => Err(ParseError::Parse("Resent-Date", Box::new(e)))
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
impl_try_from!(DateTime, ResentDate);
impl_display!(ResentDate);

// 3.6.6
// resent-from     =   "Resent-From:" mailbox-list CRLF
#[derive(Debug, Clone, PartialEq)]
pub struct ResentFrom(pub MailboxList);
impl Parsable for ResentFrom {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof("Resent-From")); }
        let mut rem = input;
        req_name!(rem, "resent-from:");
        match parse!(MailboxList, rem) {
            Ok(mbl) => {
                req_crlf!(rem);
                return Ok((ResentFrom(mbl), rem));
            },
            Err(e) => Err(ParseError::Parse("Resent-From", Box::new(e))),
        }
    }
}
impl Streamable for ResentFrom {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        Ok(try!(w.write(b"Resent-From:"))
           + try!(self.0.stream(w))
           + try!(w.write(b"\r\n")))
    }
}
impl_try_from!(MailboxList, ResentFrom);
impl_display!(ResentFrom);

// 3.6.6
// resent-sender   =   "Resent-Sender:" mailbox CRLF
#[derive(Debug, Clone, PartialEq)]
pub struct ResentSender(pub Mailbox);
impl Parsable for ResentSender {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof("Resent-Sender")); }
        let mut rem = input;
        req_name!(rem, "resent-sender:");
        match parse!(Mailbox, rem) {
            Ok(mb) => {
                req_crlf!(rem);
                return Ok((ResentSender(mb), rem));
            },
            Err(e) => Err(ParseError::Parse("Resent-Sender", Box::new(e))),
        }
    }
}
impl Streamable for ResentSender {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        Ok(try!(w.write(b"Resent-Sender:"))
           + try!(self.0.stream(w))
           + try!(w.write(b"\r\n")))
    }
}
impl_try_from!(Mailbox, ResentSender);
impl_display!(ResentSender);

// 3.6.6
// resent-to       =   "Resent-To:" address-list CRLF
#[derive(Debug, Clone, PartialEq)]
pub struct ResentTo(pub AddressList);
impl Parsable for ResentTo {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof("Resent-To")); }
        let mut rem = input;
        req_name!(rem, "resent-to:");
        match parse!(AddressList, rem) {
            Ok(x) => {
                req_crlf!(rem);
                return Ok((ResentTo(x), rem));
            },
            Err(e) => Err(ParseError::Parse("Resent-To", Box::new(e))),
        }
    }
}
impl Streamable for ResentTo {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        Ok(try!(w.write(b"Resent-To:"))
           + try!(self.0.stream(w))
           + try!(w.write(b"\r\n")))
    }
}
impl_try_from!(AddressList, ResentTo);
impl_display!(ResentTo);

// 3.6.6
// resent-cc       =   "Resent-Cc:" address-list CRLF
#[derive(Debug, Clone, PartialEq)]
pub struct ResentCc(pub AddressList);
impl Parsable for ResentCc {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof("Resent-Cc")); }
        let mut rem = input;
        req_name!(rem, "resent-cc:");
        match parse!(AddressList, rem) {
            Ok(x) => {
                req_crlf!(rem);
                return Ok((ResentCc(x), rem));
            },
            Err(e) => Err(ParseError::Parse("Resent-Cc", Box::new(e)))
        }
    }
}
impl Streamable for ResentCc {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        Ok(try!(w.write(b"Resent-Cc:"))
           + try!(self.0.stream(w))
           + try!(w.write(b"\r\n")))
    }
}
impl_try_from!(AddressList, ResentCc);
impl_display!(ResentCc);

// 3.6.6
// resent-bcc      =   "Resent-Bcc:" [address-list / CFWS] CRLF
#[derive(Debug, Clone, PartialEq)]
pub enum ResentBcc {
    AddressList(AddressList),
    CFWS(CFWS),
    Empty
}
impl Parsable for ResentBcc {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof("Resent-Bcc")); }
        let mut rem = input;
        req_name!(rem, "resent-bcc:");
        if let Ok(x) = parse!(AddressList, rem) {
            req_crlf!(rem);
            return Ok((ResentBcc::AddressList(x), rem));
        }
        if let Ok(x) = parse!(CFWS, rem) {
            req_crlf!(rem);
            return Ok((ResentBcc::CFWS(x), rem));
        }
        req_crlf!(rem);
        return Ok((ResentBcc::Empty, rem));
    }
}
impl Streamable for ResentBcc {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        let mut count: usize = 0;
        count += try!(w.write(b"Resent-Bcc:"));
        count += match *self {
            ResentBcc::AddressList(ref al) => try!(al.stream(w)),
            ResentBcc::CFWS(ref cfws) => try!(cfws.stream(w)),
            ResentBcc::Empty => 0,
        };
        count += try!(w.write(b"\r\n"));
        Ok(count)
    }
}
impl<'a> TryFrom<&'a [u8]> for ResentBcc {
    type Error = ParseError;
    fn try_from(input: &'a [u8]) -> Result<ResentBcc, ParseError> {
        let (out,rem) = try!(AddressList::parse(input));
        if rem.len() > 0 {
            return Err(ParseError::TrailingInput("Resent-Bcc", input.len() - rem.len()));
        }
        Ok(ResentBcc::AddressList(out))
    }
}
impl<'a> TryFrom<&'a str> for ResentBcc {
    type Error = ParseError;
    fn try_from(input: &'a str) -> Result<ResentBcc, ParseError> {
        TryFrom::try_from(input.as_bytes())
    }
}
impl<'a> TryFrom<AddressList> for ResentBcc {
    type Error = ParseError;
    fn try_from(input: AddressList) -> Result<ResentBcc, ParseError> {
        Ok(ResentBcc::AddressList(input))
    }
}
impl_display!(ResentBcc);

// 3.6.6
// resent-msg-id   =   "Resent-Message-ID:" msg-id CRLF
#[derive(Debug, Clone, PartialEq)]
pub struct ResentMessageId(pub MsgId);
impl Parsable for ResentMessageId {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof("Resent-Message-ID")); }
        let mut rem = input;
        req_name!(rem, "resent-message-id:");
        match parse!(MsgId, rem) {
            Ok(x) => {
                req_crlf!(rem);
                return Ok((ResentMessageId(x), rem));
            },
            Err(e) => Err(ParseError::Parse("Resent-Message-Id", Box::new(e))),
        }
    }
}
impl Streamable for ResentMessageId {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        Ok(try!(w.write(b"Resent-Message-ID:"))
           + try!(self.0.stream(w))
           + try!(w.write(b"\r\n")))
    }
}
impl_try_from!(MsgId, ResentMessageId);
impl_display!(ResentMessageId);

// 3.6.7
// received        =   "Received:" *received-token ";" date-time CRLF
// Errata ID 3979:
// received        =   "Received:" [1*received-token / CFWS]
//                     ";" date-time CRLF
#[derive(Debug, Clone, PartialEq)]
pub enum ReceivedTokens {
    Tokens(Vec<ReceivedToken>),
    Comment(CFWS),
}
#[derive(Debug, Clone, PartialEq)]
pub struct Received {
    pub received_tokens: ReceivedTokens,
    pub date_time: DateTime,
}
impl Parsable for Received {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof("Received")); }
        let mut rem = input;
        req_name!(rem, "received:");
        let mut tokens: Vec<ReceivedToken> = Vec::new();
        let err;
        loop {
            match parse!(ReceivedToken, rem) {
                Ok(r) => tokens.push(r),
                Err(e) => { err = e; break; }
            }
        }
        let received_tokens = if tokens.len()==0 {
            if let Ok(cfws) = parse!(CFWS, rem) {
                ReceivedTokens::Comment(cfws)
            } else {
                return Err(ParseError::Parse("Received", Box::new(err)));
            }
        } else {
            ReceivedTokens::Tokens(tokens)
        };
        req!(rem, b";", input);
        match parse!(DateTime, rem) {
            Ok(dt) => {
                req_crlf!(rem);
                return Ok((Received {
                    received_tokens: received_tokens,
                    date_time: dt
                }, rem));
            },
            Err(e) => Err(ParseError::Parse("Received", Box::new(e))),
        }
    }
}
impl Streamable for Received {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        let mut count: usize = 0;
        count += try!(w.write(b"Received:"));
        match self.received_tokens {
            ReceivedTokens::Tokens(ref vec) => {
                for token in vec {
                    count += try!(token.stream(w));
                }
            },
            ReceivedTokens::Comment(ref c) => {
                count += try!(c.stream(w));
            },
        }
        count += try!(w.write(b";"));
        count += try!(self.date_time.stream(w));
        count += try!(w.write(b"\r\n"));
        Ok(count)
    }
}
impl<'a> TryFrom<&'a [u8]> for Received {
    type Error = ParseError;
    fn try_from(input: &'a [u8]) -> Result<Received, ParseError> {
        let mut fudged_input: Vec<u8> = "Received:".as_bytes().to_owned();
        fudged_input.extend(&*input);
        fudged_input.extend("\r\n".as_bytes());
        let (out,rem) = try!(Received::parse(input));
        if rem.len() > 0 {
            return Err(ParseError::TrailingInput("Received", input.len() - rem.len()));
        } else {
            Ok(out)
        }
    }
}
impl<'a> TryFrom<&'a str> for Received {
    type Error = ParseError;
    fn try_from(input: &'a str) -> Result<Received, ParseError> {
        TryFrom::try_from(input.as_bytes())
    }
}
impl<'a> TryFrom<(ReceivedTokens, DateTime)> for Received {
    type Error = ParseError;
    fn try_from(input: (ReceivedTokens, DateTime)) -> Result<Received, ParseError> {
        Ok(Received {
            received_tokens: input.0,
            date_time: input.1 })
    }
}
impl_display!(Received);

// 3.6.7
// return          =   "Return-Path:" path CRLF
#[derive(Debug, Clone, PartialEq)]
pub struct Return(pub Path);
impl Parsable for Return {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        let mut rem = input;
        req_name!(rem, "return-path:");
        match parse!(Path, rem) {
            Ok(path) => {
                req_crlf!(rem);
                return Ok((Return(path), rem));
            },
            Err(e) => Err(ParseError::Parse("Return-Path", Box::new(e))),
        }
    }
}
impl Streamable for Return {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        Ok(try!(w.write(b"Return-Path:"))
           + try!(self.0.stream(w))
           + try!(w.write(b"\r\n")))
    }
}
impl_try_from!(Path, Return);
impl_display!(Return);

// 3.6.8
// optional-field  =   field-name ":" unstructured CRLF
#[derive(Debug, Clone, PartialEq)]
pub struct OptionalField {
    pub name: FieldName,
    pub value: Unstructured,
}
impl Parsable for OptionalField {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        let mut rem = input;

        match parse!(FieldName, rem) {
            Ok(name) => {
                req!(rem, b":", input);
                match parse!(Unstructured, rem) {
                    Ok(value) => {
                        req_crlf!(rem);
                        return Ok((OptionalField {
                            name: name,
                            value: value,
                        }, rem));
                    },
                    Err(e) => Err(ParseError::Parse("Optional Field", Box::new(e))),
                }
            },
            Err(e) => Err(ParseError::Parse("Optional Field", Box::new(e))),
        }
    }
}
impl Streamable for OptionalField {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        Ok(try!(self.name.stream(w))
           + try!(w.write(b":"))
           + try!(self.value.stream(w))
           + try!(w.write(b"\r\n")))
    }
}
impl<'a> TryFrom<(FieldName, Unstructured)> for OptionalField {
    type Error = ParseError;
    fn try_from(input: (FieldName, Unstructured)) -> Result<OptionalField, ParseError> {
        Ok(OptionalField {
            name: input.0,
            value: input.1 })
    }
}
impl<'a,'b> TryFrom<(&'a [u8], &'b [u8])> for OptionalField {
    type Error = ParseError;
    fn try_from(input: (&'a [u8], &'b [u8])) -> Result<OptionalField, ParseError> {
        let (name,rem) = try!(FieldName::parse(input.0));
        if rem.len() > 0 {
            return Err(ParseError::TrailingInput("Optional Field", input.0.len() - rem.len()));
        }
        let (value,rem) = try!(Unstructured::parse(input.1));
        if rem.len() > 0 {
            return Err(ParseError::TrailingInput("Optional Field", input.1.len() - rem.len()));
        }
        Ok(OptionalField {
            name: name,
            value: value,
        })
    }
}
impl<'a,'b> TryFrom<(&'a str, &'b str)> for OptionalField {
    type Error = ParseError;
    fn try_from(input: (&'a str, &'b str)) -> Result<OptionalField, ParseError> {
        TryFrom::try_from((input.0.as_bytes(), input.1.as_bytes()))
    }
}
impl_display!(OptionalField);
