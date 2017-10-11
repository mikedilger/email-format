//! This crate allows you to construct email messages in a way that assures that
//! they are compliant with relevant email standards (especially RFC 5322).  Invalid
//! data submitted will return a ParseError.
//!
//! The main structure to work with is `Email`. It has many functions to set or add
//! headers and to set the body. All of these will accept an `&str` or `&[u8]` argument
//! and attempt to parse it. These setters return a `Result<(), ParseError>` as the parse
//! may fail.
//!
//! ```
//! extern crate email_format;
//!
//! use email_format::Email;
//!
//! fn main() {
//!   let mut email = Email::new(
//!       "myself@mydomain.com",  // "From:"
//!       "Wed, 05 Jan 2015 15:13:05 +1300" // "Date:"
//!   ).unwrap();
//!   email.set_sender("from_myself@mydomain.com").unwrap();
//!   email.set_reply_to("My Mailer <no-reply@mydomain.com>").unwrap();
//!   email.set_to("You <you@yourdomain.com>").unwrap();
//!   email.set_cc("Our Friend <friend@frienddomain.com>").unwrap();
//!   email.set_message_id("<id/20161128115731.29084.maelstrom@mydomain.com>").unwrap();
//!   email.set_subject("Hello Friend").unwrap();
//!   email.set_body("Good to hear from you.\r\n\
//!                   I wish you the best.\r\n\
//!                   \r\n\
//!                   Your Friend").unwrap();
//!
//!   println!("{}", email);
//! }
//! ```
//!
//! This outputs:
//!
//! ```text
//! Date:Wed, 05 Jan 2015 15:13:05 +1300
//! From:myself@mydomain.com
//! Sender:from_myself@mydomain.com
//! Reply-To:My Mailer <no-reply@mydomain.com>
//! To:You <you@yourdomain.com>
//! Cc:Our Friend <friend@frienddomain.com>
//! Message-ID:<id/20161128115731.29084.maelstrom@mydomain.com>
//! Subject:Hello Friend
//!
//! Good to hear from you.
//! I wish you the best.
//!
//! Your Friend
//! ```
//!
//! On the other hand, the following will fail because the sender email address is invalid:
//!
//! ```rust,should_panic
//! extern crate email_format;
//!
//! use email_format::Email;
//!
//! fn main() {
//!   let mut email = Email::new(
//!       "myself@mydomain.com",  // "From:"
//!       "Wed, 05 Jan 2015 15:13:05 +1300" // "Date:"
//!   ).unwrap();
//!   email.set_sender("from_myself@(mydomain.com)").unwrap();
//! }
//! ```

extern crate buf_read_ext;

#[cfg(feature="time")]
extern crate time;
#[cfg(feature="chrono")]
extern crate chrono;

#[cfg(test)]
mod tests;

/// This module contains nitty-gritty details about parsing, storage, and streaming
/// an `Email`.
pub mod rfc5322;

use std::io::Write;
use std::io::Error as IoError;
use std::fmt;

use rfc5322::{Message, Fields, Field};
use rfc5322::{Parsable, Streamable};
use rfc5322::error::ParseError;
use rfc5322::Body;
use rfc5322::headers::{From, OrigDate, Sender, ReplyTo, To, Cc, Bcc, MessageId,
                           InReplyTo, References, Subject, Comments, Keywords,
                           OptionalField};

/// Attempt to construct `Self` via a conversion (borrowed from rust `std`)
///
/// This TryFrom trait is defined in the rust std library but is behind a
/// feature gate.  We place it here so that people using stable compilers
/// can still use our crate.  In the future, the std trait should be used.
pub trait TryFrom<T>: Sized {
    /// The type returned in the event of a conversion error.
    type Error;

    /// Performs the conversion.
    fn try_from(T) -> Result<Self, Self::Error>;
}

// We implement TryFrom from T to T with our ParseError for crate ergonomics
// (Rust won't let it be implemented with an unconstrained error type)
impl<T> TryFrom<T> for T {
    type Error = ::rfc5322::error::ParseError;
    fn try_from(input: T) -> Result<T, Self::Error> {
        Ok(input)
    }
}

#[derive(Debug, Clone)]
pub struct Email {
    message: Message,
}

