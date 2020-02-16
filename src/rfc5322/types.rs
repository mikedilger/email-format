
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
impl_display!(VChar);

// RFC 5234, B.1  Core Rules  WSP            =  SP / HTAB ; white space
#[inline]
pub fn is_wsp(c: u8) -> bool { c==SP || c==HTAB }
def_cclass!(WSP, is_wsp);
impl_display!(WSP);

// RFC 5234, B.1  Core Rules  CHAR           =  %x01-7F ; any 7-bit US-ASCII character,
//                                                      ;  excluding NUL
#[inline]
pub fn is_ascii(c: u8) -> bool { c>=1 && c<=127 }
def_cclass!(ASCII, is_ascii);
impl_display!(ASCII);

// RFC 5234, B.1  Core Rules  DIGIT          =  %x30-39   ; 0-9
#[inline]
pub fn is_digit(c: u8) -> bool { c>=0x30 && c<=0x39 }
def_cclass!(Digit, is_digit);
impl_display!(Digit);

// RFC 5234, B.1  Core Rules  ALPHA          = %x41-5A / %x61-7A   ; A-Z / a-z
#[inline]
pub fn is_alpha(c: u8) -> bool { (c>=0x41 && c<=0x5A) || (c>=0x61 && c<=0x7A) }
def_cclass!(Alpha, is_alpha);
impl_display!(Alpha);

// 3.2.1
// quoted-pair     =   ("\" (VCHAR / WSP)) / obs-qp
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuotedPair(pub u8);
impl Parsable for QuotedPair {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        let mut pos: usize = 0;
        if pos >= input.len() { return Err(ParseError::Eof("Quoted Pair")); }
        if pos + 1 >= input.len() { return Err(ParseError::NotFound("Quoted Pair")); }
        if input[pos]!=b'\\' { return Err(ParseError::NotFound("Quoted Pair")); }
        if is_vchar(input[pos + 1]) || is_wsp(input[pos + 1]) {
            pos += 2;
            let qp = QuotedPair(input[pos - 1]);
            return Ok((qp, &input[pos..]));
        }
        Err(ParseError::NotFound("Quoted Pair"))
    }
}
impl Streamable for QuotedPair {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        Ok(w.write(b"\\")? + w.write(&[self.0])?)
    }
}
impl_display!(QuotedPair);

// 3.2.2
// FWS             =   ([*WSP CRLF] 1*WSP) /  obs-FWS
//                                        ; Folding white space
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FWS;
impl Parsable for FWS {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        let mut rem = input;
        if rem.len() == 0 { return Err(ParseError::Eof("Folding White Space")); }
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
        if rem.len() == input.len() { Err(ParseError::NotFound("Folding White Space")) }
        else { Ok((FWS, rem)) }
    }
}
impl Streamable for FWS {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        Ok(w.write(b" ")?) // FIXME - fold?
    }
}
impl_display!(FWS);

// 3.2.2
// ctext           =   %d33-39 /          ; Printable US-ASCII
//                     %d42-91 /          ;  characters not including
//                     %d93-126 /         ;  "(", ")", or "\"
//                     obs-ctext
#[inline]
pub fn is_ctext(c: u8) -> bool { (c>=33 && c<=39) || (c>=42 && c<=91) || (c>=93 && c<=126) }
def_cclass!(CText, is_ctext);
impl_display!(CText);

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
            Err(ParseError::NotFound("CContent"))
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
impl_display!(CContent);

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
        if rem.len() == 0 { return Err(ParseError::Eof("Comment")); }
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
        count += w.write(b"(")?;
        for &(ws, ref cc) in &self.ccontent {
            if ws { count += w.write(b" ")? }
            count += cc.stream(w)?;
        }
        if self.trailing_ws { count += w.write(b" ")? }
        count += w.write(b")")?;
        Ok(count)
    }
}
impl_display!(Comment);

// 3.2.2
// CFWS            =   (1*([FWS] comment) [FWS]) / FWS
#[derive(Debug, Clone, PartialEq)]
pub struct CFWS {
    pub comments: Vec<(bool, Comment)>, // bool representing if whitespace preceeds it
    pub trailing_ws: bool,
}
impl Parsable for CFWS {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof("Comment Folding White Space")); }
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
            Err(ParseError::NotFound("Comment Folding White Space"))
        }
    }
}
impl Streamable for CFWS {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        let mut count: usize = 0;
        for &(ws, ref comment) in &self.comments {
            if ws { count += w.write(b" ")? }
            count += comment.stream(w)?;
        }
        if self.trailing_ws { count += w.write(b" ")? }
        Ok(count)
    }
}
impl_display!(CFWS);

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
impl_display!(AText);

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
        if input.len()==0 { return Err(ParseError::Eof("Atom")); }
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
        Err(ParseError::NotFound("Atom"))
    }
}
impl Streamable for Atom {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        let mut count: usize = 0;
        if let Some(ref cfws) = self.pre_cfws {
            count += cfws.stream(w)?;
        }
        count += self.atext.stream(w)?;
        if let Some(ref cfws) = self.post_cfws {
            count += cfws.stream(w)?;
        }
        Ok(count)
    }
}
impl_display!(Atom);

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
            if !virgin { count += w.write(b".")? }
            count += part.stream(w)?;
            virgin = false;
        }
        Ok(count)
    }
}
impl_display!(DotAtomText);

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
        if rem.len() == 0 { return Err(ParseError::Eof("DotAtom")); }
        let pre_cfws = parse!(CFWS, rem);
        if let Ok(dat) = parse!(DotAtomText, rem) {
            let post_cfws = parse!(CFWS, rem);
            Ok((DotAtom {
                pre_cfws: pre_cfws.ok(),
                dot_atom_text: dat,
                post_cfws: post_cfws.ok(),
            }, rem))
        } else {
            Err(ParseError::NotFound("DotAtom"))
        }
    }
}
impl Streamable for DotAtom {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        let mut count: usize = 0;
        if let Some(ref cfws) = self.pre_cfws {
            count += cfws.stream(w)?;
        }
        count += self.dot_atom_text.stream(w)?;
        if let Some(ref cfws) = self.post_cfws {
            count += cfws.stream(w)?;
        }
        Ok(count)
    }
}
impl_display!(DotAtom);

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
impl_display!(QText);

