
macro_rules! assert_match {
    ($left:expr, $right:pat) => {
        match $left {
            $right => true,
            _ => false
        }
    };
}

use rfc5322::{Parsable, ParseError, Streamable};

#[test]
fn test_alpha() {
    use rfc5322::types::Alpha;

    let (alpha, rem) = Alpha::parse(b"abcdEFZz123").unwrap();
    assert_eq!(alpha, Alpha(b"abcdEFZz".to_vec()));
    assert_eq!(rem, b"123");

    let err = Alpha::parse(b"").err().unwrap();
    assert_match!(err, ParseError::Eof("Alpha"));

    let err = Alpha::parse(b"123").err().unwrap();
    assert_match!(err, ParseError::NotFound("Alpha"));

    let mut output: Vec<u8> = Vec::new();
    assert_eq!(alpha.stream(&mut output).unwrap(), 8);
    assert_eq!(output, b"abcdEFZz".to_vec());
}

#[test]
fn test_parse_quoted_pair() {
    use rfc5322::types::QuotedPair;

    let err = QuotedPair::parse(b"not").err().unwrap();
    assert_match!(err, ParseError::NotFound("Quoted Pair"));
    let err = QuotedPair::parse(b"\\").err().unwrap();
    assert_match!(err, ParseError::NotFound("Quoted Pair"));
    let (token, rem) = QuotedPair::parse(b"\\n").unwrap();
    assert_eq!(token, QuotedPair(b'n'));
    assert_eq!(rem, b"");
    let qp = QuotedPair(b'n');
    let mut output: Vec<u8> = Vec::new();
    assert_eq!(qp.stream(&mut output).unwrap(), 2);
    assert_eq!(output, b"\\n");
}

#[test]
fn test_fws() {
    use rfc5322::types::FWS;

    let (token, rem) = FWS::parse(b"   ").unwrap();
    assert_eq!(token, FWS);
    assert_eq!(rem, b"");
    let (token, rem) = FWS::parse(b" \r\n  \t").unwrap();
    assert_eq!(token, FWS);
    assert_eq!(rem, b"");
    let (token, rem) = FWS::parse(b" \r ").unwrap();
    assert_eq!(token, FWS);
    assert_eq!(rem, b"\r ");
    let err = FWS::parse(b"\n ").err().unwrap();
    assert_match!(err, ParseError::NotFound("Folding White Space"));
    let err = FWS::parse(b"\r\n").err().unwrap();
    assert_match!(err, ParseError::NotFound("Folding White Space"));
    let (token, rem) = FWS::parse(b"\r\n\tx").unwrap();
    assert_eq!(token, FWS);
    assert_eq!(rem, b"x");
}

#[test]
fn test_ctext() {
    use rfc5322::types::CText;

    let input = b"Thi,s;1:23isCt_#ext".to_vec();
    let (token, remainder) = CText::parse(input.as_slice()).unwrap();
    assert_eq!(token, CText(input.clone()));
    assert_eq!(remainder, b"");
}

#[test]
fn test_ccontent() {
    use rfc5322::types::{CContent, CText, QuotedPair};

    let input = b"Thi,s;1:23isCt_#ext".to_vec();
    let (token, _) = CContent::parse(input.as_slice()).unwrap();
    assert_eq!(token, CContent::CText(CText(input.clone())));

    let input = b"\\n".to_vec();
    let (token, _) = CContent::parse(input.as_slice()).unwrap();
    assert_eq!(token, CContent::QuotedPair(QuotedPair(b'n')));

    let input = b"(Comments can contain whitespace and \\( quoted \\\\ characters, and even ( nesting ) with or (without) whitepsace, but must balance parenthesis)".to_vec();
    let (_, remainder) = CContent::parse(input.as_slice()).unwrap();
    assert_eq!(remainder, b"");
}

