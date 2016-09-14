// Format validated types representing lexical tokens defined in
// RFC 5322 (as well as some referred from RFC 5234)
// in order to support SMTP (RFC 5321)

use std::io::Write;
use std::io::Error as IoError;

pub trait Token: Sized {
    fn parse(input: &[u8], pos: &mut usize) -> Option<Self>;
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError>;
}

// Macro for defining sequences of characters within a character class
macro_rules! def_cclass {
    ( $typ:ident, $test:ident) => {
        #[derive(Debug, Clone, PartialEq)]
        pub struct $typ(pub Vec<u8>);
        impl Token for $typ {
            fn parse(input: &[u8], pos: &mut usize) -> Option<Self> {
                let mut output: Vec<u8> = Vec::new();
                while *pos < input.len() && $test(input[*pos]) {
                    output.push(input[*pos]);
                    *pos += 1;
                }
                if output.len() > 0 { Some($typ(output)) }
                else { None }
            }
            fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
                Ok(try!(w.write(&self.0[..])))
            }
        }
    };
}

// RFC 5234, B.1  Core Rules
const CR: u8 = 0x0D;     //   CR             =  %x0D      ; carriage return
const LF: u8 = 0x0A;     //   LF             =  %x0A      ; linefeed
const SP: u8 = 0x20;     //   SP             =  %x20
const HTAB: u8 = 0x09;   //   HTAB           =  %x09      ; horizontal tab
const DQUOTE: u8 = 0x22; //   DQUOTE         =  %x22      ; " (Double Quote)

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
impl Token for QuotedPair {
    fn parse(input: &[u8], pos: &mut usize) -> Option<Self> {
        if *pos >= input.len() - 1 { return None; } // too close to eof
        if input[*pos]!=b'\\' { return None; }
        if is_vchar(input[*pos + 1]) {
            *pos += 2;
            return Some(QuotedPair(input[*pos - 1]))
        }
        if is_wsp(input[*pos + 1]) {
            *pos += 2;
            return Some(QuotedPair(input[*pos - 1]))
        }
        None
    }
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
impl Token for FWS {
    fn parse(input: &[u8], pos: &mut usize) -> Option<Self> {
        if *pos >= input.len() { return None; } // eof
        let savepos = *pos;
        while *pos < input.len() {
            if is_wsp(input[*pos]) {
                *pos += 1;
            }
            else if *pos +3 <= input.len()
                && input[*pos] == CR
                && input[*pos+1] == LF
                && is_wsp(input[*pos+2])
            {
                *pos += 3;
            }
            else {
                break;
            }
        }
        if *pos > savepos {
            Some(FWS)
        } else {
            None
        }
    }
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
impl Token for CContent {
    fn parse(input: &[u8], pos: &mut usize) -> Option<Self> {
        if *pos >= input.len() { return None; } // eof
        if let Some(na) = CText::parse(input, pos) {
            Some(CContent::CText(na))
        }
        else if let Some(asp) = QuotedPair::parse(input, pos) {
            Some(CContent::QuotedPair(asp))
        }
        else if let Some(c) = Comment::parse(input, pos) {
            Some(CContent::Comment(c))
        }
        else {
            None
        }
    }
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
pub struct Comment(pub Vec<CContent>);
impl Token for Comment {
    fn parse(input: &[u8], pos: &mut usize) -> Option<Self> {
        if *pos >= input.len() { return None; } // eof
        let savepos = *pos; // Save position in case things don't work out
        if input[*pos] == b'(' {
            *pos += 1;
            let mut ccontent: Vec<CContent> = Vec::new();
            while !(*pos >= input.len()) {
                let _ = FWS::parse(input, pos);
                if let Some(cc) = CContent::parse(input, pos) {
                    ccontent.push(cc);
                    continue;
                }
                break;
            }
            let _ = FWS::parse(input, pos);
            if input[*pos]==b')' {
                *pos += 1;
                return Some(Comment(ccontent));
            }
        }
        *pos = savepos;
        None
    }
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        let mut count: usize = 0;
        count += try!(w.write(b"("));
        let mut virgin: bool = true;
        for cc in &self.0 {
            if !virgin { count += try!(w.write(b" ")) }
            count += try!(cc.stream(w));
            virgin = false;
        }
        count += try!(w.write(b")"));
        Ok(count)
    }
}

// 3.2.2
// CFWS            =   (1*([FWS] comment) [FWS]) / FWS
#[derive(Debug, Clone, PartialEq)]
pub struct CFWS(Vec<Comment>);
impl Token for CFWS {
    fn parse(input: &[u8], pos: &mut usize) -> Option<Self> {
        if *pos >= input.len() { return None; } // eof
        let savepos = *pos; // Save position in case things don't work out
        let mut comments: Vec<Comment> = Vec::new();
        while !(*pos >= input.len()) {
            let _ = FWS::parse(input, pos);
            if let Some(comment) = Comment::parse(input, pos) {
                comments.push(comment);
                continue;
            }
            break;
        }
        let _ = FWS::parse(input, pos);
        if *pos > savepos {
            Some(CFWS(comments))
        } else {
            None
        }
    }
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        let mut count: usize = 0;
        count += try!(w.write(b" ")); // FIXME - fold?
        for comment in &self.0 {
            count += try!(comment.stream(w));
            count += try!(w.write(b" ")); // FIXME - fold?
        }
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
    pub pre_comments: Vec<Comment>,
    pub atext: AText,
    pub post_comments: Vec<Comment>,
}
impl Token for Atom {
    fn parse(input: &[u8], pos: &mut usize) -> Option<Self> {
        if *pos >= input.len() { return None; } // eof
        let savepos = *pos; // Save position in case things don't work out
        let pre_comments = match CFWS::parse(input, pos) {
            Some(cfws) => cfws.0,
            None => Vec::new(),
        };
        if let Some(atext) = AText::parse(input, pos) {
            let post_comments = match CFWS::parse(input, pos) {
                Some(cfws) => cfws.0,
                None => Vec::new(),
            };
            return Some(Atom {
                pre_comments: pre_comments,
                atext: atext,
                post_comments: post_comments,
            });
        }
        *pos = savepos;
        None
    }
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        let mut count: usize = 0;
        count += try!(w.write(b" "));
        for pre_comment in &self.pre_comments {
            count += try!(pre_comment.stream(w));
            count += try!(w.write(b" "));
        }
        count += try!(self.atext.stream(w));
        for post_comment in &self.post_comments {
            count += try!(w.write(b" "));
            count += try!(post_comment.stream(w));
        }
        count += try!(w.write(b" "));
        Ok(count)
    }
}

// 3.2.3
// dot-atom-text   =   1*atext *("." 1*atext)
// dot-atom        =   [CFWS] dot-atom-text [CFWS]
#[derive(Debug, Clone, PartialEq)]
pub struct DotAtom {
    pub pre_comments: Vec<Comment>,
    pub parts: Vec<AText>,
    pub post_comments: Vec<Comment>,
}
impl Token for DotAtom {
    fn parse(input: &[u8], pos: &mut usize) -> Option<Self> {
        if *pos >= input.len() { return None; } // eof
        let savepos = *pos; // Save position in case things don't work out
        let pre_comments = match CFWS::parse(input, pos) {
            Some(cfws) => cfws.0,
            None => Vec::new(),
        };
        let mut parts: Vec<AText> = Vec::new();
        while !(*pos >= input.len()) {
            if let Some(part) = AText::parse(input, pos) {
                parts.push(part);
                if input[*pos]==b'.' {
                    *pos += 1;
                    continue;
                }
                break;
            } else {
                *pos = savepos;
                return None;
            }
        }
        if parts.len() == 0 {
            *pos = savepos;
            return None;
        }
        let post_comments = match CFWS::parse(input, pos) {
            Some(cfws) => cfws.0,
            None => Vec::new(),
        };
        Some(DotAtom {
            pre_comments: pre_comments,
            parts: parts,
            post_comments: post_comments,
        })
    }
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        let mut count: usize = 0;
        count += try!(w.write(b" "));
        for pre_comment in &self.pre_comments {
            count += try!(pre_comment.stream(w));
            count += try!(w.write(b" "));
        }
        let mut virgin: bool = true;
        for part in &self.parts {
            if !virgin { count += try!(w.write(b".")) }
            count += try!(part.stream(w));
            virgin = false;
        }
        for post_comment in &self.post_comments {
            count += try!(w.write(b" "));
            count += try!(post_comment.stream(w));
        }
        count += try!(w.write(b" "));
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

// 3.2.4
// qcontent        =   qtext / quoted-pair
#[derive(Debug, Clone, PartialEq)]
pub enum QContent {
    QText(QText),
    QuotedPair(QuotedPair),
}
impl Token for QContent {
    fn parse(input: &[u8], pos: &mut usize) -> Option<Self> {
        if *pos >= input.len() { return None; } // eof
        if let Some(x) = QText::parse(input, pos) {
            Some(QContent::QText(x))
        }
        else if let Some(x) = QuotedPair::parse(input, pos) {
            Some(QContent::QuotedPair(x))
        }
        else {
            None
        }
    }
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        match *self {
            QContent::QText(ref x) => x.stream(w),
            QContent::QuotedPair(ref x) => x.stream(w),
        }
    }
}

// 3.2.4
// quoted-string   =   [CFWS]
//                     DQUOTE *([FWS] qcontent) [FWS] DQUOTE
//                     [CFWS]
#[derive(Debug, Clone, PartialEq)]
pub struct QuotedString {
    pub pre_comments: Vec<Comment>,
    pub qcontent: Vec<QContent>,
    pub post_comments: Vec<Comment>,
}
impl Token for QuotedString {
    fn parse(input: &[u8], pos: &mut usize) -> Option<Self> {
        if *pos >= input.len() { return None; } // eof
        let savepos = *pos; // Save position in case things don't work out
        let pre_comments = match CFWS::parse(input, pos) {
            Some(cfws) => cfws.0,
            None => Vec::new(),
        };
        if input[*pos] == DQUOTE {
            *pos += 1;
            let mut qcontent: Vec<QContent> = Vec::new();
            while !(*pos >= input.len()) {
                let _ = FWS::parse(input, pos);
                if let Some(qc) = QContent::parse(input, pos) {
                    qcontent.push(qc);
                    continue;
                }
                break;
            }
            let _ = FWS::parse(input, pos);
            if input[*pos]==DQUOTE {
                *pos += 1;
                let post_comments = match CFWS::parse(input, pos) {
                    Some(cfws) => cfws.0,
                    None => Vec::new(),
                };
                return Some(QuotedString {
                    pre_comments: pre_comments,
                    qcontent: qcontent,
                    post_comments: post_comments,
                });
            }
        }
        *pos = savepos;
        None
    }
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        let mut count: usize = 0;
        count += try!(w.write(b" "));
        for pre_comment in &self.pre_comments {
            count += try!(pre_comment.stream(w));
            count += try!(w.write(b" "));
        }
        count += try!(w.write(b"\""));
        let mut virgin: bool = false;
        for qcontent in &self.qcontent {
            if !virgin { count += try!(w.write(b" ")) }
            count += try!(qcontent.stream(w));
            virgin = false;
        }
        count += try!(w.write(b"\""));
        for post_comment in &self.post_comments {
            count += try!(w.write(b" "));
            count += try!(post_comment.stream(w));
        }
        count += try!(w.write(b" "));
        Ok(count)
    }
}

