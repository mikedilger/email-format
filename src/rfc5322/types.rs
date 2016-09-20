
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

// 3.2.2
// FWS             =   ([*WSP CRLF] 1*WSP) /  obs-FWS
//                                        ; Folding white space
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FWS;
impl Parsable for FWS {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        let mut rem = input;
        if rem.len() == 0 { return Err(ParseError::Eof); }
        while rem.len() > 0 {
            if is_wsp(rem[0]) {
                rem = &rem[1..];
            }
            else if rem.len() > 2 && &rem[0..2]==b"\r\n" && is_wsp(rem[2]) {
                rem = &rem[3..];
            }
            else {
                break;
            }
        }
        if rem.len() == input.len() { Err(ParseError::NotFound) }
        else { Ok((FWS, rem)) }
    }
}
impl Streamable for FWS {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        Ok(try!(w.write(b" "))) // FIXME - fold?
    }
}

// 3.2.2
// ctext           =   %d33-39 /          ; Printable US-ASCII
//                     %d42-91 /          ;  characters not including
//                     %d93-126 /         ;  "(", ")", or "\"
//                     obs-ctext
#[inline]
pub fn is_ctext(c: u8) -> bool { (c>=33 && c<=39) || (c>=42 && c<=91) || (c>=93 && c<=126) }
def_cclass!(CText, is_ctext);

// 3.2.2
// ccontent        =   ctext / quoted-pair / comment
#[derive(Debug, Clone, PartialEq)]
pub enum CContent {
    CText(CText),
    QuotedPair(QuotedPair),
    Comment(Comment),
}
impl Parsable for CContent {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if let Ok((na, rem)) = CText::parse(input) {
            Ok((CContent::CText(na), rem))
        }
        else if let Ok((asp, rem)) = QuotedPair::parse(input) {
            Ok((CContent::QuotedPair(asp), rem))
        }
        else if let Ok((c, rem)) = Comment::parse(input) {
            Ok((CContent::Comment(c), rem))
        }
        else {
            Err(ParseError::NotFound)
        }
    }
}
impl Streamable for CContent {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        match *self {
            CContent::CText(ref x) => x.stream(w),
            CContent::QuotedPair(ref x) => x.stream(w),
            CContent::Comment(ref x) => x.stream(w),
        }
    }
}

// 3.2.2
// comment         =   "(" *([FWS] ccontent) [FWS] ")"
#[derive(Debug, Clone, PartialEq)]
pub struct Comment {
    pub ccontent: Vec<(bool, CContent)>, // bool representing if whitespace preceeds it
    pub trailing_ws: bool,
}
impl Parsable for Comment {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        let mut rem: &[u8] = input;
        if rem.len() == 0 { return Err(ParseError::Eof); }
        req!(rem, b"(", input);
        let mut ccontent: Vec<(bool, CContent)> = Vec::new();
        let mut ws: bool = false;
        while rem.len() > 0 {
            let t = parse!(FWS, rem);
            ws = t.is_ok();
            if let Ok(cc) = parse!(CContent, rem) {
                ccontent.push((ws, cc));
                continue;
            }
            break;
        }
        req!(rem, b")", input);
        return Ok((Comment {
            ccontent: ccontent,
            trailing_ws: ws,
        }, rem));
    }
}
impl Streamable for Comment {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        let mut count: usize = 0;
        count += try!(w.write(b"("));
        for &(ws, ref cc) in &self.ccontent {
            if ws { count += try!(w.write(b" ")) }
            count += try!(cc.stream(w));
        }
        if self.trailing_ws { count += try!(w.write(b" ")) }
        count += try!(w.write(b")"));
        Ok(count)
    }
}

// 3.2.2
// CFWS            =   (1*([FWS] comment) [FWS]) / FWS
#[derive(Debug, Clone, PartialEq)]
pub struct CFWS {
    pub comments: Vec<(bool, Comment)>, // bool representing if whitespace preceeds it
    pub trailing_ws: bool,
}
impl Parsable for CFWS {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof); }
        let mut comments: Vec<(bool, Comment)> = Vec::new();
        let mut rem = input;
        let mut ws: bool = false;
        while rem.len() > 0 {
            let w = parse!(FWS, rem);
            ws = w.is_ok();
            if let Ok(comment) = parse!(Comment, rem) {
                comments.push((ws, comment));
                continue;
            }
            break;
        }
        if comments.len() > 0 || ws {
            Ok((CFWS {
                comments: comments,
                trailing_ws: ws,
            }, rem))
        } else {
            Err(ParseError::NotFound)
        }
    }
}
impl Streamable for CFWS {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        let mut count: usize = 0;
        for &(ws, ref comment) in &self.comments {
            if ws { count += try!(w.write(b" ")) }
            count += try!(comment.stream(w));
        }
        if self.trailing_ws { count += try!(w.write(b" ")) }
        Ok(count)
    }
}