#[test]
fn test_comment() {
    use rfc5322::types::{Comment, CContent, CText, QuotedPair};

    let input = b"( a,b,c\t \\nYes (and so on) \r\n )".to_vec();
    let (token, rem) = Comment::parse(input.as_slice()).unwrap();
    assert_eq!(token, Comment {
        ccontent: vec![
            (true, CContent::CText( CText(b"a,b,c".to_vec()) )),
            (true, CContent::QuotedPair( QuotedPair(b'n') )),
            (false, CContent::CText( CText(b"Yes".to_vec()) )),
            (true, CContent::Comment(Comment {
                ccontent: vec![
                    (false, CContent::CText( CText(b"and".to_vec()) )),
                    (true, CContent::CText( CText(b"so".to_vec()) )),
                    (true, CContent::CText( CText(b"on".to_vec()) )) ],
                trailing_ws: false
            }))],
        trailing_ws: true,
    });
    assert_eq!(rem, b"");

    let mut output: Vec<u8> = Vec::new();
    assert_eq!(token.stream(&mut output).unwrap(), 27);
    assert_eq!(output, b"( a,b,c \\nYes (and so on) )");
}

#[test]
fn test_cfws() {
    use rfc5322::types::{CFWS, Comment, CContent, CText, QuotedPair};

    let input = b"  \t( a,b,c\t \\nYes (and so on) \r\n ) \r\n ".to_vec();
    let (token, rem) = CFWS::parse(input.as_slice()).unwrap();
    assert_eq!(token, CFWS {
        comments: vec![
            (true, Comment {
                ccontent: vec![
                    (true, CContent::CText( CText(b"a,b,c".to_vec()) )),
                    (true, CContent::QuotedPair( QuotedPair(b'n') )),
                    (false, CContent::CText( CText(b"Yes".to_vec()) )),
                    (true, CContent::Comment(Comment {
                        ccontent: vec![
                            (false, CContent::CText( CText(b"and".to_vec()) )),
                            (true, CContent::CText( CText(b"so".to_vec()) )),
                            (true, CContent::CText( CText(b"on".to_vec()) )) ],
                        trailing_ws: false
                    }))],
                trailing_ws: true,
            })],
        trailing_ws: true,
    });
    assert_eq!(rem, b"");

    let mut output: Vec<u8> = Vec::new();
    assert_eq!(token.stream(&mut output).unwrap(), 29);
    assert_eq!(output, b" ( a,b,c \\nYes (and so on) ) ");

    let input = b"(abc)(def\r\n )".to_vec();
    let (token, _) = CFWS::parse(input.as_slice()).unwrap();
    assert_eq!(token, CFWS {
        comments: vec![
            (false, Comment {
                ccontent: vec![
                    (false, CContent::CText( CText(b"abc".to_vec()) )) ],
                trailing_ws: false,
            }),
            (false, Comment {
                ccontent: vec![
                    (false, CContent::CText( CText(b"def".to_vec()) )) ],
                trailing_ws: true,
            }),
            ],
        trailing_ws: false,
    });
}

#[test]
fn test_atom() {
    use rfc5322::types::Atom;

    let input = b"  \t( a,b,c\t \\nYes (and so on) \r\n ) atom\r\n ".to_vec();
    let (atom, remainder) = Atom::parse(input.as_slice()).unwrap();
    assert_eq!(atom.atext.0, b"atom".to_vec());
    assert_eq!(remainder, b"");

    let input = b" \t AMZamz019!#$%&'*+-/=?^_`{|}~ \t ".to_vec();
    let (atom, remainder) = Atom::parse(input.as_slice()).unwrap();
    assert_eq!(atom.atext.0, b"AMZamz019!#$%&'*+-/=?^_`{|}~".to_vec());
    assert_eq!(remainder, b"");

    let input = b" John Smith ".to_vec();
    let (atom, remainder) = Atom::parse(input.as_slice()).unwrap();
    assert_eq!(atom.atext.0, b"John".to_vec());
    assert_eq!(remainder, b"Smith ");

    let mut output: Vec<u8> = Vec::new();
    assert_eq!(atom.stream(&mut output).unwrap(), 6);
    assert_eq!(output, b" John ");
}

#[test]
fn test_dot_atom() {
    use rfc5322::types::{DotAtom, AText};

    let input = b" \r\n www.google.com. ".to_vec();
    let (dot_atom, remainder) = DotAtom::parse(input.as_slice()).unwrap();
    assert_eq!(dot_atom.dot_atom_text.0, vec![
        AText(b"www".to_vec()),
        AText(b"google".to_vec()),
        AText(b"com".to_vec())]);
    assert!(dot_atom.pre_cfws.is_some());
    assert!(dot_atom.post_cfws.is_none());
    assert_eq!(remainder, b". ");
}