// 3.2.5
// word            =   atom / quoted-string
#[derive(Debug, Clone, PartialEq)]
pub enum Word {
    Atom(Atom),
    QuotedString(QuotedString),
}
impl Token for Word {
    fn parse(input: &[u8], pos: &mut usize) -> Option<Self> {
        if *pos >= input.len() { return None; } // eof
        if let Some(x) = Atom::parse(input, pos) {
            Some(Word::Atom(x))
        }
        else if let Some(x) = QuotedString::parse(input, pos) {
            Some(Word::QuotedString(x))
        }
        else {
            None
        }
    }
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        match *self {
            Word::Atom(ref x) => x.stream(w),
            Word::QuotedString(ref x) => x.stream(w),
        }
    }
}

// 3.2.5
// phrase          =   1*word / obs-phrase
#[derive(Debug, Clone, PartialEq)]
pub struct Phrase(pub Vec<Word>);
impl Token for Phrase {
    fn parse(input: &[u8], pos: &mut usize) -> Option<Self> {
        let mut output: Vec<Word> = Vec::new();
        while !(*pos >= input.len()) {
            if let Some(word) = Word::parse(input, pos) {
                output.push(word);
                continue;
            }
            break;
        }
        if output.len() == 0 {
            None
        } else {
            Some(Phrase(output))
        }
    }
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        let mut count: usize = 0;
        for word in &self.0 {
            count += try!(word.stream(w));
        }
        Ok(count)
    }
}

