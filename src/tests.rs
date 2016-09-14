
use types::Token;
use types::{QuotedPair, MailboxList};

#[test]
fn test_parse_quoted_pair() {
    let mut c: usize = 0;
    assert_eq!(QuotedPair::parse(b"not", &mut c), None);
    c = 0;
    assert_eq!(QuotedPair::parse(b"\\", &mut c), None);
    c = 0;
    assert_eq!(QuotedPair::parse(b"\\n", &mut c), Some(QuotedPair(b'n')));
    let qp = QuotedPair(b'n');
    let mut output: Vec<u8> = Vec::new();
    assert_eq!(qp.stream(&mut output).unwrap(), 2);
    assert_eq!(output, b"\\n");
}

#[test]
fn test_advanced() {
    let input = b"(stupid)mike(you suck!)@(dumb)g.ac.uk(die die die!), \"John Fitzgerald\" <jf@z.net.nz>";
    let mut c: usize = 0;
    let mbl = MailboxList::parse(input, &mut c).unwrap();
    println!("{:?}", mbl);
    let mut output: Vec<u8> = Vec::new();
    mbl.stream(&mut output).unwrap();
    println!("{}", String::from_utf8_lossy(&output[..]));
}

/*
const SAMPLE_FWS: &'static [u8] = b" \t \r\n\t ";
#[test]
fn test_consume_folding_whitespace() {
    let mut parser = Parser::new(b"  \r\n great", Behavior::CollapseWhiteSpace);
    assert_eq!(&*parser.parse_folding_whitespace().unwrap(), b" ");
    assert_eq!(parser.remaining(), 5);

    let mut parser = Parser::new(b"  \t great", Behavior::CollapseWhiteSpace);
    assert_eq!(&*parser.parse_folding_whitespace().unwrap(), b" ");
    assert_eq!(parser.remaining(), 5);

    let mut parser = Parser::new(SAMPLE_FWS, Behavior::PreserveInput);
    assert_eq!(&*parser.parse_folding_whitespace().unwrap(), SAMPLE_FWS);
    assert_eq!(parser.remaining(), 0);
}

const SAMPLE_CTEXT: &'static [u8] = b"Thi,s;1:23isCt_#ext";
#[test]
fn test_parse_text() {
    let mut parser = Parser::new(SAMPLE_CTEXT, Behavior::PreserveInput);
    assert_eq!(&*parser.parse_text(&is_ctext).unwrap(), SAMPLE_CTEXT);
    let mut parser = Parser::new(b"ctext(blah)crazy\\8s", Behavior::PreserveInput);
    assert_eq!(&*parser.parse_text(&is_ctext).unwrap(), b"ctext");
    parser.consume_byte();
    assert_eq!(&*parser.parse_text(&is_ctext).unwrap(), b"blah");
    parser.consume_byte();
    assert_eq!(&*parser.parse_text(&is_ctext).unwrap(), b"crazy");
    parser.consume_byte();
    assert_eq!(&*parser.parse_text(&is_ctext).unwrap(), b"8s");
}

const SAMPLE_COMMENT: &'static [u8] = b"( a,b,c\t \\nYes (and so on) \r\n )";
const SAMPLE_COMMENT_CWS: &'static [u8] = b"( a,b,c \\nYes (and so on) )";
#[test]
fn test_parse_ccontent() {
    let mut parser = Parser::new(SAMPLE_CTEXT, Behavior::PreserveInput);
    assert_eq!(&*parser.parse_ccontent().unwrap(), SAMPLE_CTEXT);
    let mut parser = Parser::new(SAMPLE_QP, Behavior::PreserveInput);
    assert_eq!(&*parser.parse_ccontent().unwrap(), SAMPLE_QP);
    let mut parser = Parser::new(SAMPLE_COMMENT, Behavior::PreserveInput);
    assert_eq!(&*parser.parse_ccontent().unwrap(), SAMPLE_COMMENT);
    let mut parser = Parser::new(SAMPLE_COMMENT, Behavior::CollapseWhiteSpace);
    assert_eq!(&*parser.parse_ccontent().unwrap(), SAMPLE_COMMENT_CWS);

    let mut parser = Parser::new(b"Mike+(the\r\n Bike-Man)", Behavior::CollapseWhiteSpace);
    assert_eq!(&*parser.parse_ccontent().unwrap(), b"Mike+(the Bike-Man)");
    let mut parser = Parser::new(b"This-is-a-(nested ( comment ))-dude",
                                 Behavior::CollapseWhiteSpace);
    assert_eq!(&*parser.parse_ccontent().unwrap(), "This-is-a-(nested ( comment ))-dude".as_bytes());
    let mut parser = Parser::new(b"This\\ is\\ an\\ (unclosed comment",
                                 Behavior::CollapseWhiteSpace);
    assert_eq!(&*parser.parse_ccontent().unwrap(), b"This\\ is\\ an\\ ");
    let mut parser = Parser::new(b"This(has bad \r line endings)",
                                 Behavior::CollapseWhiteSpace);
    assert_eq!(&*parser.parse_ccontent().unwrap(), b"This");
    let mut parser = Parser::new(b"This-has-bad-\n-line (endings too)",
                                 Behavior::CollapseWhiteSpace);
    assert_eq!(&*parser.parse_ccontent().unwrap(), b"This-has-bad-");
    let mut parser = Parser::new(b"This( has invalid \xEF\xBF\xBD ctext)",
                                 Behavior::CollapseWhiteSpace);
    assert_eq!(&*parser.parse_ccontent().unwrap(), b"This");
    let mut parser = Parser::new(b"This( has invalid \r\nwrapping)man",
                                 Behavior::CollapseWhiteSpace);
    assert_eq!(&*parser.parse_ccontent().unwrap(), b"This");
}

const SAMPLE_COMMENT_2: &'static [u8] = b"(Comments can contain whitespace and \\( quoted \\\\ characters, and even ( nesting ) with or (without) whitepsace, but must balance parenthesis)";
#[test]
fn test_parse_comment() {
    let mut parser = Parser::new(SAMPLE_COMMENT_2, Behavior::PreserveInput);
    assert_eq!(&*parser.parse_ccontent().unwrap(), SAMPLE_COMMENT_2);
    let mut parser = Parser::new(b"Comments start with a parenthesis)", Behavior::PreserveInput);
    assert!(parser.parse_comment().is_none());
    let mut parser = Parser::new(b"(and end with one", Behavior::PreserveInput);
    assert!(parser.parse_comment().is_none());
    let mut parser = Parser::new(b"ctext-is-not-a-comment", Behavior::PreserveInput);
    assert!(parser.parse_comment().is_none());
}

const SAMPLE_CFWS: &'static [u8] = b"  \t( a,b,c\t \\nYes (and so on) \r\n ) \r\n ";
const SAMPLE_CFWS_CWS: &'static [u8] = b" ( a,b,c \\nYes (and so on) ) ";
#[test]
fn test_parse_cfws() {
    let mut parser = Parser::new(SAMPLE_CFWS, Behavior::PreserveInput);
    assert_eq!(&*parser.parse_cfws().unwrap(), SAMPLE_CFWS);
    let mut parser = Parser::new(SAMPLE_CFWS, Behavior::CollapseWhiteSpace);
    assert_eq!(&*parser.parse_cfws().unwrap(), SAMPLE_CFWS_CWS);
}

const SAMPLE_ATOM: &'static [u8] = b"  \t( a,b,c\t \\nYes (and so on) \r\n ) atom\r\n ";
const SAMPLE_ATOM_CWS: &'static [u8] = b" ( a,b,c \\nYes (and so on) ) atom ";
const SAMPLE_ATOM_CO: &'static [u8] = b"atom";
#[test]
fn test_parse_atom() {
    let mut parser = Parser::new(SAMPLE_ATOM, Behavior::PreserveInput);
    assert_eq!(&*parser.parse_atom().unwrap(), SAMPLE_ATOM);
    let mut parser = Parser::new(SAMPLE_ATOM, Behavior::CollapseWhiteSpace);
    assert_eq!(&*parser.parse_atom().unwrap(), SAMPLE_ATOM_CWS);
    let mut parser = Parser::new(SAMPLE_ATOM, Behavior::ContentOnly);
    assert_eq!(&*parser.parse_atom().unwrap(), SAMPLE_ATOM_CO);
    let mut parser = Parser::new(b" \t AMZamz019!#$%&'*+-/=?^_`{|}~ \t ",
                                 Behavior::CollapseWhiteSpace);
    assert_eq!(&*parser.parse_atom().unwrap(), b" AMZamz019!#$%&'*+-/=?^_`{|}~ ");
    let mut parser = Parser::new(b" John Smith ", Behavior::CollapseWhiteSpace);
    assert_eq!(&*parser.parse_atom().unwrap(), b" John ");
}

const SAMPLE_DOT_ATOM: &'static [u8] = b" \r\n www.google.com. ";
const SAMPLE_DOT_ATOM_CWS: &'static [u8] = b" www.google.com";
#[test]
fn test_parse_dot_atom() {
    let mut parser = Parser::new(SAMPLE_DOT_ATOM, Behavior::PreserveInput);
    assert_eq!(&*parser.parse_dot_atom().unwrap(), b" \r\n www.google.com");
    assert_eq!(parser.remaining(), 2);
    let mut parser = Parser::new(SAMPLE_DOT_ATOM, Behavior::CollapseWhiteSpace);
    assert_eq!(&*parser.parse_dot_atom().unwrap(), SAMPLE_DOT_ATOM_CWS);
    assert_eq!(parser.remaining(), 2);

    let mut parser = Parser::new(b".config", Behavior::ContentOnly);
    assert!(parser.parse_dot_atom().is_none());
}

#[test]
fn test_parse_qcontent() {
    let mut parser = Parser::new(b"\\n>", Behavior::CollapseWhiteSpace);
    assert_eq!(&*parser.parse_qcontent().unwrap(), b"\\n");
    let input = b"!#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[]^_`abcdefghijklmnopqrstuvwxyz{|}~";
    let mut parser = Parser::new(input, Behavior::PreserveInput);
    assert_eq!(parser.parse_qcontent().unwrap().len(), input.len());
    let mut parser = Parser::new(b"a c", Behavior::PreserveInput);
    assert_eq!(&*parser.parse_qcontent().unwrap(), b"a");
    let mut parser = Parser::new(b"a\"c", Behavior::PreserveInput);
    assert_eq!(&*parser.parse_qcontent().unwrap(), b"a");
}

const SAMPLE_QUOTED_STRING: &'static [u8] = b" \t (a comment) \" \r\n bob joe\" (fred) ";
const SAMPLE_QUOTED_STRING_CWS: &'static [u8] = b" (a comment) \" bob joe\" (fred) ";
const SAMPLE_QUOTED_STRING_CO: &'static [u8] = b" bob joe";
#[test]
fn test_parse_quoted_string() {
    let mut parser = Parser::new(SAMPLE_QUOTED_STRING, Behavior::PreserveInput);
    assert_eq!(&*parser.parse_quoted_string().unwrap(), SAMPLE_QUOTED_STRING);
    let mut parser = Parser::new(SAMPLE_QUOTED_STRING, Behavior::CollapseWhiteSpace);
    assert_eq!(&*parser.parse_quoted_string().unwrap(), SAMPLE_QUOTED_STRING_CWS);
    let mut parser = Parser::new(SAMPLE_QUOTED_STRING, Behavior::ContentOnly);
    assert_eq!(&*parser.parse_quoted_string().unwrap(), SAMPLE_QUOTED_STRING_CO);
    let mut parser = Parser::new(b" \t (a comment) \" \r\n bob joe (fred) ",
                                 Behavior::PreserveInput);
    assert!(parser.parse_quoted_string().is_none());
}

#[test]
fn test_parse_word() {
    let mut parser = Parser::new(SAMPLE_ATOM, Behavior::PreserveInput);
    assert_eq!(&*parser.parse_word().unwrap(), SAMPLE_ATOM);
    let mut parser = Parser::new(SAMPLE_ATOM, Behavior::CollapseWhiteSpace);
    assert_eq!(&*parser.parse_word().unwrap(), SAMPLE_ATOM_CWS);
    let mut parser = Parser::new(SAMPLE_ATOM, Behavior::ContentOnly);
    assert_eq!(&*parser.parse_word().unwrap(), SAMPLE_ATOM_CO);

    let mut parser = Parser::new(SAMPLE_QUOTED_STRING, Behavior::PreserveInput);
    assert_eq!(&*parser.parse_word().unwrap(), SAMPLE_QUOTED_STRING);
    let mut parser = Parser::new(SAMPLE_QUOTED_STRING, Behavior::CollapseWhiteSpace);
    assert_eq!(&*parser.parse_word().unwrap(), SAMPLE_QUOTED_STRING_CWS);
    let mut parser = Parser::new(SAMPLE_QUOTED_STRING, Behavior::ContentOnly);
    assert_eq!(&*parser.parse_word().unwrap(), SAMPLE_QUOTED_STRING_CO);
}

// parse_display_name is the same
const SAMPLE_PHRASE: &'static [u8] = b" John \"the Snake\" Stevens";
#[test]
fn test_parse_phrase() {
    let mut parser = Parser::new(SAMPLE_PHRASE, Behavior::CollapseWhiteSpace);
    assert_eq!(&*parser.parse_phrase().unwrap(), SAMPLE_PHRASE);
    let mut parser = Parser::new(b" \t ", Behavior::CollapseWhiteSpace);
    assert!(parser.parse_phrase().is_none());
    let mut parser = Parser::new(b" John Smith [Doctor] ", Behavior::CollapseWhiteSpace);
    assert_eq!(&*parser.parse_phrase().unwrap(), b" John Smith ");
}
 */

