use std::{io, io::Read};

use crate::{
    errors::{Error, ErrorKind},
    token::Token,
    traits::StringReader,
};

pub struct Scanner {
    input: Box<dyn Read + 'static>,
}

impl Scanner {
    pub fn new(_input: &'static str) -> Self {
        Scanner {
            input: Box::new(StringReader::from(_input)),
        }
    }

    pub fn read_token(&mut self) -> Result<Token, Error> {
        let mut word = String::new();
        let mut buf = [b'\x00'];
        loop {
            match self.input.read(&mut buf) {
                Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Err(Error::eof()),
                Err(e) => return Err(Error::with_cause(ErrorKind::Unknown, Box::new(e))),
                _ => {}
            };

            match buf[0] {
                b'\x00' | b' ' | b'\t' | b'\r' | b'\n' | b'\x08' | b'\x0C' => continue,
                b'-' | b'0'..=b'9' => {
                    word.push(buf[0].into());
                    return self.read_numeric(word);
                }
                b'.' => {
                    word.push(buf[0].into());
                    return self.read_real(word);
                }
                b'(' => {
                    return self.read_string();
                }
                b'%' => self.read_comment(),
                _ => {
                    word.push(buf[0].into());
                    return self.read_name(word);
                }
            }?;
        }
    }

    fn next_byte(&mut self) -> Result<Option<u8>, Error> {
        let mut buf = [0];
        match self.input.read(&mut buf) {
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => Ok(None),
            Err(e) => return Err(Error::with_cause(ErrorKind::Unknown, Box::new(e))),
            _ => Ok(Some(buf[0])),
        }
    }

    fn read_comment(&mut self) -> Result<(), Error> {
        loop {
            match self.next_byte() {
                Err(e) => return Err(e),
                Ok(ch) => match ch {
                    None => break,
                    Some(ch) => match ch {
                        b'\n' | b'\x0c' => break,
                        _ => {}
                    },
                },
            }
        }

        Ok(())
    }

    fn read_numeric(&mut self, mut word: String) -> Result<Token, Error> {
        loop {
            match self.next_byte() {
                Err(e) => return Err(e),
                Ok(ch) => match ch {
                    None => break,
                    Some(ch) => match ch {
                        b'\x00' | b' ' | b'\t' | b'\r' | b'\n' | b'\x08' | b'\x0C' => break,
                        b'0'..=b'9' => word.push(ch.into()),
                        b'.' => {
                            word.push(ch.into());
                            return self.read_real(word);
                        }
                        b'#' => {
                            if word.starts_with("-") {
                                return Err(Error::new(ErrorKind::Syntax));
                            }
                            word.push(ch.into());
                            return self.read_radix(word);
                        }
                        _ => {
                            word.push(ch.into());
                            return self.read_name(word);
                        }
                    },
                },
            }
        }

        if let Ok(value) = word.parse::<i32>() {
            Ok(Token::Integer(value))
        } else {
            Err(Error::new(ErrorKind::Syntax))
        }
    }