impl Email {
    pub fn new<F,D>(from: F, date: D) -> Result<Email, ParseError>
        where From: TryFrom<F, Error=ParseError>, OrigDate: TryFrom<D, Error=ParseError>
    {
        Ok(Email {
            message: Message {
                fields: Fields {
                    trace_blocks: vec![],
                    fields: vec![
                        Field::OrigDate(try!(TryFrom::try_from(date))),
                        Field::From(try!(TryFrom::try_from(from))) ],
                },
                body: None,
            }
        })
    }

    pub fn set_date<D>(&mut self, date: D) -> Result<(), ParseError>
        where OrigDate: TryFrom<D, Error=ParseError>
    {
        let value: OrigDate = try!(TryFrom::try_from(date));
        for field in self.message.fields.fields.iter_mut() {
            if let Field::OrigDate(_) = *field {
                *field = Field::OrigDate(value);
                return Ok(())
            }
        }
        unreachable!()
    }
    pub fn get_date(&self) -> OrigDate {
        for field in self.message.fields.fields.iter() {
            if let Field::OrigDate(ref d) = *field {
                return d.clone();
            }
        }
        unreachable!()
    }

    pub fn set_from<F>(&mut self, from: F) -> Result<(), ParseError>
        where From: TryFrom<F, Error=ParseError>
    {
        let value: From = try!(TryFrom::try_from(from));
        for field in self.message.fields.fields.iter_mut() {
            if let Field::From(_) = *field {
                *field = Field::From(value);
                return Ok(());
            }
        }
        unreachable!()
    }
    pub fn get_from(&self) -> From {
        for field in self.message.fields.fields.iter() {
            if let Field::From(ref f) = *field {
                return f.clone()
            }
        }
        unreachable!()
    }

    pub fn set_sender<S>(&mut self, sender: S) -> Result<(), ParseError>
        where Sender: TryFrom<S, Error=ParseError>
    {
        let value: Sender = try!(TryFrom::try_from(sender));
        for field in self.message.fields.fields.iter_mut() {
            if let Field::Sender(_) = *field {
                *field = Field::Sender(value);
                return Ok(());
            }
        }
        self.message.fields.fields.push(Field::Sender(value));
        Ok(())
    }
    pub fn get_sender(&self) -> Option<Sender> {
        for field in self.message.fields.fields.iter() {
            if let Field::Sender(ref s) = *field {
                return Some(s.clone());
            }
        }
        None
    }
    pub fn clear_sender(&mut self) {
        self.message.fields.fields.retain(|field| {
            if let Field::Sender(_) = *field { false } else { true }
        });
    }

    pub fn set_reply_to<R>(&mut self, reply_to: R) -> Result<(), ParseError>
        where ReplyTo: TryFrom<R, Error=ParseError>
    {
        let value: ReplyTo = try!(TryFrom::try_from(reply_to));
        for field in self.message.fields.fields.iter_mut() {
            if let Field::ReplyTo(_) = *field {
                *field = Field::ReplyTo(value);
                return Ok(());
            }
        }
        self.message.fields.fields.push(Field::ReplyTo(value));
        Ok(())
    }
    pub fn get_reply_to(&self) -> Option<ReplyTo> {
        for field in self.message.fields.fields.iter() {
            if let Field::ReplyTo(ref rt) = *field {
                return Some(rt.clone())
            }
        }
        None
    }
    pub fn clear_reply_to(&mut self) {
        self.message.fields.fields.retain(|field| {
            if let Field::ReplyTo(_) = *field { false } else { true }
        });
    }

    pub fn set_to<T>(&mut self, to: T) -> Result<(), ParseError>
        where To: TryFrom<T, Error=ParseError>
    {
        let value: To = try!(TryFrom::try_from(to));
        for field in self.message.fields.fields.iter_mut() {
            if let Field::To(_) = *field {
                *field = Field::To(value);
                return Ok(());
            }
        }
        self.message.fields.fields.push(Field::To(value));
        Ok(())
    }
    pub fn get_to(&self) -> Option<To> {
        for field in self.message.fields.fields.iter() {
            if let Field::To(ref t) = *field {
                return Some(t.clone())
            }
        }
        None
    }
    pub fn clear_to(&mut self) {
        self.message.fields.fields.retain(|field| {
            if let Field::To(_) = *field { false } else { true }
        });
    }