// 3.2.4
// qcontent        =   qtext / quoted-pair
#[derive(Debug, Clone, PartialEq)]
pub enum QContent {
    QText(QText),
    QuotedPair(QuotedPair),
}
impl Parsable for QContent {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof("QContent")); }
        if let Ok((x, rem)) = QText::parse(input) {
            Ok((QContent::QText(x), rem))
        }
        else if let Ok((x, rem)) = QuotedPair::parse(input) {
            Ok((QContent::QuotedPair(x), rem))
        }
        else {
            Err(ParseError::NotFound("QContent"))
        }
    }
}
impl Streamable for QContent {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        match *self {
            QContent::QText(ref x) => x.stream(w),
            QContent::QuotedPair(ref x) => x.stream(w),
        }
    }
}
impl_display!(QContent);

// 3.2.4
// quoted-string   =   [CFWS]
//                     DQUOTE *([FWS] qcontent) [FWS] DQUOTE
//                     [CFWS]
#[derive(Debug, Clone, PartialEq)]
pub struct QuotedString {
    pub pre_cfws: Option<CFWS>,
    pub qcontent: Vec<(bool, QContent)>, // bool representing if whitespace preceeds it
    pub trailing_ws: bool,
    pub post_cfws: Option<CFWS>,
}
impl Parsable for QuotedString {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof("QuotedString")); }
        let mut rem = input;
        let pre_cfws = parse!(CFWS, rem);
        req!(rem, b"\"", input);
        let mut qcontent: Vec<(bool, QContent)> = Vec::new();
        let mut ws: bool = false;
        while rem.len() > 0 {
            let t = parse!(FWS, rem);
            ws = t.is_ok();
            if let Ok(qc) = parse!(QContent, rem) {
                qcontent.push((ws, qc));
                continue;
            }
            break;
        }
        req!(rem, b"\"", input);
        let post_cfws = parse!(CFWS, rem);
        Ok((QuotedString {
            pre_cfws: pre_cfws.ok(),
            qcontent: qcontent,
            trailing_ws: ws,
            post_cfws: post_cfws.ok() }, rem))
    }
}
impl Streamable for QuotedString {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        let mut count: usize = 0;
        if let Some(ref cfws) = self.pre_cfws {
            count += cfws.stream(w)?;
        }
        count += w.write(b"\"")?;
        for &(ws, ref qc) in &self.qcontent {
            if ws {
                count += w.write(b" ")?;
            }
            count += qc.stream(w)?;
        }
        if self.trailing_ws {
            count += w.write(b" ")?;
        }
        count += w.write(b"\"")?;
        if let Some(ref cfws) = self.post_cfws {
            count += cfws.stream(w)?;
        }
        Ok(count)
    }
}
impl_display!(QuotedString);

// 3.2.5
// word            =   atom / quoted-string
#[derive(Debug, Clone, PartialEq)]
pub enum Word {
    Atom(Atom),
    QuotedString(QuotedString),
}
impl Parsable for Word {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof("Word")); }
        if let Ok((x, rem)) = Atom::parse(input) {
            Ok((Word::Atom(x), rem))
        }
        else if let Ok((x, rem)) = QuotedString::parse(input) {
            Ok((Word::QuotedString(x), rem))
        }
        else {
            Err(ParseError::NotFound("Word"))
        }
    }
}
impl Streamable for Word {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        match *self {
            Word::Atom(ref x) => x.stream(w),
            Word::QuotedString(ref x) => x.stream(w),
        }
    }
}
impl_display!(Word);

// 3.2.5
// phrase          =   1*word / obs-phrase
#[derive(Debug, Clone, PartialEq)]
pub struct Phrase(pub Vec<Word>);
impl Parsable for Phrase {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof("Phrase")); }
        let mut rem = input;
        let mut output: Vec<Word> = Vec::new();
        while let Ok(word) = parse!(Word, rem) {
            output.push(word);
        }
        if output.len() == 0 {
            Err(ParseError::NotFound("Phrase"))
        } else {
            Ok((Phrase(output), rem))
        }
    }
}
impl Streamable for Phrase {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        let mut count: usize = 0;
        for word in &self.0 {
            count += word.stream(w)?;
        }
        Ok(count)
    }
}
impl_display!(Phrase);

// 3.2.5
// unstructured    = (*([FWS] VCHAR) *WSP) / obs-unstruct
#[derive(Debug, Clone, PartialEq)]
pub struct Unstructured {
    pub leading_ws: bool,
    pub parts: Vec<VChar>, // always separated by whitespace
    pub trailing_ws: bool,
}
impl Parsable for Unstructured {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof("Unstructured")); }
        let mut rem = input;
        let mut output: Vec<VChar> = Vec::new();
        let t = parse!(FWS, rem);
        let leading_ws: bool = t.is_ok();
        while rem.len() > 0 {
            let mut rem2 = match FWS::parse(rem) {
                Ok((_, rem2)) => rem2,
                Err(_) => rem,
            };
            if let Ok(vchar) = parse!(VChar, rem2) {
                rem = rem2;
                output.push(vchar);
                continue;
            }
            break;
        }
        if output.len() == 0 { return Err(ParseError::NotFound("Unstructured")); }
        let t = parse!(WSP, rem);
        Ok((Unstructured {
            leading_ws: leading_ws,
            parts: output,
            trailing_ws: t.is_ok()
        }, rem))
    }
}
impl Streamable for Unstructured {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        let mut count: usize = 0;
        if self.leading_ws { count += w.write(b" ")?; }
        let mut first: bool = true;
        for vc in &self.parts {
            if !first {
                count += w.write(b" ")?;
            }
            count += vc.stream(w)?;
            first = false;
        }
        if self.trailing_ws { count += w.write(b" ")?; }
        Ok(count)
    }
}
impl_display!(Unstructured);

