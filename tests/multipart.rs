extern crate email_format;
extern crate mime_multipart;
extern crate hyper;

use email_format::Email;
use mime_multipart::{Part, Node};
use hyper::header::{Headers, ContentType};
use hyper::mime::{Mime, TopLevel, SubLevel, Attr, Value};

fn main() {

    let body = "Good to hear from you, Hans Müeller.\r\n\
                I wish you the best.\r\n\
                \r\n\
                Your Friend,
                黛安娜";

    let part = Part {
        headers: {
            let mut h = Headers::new();
            h.set(ContentType(Mime(TopLevel::Text, SubLevel::Plain,
                                   vec![(Attr::Charset, Value::Utf8)])));
            h
        },
        body: body.as_bytes().to_owned(),
    };
    let nodes: Vec<Node> = vec![ Node::Part(part) ];
    let boundary = ::mime_multipart::generate_boundary();
    let mut part_bytes: Vec<u8> = Vec::new();
    ::mime_multipart::write_multipart(&mut part_bytes, &boundary, &nodes).unwrap();

    let mut email = Email::new(
        "myself@mydomain.com",  // "From:"
        "Wed, 05 Jan 2015 15:13:05 +1300" // "Date:"
    ).unwrap();
    email.set_sender("from_myself@mydomain.com").unwrap();
    email.set_reply_to("My Mailer <no-reply@mydomain.com>").unwrap();
    email.set_to("You <you@yourdomain.com>").unwrap();
    email.set_cc("Our Friend <friend@frienddomain.com>").unwrap();
    email.set_message_id("<id/20161128115731.29084.maelstrom@mydomain.com>").unwrap();
    email.set_subject("Hello Friend").unwrap();
    email.add_optional_field(("MIME-Version", "1.0")).unwrap();
    email.add_optional_field(("Content-Type",
                             &*format!("multipart/alternative; boundary=\"{}\"",
                             unsafe { String::from_utf8_unchecked(boundary) } ))).unwrap();
    email.set_body(&*part_bytes).unwrap();

    println!("{}", email);
}