    pub fn set_cc<C>(&mut self, cc: C) -> Result<(), ParseError>
        where Cc: TryFrom<C, Error=ParseError>
    {
        let value: Cc = try!(TryFrom::try_from(cc));
        for field in self.message.fields.fields.iter_mut() {
            if let Field::Cc(_) = *field {
                *field = Field::Cc(value);
                return Ok(());
            }
        }
        self.message.fields.fields.push(Field::Cc(value));
        Ok(())
    }
    pub fn get_cc(&self) -> Option<Cc> {
        for field in self.message.fields.fields.iter() {
            if let Field::Cc(ref cc) = *field {
                return Some(cc.clone())
            }
        }
        None
    }
    pub fn clear_cc(&mut self) {
        self.message.fields.fields.retain(|field| {
            if let Field::Cc(_) = *field { false } else { true }
        });
    }

    pub fn set_bcc<B>(&mut self, bcc: B) -> Result<(), ParseError>
        where Bcc: TryFrom<B, Error=ParseError>
    {
        let value: Bcc = try!(TryFrom::try_from(bcc));
        for field in self.message.fields.fields.iter_mut() {
            if let Field::Bcc(_) = *field {
                *field = Field::Bcc(value);
                return Ok(());
            }
        }
        self.message.fields.fields.push(Field::Bcc(value));
        Ok(())
    }
    pub fn get_bcc(&self) -> Option<Bcc> {
        for field in self.message.fields.fields.iter() {
            if let Field::Bcc(ref b) = *field {
                return Some(b.clone())
            }
        }
        None
    }
    pub fn clear_bcc(&mut self) {
        self.message.fields.fields.retain(|field| {
            if let Field::Bcc(_) = *field { false } else { true }
        });
    }

    pub fn set_message_id<M>(&mut self, message_id: M) -> Result<(), ParseError>
        where MessageId: TryFrom<M, Error=ParseError>
    {
        let value: MessageId = try!(TryFrom::try_from(message_id));
        for field in self.message.fields.fields.iter_mut() {
            if let Field::MessageId(_) = *field {
                *field = Field::MessageId(value);
                return Ok(());
            }
        }
        self.message.fields.fields.push(Field::MessageId(value));
        Ok(())
    }
    pub fn get_message_id(&self) -> Option<MessageId> {
        for field in self.message.fields.fields.iter() {
            if let Field::MessageId(ref m) = *field {
                return Some(m.clone())
            }
        }
        None
    }
    pub fn clear_message_id(&mut self) {
        self.message.fields.fields.retain(|field| {
            if let Field::MessageId(_) = *field { false } else { true }
        });
    }

    pub fn set_in_reply_to<I>(&mut self, in_reply_to: I) -> Result<(), ParseError>
        where InReplyTo: TryFrom<I, Error=ParseError>
    {
        let value: InReplyTo = try!(TryFrom::try_from(in_reply_to));
        for field in self.message.fields.fields.iter_mut() {
            if let Field::InReplyTo(_) = *field {
                *field = Field::InReplyTo(value);
                return Ok(());
            }
        }
        self.message.fields.fields.push(Field::InReplyTo(value));
        Ok(())
    }
    pub fn get_in_reply_to(&self) -> Option<InReplyTo> {
        for field in self.message.fields.fields.iter() {
            if let Field::InReplyTo(ref x) = *field {
                return Some(x.clone())
            }
        }
        None
    }
    pub fn clear_in_reply_to(&mut self) {
        self.message.fields.fields.retain(|field| {
            if let Field::InReplyTo(_) = *field { false } else { true }
        });
    }

    pub fn set_references<R>(&mut self, references: R) -> Result<(), ParseError>
        where References: TryFrom<R, Error=ParseError>
    {
        let value: References = try!(TryFrom::try_from(references));
        for field in self.message.fields.fields.iter_mut() {
            if let Field::References(_) = *field {
                *field = Field::References(value);
                return Ok(());
            }
        }
        self.message.fields.fields.push(Field::References(value));
        Ok(())
    }
    pub fn get_references(&self) -> Option<References> {
        for field in self.message.fields.fields.iter() {
            if let Field::References(ref x) = *field {
                return Some(x.clone())
            }
        }
        None
    }
    pub fn clear_references(&mut self) {
        self.message.fields.fields.retain(|field| {
            if let Field::References(_) = *field { false } else { true }
        });
    }

