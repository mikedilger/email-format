
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
    assert_match!(err, ParseError::Eof);

    let err = Alpha::parse(b"123").err().unwrap();
    assert_match!(err, ParseError::NotFound);

    let mut output: Vec<u8> = Vec::new();
    assert_eq!(alpha.stream(&mut output).unwrap(), 8);
    assert_eq!(output, b"abcdEFZz".to_vec());
}

#[test]
fn test_parse_quoted_pair() {
    use rfc5322::types::QuotedPair;

    let err = QuotedPair::parse(b"not").err().unwrap();
    assert_match!(err, ParseError::NotFound);
    let err = QuotedPair::parse(b"\\").err().unwrap();
    assert_match!(err, ParseError::NotFound);
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
    assert_match!(err, ParseError::NotFound);
    let err = FWS::parse(b"\r\n").err().unwrap();
    assert_match!(err, ParseError::NotFound);
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