#[test]
fn test_qcontent() {
    use rfc5322::types::{QContent, QText, QuotedPair};

    let input = b"!#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[]\
                 ^_`abcdefghijklmnopqrstuvwxyz{|}~".to_vec();
    let input2 = input.clone();
    let (token, remainder) = QContent::parse(input.as_slice()).unwrap();
    assert_eq!(token, QContent::QText( QText(input2) ));
    assert_eq!(remainder, b"");

    let input = b"\\nc".to_vec();
    let (token, remainder) = QContent::parse(input.as_slice()).unwrap();
    assert_eq!(token, QContent::QuotedPair( QuotedPair(b'n') ));
    assert_eq!(remainder, b"c");

    let mut output: Vec<u8> = Vec::new();
    assert_eq!(token.stream(&mut output).unwrap(), 2);
    assert_eq!(output, b"\\n");
}

#[test]
fn test_quoted_string() {
    use rfc5322::types::{QuotedString, QContent, QText};

    let input = b" \t (a comment) \" \r\n bob joe\" (fred) ".to_vec();
    let (token, remainder) = QuotedString::parse(input.as_slice()).unwrap();
    assert_eq!(remainder, b"");
    assert!(token.pre_cfws.is_some());
    assert_eq!(token.qcontent, vec![
        (true, QContent::QText( QText(b"bob".to_vec()) )),
        (true, QContent::QText( QText(b"joe".to_vec()) )),
        ]);
    assert_eq!(token.trailing_ws, false);
    assert!(token.post_cfws.is_some());

    let unterminated = b" \t (a comment) \" \r\n bob joe (fred) ".to_vec();
    assert!(QuotedString::parse(unterminated.as_slice()).is_err());
}

#[test]
fn test_phrase() {
    use rfc5322::types::Phrase;

    let input = b" John \"the Snake\" Stevens".to_vec();
    let (phrase, remainder) = Phrase::parse(input.as_slice()).unwrap();
    assert_eq!(phrase.0.len(), 3);
    assert_eq!(remainder, b"");

    let input = b" John Smith [Doctor]".to_vec();
    let (phrase, remainder) = Phrase::parse(input.as_slice()).unwrap();
    assert_eq!(phrase.0.len(), 2);
    assert_eq!(remainder, b"[Doctor]");
}

#[test]
fn test_unstructured() {
    use rfc5322::types::{Unstructured, VChar};

    let input = b"This is; unstructured=5 \r\n ".to_vec();
    let (u, remainder) = Unstructured::parse(input.as_slice()).unwrap();
    assert_eq!(u, Unstructured {
        leading_ws: false,
        parts: vec![
            VChar(b"This".to_vec()),
            VChar(b"is;".to_vec()),
            VChar(b"unstructured=5".to_vec())],
        trailing_ws: true,
    });
    assert_eq!(remainder, b"\r\n "); // because trailing ws is only WSP not FWS
}

#[test]
fn test_domain_literal() {
    use rfc5322::types::{DomainLiteral, DText};

    let input = b"\r\n \t[ 2001:db8:85a3:8d3:1319:8a2e:370:7348]".to_vec();
    let (token, _) = DomainLiteral::parse(input.as_slice()).unwrap();
    assert!(token.pre_cfws.is_some());
    assert_eq!(token.dtext, vec![
        (true, DText(b"2001:db8:85a3:8d3:1319:8a2e:370:7348".to_vec()))
        ]);
    assert_eq!(token.trailing_ws, false);
    assert!(token.post_cfws.is_none());
}