// 3.2.3
// atext           =   ALPHA / DIGIT /    ; Printable US-ASCII
//                     "!" / "#" /        ;  characters not including
//                     "$" / "%" /        ;  specials.  Used for atoms.
//                     "&" / "'" /
//                     "*" / "+" /
//                     "-" / "/" /
//                     "=" / "?" /
//                     "^" / "_" /
//                     "`" / "{" /
//                     "|" / "}" /
//                     "~"
#[inline]
pub fn is_atext(c: u8) -> bool {
    is_alpha(c) || is_digit(c)
        || c==b'!' || c==b'#'  || c==b'$' || c==b'%'
        || c==b'&' || c==b'\'' || c==b'*' || c==b'+'
        || c==b'-' || c==b'/'  || c==b'=' || c==b'?'
        || c==b'^' || c==b'_'  || c==b'`' || c==b'{'
        || c==b'|' || c==b'}'  || c==b'~'
}
def_cclass!(AText, is_atext);

// 3.2.3
// atom            =   [CFWS] 1*atext [CFWS]
#[derive(Debug, Clone, PartialEq)]
pub struct Atom {
    pub pre_cfws: Option<CFWS>,
    pub atext: AText,
    pub post_cfws: Option<CFWS>,
}
impl Parsable for Atom {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len()==0 { return Err(ParseError::Eof); }
        let mut rem = input;
        let pre_cfws = parse!(CFWS, rem);
        if let Ok(atext) = parse!(AText, rem) {
            let post_cfws = parse!(CFWS, rem);
            return Ok((Atom {
                pre_cfws: pre_cfws.ok(),
                atext: atext,
                post_cfws: post_cfws.ok(),
            }, rem));
        }
        Err(ParseError::NotFound)
    }
}
impl Streamable for Atom {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        let mut count: usize = 0;
        if let Some(ref cfws) = self.pre_cfws {
            count += try!(cfws.stream(w));
        }
        count += try!(self.atext.stream(w));
        if let Some(ref cfws) = self.post_cfws {
            count += try!(cfws.stream(w));
        }
        Ok(count)
    }
}

// 3.2.3
// dot-atom-text   =   1*atext *("." 1*atext)
#[derive(Debug, Clone, PartialEq)]
pub struct DotAtomText(pub Vec<AText>);
impl Parsable for DotAtomText {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        let mut rem = input;
        let mut parts: Vec<AText> = Vec::new();
        match parse!(AText, rem) {
            Ok(part) => parts.push(part),
            Err(e) => return Err(e),
        }
        while rem.len() > 0 {
            if rem[0]!=b'.' { break; };
            let rem2 = &rem[1..];
            if let Ok((part, r)) = AText::parse(rem2) {
                rem = r;
                parts.push(part);
                continue;
            } else {
                break;
            }
        }
        Ok((DotAtomText(parts), rem))
    }
}
impl Streamable for DotAtomText {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        let mut count: usize = 0;
        let mut virgin: bool = true;
        for part in &self.0 {
            if !virgin { count += try!(w.write(b".")) }
            count += try!(part.stream(w));
            virgin = false;
        }
        Ok(count)
    }
}

// 3.2.3
// dot-atom        =   [CFWS] dot-atom-text [CFWS]
#[derive(Debug, Clone, PartialEq)]
pub struct DotAtom {
    pub pre_cfws: Option<CFWS>,
    pub dot_atom_text: DotAtomText,
    pub post_cfws: Option<CFWS>,
}
impl Parsable for DotAtom {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        let mut rem = input;
        if rem.len() == 0 { return Err(ParseError::Eof); }
        let pre_cfws = parse!(CFWS, rem);
        if let Ok(dat) = parse!(DotAtomText, rem) {
            let post_cfws = parse!(CFWS, rem);
            Ok((DotAtom {
                pre_cfws: pre_cfws.ok(),
                dot_atom_text: dat,
                post_cfws: post_cfws.ok(),
            }, rem))
        } else {
            Err(ParseError::NotFound)
        }
    }
}
impl Streamable for DotAtom {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        let mut count: usize = 0;
        if let Some(ref cfws) = self.pre_cfws {
            count += try!(cfws.stream(w));
        }
        count += try!(self.dot_atom_text.stream(w));
        if let Some(ref cfws) = self.post_cfws {
            count += try!(cfws.stream(w));
        }
        Ok(count)
    }
}

// 3.2.3 (we don't need to parse this one, it is not used.  could be used as a tokenization
//        point in lexical analysis)
// specials            = "(" / ")" /        ; Special characters that do
//                       "<" / ">" /        ;  not appear in atext
//                       "[" / "]" /
//                       ":" / ";" /
//                       "@" / "\" /
//                       "," / "." /
//                       DQUOTE

// 3.2.4
// qtext           =   %d33 /             ; Printable US-ASCII
//                     %d35-91 /          ;  characters not including
//                     %d93-126 /         ;  "\" or the quote character
//                     obs-qtext
#[inline]
pub fn is_qtext(c: u8) -> bool { c==33 || (c>=35 && c<=91) || (c>=93 && c<=126) }
def_cclass!(QText, is_qtext);