// 3.4.1
// local-part      =   dot-atom / quoted-string / obs-local-part
#[derive(Debug, Clone, PartialEq)]
pub enum LocalPart {
    DotAtom(DotAtom),
    QuotedString(QuotedString),
}
impl Parsable for LocalPart {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof("LocalPart")); }
        if let Ok((x, rem)) = DotAtom::parse(input) {
            Ok((LocalPart::DotAtom(x), rem))
        }
        else if let Ok((x, rem)) = QuotedString::parse(input) {
            Ok((LocalPart::QuotedString(x), rem))
        }
        else {
            Err(ParseError::NotFound("LocalPart"))
        }
    }
}
impl Streamable for LocalPart {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        match *self {
            LocalPart::DotAtom(ref x) => x.stream(w),
            LocalPart::QuotedString(ref x) => x.stream(w),
        }
    }
}
impl_display!(LocalPart);

// 3.4.1
// dtext           =   %d33-90 /          ; Printable US-ASCII
//                     %d94-126 /         ;  characters not including
//                     obs-dtext          ;  "[", "]", or "\"
#[inline]
pub fn is_dtext(c: u8) -> bool { (c>=33 && c<=90) || (c>=94 && c<=126) }
def_cclass!(DText, is_dtext);
impl_display!(DText);

// 3.4.1
// domain-literal  =   [CFWS] "[" *([FWS] dtext) [FWS] "]" [CFWS]
#[derive(Debug, Clone, PartialEq)]
pub struct DomainLiteral {
    pub pre_cfws: Option<CFWS>,
    pub dtext: Vec<(bool, DText)>, // bool representing if whitespace preceeds it
    pub trailing_ws: bool,
    pub post_cfws: Option<CFWS>,
}
impl Parsable for DomainLiteral {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof("DomainLiteral")); }
        let mut rem = input;
        let mut dtext: Vec<(bool, DText)> = Vec::new();
        let pre_cfws = parse!(CFWS, rem);
        req!(rem, b"[", input);
        let mut ws: bool = false;
        while rem.len() > 0 {
            let t = parse!(FWS, rem);
            ws = t.is_ok();
            if let Ok(d) = parse!(DText, rem) {
                dtext.push((ws,d));
                continue;
            }
            break;
        }
        req!(rem, b"]", input);
        let post_cfws = parse!(CFWS, rem);
        Ok((DomainLiteral {
            pre_cfws: pre_cfws.ok(),
            dtext: dtext,
            trailing_ws: ws,
            post_cfws: post_cfws.ok(),
        }, rem))
    }
}
impl Streamable for DomainLiteral {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        let mut count: usize = 0;
        if let Some(ref cfws) = self.pre_cfws {
            count += cfws.stream(w)?;
        }
        count += w.write(b"[")?;
        for &(ws, ref dt) in &self.dtext {
            if ws {  count += w.write(b" ")?; }
            count += dt.stream(w)?;
        }
        count += w.write(b"]")?;
        if let Some(ref cfws) = self.post_cfws {
            count += cfws.stream(w)?;
        }
        Ok(count)
    }
}
impl_display!(DomainLiteral);

// 3.4.1
// domain          =   dot-atom / domain-literal / obs-domain
#[derive(Debug, Clone, PartialEq)]
pub enum Domain {
    DotAtom(DotAtom),
    DomainLiteral(DomainLiteral),
}
impl Parsable for Domain {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof("Domain")); }
        if let Ok((x, rem)) = DotAtom::parse(input) {
            Ok((Domain::DotAtom(x), rem))
        }
        else if let Ok((x, rem)) = DomainLiteral::parse(input) {
            Ok((Domain::DomainLiteral(x), rem))
        }
        else {
            Err(ParseError::NotFound("Domain"))
        }
    }
}
impl Streamable for Domain {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        match *self {
            Domain::DotAtom(ref x) => x.stream(w),
            Domain::DomainLiteral(ref x) => x.stream(w),
        }
    }
}
impl_display!(Domain);

// 3.4.1
// addr-spec       =   local-part "@" domain
#[derive(Debug, Clone, PartialEq)]
pub struct AddrSpec {
    pub local_part: LocalPart,
    pub domain: Domain,
}
impl Parsable for AddrSpec {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof("AddrSpec")); }
        if let Ok((lp, rem)) = LocalPart::parse(input) {
            if rem.len() > 0 && rem[0]==b'@' {
                if let Ok((d, rem)) = Domain::parse(&rem[1..]) {
                    return Ok((AddrSpec {
                        local_part: lp,
                        domain: d
                    }, rem));
                }
            }
        }
        Err(ParseError::NotFound("AddrSpec"))
    }
}
impl Streamable for AddrSpec {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        Ok(self.local_part.stream(w)?
           + w.write(b"@")?
           + self.domain.stream(w)?)
    }
}
impl_display!(AddrSpec);

// 3.4
// angle-addr      =   [CFWS] "<" addr-spec ">" [CFWS] /
//                     obs-angle-addr
#[derive(Debug, Clone, PartialEq)]
pub struct AngleAddr{
    pub pre_cfws: Option<CFWS>,
    pub addr_spec: AddrSpec,
    pub post_cfws: Option<CFWS>,
}
impl Parsable for AngleAddr {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof("AngleAddr")); }
        let mut rem = input;
        let pre_cfws = parse!(CFWS, rem);
        req!(rem, b"<", input);
        if let Ok(aspec) = parse!(AddrSpec, rem) {
            req!(rem, b">", input);
            let post_cfws = parse!(CFWS, rem);
            return Ok((AngleAddr {
                pre_cfws: pre_cfws.ok(),
                addr_spec: aspec,
                post_cfws: post_cfws.ok(),
            }, rem));
        }
        Err(ParseError::NotFound("AngleAddr"))
    }
}
impl Streamable for AngleAddr {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        let mut count: usize = 0;
        if let Some(ref cfws) = self.pre_cfws {
            count += cfws.stream(w)?
        }
        count += w.write(b"<")?;
        count += self.addr_spec.stream(w)?;
        count += w.write(b">")?;
        if let Some(ref cfws) = self.post_cfws {
            count += cfws.stream(w)?
        }
        Ok(count)
    }
}
impl_display!(AngleAddr);