#[test]
fn test_addr_spec() {
    use rfc5322::types::{AddrSpec, LocalPart, Domain, DotAtom, DotAtomText,
                         QuotedString, QContent, DomainLiteral, AText, DText, QText};

    let input = b"joe.smith@gmail.com".to_vec();
    let (a, rem) = AddrSpec::parse(input.as_slice()).unwrap();
    assert_eq!(a.local_part, LocalPart::DotAtom( DotAtom {
        pre_cfws: None,
        dot_atom_text: DotAtomText(vec![ AText(b"joe".to_vec()),
                                         AText(b"smith".to_vec()) ]),
        post_cfws: None,
    }));
    assert_eq!(a.domain, Domain::DotAtom( DotAtom {
        pre_cfws: None,
        dot_atom_text: DotAtomText(vec![ AText(b"gmail".to_vec()),
                                         AText(b"com".to_vec()) ]),
        post_cfws: None,
    }));
    assert_eq!(rem, b"");

    let mut output: Vec<u8> = Vec::new();
    assert_eq!(a.stream(&mut output).unwrap(), 19);
    assert_eq!(output, b"joe.smith@gmail.com".to_vec());

    let input = b"\"joe smith\"@[2001:db8:85a3:8d3:1319:8a2e:370:7348]".to_vec();
    let (a, rem) = AddrSpec::parse(input.as_slice()).unwrap();
    assert_eq!(a.local_part, LocalPart::QuotedString( QuotedString {
        pre_cfws: None,
        qcontent: vec![ (false, QContent::QText(QText(b"joe".to_vec()))),
                         (true, QContent::QText(QText(b"smith".to_vec()))) ],
        trailing_ws: false,
        post_cfws: None,
    }));
    assert_eq!(a.domain, Domain::DomainLiteral( DomainLiteral {
        pre_cfws: None,
        dtext: vec![(false, DText(b"2001:db8:85a3:8d3:1319:8a2e:370:7348".to_vec()))],
        trailing_ws: false,
        post_cfws: None,
    }));
    assert_eq!(rem, b"");
}

#[test]
fn test_angle_addr() {
    use rfc5322::types::AngleAddr;

    let input = b"< admin@example.com >".to_vec();
    let (token, rem) = AngleAddr::parse(input.as_slice()).unwrap();
    assert_eq!(rem, b"");

    let mut output: Vec<u8> = Vec::new();
    assert_eq!(token.stream(&mut output).unwrap(), 21);
    assert_eq!(output, input);
}

#[test]
fn test_name_addr() {
    use rfc5322::types::NameAddr;

    let input = b" Bruce \"The Boss\" < bruce@net> \r\n ".to_vec();
    let (token, rem) = NameAddr::parse(input.as_slice()).unwrap();
    assert_eq!(rem, b"");

    let mut output: Vec<u8> = Vec::new();
    assert_eq!(token.stream(&mut output).unwrap(), 31);
    assert_eq!(output, b" Bruce \"The Boss\" < bruce@net> ".to_vec());
}

#[test]
fn test_mailbox_list() {
    use rfc5322::types::{MailboxList, Mailbox};

    let input = b"a@b.c, \"j p\" <d.e@e.f>,,".to_vec();
    let (mbl, rem) = MailboxList::parse(input.as_slice()).unwrap();
    assert_eq!(mbl.0.len(), 2);
    let mb2 = &mbl.0[1];
    assert_eq!(match mb2 {
        &Mailbox::NameAddr(_) => true,
        &Mailbox::AddrSpec(_) => false,
    }, true);
    assert_eq!(rem, b",,");

    let mut output: Vec<u8> = Vec::new();
    assert_eq!(mbl.stream(&mut output).unwrap(), 22);
    assert_eq!(output, b"a@b.c, \"j p\" <d.e@e.f>".to_vec());
}

#[test]
fn test_zone() {
    use rfc5322::types::Zone;

    let input = b" +1135".to_vec();
    let (v, rem) = Zone::parse(input.as_slice()).unwrap();
    assert_eq!(rem, b"");
    assert_eq!(v.0, 1135_i32);

    let input = b" \r\n -0700".to_vec();
    let (v, rem) = Zone::parse(input.as_slice()).unwrap();
    assert_eq!(rem, b"");
    assert_eq!(v.0, -700_i32);

    let mut output: Vec<u8> = Vec::new();
    assert_eq!(v.stream(&mut output).unwrap(), 6);
    assert_eq!(output, b" -0700".to_vec());
}