// 3.2.5
// unstructured    = (*([FWS] VCHAR) *WSP) / obs-unstruct
#[derive(Debug, Clone, PartialEq)]
pub struct Unstructured(pub Vec<VChar>);
impl Token for Unstructured {
    fn parse(input: &[u8], pos: &mut usize) -> Option<Self> {
        let savepos = *pos; // Save position in case things don't work out
        let mut bumppos = *pos;
        let mut output: Vec<VChar> = Vec::new();
        while !(*pos >= input.len()) {
            let _ = FWS::parse(input, pos);
            if let Some(vchar) = VChar::parse(input, pos) {
                output.push(vchar);
                bumppos = *pos; // bump
            } else {
                break;
            }
        }
        *pos = bumppos; // back up (any FWS::parse gets undone)
        let _ = WSP::parse(input, pos);
        if output.len() == 0 {
            *pos = savepos;
            None
        } else {
            Some(Unstructured(output))
        }
    }
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        let mut count: usize = 0;
        for t in &self.0 {
            count += try!(w.write(b" "));
            count += try!(t.stream(w));
        }
        Ok(count)
    }
}

// 3.3
// We are not coding section 3.3 because we are not accepting strings from the
// end user, we accept strongly typed dates only, and stream appropriately

// 3.4
// address         =   mailbox / group
#[derive(Debug, Clone, PartialEq)]
pub enum Address {
    Mailbox(Mailbox),
    Group(Group),
}
impl Token for Address {
    fn parse(input: &[u8], pos: &mut usize) -> Option<Self> {
        if *pos >= input.len() { return None; } // eof
        if let Some(m) = Mailbox::parse(input, pos) {
            Some(Address::Mailbox(m))
        }
        else if let Some(g) = Group::parse(input, pos) {
            Some(Address::Group(g))
        }
        else {
            None
        }
    }
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        match *self {
            Address::Mailbox(ref x) => x.stream(w),
            Address::Group(ref x) => x.stream(w),
        }
    }
}