// 3.4
// display-name    =   phrase
#[derive(Debug, Clone, PartialEq)]
pub struct DisplayName(pub Phrase);
impl Parsable for DisplayName {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        Phrase::parse(input).map(|(p,rem)| (DisplayName(p),rem))
    }
}
impl Streamable for DisplayName {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        self.0.stream(w)
    }
}
impl_display!(DisplayName);

// 3.4
// name-addr       =   [display-name] angle-addr
#[derive(Debug, Clone, PartialEq)]
pub struct NameAddr {
    pub display_name: Option<DisplayName>,
    pub angle_addr: AngleAddr
}
impl Parsable for NameAddr {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof("NameAddr")); }
        let mut rem = input;
        let maybe_dn = parse!(DisplayName, rem);
        if let Ok(aa) = parse!(AngleAddr, rem) {
            return Ok((NameAddr {
                display_name: maybe_dn.ok(),
                angle_addr: aa,
            }, rem));
        }
        Err(ParseError::NotFound("NameAddr"))
    }
}
impl Streamable for NameAddr {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        let mut count: usize = 0;
        if self.display_name.is_some() {
            count += self.display_name.as_ref().unwrap().stream(w)?;
        }
        count += self.angle_addr.stream(w)?;
        Ok(count)
    }
}
impl_display!(NameAddr);

// 3.4
// mailbox         =   name-addr / addr-spec
#[derive(Debug, Clone, PartialEq)]
pub enum Mailbox {
    NameAddr(NameAddr),
    AddrSpec(AddrSpec),
}
impl Parsable for Mailbox {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof("Mailbox")); }
        if let Ok((x, rem)) = NameAddr::parse(input) {
            Ok((Mailbox::NameAddr(x), rem))
        }
        else if let Ok((x, rem)) = AddrSpec::parse(input) {
            Ok((Mailbox::AddrSpec(x), rem))
        }
        else {
            Err(ParseError::NotFound("Mailbox"))
        }
    }
}
impl Streamable for Mailbox {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        match *self {
            Mailbox::NameAddr(ref na) => na.stream(w),
            Mailbox::AddrSpec(ref asp) => asp.stream(w),
        }
    }
}
impl_display!(Mailbox);

// 3.4
// mailbox-list    =   (mailbox *("," mailbox)) / obs-mbox-list
#[derive(Debug, Clone, PartialEq)]
pub struct MailboxList(pub Vec<Mailbox>);
impl Parsable for MailboxList {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof("Mailbox List")); }
        let mut rem = input;
        let mut output: Vec<Mailbox> = Vec::new();
        let mut savedrem = rem;
        while let Ok(mailbox) = parse!(Mailbox, rem) {
            savedrem = rem;
            output.push(mailbox);
            if rem.len()==0 || rem[0]!=b',' {
                break;
            }
            rem = &rem[1..];
        }
        rem = savedrem;
        if output.len() == 0 {
            Err(ParseError::NotFound("MailboxList"))
        } else {
            Ok((MailboxList(output), rem))
        }
    }
}
impl Streamable for MailboxList {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        let mut count: usize = 0;
        let mut virgin: bool = true;
        for mb in &self.0 {
            if ! virgin {
                count += w.write(b",")?;
            }
            count += mb.stream(w)?;
            virgin = false;
        }
        Ok(count)
    }
}
impl_display!(MailboxList);

// 3.4
// group-list      =   mailbox-list / CFWS / obs-group-list
#[derive(Debug, Clone, PartialEq)]
pub enum GroupList {
    MailboxList(MailboxList),
    CFWS(CFWS),
}
impl Parsable for GroupList {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof("Group List")); }
        if let Ok((x, rem)) = MailboxList::parse(input) {
            Ok((GroupList::MailboxList(x), rem))
        }
        else if let Ok((x, rem)) = CFWS::parse(input) {
            Ok((GroupList::CFWS(x), rem))
        }
        else {
            Err(ParseError::NotFound("GroupList"))
        }
    }
}
impl Streamable for GroupList {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        match *self {
            GroupList::MailboxList(ref na) => na.stream(w),
            GroupList::CFWS(ref asp) => asp.stream(w),
        }
    }
}
impl_display!(GroupList);

// 3.4
// group           =   display-name ":" [group-list] ";" [CFWS]
#[derive(Debug, Clone, PartialEq)]
pub struct Group {
    pub display_name: DisplayName,
    pub group_list: Option<GroupList>,
    pub cfws: Option<CFWS>,
}
impl Parsable for Group {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof("Group")); }
        let mut rem = input;
        if let Ok(dn) = parse!(DisplayName, rem) {
            req!(rem, b":", input);
            let group_list = parse!(GroupList, rem);
            req!(rem, b";", input);
            let cfws = parse!(CFWS, rem);
            return Ok((Group {
                display_name: dn,
                group_list: group_list.ok(),
                cfws: cfws.ok(),
            }, rem));
        }
        Err(ParseError::NotFound("Group"))
    }
}
impl Streamable for Group {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        let mut count: usize = 0;
        count += self.display_name.stream(w)?;
        count += w.write(b":")?;
        if let Some(ref gl) = self.group_list {
            count += gl.stream(w)?;
        }
        count += w.write(b";")?;
        if let Some(ref cfws) = self.cfws {
            count += cfws.stream(w)?;
        }
        Ok(count)
    }
}
impl_display!(Group);