/*
#[test]
fn test_parse_name_addr() {
    let mut parser = Parser::new(b" Bruce \"The Boss\" < bruce@net> \r\n ");
    assert_eq!(&*parser.parse_name_addr().unwrap(), b" Bruce \"The Boss\" < bruce@net> ");
}

#[test]
fn test_parse_mailbox() {
    // do addr spec first
}

#[test]
fn test_parse_mailbox_list() {
    // do mailbox first
}

#[test]
fn test_parse_angle_addr() {
}

#[test]
fn test_parse_domain_literal() {
}


#[test]
fn test_parse_domain() {
}

#[test]
fn test_parse_local_part() {
}

#[test]
fn test_parse_addr_spec() {
}
 */

/*
#[test]
fn test_type_us_ascii() {
    let valid: Vec<u8> = vec![1,127,14,8];
    let invalid1: Vec<u8> = vec![1,127,0,14,8];
    let invalid2: Vec<u8> = vec![1,127,128,14,8];
    assert!(UsAscii::from_bytes(&*valid).is_ok());
    assert!(UsAscii::from_bytes(&*invalid1).is_err());
    assert!(UsAscii::from_bytes(&*invalid2).is_err());
}

#[test]
fn test_type_line() {
    {
        let mut valid: Vec<u8> = Vec::with_capacity(998);
        for _ in 0..998 {
            valid.push(b'c');
        }
        assert!(
            Line::from_ascii(
                UsAscii::from_bytes(&*valid).unwrap()).is_ok());
    }
    {
        let mut too_long: Vec<u8> = Vec::with_capacity(999);
        for _ in 0..999 {
            too_long.push(b'c');
        }
        assert!(
            Line::from_ascii(
                UsAscii::from_bytes(&*too_long).unwrap()).is_err());
    }
    {
        let has_cr: Vec<u8> = vec![8, 18, 0x0D, 12, 31];
        assert!(
            Line::from_ascii(
                UsAscii::from_bytes(&*has_cr).unwrap()).is_err());
    }
}

#[test]
fn test_type_header_name() {
    assert!(HeaderName::from_bytes(b"hEaDerNamE1748").is_ok());
    assert!(HeaderName::from_bytes(b"my header").is_err()); // space is not printable
    assert!(HeaderName::from_bytes(b"my:header").is_err()); // contains colon
    assert!(HeaderName::from_bytes(b"my\nheader").is_err()); // LF is not printable
}

#[test]
fn test_type_header_body() {
    assert!(HeaderBody::from_bytes(b"This:is \r\n a header\tbody").is_ok());
    assert!(HeaderBody::from_bytes(b"This is not\r\na header body").is_err()); // bad folding
    assert!(HeaderBody::from_bytes(b"This is \x07not valid").is_err()); // bad character
}
 */