// 3.4
// mailbox         =   name-addr / addr-spec
#[derive(Debug, Clone, PartialEq)]
pub enum Mailbox {
    NameAddr(NameAddr),
    AddrSpec(AddrSpec),
}
impl Token for Mailbox {
    fn parse(input: &[u8], pos: &mut usize) -> Option<Self> {
        if *pos >= input.len() { return None; } // eof
        if let Some(na) = NameAddr::parse(input, pos) {
            Some(Mailbox::NameAddr(na))
        }
        else if let Some(asp) = AddrSpec::parse(input, pos) {
            Some(Mailbox::AddrSpec(asp))
        }
        else {
            None
        }
    }
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        match *self {
            Mailbox::NameAddr(ref na) => na.stream(w),
            Mailbox::AddrSpec(ref asp) => asp.stream(w),
        }
    }
}

// 3.4
// name-addr       =   [display-name] angle-addr
#[derive(Debug, Clone, PartialEq)]
pub struct NameAddr {
    pub display_name: Option<DisplayName>,
    pub angle_addr: AngleAddr
}
impl Token for NameAddr {
    fn parse(input: &[u8], pos: &mut usize) -> Option<Self> {
        if *pos >= input.len() { return None; } // eof
        let savepos = *pos; // Save position in case things don't work out
        let maybe_dn = DisplayName::parse(input, pos);
        if let Some(aa) = AngleAddr::parse(input, pos) {
            return Some(NameAddr {
                display_name: maybe_dn,
                angle_addr: aa,
            });
        }
        *pos = savepos;
        None
    }
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        let mut count: usize = 0;
        if self.display_name.is_some() {
            count += try!(self.display_name.as_ref().unwrap().stream(w));
        }
        count += try!(w.write(b" "));
        count += try!(self.angle_addr.stream(w));
        Ok(count)
    }
}