#[test]
fn test_time_of_day() {
    use rfc5322::types::TimeOfDay;

    let input = b"17:25:049".to_vec();
    let (t, rem) = TimeOfDay::parse(input.as_slice()).unwrap();
    assert_eq!(rem, b"9");
    assert_eq!(t.hour.0, 17);
    assert_eq!(t.minute.0, 25);
    assert_eq!(t.second.as_ref().unwrap().0, 4);

    let mut output: Vec<u8> = Vec::new();
    assert_eq!(t.stream(&mut output).unwrap(), 8);
    assert_eq!(output, b"17:25:04".to_vec());

    let input = b"01:019".to_vec();
    let (t, rem) = TimeOfDay::parse(input.as_slice()).unwrap();
    assert_eq!(rem, b"9");
    assert_eq!(t.hour.0, 1);
    assert_eq!(t.minute.0, 1);
    assert_eq!(t.second, None);

    let mut output: Vec<u8> = Vec::new();
    assert_eq!(t.stream(&mut output).unwrap(), 5);
    assert_eq!(output, b"01:01".to_vec());
}

#[test]
fn test_date() {
    use rfc5322::types::Date;

    let input = b" 22 Sep 2016 ".to_vec();
    let (t, rem) = Date::parse(input.as_slice()).unwrap();
    assert_eq!(rem, b"");
    assert_eq!(t.day.0, 22);
    assert_eq!(t.month.0, 9);
    assert_eq!(t.year.0, 2016);

    let mut output: Vec<u8> = Vec::new();
    assert_eq!(t.stream(&mut output).unwrap(), 13);
    assert_eq!(output, b" 22 Sep 2016 ".to_vec());
}

#[test]
fn test_date_time() {
    use rfc5322::types::DateTime;

    let input = b"suN, 01 DEC 2000 12:12:12 -1300 (or thereabouts) \r\n ".to_vec();
    let (t, rem) = DateTime::parse(input.as_slice()).unwrap();
    assert_eq!(rem, b"");

    let mut output: Vec<u8> = Vec::new();
    assert_eq!(t.stream(&mut output).unwrap(), 49);
    assert_eq!(output, b"Sun, 01 Dec 2000 12:12:12 -1300 (or thereabouts) ".to_vec());
}

#[test]
fn test_orig_date() {
    use rfc5322::headers::OrigDate;

    let input = b"DATE: SAT, 11 Jan 2000 00:00:00 +0000\r\n".to_vec();
    let (od, rem) = OrigDate::parse(input.as_slice()).unwrap();
    assert_eq!(rem, b"");

    let mut output: Vec<u8> = Vec::new();
    assert_eq!(od.stream(&mut output).unwrap(), 39);
    assert_eq!(output, b"Date: Sat, 11 Jan 2000 00:00:00 +0000\r\n".to_vec());
}

#[test]
fn test_from() {
    use rfc5322::headers::From;

    let input = b"froM:steven@a.b.c\r\n".to_vec();
    let (from, rem) = From::parse(input.as_slice()).unwrap();
    assert_eq!(rem, b"");

    let mut output: Vec<u8> = Vec::new();
    assert_eq!(from.stream(&mut output).unwrap(), 19);
    assert_eq!(output, b"From:steven@a.b.c\r\n".to_vec());
}

#[test]
fn test_bcc() {
    use rfc5322::headers::Bcc;

    let input1 = b"bcc: (hah)\r\n".to_vec();
    let (token, rem) = Bcc::parse(input1.as_slice()).unwrap();
    assert_eq!(rem, b"");
    assert!(match token {
        Bcc::AddressList(_) => false,
        Bcc::CFWS(_) => true,
        Bcc::Empty => false,
    });

    let input1 = b"bcc: a@b,c@d\r\n".to_vec();
    let (token, rem) = Bcc::parse(input1.as_slice()).unwrap();
    assert_eq!(rem, b"");
    assert!(match token {
        Bcc::AddressList(_) => true,
        Bcc::CFWS(_) => false,
        Bcc::Empty => false,
    });
}

#[test]
fn test_msg_id() {
    use rfc5322::types::{MsgId, IdLeft, IdRight, DotAtomText, AText};

    let input = b"<950910bae2c7eff8d34297870a93dbb8@a.b.co.nz>".to_vec();
    let (msgid, rem) = MsgId::parse(input.as_slice()).unwrap();
    assert_eq!(rem, b"");
    assert_eq!(msgid, MsgId {
        pre_cfws: None,
        id_left: IdLeft(DotAtomText(vec![
            AText("950910bae2c7eff8d34297870a93dbb8".as_bytes().to_owned()),
            ])),
        id_right: IdRight::DotAtomText(DotAtomText(vec![
            AText("a".as_bytes().to_owned()),
            AText("b".as_bytes().to_owned()),
            AText("co".as_bytes().to_owned()),
            AText("nz".as_bytes().to_owned()),
            ])),
        post_cfws: None,
    });
}