    fn read_real(&mut self, mut word: String) -> Result<Token, Error> {
        loop {
            match self.next_byte() {
                Err(e) => return Err(e),
                Ok(ch) => match ch {
                    None => break,
                    Some(ch) => match ch {
                        b'\x00' | b' ' | b'\t' | b'\r' | b'\n' | b'\x08' | b'\x0C' => {
                            if word.to_lowercase().ends_with('e') {
                                return Err(Error::new(ErrorKind::Syntax));
                            }
                            break;
                        }
                        b'e' | b'E' => {
                            if word.to_lowercase().contains('e') {
                                return self.read_name(word);
                            }
                            word.push(ch.into());
                        }
                        b'-' => {
                            if !word.to_lowercase().ends_with('e') {
                                return self.read_name(word);
                            }
                            word.push(ch.into());
                        }
                        b'0'..=b'9' => word.push(ch.into()),
                        _ => {
                            word.push(ch.into());
                            return self.read_name(word);
                        }
                    },
                },
            }
        }

        word = word.to_lowercase();
        if word.contains('e') {
            let mut parts = word.split('e');
            let decimal = parts
                .next()
                .expect("TODO: figure out a clean way to do this");
            let exponent = parts
                .next()
                .expect("TODO: figure out a clean way to do this");

            if let Ok(decimal) = decimal.parse::<f64>() {
                if let Ok(exponent) = exponent.parse::<i32>() {
                    let value = decimal * 10.0_f64.powi(exponent);
                    return Ok(Token::Real(value));
                }
                return Err(Error::new(ErrorKind::Syntax));
            }
            return Err(Error::new(ErrorKind::Syntax));
        }

        if let Ok(value) = word.parse::<f64>() {
            Ok(Token::Real(value))
        } else {
            Err(Error::new(ErrorKind::Syntax))
        }
    }

    fn read_radix(&mut self, mut word: String) -> Result<Token, Error> {
        loop {
            match self.next_byte() {
                Err(e) => return Err(e),
                Ok(ch) => match ch {
                    None => break,
                    Some(ch) => match ch {
                        b'\x00' | b' ' | b'\t' | b'\r' | b'\n' | b'\x08' | b'\x0C' => {
                            if word.ends_with('#') {
                                return Err(Error::new(ErrorKind::Syntax));
                            }
                            break;
                        }
                        b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z' => word.push(ch.into()),
                        _ => {
                            word.push(ch.into());
                            // read_name
                        }
                    },
                },
            }
        }

        let mut parts = word.split('#');
        let base = parts.next().expect("TODO: handle this neatly");
        let digits = parts.next().expect("TODO: handle this neatly");

        if let Ok(base) = base.parse::<u32>() {
            if let Ok(value) = i32::from_str_radix(digits, base) {
                Ok(Token::Integer(value))
            } else {
                Err(Error::new(ErrorKind::Syntax))
            }
        } else {
            Err(Error::new(ErrorKind::Syntax))
        }
    }

    fn read_string(&mut self) -> Result<Token, Error> {
        let mut word = String::new();
        let mut active_parenthesis = 0;

        loop {
            match self.next_byte() {
                Err(e) => return Err(e),
                Ok(ch) => match ch {
                    None => return Err(Error::new(ErrorKind::UnterminatedString)),
                    Some(ch) => match ch {
                        b'(' => {
                            word.push(ch.into());
                            active_parenthesis += 1;
                        }
                        b')' => {
                            if active_parenthesis < 1 {
                                break;
                            }
                            word.push(ch.into());
                            active_parenthesis -= 1;
                        }
                        b'\\' => {
                            let next_ch = match self.next_byte() {
                                Err(e) => Err(e),
                                Ok(next_ch) => match next_ch {
                                    None => Err(Error::eof()),
                                    Some(next_ch) => Ok(next_ch),
                                },
                            }?;
                            match next_ch {
                                b'\n' => continue,
                                b'r' => word.push('\r'),
                                b'n' => word.push('\n'),
                                b't' => word.push('\t'),
                                b'b' => word.push('\x08'),
                                b'f' => word.push('\x0C'),
                                b'\\' => word.push('\\'),
                                b'(' => word.push('('),
                                b')' => word.push(')'),
                                // TODO: support \r\n and octals
                                _ => word.push(next_ch.into()),
                            }
                        }
                        _ => word.push(ch.into()),
                    },
                },
            }
        }

        Ok(Token::String(word))
    }

    // TODO: support hex and base85 strings

    fn read_name(&mut self, mut word: String) -> Result<Token, Error> {
        loop {
            match self.next_byte() {
                Err(e) => return Err(e),
                Ok(ch) => match ch {
                    None => break,
                    Some(ch) => match ch {
                        b'\x00' | b' ' | b'\t' | b'\r' | b'\n' | b'\x08' | b'\x0C' => break,
                        _ => word.push(ch.into()),
                    },
                },
            }
        }

        Ok(Token::Name(word))
    }
}