    pub fn set_subject<S>(&mut self, subject: S) -> Result<(), ParseError>
        where Subject: TryFrom<S, Error=ParseError>
    {
        let value: Subject = try!(TryFrom::try_from(subject));
        for field in self.message.fields.fields.iter_mut() {
            if let Field::Subject(_) = *field {
                *field = Field::Subject(value);
                return Ok(());
            }
        }
        self.message.fields.fields.push(Field::Subject(value));
        Ok(())
    }
    pub fn get_subject(&self) -> Option<Subject> {
        for field in self.message.fields.fields.iter() {
            if let Field::Subject(ref x) = *field {
                return Some(x.clone())
            }
        }
        None
    }
    pub fn clear_subject(&mut self) {
        self.message.fields.fields.retain(|field| {
            if let Field::Subject(_) = *field { false } else { true }
        });
    }

    pub fn add_comments<C>(&mut self, comments: C) -> Result<(), ParseError>
        where Comments: TryFrom<C, Error=ParseError>
    {
        let value: Comments = try!(TryFrom::try_from(comments));
        self.message.fields.fields.push(Field::Comments(value));
        Ok(())
    }
    pub fn get_comments(&self) -> Vec<Comments> {
        let mut output: Vec<Comments> = Vec::new();
        for field in self.message.fields.fields.iter() {
            if let Field::Comments(ref x) = *field {
                output.push(x.clone());
            }
        }
        output
    }
    pub fn clear_comments(&mut self) {
        self.message.fields.fields.retain(|field| {
            if let Field::Comments(_) = *field { false } else { true }
        });
    }

    pub fn add_keywords<K>(&mut self, keywords: K) -> Result<(), ParseError>
        where Keywords: TryFrom<K, Error=ParseError>
    {
        let value: Keywords = try!(TryFrom::try_from(keywords));
        self.message.fields.fields.push(Field::Keywords(value));
        Ok(())
    }
    pub fn get_keywords(&self) -> Vec<Keywords> {
        let mut output: Vec<Keywords> = Vec::new();
        for field in self.message.fields.fields.iter() {
            if let Field::Keywords(ref x) = *field {
                output.push(x.clone());
            }
        }
        output
    }
    pub fn clear_keywords(&mut self) {
        self.message.fields.fields.retain(|field| {
            if let Field::Keywords(_) = *field { false } else { true }
        });
    }

    pub fn add_optional_field<O>(&mut self, optional_field: O) -> Result<(), ParseError>
        where OptionalField: TryFrom<O, Error=ParseError>
    {
        let value: OptionalField = try!(TryFrom::try_from(optional_field));
        self.message.fields.fields.push(Field::OptionalField(value));
        Ok(())
    }
    pub fn get_optional_fields(&self) -> Vec<OptionalField> {
        let mut output: Vec<OptionalField> = Vec::new();
        for field in self.message.fields.fields.iter() {
            if let Field::OptionalField(ref x) = *field {
                output.push(x.clone());
            }
        }
        output
    }

    // TBD: trace
    // TBD: resent-date
    // TBD: resent-from
    // TBD: resent-sender
    // TBD: resent-to
    // TBD: resent-cc
    // TBD: resent-bcc
    // TBD: resent-msg-id

    pub fn set_body<B>(&mut self, body: B) -> Result<(), ParseError>
        where Body: TryFrom<B, Error=ParseError>
    {
        let value: Body = try!(TryFrom::try_from(body));
        self.message.body = Some(value);
        Ok(())
    }
    pub fn get_body(&self) -> Option<Body> {
        self.message.body.clone()
    }
    pub fn clear_body(&mut self) {
        self.message.body = None;
    }
}

impl Parsable for Email {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        let mut rem = input;
        match Message::parse(rem).map(|(value, r)| { rem = r; value }) {
            Ok(message) => Ok((Email { message: message}, rem)),
            Err(e) => Err(ParseError::Parse("Email", Box::new(e)))
        }
    }
}

impl Streamable for Email {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        self.message.stream(w)
    }
}

impl fmt::Display for Email {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let mut output: Vec<u8> = Vec::new();
        if let Err(_) = self.stream(&mut output) {
            return Err(fmt::Error);
        }
        unsafe {
            // rfc5322 formatted emails fall within utf8
            write!(f, "{}", ::std::str::from_utf8_unchecked(&*output))
        }
    }
}