#[test]
fn test_body() {
    use rfc5322::Body;

    let input = b"This is a test email".to_vec();
    let (body, rem) = Body::parse(input.as_slice()).unwrap();
    assert_eq!(rem, b"");
    assert_eq!(body.0, input);

    let input = b"This is a test email\r\n".to_vec();
    let (body, rem) = Body::parse(input.as_slice()).unwrap();
    assert_eq!(rem, b"");
    assert_eq!(body.0, input);

    let input = b"This is a test email\r\nVery simple, though.\r\n".to_vec();
    let (body, rem) = Body::parse(input.as_slice()).unwrap();
    assert_eq!(rem, b"");
    assert_eq!(body.0, input);

    let input = b"This is a test email\r\n\r\nok\r\n".to_vec();
    let (body, rem) = Body::parse(input.as_slice()).unwrap();
    assert_eq!(rem, b"");
    assert_eq!(body.0, input);

    let input = b"This is a test email\r\n\r\nbad\rbad\r\n".to_vec();
    assert_match!(Body::parse(input.as_slice()), Err(_));
}

#[test]
fn test_message_1() {
    use rfc5322::{Message, Fields, Field, Body};
    use rfc5322::headers::{Subject, From, To};
    use rfc5322::types::{Unstructured, MailboxList, VChar, AddrSpec, DotAtom, CFWS,
                         AText, DotAtomText, LocalPart, Domain,
                         Mailbox, Address, AddressList};

    let input = b"Subject: This is a test\r\n\
From: me@mydomain.net\r\n\
To: you@yourdomain.net\r\n\
\r\n\
This is the body.\r\n\
Simple.".to_vec();

    let (message, rem) = Message::parse(input.as_slice()).unwrap();
    assert_eq!(rem, b"");
    assert_eq!(message, Message {
        fields: Fields {
            trace_blocks: vec![],
            fields: vec![
                Field::Subject(Subject(Unstructured {
                    leading_ws: true,
                    parts: vec![VChar(b"This".to_vec()),
                                VChar(b"is".to_vec()),
                                VChar(b"a".to_vec()),
                                VChar(b"test".to_vec())],
                    trailing_ws: false,
                })),
                Field::From(From::Mailboxes(MailboxList(vec![Mailbox::AddrSpec(AddrSpec {
                    local_part: LocalPart::DotAtom(DotAtom {
                        pre_cfws: Some(CFWS {
                            comments: vec![],
                            trailing_ws: true,
                        }),
                        dot_atom_text: DotAtomText(vec![AText(b"me".to_vec())]),
                        post_cfws: None,
                    }),
                    domain: Domain::DotAtom(DotAtom {
                        pre_cfws: None,
                        dot_atom_text: DotAtomText(vec![AText(b"mydomain".to_vec()),
                                                        AText(b"net".to_vec())]),
                        post_cfws: None })
                })]))),
                Field::To(To(AddressList( vec![
                    Address::Mailbox(
                        Mailbox::AddrSpec(AddrSpec {
                            local_part: LocalPart::DotAtom(DotAtom {
                                pre_cfws: Some(CFWS {
                                    comments: vec![],
                                    trailing_ws: true,
                                }),
                                dot_atom_text: DotAtomText(vec![AText(b"you".to_vec())]),
                                post_cfws: None,
                            }),
                            domain: Domain::DotAtom(DotAtom {
                                pre_cfws: None,
                                dot_atom_text: DotAtomText(vec![AText(b"yourdomain".to_vec()),
                                                                AText(b"net".to_vec())]),
                                post_cfws: None })
                        }))]))),
                ]
        },
        body: Some(Body(b"This is the body.\r\nSimple.".to_vec())),
    });
}