// 3.4
// angle-addr      =   [CFWS] "<" addr-spec ">" [CFWS] /
//                     obs-angle-addr
#[derive(Debug, Clone, PartialEq)]
pub struct AngleAddr{
    pub leading_ws: Option<CFWS>,
    pub addr_spec: AddrSpec,
    pub trailing_ws: Option<CFWS>,
}
impl Token for AngleAddr {
    fn parse(input: &[u8], pos: &mut usize) -> Option<Self> {
        if *pos >= input.len() { return None; } // eof
        let savepos = *pos; // Save position in case things don't work out
        let maybe_leading_ws = CFWS::parse(input, pos);
        if input[*pos] == b'<' {
            *pos += 1;
            if let Some(asp) = AddrSpec::parse(input, pos) {
                if input[*pos] == b'>' {
                    *pos += 1;
                    let maybe_trailing_ws = CFWS::parse(input, pos);
                    return Some(AngleAddr {
                        leading_ws: maybe_leading_ws,
                        addr_spec: asp,
                        trailing_ws: maybe_trailing_ws,
                    });
                }
            }
        }
        *pos = savepos;
        None
    }
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        let mut count: usize = 0;
        if let Some(ref ws) = self.leading_ws {
            count += try!(ws.stream(w))
        }
        count += try!(w.write(b"<"));
        count += try!(self.addr_spec.stream(w));
        count += try!(w.write(b">"));
        if let Some(ref ws) = self.trailing_ws {
            count += try!(ws.stream(w))
        }
        Ok(count)
    }
}

// 3.4
// group           =   display-name ":" [group-list] ";" [CFWS]
#[derive(Debug, Clone, PartialEq)]
pub struct Group {
    display_name: DisplayName,
    group_list: Option<GroupList>,
    post_comments: Vec<Comment>
}
impl Token for Group {
    fn parse(input: &[u8], pos: &mut usize) -> Option<Self> {
        if *pos >= input.len() { return None; } // eof
        let savepos = *pos; // Save position in case things don't work out
        if let Some(dn) = DisplayName::parse(input, pos) {
            if input[*pos] == b':' {
                *pos += 1;
                let maybe_group_list = GroupList::parse(input, pos);
                if input[*pos] == b';' {
                    *pos += 1;
                    let post_comments = match CFWS::parse(input, pos) {
                        Some(cfws) => cfws.0,
                        None => Vec::new(),
                    };
                    return Some(Group {
                        display_name: dn,
                        group_list: maybe_group_list,
                        post_comments: post_comments,
                    });
                }
            }
        }
        *pos = savepos;
        None
    }
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        let mut count: usize = 0;
        count += try!(self.display_name.stream(w));
        count += try!(w.write(b":"));
        if let Some(ref gl) = self.group_list {
            count += try!(gl.stream(w));
        }
        count += try!(w.write(b";"));
        for post_comment in &self.post_comments {
            count += try!(w.write(b" "));
            count += try!(post_comment.stream(w));
        }
        count += try!(w.write(b" "));
        Ok(count)
    }
}

// 3.4
// display-name    =   phrase
#[derive(Debug, Clone, PartialEq)]
pub struct DisplayName(pub Phrase);
impl Token for DisplayName {
    fn parse(input: &[u8], pos: &mut usize) -> Option<Self> {
        if let Some(p) = Phrase::parse(input, pos) {
            Some(DisplayName(p))
        } else {
            None
        }
    }
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        self.0.stream(w)
    }
}

