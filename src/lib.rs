
#![feature(associated_consts)]

extern crate uuid;
extern crate chrono;

pub mod error;
pub mod types;
#[cfg(test)]
mod tests;

use std::io::Write;
use std::io::Error as IoError;
use std::str::FromStr;
use uuid::Uuid;
use chrono::DateTime;
use chrono::offset::fixed::FixedOffset;
use chrono::offset::local::Local;
use error::Error;

trait HeaderStream {
    const NAME: &'static str;

    /// Stream out the header value
    fn stream_value<W: Write>(&self, w: &mut W) -> Result<usize, IoError>;

    /// Stream out the header
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError>
    {
        Ok(try!(w.write(Self::NAME.as_bytes()))
           + try!(w.write(b": "))
           + try!(self.stream_value(w))
           + try!(w.write(b"\r\n")) )
    }
}

#[derive(Debug, Clone)]
pub struct OrigDate(DateTime<FixedOffset>);
impl HeaderStream for OrigDate {
    const NAME: &'static str = "Date";
    fn stream_value<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        Ok(try!(w.write(
            self.0.format("%a, %e %b %Y %H:%M:%S %z").to_string().as_bytes())))
    }
}

#[derive(Debug, Clone)]
pub struct From(String);
impl HeaderStream for From {
    const NAME: &'static str = "From";
    fn stream_value<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        Ok(try!(w.write(self.0.as_bytes())))
    }
}
impl FromStr for From {
    type Err = Error;
    fn from_str(s: &str) -> Result<From, Error> {
        // FIXME validate
        Ok(From(s.to_owned()))
    }
}

#[derive(Debug, Clone)]
pub struct Sender(String);
impl HeaderStream for Sender {
    const NAME: &'static str = "Sender";
    fn stream_value<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        Ok(try!(w.write(self.0.as_bytes())))
    }
}
impl FromStr for Sender {
    type Err = Error;
    fn from_str(s: &str) -> Result<Sender, Error> {
        // FIXME validate
        Ok(Sender(s.to_owned()))
    }
}

#[derive(Debug, Clone)]
pub struct ReplyTo(String);
impl HeaderStream for ReplyTo {
    const NAME: &'static str = "Reply-To";
    fn stream_value<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        Ok(try!(w.write(self.0.as_bytes())))
    }
}
impl FromStr for ReplyTo {
    type Err = Error;
    fn from_str(s: &str) -> Result<ReplyTo, Error> {
        // FIXME validate
        Ok(ReplyTo(s.to_owned()))
    }
}

#[derive(Debug, Clone)]
pub struct To(String);
impl HeaderStream for To {
    const NAME: &'static str = "To";
    fn stream_value<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        Ok(try!(w.write(self.0.as_bytes())))
    }
}
impl FromStr for To {
    type Err = Error;
    fn from_str(s: &str) -> Result<To, Error> {
        // FIXME validate
        Ok(To(s.to_owned()))
    }
}

#[derive(Debug, Clone)]
pub struct Cc(String);
impl HeaderStream for Cc {
    const NAME: &'static str = "Cc";
    fn stream_value<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        Ok(try!(w.write(self.0.as_bytes())))
    }
}
impl FromStr for Cc {
    type Err = Error;
    fn from_str(s: &str) -> Result<Cc, Error> {
        // FIXME validate
        Ok(Cc(s.to_owned()))
    }
}

#[derive(Debug, Clone)]
pub struct Bcc(String);
impl HeaderStream for Bcc {
    const NAME: &'static str = "Bcc";
    fn stream_value<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        Ok(try!(w.write(self.0.as_bytes())))
    }
}
impl FromStr for Bcc {
    type Err = Error;
    fn from_str(s: &str) -> Result<Bcc, Error> {
        // FIXME validate
        Ok(Bcc(s.to_owned()))
    }
}

#[derive(Debug, Clone)]
pub struct MessageId(Uuid);
impl MessageId {
    pub fn new() -> MessageId {
        MessageId( Uuid::new_v4() )
    }
}
impl HeaderStream for MessageId {
    const NAME: &'static str = "Message-ID";
    fn stream_value<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        let s = format!("{}", self.0);
        Ok(try!(w.write(s.as_bytes())))
    }
}

#[derive(Debug, Clone)]
pub struct InReplyTo(String);
impl HeaderStream for InReplyTo {
    const NAME: &'static str = "In-Reply-To";
    fn stream_value<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        Ok(try!(w.write(self.0.as_bytes())))
    }
}
impl FromStr for InReplyTo {
    type Err = Error;
    fn from_str(s: &str) -> Result<InReplyTo, Error> {
        // FIXME validate
        Ok(InReplyTo(s.to_owned()))
    }
}