// 3.4
// address         =   mailbox / group
#[derive(Debug, Clone, PartialEq)]
pub enum Address {
    Mailbox(Mailbox),
    Group(Group),
}
impl Parsable for Address {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof("Address")); }
        if let Ok((x, rem)) = Mailbox::parse(input) {
            Ok((Address::Mailbox(x), rem))
        }
        else if let Ok((x, rem)) = Group::parse(input) {
            Ok((Address::Group(x), rem))
        }
        else {
            Err(ParseError::NotFound("Address"))
        }
    }
}
impl Streamable for Address {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        match *self {
            Address::Mailbox(ref x) => x.stream(w),
            Address::Group(ref x) => x.stream(w),
        }
    }
}
impl_display!(Address);

// 3.4
// address-list    =   (address *("," address)) / obs-addr-list
#[derive(Debug, Clone, PartialEq)]
pub struct AddressList(pub Vec<Address>);
impl Parsable for AddressList {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof("Address List")); }
        let mut rem = input;
        let mut output: Vec<Address> = Vec::new();
        let mut savedrem = rem;
        while let Ok(mailbox) = parse!(Address, rem) {
            savedrem = rem;
            output.push(mailbox);
            if rem.len()==0 || rem[0]!=b',' {
                break;
            }
            rem = &rem[1..];
        }
        rem = savedrem;
        if output.len() == 0 {
            Err(ParseError::NotFound("AddressList"))
        } else {
            Ok((AddressList(output), rem))
        }
    }
}
impl Streamable for AddressList {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        let mut count: usize = 0;
        let mut virgin: bool = true;
        for a in &self.0 {
            if ! virgin {
                count += w.write(b",")?;
            }
            count += a.stream(w)?;
            virgin = false;
        }
        Ok(count)
    }
}
impl_display!(AddressList);

// 3.3
// zone            =   (FWS ( "+" / "-" ) 4DIGIT) / obs-zone
#[derive(Debug, Clone, PartialEq)]
pub struct Zone(pub i32);
impl Parsable for Zone {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof("Zone")); }
        let mut rem = input;
        let fws = parse!(FWS, rem);
        if fws.is_err() { return Err(ParseError::NotFound("Zone")); }
        if rem.len() < 5 { return Err(ParseError::NotFound("Zone")); }
        let sign: i32 = match rem[0] {
            b'+' => 1,
            b'-' => -1,
            _ => return Err(ParseError::NotFound("Zone")),
        };
        if !is_digit(rem[1]) || !is_digit(rem[2]) || !is_digit(rem[3]) || !is_digit(rem[4]) {
            return Err(ParseError::NotFound("Zone"));
        }
        let v: i32 = (1000 * ((rem[1]-48) as i32)
                      + 100 * ((rem[2]-48) as i32)
                      + 10 * ((rem[3]-48) as i32)
                      + ((rem[4]-48) as i32)) * sign;
        Ok((Zone(v), &rem[5..]))
    }
}
impl Streamable for Zone {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        let v = if self.0 < 0 {
            w.write(b" -")?;
            -self.0
        } else {
            w.write(b" +")?;
            self.0
        };
        write!(w, "{:04}", v)?;
        Ok(6)
    }
}
impl_display!(Zone);

// 3.3
// second          =   2DIGIT / obs-second
#[derive(Debug, Clone, PartialEq)]
pub struct Second(pub u8);
impl Parsable for Second {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof("Second")); }
        if input.len() < 2 { return Err(ParseError::NotFound("Second")); }
        if !is_digit(input[0]) || !is_digit(input[1]) {
            return Err(ParseError::NotFound("Second"));
        }
        let v: u8 = (10 * (input[0]-48)) + (input[1]-48);
        Ok((Second(v), &input[2..]))
    }
}
impl Streamable for Second {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        write!(w, "{:02}", self.0)?;
        Ok(2)
    }
}
impl_display!(Second);

// 3.3
// minute          =   2DIGIT / obs-minute
#[derive(Debug, Clone, PartialEq)]
pub struct Minute(pub u8);
impl Parsable for Minute {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof("Minute")); }
        if input.len() < 2 { return Err(ParseError::NotFound("Minute")); }
        if !is_digit(input[0]) || !is_digit(input[1]) {
            return Err(ParseError::NotFound("Minute"));
        }
        let v: u8 = (10 * (input[0]-48)) + (input[1]-48);
        Ok((Minute(v), &input[2..]))
    }
}
impl Streamable for Minute {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        write!(w, "{:02}", self.0)?;
        Ok(2)
    }
}
impl_display!(Minute);

// 3.3
// hour          =   2DIGIT / obs-hour
#[derive(Debug, Clone, PartialEq)]
pub struct Hour(pub u8);
impl Parsable for Hour {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof("Hour")); }
        if input.len() < 2 { return Err(ParseError::NotFound("Hour")); }
        if !is_digit(input[0]) || !is_digit(input[1]) {
            return Err(ParseError::NotFound("Hour"));
        }
        let v: u8 = (10 * (input[0]-48)) + (input[1]-48);
        Ok((Hour(v), &input[2..]))
    }
}
impl Streamable for Hour {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        write!(w, "{:02}", self.0)?;
        Ok(2)
    }
}
impl_display!(Hour);

// 3.3
// time-of-day     =   hour ":" minute [ ":" second ]
#[derive(Debug, Clone, PartialEq)]
pub struct TimeOfDay {
    pub hour: Hour,
    pub minute: Minute,
    pub second: Option<Second>
}
impl Parsable for TimeOfDay {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof("TimeOfDay")); }
        let mut rem = input;
        if let Ok(hour) = parse!(Hour, rem) {
            req!(rem, b":", input);
            if let Ok(minute) = parse!(Minute, rem) {
                let saved = rem;
                if rem.len() > 0 && rem[0]==b':' {
                    rem = &rem[1..];
                    if let Ok(second) = parse!(Second, rem) {
                        return Ok((TimeOfDay {
                            hour: hour,
                            minute: minute,
                            second: Some(second),
                        }, rem));
                    }
                }
                return Ok((TimeOfDay {
                    hour: hour,
                    minute: minute,
                    second: None,
                }, saved));
            }
        }
        Err(ParseError::NotFound("TimeOfDay"))
    }
}
impl Streamable for TimeOfDay {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        if self.second.is_some() {
            write!(w, "{:02}:{:02}:{:02}", self.hour.0, self.minute.0,
                   self.second.as_ref().unwrap().0)?;
            Ok(8)
        } else {
            write!(w, "{:02}:{:02}", self.hour.0, self.minute.0)?;
            Ok(5)
        }
    }
}
impl_display!(TimeOfDay);