// 3.4
// mailbox-list    =   (mailbox *("," mailbox)) / obs-mbox-list
#[derive(Debug, Clone, PartialEq)]
pub struct MailboxList(pub Vec<Mailbox>);
impl Token for MailboxList {
    fn parse(input: &[u8], pos: &mut usize) -> Option<Self> {
        if *pos >= input.len() { return None; } // eof
        let mut output: Vec<Mailbox> = Vec::new();
        if let Some(mailbox) = Mailbox::parse(input, pos) {
            output.push(mailbox);
        } else {
            return None;
        }
        while !(*pos >= input.len()) {
            if input[*pos] != b',' {
                break;
            }
            *pos += 1;
            if let Some(mailbox) = Mailbox::parse(input, pos) {
                output.push(mailbox);
            } else {
                *pos -= 1; // return the comma
                break;
            }
        }
        Some(MailboxList(output))
    }
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        let mut count: usize = 0;
        let mut virgin: bool = true;
        for mb in &self.0 {
            if ! virgin {
                count += try!(w.write(b", "));
            }
            count += try!(mb.stream(w));
            virgin = false;
        }
        Ok(count)
    }
}

// 3.4
// address-list    =   (address *("," address)) / obs-addr-list
#[derive(Debug, Clone, PartialEq)]
pub struct AddressList(pub Vec<Address>);
impl Token for AddressList {
    fn parse(input: &[u8], pos: &mut usize) -> Option<Self> {
        if *pos >= input.len() { return None; } // eof
        let mut output: Vec<Address> = Vec::new();
        if let Some(mailbox) = Address::parse(input, pos) {
            output.push(mailbox);
        } else {
            return None;
        }
        while !(*pos >= input.len()) {
            if input[*pos] != b',' {
                break;
            }
            *pos += 1;
            if let Some(mailbox) = Address::parse(input, pos) {
                output.push(mailbox);
            } else {
                *pos -= 1; // return the comma
                break;
            }
        }
        Some(AddressList(output))
    }
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        let mut count: usize = 0;
        let mut virgin: bool = true;
        for a in &self.0 {
            if ! virgin {
                count += try!(w.write(b", "));
            }
            count += try!(a.stream(w));
            virgin = false;
        }
        Ok(count)
    }
}

// 3.4
// group-list      =   mailbox-list / CFWS / obs-group-list
#[derive(Debug, Clone, PartialEq)]
pub enum GroupList {
    MailboxList(MailboxList),
    Comments(Vec<Comment>),
}
impl Token for GroupList {
    fn parse(input: &[u8], pos: &mut usize) -> Option<Self> {
        if let Some(mbl) = MailboxList::parse(input, pos) {
            return Some(GroupList::MailboxList(mbl));
        }
        if let Some(cfws) = CFWS::parse(input, pos) {
            return Some(GroupList::Comments(cfws.0));
        }
        None
    }
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        match *self {
            GroupList::MailboxList(ref mbl) => {
                mbl.stream(w)
            },
            GroupList::Comments(ref cs) => {
                let mut count: usize = 0;
                for c in cs {
                    count += try!(w.write(b" "));
                    count += try!(c.stream(w));
                }
                count += try!(w.write(b" "));
                Ok(count)
            }
        }
    }
}

// 3.4.1
// addr-spec       =   local-part "@" domain
#[derive(Debug, Clone, PartialEq)]
pub struct AddrSpec {
    pub local_part: LocalPart,
    pub domain: Domain,
}
impl Token for AddrSpec {
    fn parse(input: &[u8], pos: &mut usize) -> Option<Self> {
        if *pos >= input.len() { return None; } // eof
        let savepos = *pos; // Save position in case things don't work out
        if let Some(lp) = LocalPart::parse(input, pos) {
            if input[*pos] == b'@' {
                *pos += 1;
                if let Some(d) = Domain::parse(input, pos) {
                    return Some(AddrSpec {
                        local_part: lp,
                        domain: d,
                    });
                }
            }
        }
        *pos = savepos;
        None
    }
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        Ok(try!(self.local_part.stream(w))
           + try!(w.write(b"@"))
           + try!(self.domain.stream(w)))
    }
}

