
macro_rules! req_name {
    ($rem:ident, $str:expr, $input:ident) => {
        let len: usize = $str.len();
        if $rem.len() < len || &(&$rem[0..len]).to_ascii_lowercase()!=$str {
            return Err(ParseError::NotFound);
        }
        $rem = &$rem[len..];
    };
}

macro_rules! req_crlf {
    ($rem:ident, $input:ident) => {
        if &$rem[..2] != b"\r\n" {
            return Err(ParseError::NotFound);
        }
        $rem = &$rem[2..];
    }
}
