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
use buf_read_ext::BufReadExt;
use self::headers::{Return, Received};
use self::headers::{ResentDate, ResentFrom, ResentSender, ResentTo, ResentCc, ResentBcc,
                    ResentMessageId};
use self::headers::{OrigDate, From, Sender, ReplyTo, To, Cc, Bcc, MessageId, InReplyTo,
                    References, Subject, Comments, Keywords, OptionalField};

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

// 3.6
// a sub part of the Fields definition
#[derive(Debug, Clone, PartialEq)]
pub enum Field {
    OrigDate(OrigDate),
    From(From),
    Sender(Sender),
    ReplyTo(ReplyTo),
    To(To),
    Cc(Cc),
    Bcc(Bcc),
    MessageId(MessageId),
    InReplyTo(InReplyTo),
    References(References),
    Subject(Subject),
    Comments(Comments),
    Keywords(Keywords),
    OptionalField(OptionalField),
}
impl Parsable for Field {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        let mut rem = input;
        if let Ok(x) = parse!(OrigDate, rem) {
            return Ok((Field::OrigDate(x), rem));
        }
        if let Ok(x) = parse!(From, rem) {
            return Ok((Field::From(x), rem));
        }
        if let Ok(x) = parse!(Sender, rem) {
            return Ok((Field::Sender(x), rem));
        }
        if let Ok(x) = parse!(ReplyTo, rem) {
            return Ok((Field::ReplyTo(x), rem));
        }
        if let Ok(x) = parse!(To, rem) {
            return Ok((Field::To(x), rem));
        }
        if let Ok(x) = parse!(Cc, rem) {
            return Ok((Field::Cc(x), rem));
        }
        if let Ok(x) = parse!(Bcc, rem) {
            return Ok((Field::Bcc(x), rem));
        }
        if let Ok(x) = parse!(MessageId, rem) {
            return Ok((Field::MessageId(x), rem));
        }
        if let Ok(x) = parse!(InReplyTo, rem) {
            return Ok((Field::InReplyTo(x), rem));
        }
        if let Ok(x) = parse!(References, rem) {
            return Ok((Field::References(x), rem));
        }
        if let Ok(x) = parse!(Subject, rem) {
            return Ok((Field::Subject(x), rem));
        }
        if let Ok(x) = parse!(Comments, rem) {
            return Ok((Field::Comments(x), rem));
        }
        if let Ok(x) = parse!(Keywords, rem) {
            return Ok((Field::Keywords(x), rem));
        }
        if let Ok(x) = parse!(OptionalField, rem) {
            return Ok((Field::OptionalField(x), rem));
        }
        Err(ParseError::NotFound)
    }
}
impl Streamable for Field {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        match *self {
            Field::OrigDate(ref x) => x.stream(w),
            Field::From(ref x) => x.stream(w),
            Field::Sender(ref x) => x.stream(w),
            Field::ReplyTo(ref x) => x.stream(w),
            Field::To(ref x) => x.stream(w),
            Field::Cc(ref x) => x.stream(w),
            Field::Bcc(ref x) => x.stream(w),
            Field::MessageId(ref x) => x.stream(w),
            Field::InReplyTo(ref x) => x.stream(w),
            Field::References(ref x) => x.stream(w),
            Field::Subject(ref x) => x.stream(w),
            Field::Comments(ref x) => x.stream(w),
            Field::Keywords(ref x) => x.stream(w),
            Field::OptionalField(ref x) => x.stream(w),
        }
    }
}

// 3.6
// a sub part of the Fields definition
#[derive(Debug, Clone, PartialEq)]
pub struct ResentTraceBlock {
    pub trace: Trace,
    pub resent_fields: Vec<ResentField>,
}
impl Parsable for ResentTraceBlock {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        let mut rem = input;
        if let Ok(t) = parse!(Trace, rem) {
            let mut fields: Vec<ResentField> = Vec::new();
            while let Ok(f) = parse!(ResentField, rem) {
                fields.push(f);
            }
            if fields.len() == 0 {
                Err(ParseError::ExpectedType("Resent Field"))
            } else {
                Ok((ResentTraceBlock {
                    trace: t,
                    resent_fields: fields
                }, rem))
            }
        } else {
            Err(ParseError::NotFound)
        }
    }
}
impl Streamable for ResentTraceBlock {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        let mut count: usize = 0;
        count += try!(self.trace.stream(w));
        for field in &self.resent_fields {
            count += try!(field.stream(w));
        }
        Ok(count)
    }
}

// 3.6
// a sub part of the Fields definition
#[derive(Debug, Clone, PartialEq)]
pub struct OptTraceBlock {
    pub trace: Trace,
    pub opt_fields: Vec<OptionalField>,
}
impl Parsable for OptTraceBlock {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        let mut rem = input;
        if let Ok(t) = parse!(Trace, rem) {
            let mut fields: Vec<OptionalField> = Vec::new();
            while let Ok(f) = parse!(OptionalField, rem) {
                fields.push(f);
            }
            if fields.len() == 0 {
                Err(ParseError::ExpectedType("Optional Field"))
            } else {
                Ok((OptTraceBlock {
                    trace: t,
                    opt_fields: fields
                }, rem))
            }
        } else {
            Err(ParseError::NotFound)
        }
    }
}
impl Streamable for OptTraceBlock {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        let mut count: usize = 0;
        count += try!(self.trace.stream(w));
        for field in &self.opt_fields {
            count += try!(field.stream(w));
        }
        Ok(count)
    }
}