/*
#[test]
fn test_dates() {
    use Email;
    use chrono::offset::fixed::FixedOffset;
    use chrono::offset::TimeZone;

    let mut email = Email::new("nobody@localhost").unwrap();

    println!("ORIG DATE WAS: {}", email.get_orig_date());

    email.set_orig_date( FixedOffset::east(1*60*60).ymd(2015, 2, 18).and_hms(23, 16, 9) );
    let date = email.get_orig_date();
    println!("SET FIXED DATE IS: {}", date);
}

#[test]
fn test_email_stream() {
    use Email;
    use chrono::offset::fixed::FixedOffset;
    use chrono::offset::TimeZone;
    use uuid::Uuid;

    let mut output: Vec<u8> = Vec::new();

    let d4 = [12, 3, 9, 56, 54, 43, 8, 9];
    let uuid = Uuid::from_fields(42, 12, 5, &d4).unwrap();

    let email = Email::new("nobody@localhost").unwrap()
        .message_id(uuid)
        .orig_date(FixedOffset::east(0).ymd(2000, 1, 1).and_hms(0, 0, 0))
        .subject("Hi").unwrap()
        .body("Hello").unwrap();
    assert!(email.stream(&mut output).is_ok());

    assert_eq!(&*output, "Date: Sat,  1 Jan 2000 00:00:00 +0000\r\n\
                          From: nobody@localhost\r\n\
                          Message-ID: 0000002a-000c-0005-0c03-0938362b0809\r\n\
                          Subject: Hi\r\n\
                          \r\n\
                          Hello".as_bytes());
}
 */