// 3.3
// time            =   time-of-day zone
#[derive(Debug, Clone, PartialEq)]
pub struct Time {
    pub time_of_day: TimeOfDay,
    pub zone: Zone,
}
impl Parsable for Time {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof("Time")); }
        let mut rem = input;
        if let Ok(tod) = parse!(TimeOfDay, rem) {
            if let Ok(zone) = parse!(Zone, rem) {
                return Ok((Time {
                    time_of_day: tod,
                    zone: zone
                }, rem));
            }
        }
        Err(ParseError::NotFound("Time"))
    }
}
impl Streamable for Time {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        Ok(self.time_of_day.stream(w)? + self.zone.stream(w)?)
    }
}
impl_display!(Time);

// 3.3
// year            =   (FWS 4*DIGIT FWS) / obs-year
#[derive(Debug, Clone, PartialEq)]
pub struct Year(pub u32);
impl Parsable for Year {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof("Year")); }
        let mut rem = input;
        let fws = parse!(FWS, rem);
        if fws.is_err() { return Err(ParseError::NotFound("Year")); }
        if rem.len() < 5 { return Err(ParseError::NotFound("Year")); }
        if !is_digit(rem[0]) || !is_digit(rem[1]) || !is_digit(rem[2]) || !is_digit(rem[3]) {
            return Err(ParseError::NotFound("Year"));
        }
        let v: u32 = 1000 * ((rem[0]-48) as u32)
                      + 100 * ((rem[1]-48) as u32)
                      + 10 * ((rem[2]-48) as u32)
                      + ((rem[3]-48) as u32);
        rem = &rem[4..];
        let fws = parse!(FWS, rem);
        if fws.is_err() { return Err(ParseError::NotFound("Year")); }
        Ok((Year(v), rem))
    }
}
impl Streamable for Year {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        write!(w, " {:04} ", self.0)?;
        Ok(6)
    }
}
impl_display!(Year);

// 3.3
// month           =   "Jan" / "Feb" / "Mar" / "Apr" /
//                     "May" / "Jun" / "Jul" / "Aug" /
//                     "Sep" / "Oct" / "Nov" / "Dec"
#[derive(Debug, Clone, PartialEq)]
pub struct Month(pub u8);
impl Parsable for Month {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof("Month")); }
        if input.len() < 3 { return Err(ParseError::NotFound("Month")); }
        let three = &input[0..3].to_ascii_lowercase();
        let rem = &input[3..];
        if three==b"jan" { Ok((Month(1), rem)) }
        else if three==b"feb" { Ok((Month(2), rem)) }
        else if three==b"mar" { Ok((Month(3), rem)) }
        else if three==b"apr" { Ok((Month(4), rem)) }
        else if three==b"may" { Ok((Month(5), rem)) }
        else if three==b"jun" { Ok((Month(6), rem)) }
        else if three==b"jul" { Ok((Month(7), rem)) }
        else if three==b"aug" { Ok((Month(8), rem)) }
        else if three==b"sep" { Ok((Month(9), rem)) }
        else if three==b"oct" { Ok((Month(10), rem)) }
        else if three==b"nov" { Ok((Month(11), rem)) }
        else if three==b"dec" { Ok((Month(12), rem)) }
        else { Err(ParseError::NotFound("Month")) }
    }
}
impl Streamable for Month {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        match self.0 {
            1 => Ok(w.write(b"Jan")?),
            2 => Ok(w.write(b"Feb")?),
            3 => Ok(w.write(b"Mar")?),
            4 => Ok(w.write(b"Apr")?),
            5 => Ok(w.write(b"May")?),
            6 => Ok(w.write(b"Jun")?),
            7 => Ok(w.write(b"Jul")?),
            8 => Ok(w.write(b"Aug")?),
            9 => Ok(w.write(b"Sep")?),
            10 => Ok(w.write(b"Oct")?),
            11 => Ok(w.write(b"Nov")?),
            12 => Ok(w.write(b"Dec")?),
            _ => Err(IoError::new(::std::io::ErrorKind::InvalidData, "Month out of range"))
        }
    }
}
impl_display!(Month);

// 3.3
// day             =   ([FWS] 1*2DIGIT FWS) / obs-day
#[derive(Debug, Clone, PartialEq)]
pub struct Day(pub u8);
impl Parsable for Day {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof("Day")); }
        let mut rem = input;
        let _ = parse!(FWS, rem);
        if rem.len() < 3 { return Err(ParseError::NotFound("Day")); }
        if !is_digit(rem[0]) || (!is_digit(rem[1]) && !is_wsp(rem[1])) {
            return Err(ParseError::NotFound("Day"));
        }
        let mut v: u8 = rem[0] - 48;
        let mut num_consumed = 1;
        // the day field may be 1 or 2 digits
        if is_digit(rem[1]) {
            v = 10 * v + rem[1] - 48;
            num_consumed += 1;
        }
        rem = &rem[num_consumed..];
        let fws = parse!(FWS, rem);
        if fws.is_err() { return Err(ParseError::NotFound("Day")); }
        Ok((Day(v), rem))
    }
}
impl Streamable for Day {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        write!(w, " {} ", self.0)?;
        Ok(4)
    }
}
impl_display!(Day);

// 3.3
// date            =   day month year
#[derive(Debug, Clone, PartialEq)]
pub struct Date {
    pub day: Day,
    pub month: Month,
    pub year: Year,
}
impl Parsable for Date {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof("Date")); }
        let mut rem = input;
        if let Ok(day) = parse!(Day, rem) {
            if let Ok(month) = parse!(Month, rem) {
                if let Ok(year) = parse!(Year, rem) {
                    return Ok((Date {
                        day: day,
                        month: month,
                        year: year,
                    }, rem));
                }
            }
        }
        Err(ParseError::NotFound("Date"))
    }
}
impl Streamable for Date {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        Ok(self.day.stream(w)?
           + self.month.stream(w)?
           + self.year.stream(w)?)
    }
}
impl_display!(Date);