#[derive(Debug, Clone)]
pub struct References(String);
impl HeaderStream for References {
    const NAME: &'static str = "References";
    fn stream_value<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        Ok(try!(w.write(self.0.as_bytes())))
    }
}
impl FromStr for References {
    type Err = Error;
    fn from_str(s: &str) -> Result<References, Error> {
        // FIXME validate
        Ok(References(s.to_owned()))
    }
}

#[derive(Debug, Clone)]
pub struct Subject(String);
impl HeaderStream for Subject {
    const NAME: &'static str = "Subject";
    fn stream_value<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        Ok(try!(w.write(self.0.as_bytes())))
    }
}
impl FromStr for Subject {
    type Err = Error;
    fn from_str(s: &str) -> Result<Subject, Error> {
        // FIXME validate
        Ok(Subject(s.to_owned()))
    }
}

#[derive(Debug, Clone)]
pub struct Comments(String);
impl HeaderStream for Comments {
    const NAME: &'static str = "Comments";
    fn stream_value<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        Ok(try!(w.write(self.0.as_bytes())))
    }
}
impl FromStr for Comments {
    type Err = Error;
    fn from_str(s: &str) -> Result<Comments, Error> {
        // FIXME validate
        Ok(Comments(s.to_owned()))
    }
}

#[derive(Debug, Clone)]
pub struct Keywords(String);
impl HeaderStream for Keywords {
    const NAME: &'static str = "Keywords";
    fn stream_value<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        Ok(try!(w.write(self.0.as_bytes())))
    }
}
impl FromStr for Keywords {
    type Err = Error;
    fn from_str(s: &str) -> Result<Keywords, Error> {
        // FIXME validate
        Ok(Keywords(s.to_owned()))
    }
}

#[derive(Debug, Clone)]
pub struct OptionalField(String, String);
impl OptionalField {
    pub fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        Ok(try!(w.write(self.0.as_bytes()))
           + try!(w.write(b": "))
           + try!(w.write(self.1.as_bytes()))
           + try!(w.write(b"\r\n")) )
    }
    pub fn new(name: &str, value: &str) -> Result<OptionalField, Error>
    {
        // FIXME validate
        Ok(OptionalField(name.to_owned(), value.to_owned()))
    }
}

#[derive(Debug, Clone)]
pub struct Body(String);
impl Body {
    pub fn empty() -> Body {
        Body("".to_owned())
    }
    pub fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        Ok(try!(w.write(self.0.as_bytes())))
    }
}
impl FromStr for Body {
    type Err = Error;
    fn from_str(s: &str) -> Result<Body, Error> {
        // FIXME validate
        Ok(Body(s.to_owned()))
    }
}

/// Email structure.  Streamable.  Content is validated and type-safe, and is compliant with
/// RFC 5322 (partial, work TBD).
///
/// ```
/// # extern crate chrono;
/// # extern crate email_format;
/// # pub fn main() {
/// use email_format::Email;
/// use chrono::offset::local::Local;
/// use std::io;
///
/// let mut email = Email::new("nobody@localhost").unwrap();
/// let now = Local::now();
/// email.set_orig_date(now.with_timezone(now.offset()))
///      .set_subject("Hi").unwrap().set_body("Hello").unwrap();
/// email.stream(&mut io::stdout()).unwrap();
/// # }
/// ```

#[derive(Debug, Clone)]
pub struct Email {
    orig_date: OrigDate,
    from: From,
    sender: Option<Sender>,
    reply_to: Option<ReplyTo>,
    to: Option<To>,
    cc: Option<Cc>,
    bcc: Option<Bcc>,
    message_id: Option<MessageId>,
    in_reply_to: Option<InReplyTo>,
    references: Option<References>,
    subject: Option<Subject>,
    comments: Vec<Comments>,
    keywords: Vec<Keywords>,
    optional_fields: Vec<OptionalField>,

    // We will support trace/resent headers at a later time.
    //trace: Vec<Trace>,
    //resent_date: Vec<ResentDate>,
    //resent_from: Vec<ResentFrom>,
    //resent_sender: Vec<ResentSender>,
    //resent_to: Vec<ResentTo>,
    //resent_cc: Vec<ResentCc>,
    //recent_bcc: Vec<ResentBcc>,
    //recent_msg_id: Vec<RecentMsgId>,