// 3.6
// a sub part of the Fields definition
#[derive(Debug, Clone, PartialEq)]
pub enum TraceBlock {
    Resent(ResentTraceBlock),
    Opt(OptTraceBlock),
}
impl Parsable for TraceBlock {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        let mut rem = input;
        if let Ok(block) = parse!(ResentTraceBlock, rem) {
            Ok((TraceBlock::Resent(block), rem))
        }
        else if let Ok(block) = parse!(OptTraceBlock, rem) {
            Ok((TraceBlock::Opt(block), rem))
        }
        else {
            Err(ParseError::NotFound)
        }
    }
}
impl Streamable for TraceBlock {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        match *self {
            TraceBlock::Resent(ref block) => block.stream(w),
            TraceBlock::Opt(ref block) => block.stream(w),
        }
    }
}

// 3.6
// fields          =   *(trace
//                       *optional-field /
//                       *(resent-date /
//                        resent-from /
//                        resent-sender /
//                        resent-to /
//                        resent-cc /
//                        resent-bcc /
//                        resent-msg-id))
//                     *(orig-date /
//                     from /
//                     sender /
//                     reply-to /
//                     to /
//                     cc /
//                     bcc /
//                     message-id /
//                     in-reply-to /
//                     references /
//                     subject /
//                     comments /
//                     keywords /
//                     optional-field)
#[derive(Debug, Clone, PartialEq)]
pub struct Fields {
    pub trace_blocks: Vec<TraceBlock>,
    pub fields: Vec<Field>,
}
impl Parsable for Fields {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        let mut rem = input;
        let mut trace_blocks: Vec<TraceBlock> = Vec::new();
        while let Ok(tb) = parse!(TraceBlock, rem) {
            trace_blocks.push(tb);
        }
        let mut fields: Vec<Field> = Vec::new();
        while let Ok(f) = parse!(Field, rem) {
            fields.push(f);
        }
        Ok((Fields {
            trace_blocks: trace_blocks,
            fields: fields,
        }, rem))
    }
}
impl Streamable for Fields {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        let mut count: usize = 0;
        for tb in &self.trace_blocks {
            count += try!(tb.stream(w));
        }
        for f in &self.fields {
            count += try!(f.stream(w));
        }
        Ok(count)
    }
}

// 3.5
// text            =   %d1-9 /            ; Characters excluding CR
//                     %d11 /             ;  and LF
//                     %d12 /
//                     %d14-127
#[inline]
pub fn is_text(c: u8) -> bool {
    (c>=1 && c<=9) || c==11 || c==12 || (c>=14 && c<=127)
}
def_cclass!(Text, is_text);

// 3.5
// body            =   (*(*998text CRLF) *998text) / obs-body
#[derive(Debug, Clone, PartialEq)]
// for performance/memory reasons, we store as a Vec<u8>
// rather than Vec<Line> where Line is Vec<Text>.
pub struct Body(pub Vec<u8>);
impl Parsable for Body {
    fn parse(mut input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        let mut body: Vec<u8> = Vec::new();
        let mut line_number: usize = 0;
        loop {
            line_number += 1;
            let mut line: Vec<u8> = Vec::new();
            match input.stream_until_token(b"\r\n", &mut line) {
                Err(e) => return Err(ParseError::Io(e)),
                Ok((_, found)) => {
                    let mut rem = &*line;
                    if let Ok(text) = parse!(Text, rem) {
                        if rem.len() > 0 {
                            return Err(ParseError::InvalidBodyChar(rem[0]));
                        }
                        if text.0.len() > 998 {
                            return Err(ParseError::LineTooLong(line_number));
                        }
                        body.extend(text.0.clone());
                    }
                    if !found { break; } // end of input
                    else { body.extend_from_slice(b"\r\n"); }
                }
            }
        }
        Ok((Body(body), input))
    }
}
impl Streamable for Body {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        w.write(&self.0)
    }
}

// 3.5
// message         =   (fields / obs-fields)
//                     [CRLF body]
#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    pub fields: Fields,
    pub body: Option<Body>
}
impl Parsable for Message {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        let mut rem = input;
        if let Ok(fields) = parse!(Fields, rem) {
            if &rem[..2] != b"\r\n" {
                return Ok((Message {
                    fields: fields,
                    body: None,
                }, rem));
            }
            rem = &rem[2..];
            parse!(Body, rem).map(|b| (Message {
                fields: fields,
                body: Some(b),
            }, rem))
        } else {
            Err(ParseError::NotFound)
        }
    }
}
impl Streamable for Message {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        let mut count: usize = 0;
        count += try!(self.fields.stream(w));
        if let Some(ref body) = self.body {
            count += try!(body.stream(w));
        }
        Ok(count)
    }

}