// 3.3
// day-name        =   "Mon" / "Tue" / "Wed" / "Thu" /
//                     "Fri" / "Sat" / "Sun"
#[derive(Debug, Clone, PartialEq)]
pub struct DayName(pub u8);
impl Parsable for DayName {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof("DayName")); }
        if input.len() < 3 { return Err(ParseError::NotFound("DayName")); }
        let three = &input[0..3].to_ascii_lowercase();
        let rem = &input[3..];
        if three==b"sun" { Ok((DayName(1), rem)) }
        else if three==b"mon" { Ok((DayName(2), rem)) }
        else if three==b"tue" { Ok((DayName(3), rem)) }
        else if three==b"wed" { Ok((DayName(4), rem)) }
        else if three==b"thu" { Ok((DayName(5), rem)) }
        else if three==b"fri" { Ok((DayName(6), rem)) }
        else if three==b"sat" { Ok((DayName(7), rem)) }
        else { Err(ParseError::NotFound("DayName")) }
    }
}
impl Streamable for DayName {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        match self.0 {
            1 => Ok(w.write(b"Sun")?),
            2 => Ok(w.write(b"Mon")?),
            3 => Ok(w.write(b"Tue")?),
            4 => Ok(w.write(b"Wed")?),
            5 => Ok(w.write(b"Thu")?),
            6 => Ok(w.write(b"Fri")?),
            7 => Ok(w.write(b"Sat")?),
            _ => Err(IoError::new(::std::io::ErrorKind::InvalidData, "Day out of range"))
        }
    }
}
impl_display!(DayName);

// 3.3
// day-of-week     =   ([FWS] day-name) / obs-day-of-week
#[derive(Debug, Clone, PartialEq)]
pub struct DayOfWeek {
    pub pre_fws: Option<FWS>,
    pub day_name: DayName,
}
impl Parsable for DayOfWeek {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof("DayOfWeek")); }
        let mut rem = input;
        let pre_fws = parse!(FWS, rem);
        if let Ok(dn) = parse!(DayName, rem) {
            Ok((DayOfWeek {
                pre_fws: pre_fws.ok(),
                day_name: dn,
            }, rem))
        } else {
            Err(ParseError::NotFound("DayOfWeek"))
        }
    }
}
impl Streamable for DayOfWeek {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        let mut count: usize = 0;
        if let Some(ref fws) = self.pre_fws {
            count += fws.stream(w)?;
        }
        count += self.day_name.stream(w)?;
        Ok(count)
    }
}
impl_display!(DayOfWeek);

// 3.3
// date-time       =   [ day-of-week "," ] date time [CFWS]
#[derive(Debug, Clone, PartialEq)]
pub struct DateTime {
    pub day_of_week: Option<DayOfWeek>,
    pub date: Date,
    pub time: Time,
    pub post_cfws: Option<CFWS>
}
impl Parsable for DateTime {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof("DateTime")); }
        let mut rem = input;
        let mut day_of_week: Option<DayOfWeek> = None;
        if let Ok(dow) = parse!(DayOfWeek, rem) {
            if rem.len() != 0 && rem[0]==b',' {
                rem = &rem[1..];
                day_of_week = Some(dow);
            } else {
                rem = input;
            }
        }
        if let Ok(date) = parse!(Date, rem) {
            if let Ok(time) = parse!(Time, rem) {
                let post_cfws = parse!(CFWS, rem);
                return Ok((DateTime {
                    day_of_week: day_of_week,
                    date: date,
                    time: time,
                    post_cfws: post_cfws.ok()
                }, rem));
            }
        }
        Err(ParseError::NotFound("DateTime"))
    }
}
impl Streamable for DateTime {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        let mut count: usize = 0;
        if let Some(ref dow) = self.day_of_week {
            count += dow.stream(w)?;
            count += w.write(b",")?;
        }
        count += self.date.stream(w)?;
        count += self.time.stream(w)?;
        if let Some(ref cfws) = self.post_cfws {
            count += cfws.stream(w)?;
        }
        Ok(count)
    }
}
impl_display!(DateTime);

// 3.6.4
// no-fold-literal =   "[" *dtext "]"
#[derive(Debug, Clone, PartialEq)]
pub struct NoFoldLiteral(pub DText);
impl Parsable for NoFoldLiteral {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof("No-Fold Literal")); }
        let mut rem = input;
        req!(rem, b"[", input);
        if let Ok(dtext) = parse!(DText, rem) {
            req!(rem, b"]", input);
            return Ok((NoFoldLiteral(dtext), rem));
        }
        Err(ParseError::NotFound("No-Fold Literal"))
    }
}
impl Streamable for NoFoldLiteral {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        Ok(w.write(b"[")?
           + self.0.stream(w)?
           + w.write(b"]")?)
    }
}
impl_display!(NoFoldLiteral);

// 3.6.4
// id-right        =   dot-atom-text / no-fold-literal / obs-id-right
#[derive(Debug, Clone, PartialEq)]
pub enum IdRight {
    DotAtomText(DotAtomText),
    NoFoldLiteral(NoFoldLiteral),
}
impl Parsable for IdRight {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof("Id-right")); }
        if let Ok((x, rem)) = DotAtomText::parse(input) {
            Ok((IdRight::DotAtomText(x), rem))
        }
        else if let Ok((x, rem)) = NoFoldLiteral::parse(input) {
            Ok((IdRight::NoFoldLiteral(x), rem))
        }
        else {
            Err(ParseError::NotFound("Id-right"))
        }
    }
}
impl Streamable for IdRight {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        match *self {
            IdRight::DotAtomText(ref x) => x.stream(w),
            IdRight::NoFoldLiteral(ref x) => x.stream(w),
        }
    }
}
impl_display!(IdRight);