// 3.4.1
// local-part      =   dot-atom / quoted-string / obs-local-part
#[derive(Debug, Clone, PartialEq)]
pub enum LocalPart {
    DotAtom(DotAtom),
    QuotedString(QuotedString),
}
impl Token for LocalPart {
    fn parse(input: &[u8], pos: &mut usize) -> Option<Self> {
        if *pos >= input.len() { return None; } // eof
        if let Some(x) = DotAtom::parse(input, pos) {
            Some(LocalPart::DotAtom(x))
        }
        else if let Some(x) = QuotedString::parse(input, pos) {
            Some(LocalPart::QuotedString(x))
        }
        else {
            None
        }
    }
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        match *self {
            LocalPart::DotAtom(ref x) => x.stream(w),
            LocalPart::QuotedString(ref x) => x.stream(w),
        }
    }
}

// 3.4.1
// domain          =   dot-atom / domain-literal / obs-domain
#[derive(Debug, Clone, PartialEq)]
pub enum Domain {
    DotAtom(DotAtom),
    DomainLiteral(DomainLiteral),
}
impl Token for Domain {
    fn parse(input: &[u8], pos: &mut usize) -> Option<Self> {
        if *pos >= input.len() { return None; } // eof
        if let Some(x) = DotAtom::parse(input, pos) {
            Some(Domain::DotAtom(x))
        }
        else if let Some(x) = DomainLiteral::parse(input, pos) {
            Some(Domain::DomainLiteral(x))
        }
        else {
            None
        }
    }
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        match *self {
            Domain::DotAtom(ref x) => x.stream(w),
            Domain::DomainLiteral(ref x) => x.stream(w),
        }
    }
}

// 3.4.1
// domain-literal  =   [CFWS] "[" *([FWS] dtext) [FWS] "]" [CFWS]
#[derive(Debug, Clone, PartialEq)]
pub struct DomainLiteral {
    pub leading_ws: Option<CFWS>,
    pub dtext: Vec<DText>,
    pub trailing_ws: Option<CFWS>,
}
impl Token for DomainLiteral {
    fn parse(input: &[u8], pos: &mut usize) -> Option<Self> {
        if *pos >= input.len() { return None; } // eof
        let savepos = *pos; // Save position in case things don't work out
        let maybe_leading_ws = CFWS::parse(input, pos);
        if input[*pos]==b'[' {
            *pos += 1;
            let mut dtext: Vec<DText> = Vec::new();
            while !(*pos >= input.len()) {
                let _ = FWS::parse(input, pos);
                if let Some(d) = DText::parse(input, pos) {
                    dtext.push(d);
                    continue;
                }
                break;
            }
            let _ = FWS::parse(input, pos);
            if input[*pos]==b']' {
                *pos += 1;
                let maybe_trailing_ws = CFWS::parse(input, pos);
                return Some(DomainLiteral {
                    leading_ws: maybe_leading_ws,
                    dtext: dtext,
                    trailing_ws: maybe_trailing_ws,
                });
            }
        }
        *pos = savepos;
        None
    }
    fn stream<W: Write>(&self, w: &mut W) -> Result<usize, IoError> {
        let mut count: usize = 0;
        if let Some(ref ws) = self.leading_ws {
            count += try!(ws.stream(w));
        }
        count += try!(w.write(b"["));
        let mut virgin: bool = true;
        for d in &self.dtext {
            if !virgin { count += try!(w.write(b" ")) }
            count += try!(d.stream(w));
            virgin = false;
        }
        count += try!(w.write(b"]"));
        if let Some(ref ws) = self.trailing_ws {
            count += try!(ws.stream(w));
        }
        Ok(count)
    }
}

// 3.4.1
// dtext           =   %d33-90 /          ; Printable US-ASCII
//                     %d94-126 /         ;  characters not including
//                     obs-dtext          ;  "[", "]", or "\"
#[inline]
pub fn is_dtext(c: u8) -> bool { (c>=33 && c<=90) || (c>=94 && c<=126) }
def_cclass!(DText, is_dtext);

// Section 3.5 and onwards has not been written yet

// 3.5
// message         =   (fields / obs-fields)
//                     [CRLF body]

// 3.5
// body            =   (*(*998text CRLF) *998text) / obs-body

// 3.5
// text            =   %d1-9 /            ; Characters excluding CR
//                     %d11 /             ;  and LF
//                     %d12 /
//                     %d14-127

// 3.6.1
// orig-date       =   "Date:" date-time CRLF