    body: Body,
}

impl Email  {
    /// Create a new, default, empty email.
    pub fn new(from: &str) -> Result<Email, Error> {
        Ok(Email {
            orig_date: {
                let now = Local::now();
                OrigDate(now.with_timezone(now.offset()))
            },
            from: try!(from.parse()),
            sender: None,
            reply_to: None,
            to: None,
            cc: None,
            bcc: None,
            message_id: Some(MessageId::new()),
            in_reply_to: None,
            references: None,
            subject: None,
            comments: Vec::new(),
            keywords: Vec::new(),
            optional_fields: Vec::new(),
            body: Body::empty(),
        })
    }

    /// Set the Orig-Date header field (the date when the creator created the email, as
    /// opposed to when it was transported or forwarded)
    pub fn orig_date(mut self, date: DateTime<FixedOffset>) -> Self {
        self.orig_date = OrigDate(date);
        self
    }

    /// Set the Orig-Date header field (the date when the creator created the email, as
    /// opposed to when it was transported or forwarded)
    pub fn set_orig_date<'a>(&'a mut self, date: DateTime<FixedOffset>) -> &'a mut Self {
        self.orig_date = OrigDate(date);
        self
    }

    /// Get the Orig-Date header field (the date when the creator created the email, as
    /// opposed to when it was transported or forwarded)
    pub fn get_orig_date<'a>(&'a self) -> DateTime<FixedOffset>
    {
        self.orig_date.0
    }

    /// Set the From address header field
    pub fn from(mut self, from: &str) -> Result<Self, Error> {
        self.from = try!(from.parse());
        Ok(self)
    }

    /// Set the From address header field
    pub fn set_from<'a>(&'a mut self, from: String) -> Result<&'a mut Self, Error> {
        self.from = try!(from.parse());
        Ok(self)
    }

    /// Get the From address header field
    pub fn get_from<'a>(&'a self) -> &'a String
    {
        &self.from.0
    }

    /// Set the Sender header field
    pub fn sender(mut self, sender: &str) -> Result<Self, Error> {
        self.sender = Some(try!(sender.parse()));
        Ok(self)
    }

    /// Set the Sender header field
    pub fn set_sender<'a>(&'a mut self, sender: &str) -> Result<&'a mut Self, Error> {
        self.sender = Some(try!(sender.parse()));
        Ok(self)
    }

    /// Unset the Sender header field
    pub fn unset_sender<'a>(&'a mut self) -> &'a mut Self {
        self.sender = None;
        self
    }

    /// Get the Sender header field
    pub fn get_sender<'a>(&'a self) -> Option<&'a String>
    {
        match self.sender {
            None => None,
            Some(ref s) => Some(&s.0)
        }
    }

    /// Set the Reply-To header field
    pub fn reply_to(mut self, reply_to: &str) -> Result<Self, Error> {
        self.reply_to = Some(try!(reply_to.parse()));
        Ok(self)
    }

    /// Set the Reply-To header field
    pub fn set_reply_to<'a>(&'a mut self, reply_to: &str) -> Result<&'a mut Self, Error> {
        self.reply_to = Some(try!(reply_to.parse()));
        Ok(self)
    }

    pub fn unset_reply_to<'a>(&'a mut self) -> &'a mut Self {
        self.reply_to = None;
        self
    }

    /// Get the Reply-To header field
    pub fn get_reply_to<'a>(&'a self) -> Option<&'a String>
    {
        match self.reply_to {
            None => None,
            Some(ref s) => Some(&s.0)
        }
    }

    /// Set the To header field
    pub fn to(mut self, to: &str) -> Result<Self, Error> {
        self.to = Some(try!(to.parse()));
        Ok(self)
    }

    /// Set the To header field
    pub fn set_to<'a>(&'a mut self, to: &str) -> Result<&'a mut Self, Error> {
        self.to = Some(try!(to.parse()));
        Ok(self)
    }

    /// Unset the To header field
    pub fn unset_to<'a>(&'a mut self) -> &'a mut Self {
        self.to = None;
        self
    }

    /// Get the To header field
    pub fn get_to<'a>(&'a self) -> Option<&'a String>
    {
        match self.to {
            None => None,
            Some(ref s) => Some(&s.0)
        }
    }

    /// Set the Cc header field
    pub fn cc(mut self, cc: &str) -> Result<Self, Error> {
        self.cc = Some(try!(cc.parse()));
        Ok(self)
    }

    /// Set the Cc header field
    pub fn set_cc<'a>(&'a mut self, cc: &str) -> Result<&'a mut Self, Error> {
        self.cc = Some(try!(cc.parse()));
        Ok(self)
    }

    /// Unset the Cc header field
    pub fn unset_cc<'a>(&'a mut self) -> &'a mut Self {
        self.cc = None;
        self
    }

    /// Get the Cc header field
    pub fn get_cc<'a>(&'a self) -> Option<&'a String>
    {
        match self.cc {
            None => None,
            Some(ref s) => Some(&s.0)
        }
    }

    /// Set the Bcc header field
    pub fn bcc(mut self, bcc: &str) -> Result<Self, Error> {
        self.bcc = Some(try!(bcc.parse()));
        Ok(self)
    }

    /// Set the Bcc header field
    pub fn set_bcc<'a>(&'a mut self, bcc: &str) -> Result<&'a mut Self, Error> {
        self.bcc = Some(try!(bcc.parse()));
        Ok(self)
    }

    pub fn unset_bcc<'a>(&'a mut self) -> &'a mut Self {
        self.bcc = None;
        self
    }

    /// Get the Bcc header field
    pub fn get_bcc<'a>(&'a self) -> Option<&'a String>
    {
        match self.bcc {
            None => None,
            Some(ref s) => Some(&s.0)
        }
    }

    /// Set the Message-Id header field (this field is already set by new() to a randomly
    /// generated Uuid, but with this function you can override that)
    pub fn message_id(mut self, message_id: Uuid) -> Self {
        self.message_id = Some(MessageId(message_id));
        self
    }

    /// Set the Message-Id header field (this field is already set by new() to a randomly
    /// generated Uuid, but with this function you can override that)
    pub fn set_message_id<'a>(&'a mut self, message_id: Uuid) -> &'a mut Self {
        self.message_id = Some(MessageId(message_id));
        self
    }

    /// Unset the Message-Id header field
    pub fn unset_message_id<'a>(&'a mut self) -> &'a mut Self {
        self.message_id = None;
        self
    }

    /// Get the Message-Id header field
    pub fn get_message_id<'a>(&'a self) -> Option<&'a Uuid> {
        match self.message_id {
            None => None,
            Some(ref s) => Some(&s.0)
        }
    }

    /// Set the In-Reply-To header field
    pub fn in_reply_to(mut self, in_reply_to: &str) -> Result<Self, Error> {
        self.in_reply_to = Some(try!(in_reply_to.parse()));
        Ok(self)
    }

    /// Set the In-Reply-To header field
    pub fn set_in_reply_to<'a>(&'a mut self, in_reply_to: &str) -> Result<&'a mut Self, Error> {
        self.in_reply_to = Some(try!(in_reply_to.parse()));
        Ok(self)
    }

    /// Unset the In-Reply-To header field
    pub fn unset_in_reply_to<'a>(&'a mut self) -> &'a mut Self {
        self.in_reply_to = None;
        self
    }

    /// Get the In-Reply-To header field
    pub fn get_in_reply_to<'a>(&'a self) -> Option<&'a String> {
        match self.in_reply_to {
            None => None,
            Some(ref s) => Some(&s.0)
        }
    }

    /// Set the References header field
    pub fn references(mut self, references: &str) -> Result<Self, Error> {
        self.references = Some(try!(references.parse()));
        Ok(self)
    }

    /// Set the References header field
    pub fn set_references<'a>(&'a mut self, references: &str) -> Result<&'a mut Self, Error> {
        self.references = Some(try!(references.parse()));
        Ok(self)
    }

    /// Unset the References header field
    pub fn unset_references<'a>(&'a mut self) -> &'a mut Self {
        self.references = None;
        self
    }

    /// Get the References header field
    pub fn get_references<'a>(&'a self) -> Option<&'a String> {
        match self.references {
            None => None,
            Some(ref s) => Some(&s.0)
        }
    }

    /// Set the Subject header field
    pub fn subject(mut self, subject: &str) -> Result<Self, Error> {
        self.subject = Some(try!(subject.parse()));
        Ok(self)
    }

    /// Set the Subject header field
    pub fn set_subject<'a>(&'a mut self, subject: &str) -> Result<&'a mut Self, Error> {
        self.subject = Some(try!(subject.parse()));
        Ok(self)
    }

    /// Unset the Subject header field
    pub fn unset_subject<'a>(&'a mut self) -> &'a mut Self {
        self.subject = None;
        self
    }

    /// Get the Subject header field
    pub fn get_subject<'a>(&'a self) -> Option<&'a String> {
        match self.subject {
            None => None,
            Some(ref s) => Some(&s.0)
        }
    }

    /// Add a Comments header field
    pub fn comments(mut self, comments: &str) -> Result<Self, Error> {
        self.comments.push(try!(comments.parse()));
        Ok(self)
    }

    /// Add a Comments header field
    pub fn add_comments<'a>(&'a mut self, comments: &str) -> Result<&'a mut Self, Error> {
        self.comments.push(try!(comments.parse()));
        Ok(self)
    }

    /// Clear all Comments header fields
    pub fn clear_comments<'a>(&'a mut self) -> &'a mut Self {
        self.comments = Vec::new();
        self
    }

    /// Get Comments header fields
    pub fn get_comments<'a>(&'a self) -> Vec<&String> {
        self.comments.iter().map(|x| &x.0).collect()
    }

    /// Add a Keywords header field
    pub fn keywords(mut self, keywords: &str) -> Result<Self, Error> {
        self.keywords.push(try!(keywords.parse()));
        Ok(self)
    }

    /// Add a Keywords header field
    pub fn add_keywords<'a>(&'a mut self, keywords: &str) -> Result<&'a mut Self, Error> {
        self.keywords.push(try!(keywords.parse()));
        Ok(self)
    }

    /// Clear all Keywords header fields
    pub fn clear_keywords<'a>(&'a mut self) -> &'a mut Self {
        self.keywords = Vec::new();
        self
    }

    /// Get Keywords header fields
    pub fn get_keywords<'a>(&'a self) -> Vec<&String> {
        self.keywords.iter().map(|x| &x.0).collect()
    }

    /// Add an optional (user-defined) header field
    pub fn optional_field(mut self, field: &str, value: &str) -> Result<Self, Error> {
        self.optional_fields.push(try!(OptionalField::new(field, value)));
        Ok(self)
    }

    /// Add an optional (user-defined) header field
    pub fn add_optional_field<'a>(&'a mut self, field: &str, value: &str)
                                  -> Result<&'a mut Self, Error> {
        self.optional_fields.push(try!(OptionalField::new(field, value)));
        Ok(self)
    }

    /// Clear all optional (user-defined) header fields
    pub fn clear_optional_fields<'a>(&'a mut self) -> &'a mut Self {
        self.optional_fields = Vec::new();
        self
    }

    /// Get optional (user-defined) header fields
    pub fn get_optional_fields<'a>(&'a self) -> Vec<(&String, &String)>
    {
        self.optional_fields.iter().map(|x| (&x.0, &x.1)).collect()
    }

    /// Set body
    pub fn body(mut self, body: &str) -> Result<Self, Error> {
        self.body = try!(body.parse());
        Ok(self)
    }

    /// Set body
    pub fn set_body<'a>(&'a mut self, body: &str) -> Result<&'a mut Self, Error> {
        self.body = try!(body.parse());
        Ok(self)
    }

    /// Unset body
    pub fn unset_body<'a>(&'a mut self) -> &'a mut Self {
        self.body = Body::empty();
        self
    }

    /// Stream out the email.  Returns the number of bytes written, on success.
    pub fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        let mut count = 0;
        count += try!(self.orig_date.stream(w));
        count += try!(self.from.stream(w));
        if let Some(ref sender) = self.sender {
            count += try!(sender.stream(w));
        }
        if let Some(ref reply_to) = self.reply_to {
            count += try!(reply_to.stream(w));
        }
        if let Some(ref to) = self.to {
            count += try!(to.stream(w));
        }
        if let Some(ref cc) = self.cc {
            count += try!(cc.stream(w));
        }
        if let Some(ref bcc) = self.bcc {
            count += try!(bcc.stream(w));
        }
        if let Some(ref message_id) = self.message_id {
            count += try!(message_id.stream(w));
        }
        if let Some(ref in_reply_to) = self.in_reply_to {
            count += try!(in_reply_to.stream(w));
        }
        if let Some(ref references) = self.references {
            count += try!(references.stream(w));
        }
        if let Some(ref subject) = self.subject {
            count += try!(subject.stream(w));
        }
        for comment in &self.comments {
            count += try!(comment.stream(w));
        }
        for keywords in &self.keywords {
            count += try!(keywords.stream(w));
        }
        for optional_field in &self.optional_fields {
            count += try!(optional_field.stream(w));
        }
        count += try!(w.write(b"\r\n"));

        count += try!(self.body.stream(w));

        Ok(count)
    }
}