#[cfg(test)]
mod tests {
    use std::str;

    use super::*;

    #[test]
    fn test_comment() {
        let mut scanner = Scanner::new("% this is a comment");
        let token = scanner.read_token();

        assert!(token.is_err_and(|e| e.kind() == ErrorKind::UnexpectedEof));
    }

    #[test]
    fn test_bad_numeric() -> Result<(), Error> {
        let inputs = ["1x0", "1.x0"];
        for input in inputs {
            let mut scanner = Scanner::new(input);
            let token = scanner.read_token()?;

            assert_eq!(Token::Name(String::from(input)), token);
        }

        Ok(())
    }

    #[test]
    fn test_numeric() -> Result<(), Error> {
        let cases = [
            ("1", Token::Integer(1)),
            ("-1", Token::Integer(-1)),
            ("1234567890", Token::Integer(1_234_567_890)),
            (".1", Token::Real(0.1)),
            ("-.1", Token::Real(-0.1)),
            ("1.234567890", Token::Real(1.23456789)),
            ("1.2E7", Token::Real(12_000_000.0)),
            ("1.2e7", Token::Real(12_000_000.0)),
            ("-1.2e7", Token::Real(-12_000_000.0)),
            ("1.2e-7", Token::Real(0.00000012)),
            ("-1.2e-7", Token::Real(-0.00000012)),
            ("2#1000", Token::Integer(0b1000)),
            ("8#1777", Token::Integer(0o1777)),
            ("16#fffe", Token::Integer(0xFFFE)),
            ("16#FFFE", Token::Integer(0xFFFE)),
            ("16#ffFE", Token::Integer(0xFFFE)),
        ];

        for (input, expect) in cases {
            let mut scanner = Scanner::new(input);
            let token = scanner.read_token()?;

            assert_eq!(expect, token);
        }

        Ok(())
    }

    #[test]
    fn test_bad_string() {
        let inputs = ["(this is a string", "(this is a string\\)"];
        for input in inputs {
            let mut scanner = Scanner::new(input);
            let token = scanner.read_token();

            assert!(token.is_err_and(|e| e.kind() == ErrorKind::UnterminatedString));
        }
    }

    #[test]
    fn test_string() -> Result<(), Error> {
        let cases = [
            ("(this is a string)", "this is a string"),
            (
                "(this is a multiline\nstring)",
                "this is a multiline\nstring",
            ),
            (
                "(this is a multiline\r\nstring)",
                "this is a multiline\r\nstring",
            ),
            (
                "(this has (nested) parenthesis)",
                "this has (nested) parenthesis",
            ),
        ];

        for (input, expect) in cases {
            let mut scanner = Scanner::new(input);
            let token = scanner.read_token()?;

            assert_eq!(Token::String(String::from(expect)), token);
        }

        Ok(())
    }

    #[test]
    fn test_escaped_string() -> Result<(), Error> {
        let cases = [
            ("()", ""),
            ("(\\n)", "\n"),
            ("(\\r)", "\r"),
            ("(\\t)", "\t"),
            ("(\\b)", "\x08"),
            ("(\\f)", "\x0C"),
            ("(\\\\)", "\\"),
            ("(\\()", "("),
            ("(\\))", ")"),
            ("(\\\n)", ""),
            ("(\\\r)", ""),
            ("(\\\r\n)", ""),
        ];

        for (input, expect) in cases {
            let mut scanner = Scanner::new(input);
            let token = scanner.read_token()?;

            assert_eq!(Token::String(String::from(expect)), token);
        }

        Ok(())
    }

    #[test]
    fn test_escaped_string_ignore() -> Result<(), Error> {
        let mut scanner = Scanner::new("(\\ii)");
        let token = scanner.read_token()?;

        assert_eq!(Token::String(String::from("ii")), token);
        Ok(())
    }