// 3.6.4
// id-left         =   dot-atom-text / obs-id-left
#[derive(Debug, Clone, PartialEq)]
pub struct IdLeft(pub DotAtomText);
impl Parsable for IdLeft {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof("Id-left")); }
        let mut rem = input;
        if let Ok(dat) = parse!(DotAtomText, rem) {
            return Ok((IdLeft(dat), rem));
        }
        Err(ParseError::NotFound("Id-left"))
    }
}
impl Streamable for IdLeft {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        Ok(self.0.stream(w)?)
    }
}
impl_display!(IdLeft);

// 3.6.4
// msg-id          =   [CFWS] "<" id-left "@" id-right ">" [CFWS]
#[derive(Debug, Clone, PartialEq)]
pub struct MsgId {
    pub pre_cfws: Option<CFWS>,
    pub id_left: IdLeft,
    pub id_right: IdRight,
    pub post_cfws: Option<CFWS>,
}
impl Parsable for MsgId {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof("MsgId")); }
        let mut rem = input;
        let pre_cfws = parse!(CFWS, rem);
        req!(rem, b"<", input);
        let idl = match parse!(IdLeft, rem) {
            Err(e) => return Err(e),
            Ok(idl) => idl
        };
        req!(rem, b"@", input);
        let idr = match parse!(IdRight, rem) {
            Err(e) => return Err(e),
            Ok(idr) => idr
        };
        req!(rem, b">", input);
        let post_cfws = parse!(CFWS, rem);
        Ok((MsgId {
            pre_cfws: pre_cfws.ok(),
            id_left: idl,
            id_right: idr,
            post_cfws: post_cfws.ok(),
        }, rem))
    }
}
impl Streamable for MsgId {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        let mut count: usize = 0;
        if let Some(ref cfws) = self.pre_cfws {
            count += cfws.stream(w)?;
        }
        count += w.write(b"<")?;
        count += self.id_left.stream(w)?;
        count += w.write(b"@")?;
        count += self.id_right.stream(w)?;
        count += w.write(b">")?;
        if let Some(ref cfws) = self.post_cfws {
            count += cfws.stream(w)?;
        }
        Ok(count)
    }
}
impl_display!(MsgId);

// 3.6.7
// received-token  =   word / angle-addr / addr-spec / domain
#[derive(Debug, Clone, PartialEq)]
pub enum ReceivedToken {
    Word(Word),
    AngleAddr(AngleAddr),
    AddrSpec(AddrSpec),
    Domain(Domain),
}
impl Parsable for ReceivedToken {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        if input.len() == 0 { return Err(ParseError::Eof("Received Token")); }
        if let Ok((x, rem)) = Word::parse(input) {
            Ok((ReceivedToken::Word(x), rem))
        }
        else if let Ok((x, rem)) = AngleAddr::parse(input) {
            Ok((ReceivedToken::AngleAddr(x), rem))
        }
        else if let Ok((x, rem)) = AddrSpec::parse(input) {
            Ok((ReceivedToken::AddrSpec(x), rem))
        }
        else if let Ok((x, rem)) = Domain::parse(input) {
            Ok((ReceivedToken::Domain(x), rem))
        }
        else {
            Err(ParseError::NotFound("Received Token"))
        }
    }
}
impl Streamable for ReceivedToken {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        match *self {
            ReceivedToken::Word(ref x) => x.stream(w),
            ReceivedToken::AngleAddr(ref x) => x.stream(w),
            ReceivedToken::AddrSpec(ref x) => x.stream(w),
            ReceivedToken::Domain(ref x) => x.stream(w),
        }
    }
}
impl_display!(ReceivedToken);

// 3.6.7
// path            =   angle-addr / ([CFWS] "<" [CFWS] ">" [CFWS])
#[derive(Debug, Clone, PartialEq)]
pub enum Path {
    AngleAddr(AngleAddr),
    Other(Option<CFWS>, Option<CFWS>, Option<CFWS>),
}
impl Parsable for Path {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        let mut rem = input;
        if let Ok(aa) = parse!(AngleAddr, rem) {
            return Ok((Path::AngleAddr(aa), rem));
        }
        let c1 = parse!(CFWS, rem).ok();
        req!(rem, b"<", input);
        let c2 = parse!(CFWS, rem).ok();
        req!(rem, b">", input);
        let c3 = parse!(CFWS, rem).ok();
        Ok((Path::Other(c1,c2,c3), rem))
    }
}
impl Streamable for Path {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        match *self {
            Path::AngleAddr(ref aa) => aa.stream(w),
            Path::Other(ref c1, ref c2, ref c3) => {
                let mut count: usize = 0;
                if let &Some(ref c) = c1 {
                    count += c.stream(w)?;
                }
                count += w.write(b"<")?;
                if let &Some(ref c) = c2 {
                    count += c.stream(w)?;
                }
                count += w.write(b">")?;
                if let &Some(ref c) = c3 {
                    count += c.stream(w)?;
                }
                Ok(count)
            }
        }
    }
}
impl_display!(Path);

// 3.6.8
// ftext           =   %d33-57 /          ; Printable US-ASCII
//                     %d59-126           ;  characters not including
//                                        ;  ":".
#[inline]
pub fn is_ftext(c: u8) -> bool { (c>=33 && c<=57) || (c>=59 && c<=126) }
def_cclass!(FText, is_ftext);
impl_display!(FText);

// 3.6.8
// field-name      =   1*ftext
#[derive(Debug, Clone, PartialEq)]
pub struct FieldName(pub FText);
impl Parsable for FieldName {
    fn parse(input: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        let mut rem = input;
        if let Ok(ftext) = parse!(FText, rem) {
            Ok((FieldName(ftext), rem))
        } else {
            Err(ParseError::NotFound("Field Name"))
        }
    }
}
impl Streamable for FieldName {
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        self.0.stream(w)
    }
}
impl_display!(FieldName);