#[test]
fn test_email_struct_functions() {
    use ::Email;
    let mut email = Email::new("mike@sample.com",
                               "Wed, 05 Jan 2015 15:13:05 +1300").unwrap();

    email.set_date("Wed, 06 Jan 2015 15:13:05 +1300".as_bytes()).unwrap();
    let date1 = email.get_date();
    email.set_date("Fri, 30 Dec 2000 09:11:56 -1100").unwrap();
    let date2 = email.get_date();
    assert!(date1 != date2);
    email.set_date(date2).unwrap();

    email.set_from("mike@sample.com".as_bytes()).unwrap();
    let from1 = email.get_from();
    email.set_from("mike@sample2.com").unwrap();
    let from2 = email.get_from();
    assert!(from1 != from2);
    email.set_from(from2).unwrap();

    assert!(email.get_sender().is_none());
    email.set_sender("mike@sample.com".as_bytes()).unwrap();
    let sender1 = email.get_sender().unwrap();
    email.set_sender("mike@sample2.com").unwrap();
    let sender2 = email.get_sender().unwrap();
    assert!(sender1 != sender2);
    email.set_sender(sender2).unwrap();

    assert!(email.get_reply_to().is_none());
    email.set_reply_to("mike@sample.com".as_bytes()).unwrap();
    let reply_to1 = email.get_reply_to().unwrap();
    email.set_reply_to("mike@sample2.com").unwrap();
    let reply_to2 = email.get_reply_to().unwrap();
    assert!(reply_to1 != reply_to2);
    email.set_reply_to(reply_to2).unwrap();

    assert!(email.get_to().is_none());
    email.set_to("mike@sample.com".as_bytes()).unwrap();
    let to1 = email.get_to().unwrap();
    email.set_to("mike@sample2.com").unwrap();
    let to2 = email.get_to().unwrap();
    assert!(to1 != to2);
    email.set_to(to2).unwrap();

    assert!(email.get_cc().is_none());
    email.set_cc("mike@sample.com, webmaster@sample.com".as_bytes()).unwrap();
    let cc1 = email.get_cc().unwrap();
    email.set_cc("mike@sample2.com, mike@sample.com").unwrap();
    let cc2 = email.get_cc().unwrap();
    assert!(cc1 != cc2);
    email.set_cc(cc2).unwrap();
}

#[test]
fn test_email_example() {
    use ::Email;

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
    email.set_body("Good to hear from you.\r\n\
                    I wish you the best.\r\n\
                    \r\n\
                    Your Friend").unwrap();


    let mut output: Vec<u8> = Vec::new();
    email.stream(&mut output).unwrap();

    assert_eq!(output,
               "Date:Wed, 05 Jan 2015 15:13:05 +1300\r\n\
                From:myself@mydomain.com\r\n\
                Sender:from_myself@mydomain.com\r\n\
                Reply-To:My Mailer <no-reply@mydomain.com>\r\n\
                To:You <you@yourdomain.com>\r\n\
                Cc:Our Friend <friend@frienddomain.com>\r\n\
                Message-ID:<id/20161128115731.29084.maelstrom@mydomain.com>\r\n\
                Subject:Hello Friend\r\n\
                \r\n\
                Good to hear from you.\r\n\
                I wish you the best.\r\n\
                \r\n\
                Your Friend".as_bytes());
}

#[test]
fn test_email_parse_stream() {
    use ::Email;
    use ::rfc5322::{Parsable, Streamable};

    let input = "Date: Wed, 05 Jan 2015 15:13:05 +1300\r\n\
                 From: myself@mydomain.com\r\n\
                 Sender: from_myself@mydomain.com\r\n\
                 Reply-To: My Mailer <no-reply@mydomain.com>\r\n\
                 To: You <you@yourdomain.com>\r\n\
                 Cc: Our Friend <friend@frienddomain.com>\r\n\
                 Message-ID: <id/20161128115731.29084.maelstrom@mydomain.com>\r\n\
                 Subject: Hello Friend\r\n\
                 \r\n\
                 Good to hear from you.\r\n\
                 I wish you the best.\r\n\
                 \r\n\
                 Your Friend".as_bytes();

    let (email, remainder) = Email::parse(&input).unwrap();
    assert_eq!(remainder.len(), 0);

    let mut output: Vec<u8> = Vec::new();
    email.stream(&mut output).unwrap();

    assert_eq!(input, &*output);
}

#[test]
#[should_panic]
fn test_trailing_input() {
    use ::TryFrom;
    use ::rfc5322::headers::Sender;

    let _: Sender = TryFrom::try_from("mike@optcomp.nz[.xyz]").unwrap();
}