    #[test]
    fn test_octal_string() -> Result<(), Error> {
        let cases = [("(\\000)", 0), ("(\\377)", 255)];

        for (input, expect) in cases {
            let mut scanner = Scanner::new(input);
            let token = scanner.read_token()?;

            let expect = unsafe { str::from_boxed_utf8_unchecked(Box::new([expect])) };

            assert_eq!(Token::String(expect.to_string()), token);
        }

        Ok(())
    }

    #[test]
    fn test_hex_string() -> Result<(), Error> {
        let cases = [
            ("<0>", b"\x00".to_vec()),
            ("<FFFFFFFF>", b"\xFF\xFF\xFF\xFF".to_vec()),
            ("<ffffffff>", b"\xFF\xFF\xFF\xFF".to_vec()),
            ("<ffffFFFF>", b"\xFF\xFF\xFF\xFF".to_vec()),
            ("<901fa>", b"\x90\x1F\xA0".to_vec()),
            ("<901fa3>", b"\x90\x1F\xA3".to_vec()),
        ];

        for (input, expect) in cases {
            let mut scanner = Scanner::new(input);
            let token = scanner.read_token()?;

            let expect = unsafe { str::from_boxed_utf8_unchecked(expect.into()) };

            assert_eq!(Token::String(expect.to_string()), token);
        }

        Ok(())
    }

    #[test]
    fn test_base85_string() -> Result<(), Error> {
        let input = "FD,B0+DGm>F)Po,+EV1>F8";
        let mut scanner = Scanner::new(input);
        let token = scanner.read_token()?;

        assert_eq!(Token::String(String::from("this is some text")), token);

        Ok(())
    }

    #[test]
    fn test_multiple_string() -> Result<(), Error> {
        let input = "(this is a literal string) <7468697320697320612068657820737472696E67> <~FD,B0+DGm>@3B#fF(I<g+EMXFBl7P~>";
        let mut scanner = Scanner::new(input);

        let token = scanner.read_token()?;
        assert_eq!(
            Token::String(String::from("this is a literal string")),
            token
        );

        let token = scanner.read_token()?;
        assert_eq!(Token::String(String::from("this is a hex string")), token);

        let token = scanner.read_token()?;
        assert_eq!(
            Token::String(String::from("this is a base85 string")),
            token
        );

        Ok(())
    }

    #[test]
    fn test_name() -> Result<(), Error> {
        let inputs = [
            "abc", "Offset", "$$", "23A", "13-456", "a.b", "$MyDict", "@pattern",
        ];

        for input in inputs {
            let mut scanner = Scanner::new(input);
            let token = scanner.read_token()?;

            assert_eq!(Token::Name(input.to_string()), token);
        }

        Ok(())
    }

    #[test]
    fn test_all() -> Result<(), Error> {
        let input = "
myStr (i have a string right here)
myOtherStr (and
another \
right \
here)
% this is a comment
myInt 1234567890
myNegativeInt -1234567890
myReal 3.1456
myNegativeReal -3.1456
        ";

        let expected_tokens = [
            Token::Name("myStr".to_string()),
            Token::String("i have a string right here".to_string()),
            Token::Name("myOtherStr".to_string()),
            Token::String("and\nanother right here".to_string()),
            Token::Name("myInt".to_string()),
            Token::Integer(1234567890),
            Token::Name("myNegativeInt".to_string()),
            Token::Integer(-1234567890),
            Token::Name("myReal".to_string()),
            Token::Real(3.1456),
            Token::Name("myNegativeReal".to_string()),
            Token::Real(-3.1456),
        ];

        let mut scanner = Scanner::new(input);
        for expect in expected_tokens {
            let received = scanner.read_token()?;

            assert_eq!(expect, received);
        }

        Ok(())
    }
}
