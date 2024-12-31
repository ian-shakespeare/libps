use std::{io, str};

use crate::{encoding::decode_ascii85, token::Token, traits::StringReadPeeker, Error, ErrorKind};

pub struct Scanner<'a> {
    input: Box<dyn crate::PeekRead + 'a>,
}

impl<'a> Scanner<'a> {
    pub fn new(_input: &'a str) -> Self {
        Scanner {
            input: Box::new(StringReadPeeker::from(_input)),
        }
    }

    pub fn read_token(&mut self) -> crate::Result<Token> {
        let mut word = String::new();
        let mut buf = [b'\x00'];
        loop {
            match self.input.read(&mut buf) {
                Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                    return Err(Error::from(ErrorKind::UnexpectedEof))
                }
                Err(e) => return Err(Error::new(ErrorKind::Unknown, Box::new(e))),
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
                    return self.read_string_literal();
                }
                b'<' => {
                    return match self.next_byte() {
                        Err(e) => Err(e),
                        Ok(next_ch) => match next_ch {
                            None => Err(Error::from(ErrorKind::UnterminatedString)),
                            Some(b'~') => self.read_string_base85(),
                            Some(next_ch) => {
                                word.push(next_ch.into());
                                self.read_string_hex(word)
                            }
                        },
                    }
                }
                b'%' => self.read_comment(),
                _ => {
                    word.push(buf[0].into());
                    return self.read_name(word);
                }
            }?;
        }
    }

    fn next_byte(&mut self) -> crate::Result<Option<u8>> {
        let mut buf = [0];
        match self.input.read(&mut buf) {
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => Ok(None),
            Err(e) => Err(Error::new(ErrorKind::Unknown, Box::new(e))),
            _ => Ok(Some(buf[0])),
        }
    }

    fn peek_next_byte(&self) -> crate::Result<Option<u8>> {
        let mut buf = [0];
        match self.input.peek(&mut buf) {
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => Ok(None),
            Err(e) => Err(Error::new(ErrorKind::Unknown, Box::new(e))),
            _ => Ok(Some(buf[0])),
        }
    }

    fn read_comment(&mut self) -> crate::Result<()> {
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

    fn read_numeric(&mut self, mut word: String) -> crate::Result<Token> {
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
                                return Err(Error::from(ErrorKind::Syntax));
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
            Err(Error::from(ErrorKind::Syntax))
        }
    }

    fn read_real(&mut self, mut word: String) -> crate::Result<Token> {
        loop {
            match self.next_byte() {
                Err(e) => return Err(e),
                Ok(ch) => match ch {
                    None => break,
                    Some(ch) => match ch {
                        b'\x00' | b' ' | b'\t' | b'\r' | b'\n' | b'\x08' | b'\x0C' => {
                            if word.to_lowercase().ends_with('e') {
                                return Err(Error::from(ErrorKind::Syntax));
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
                return Err(Error::from(ErrorKind::Syntax));
            }
            return Err(Error::from(ErrorKind::Syntax));
        }

        if let Ok(value) = word.parse::<f64>() {
            Ok(Token::Real(value))
        } else {
            Err(Error::from(ErrorKind::Syntax))
        }
    }

    fn read_radix(&mut self, mut word: String) -> crate::Result<Token> {
        loop {
            match self.next_byte() {
                Err(e) => return Err(e),
                Ok(ch) => match ch {
                    None => break,
                    Some(ch) => match ch {
                        b'\x00' | b' ' | b'\t' | b'\r' | b'\n' | b'\x08' | b'\x0C' => {
                            if word.ends_with('#') {
                                return Err(Error::from(ErrorKind::Syntax));
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
                Err(Error::from(ErrorKind::Syntax))
            }
        } else {
            Err(Error::from(ErrorKind::Syntax))
        }
    }

    fn read_string_literal(&mut self) -> crate::Result<Token> {
        let mut word = String::new();
        let mut active_parenthesis = 0;

        loop {
            match self.next_byte() {
                Err(e) => return Err(e),
                Ok(ch) => match ch {
                    None => return Err(Error::from(ErrorKind::UnterminatedString)),
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
                                    None => Err(Error::from(ErrorKind::UnexpectedEof)),
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
                                b'\r' => match self.peek_next_byte() {
                                    Err(e) => return Err(e),
                                    Ok(next_ch) => match next_ch {
                                        None => {
                                            return Err(Error::from(ErrorKind::UnterminatedString))
                                        }
                                        Some(b'\n') => {
                                            let _ = self.next_byte()?;
                                        }
                                        _ => {}
                                    },
                                },
                                b'0'..=b'9' => {
                                    let mut octal = String::new();
                                    octal.push(next_ch.into());

                                    let mut next_digits = [0, 0];
                                    let _ = self.input.peek(&mut next_digits)?;
                                    match str::from_utf8(&next_digits) {
                                        Err(_) => return Err(Error::from(ErrorKind::Syntax)),
                                        Ok(next_digits) => {
                                            octal.push_str(next_digits);
                                        }
                                    }

                                    match u8::from_str_radix(&octal, 8) {
                                        Err(_) => return Err(Error::from(ErrorKind::Syntax)),
                                        Ok(value) => word.push(value.into()),
                                    }

                                    for _ in 0..2 {
                                        let _ = self.next_byte()?;
                                    }
                                }
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

    fn read_string_hex(&mut self, mut word: String) -> crate::Result<Token> {
        loop {
            match self.next_byte() {
                Err(e) => return Err(e),
                Ok(ch) => match ch {
                    None => return Err(Error::from(ErrorKind::UnterminatedString)),
                    Some(ch) => match ch {
                        b'>' => break,
                        b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z' => word.push(ch.into()),
                        _ => return Err(Error::new(ErrorKind::Syntax, "invalid hex string")),
                    },
                },
            }
        }

        // TODO: Actually decode the hex
        Ok(Token::String(word))
    }

    fn read_string_base85(&mut self) -> crate::Result<Token> {
        let mut word = String::new();
        loop {
            match self.next_byte() {
                Err(e) => return Err(e),
                Ok(ch) => match ch {
                    None => return Err(Error::from(ErrorKind::UnterminatedString)),
                    Some(ch) => match ch {
                        b'~' => match self.peek_next_byte() {
                            Err(e) => return Err(e),
                            Ok(next_ch) => match next_ch {
                                None => return Err(Error::from(ErrorKind::UnterminatedString)),
                                Some(b'>') => break,
                                _ => continue,
                            },
                        },
                        _ => word.push(ch.into()),
                    },
                },
            }
        }

        Ok(Token::String(decode_ascii85(&word)?))
    }

    fn read_name(&mut self, mut word: String) -> crate::Result<Token> {
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
    use std::{error, str};

    use super::*;

    #[test]
    fn test_comment() {
        let mut scanner = Scanner::new("% this is a comment");
        let token = scanner.read_token();

        assert!(token.is_err_and(|e| e.kind() == crate::ErrorKind::UnexpectedEof));
    }

    #[test]
    fn test_bad_numeric() -> Result<(), Box<dyn error::Error>> {
        let inputs = ["1x0", "1.x0"];
        for input in inputs {
            let mut scanner = Scanner::new(input);
            let token = scanner.read_token()?;

            assert_eq!(Token::Name(String::from(input)), token);
        }

        Ok(())
    }

    #[test]
    fn test_numeric() -> Result<(), Box<dyn error::Error>> {
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
    fn test_string() -> Result<(), Box<dyn error::Error>> {
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
    fn test_escaped_string() -> Result<(), Box<dyn error::Error>> {
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
    fn test_ignore_escaped_string() -> Result<(), Box<dyn error::Error>> {
        let mut scanner = Scanner::new("(\\ii)");
        let token = scanner.read_token()?;

        assert_eq!(Token::String(String::from("ii")), token);
        Ok(())
    }

    #[test]
    fn test_octal_string() -> Result<(), Box<dyn error::Error>> {
        let cases = [("(\\000)", "\0"), ("(\\377)", "ÿ")];

        for (input, expect) in cases {
            let mut scanner = Scanner::new(input);
            let token = scanner.read_token()?;

            assert_eq!(Token::String(expect.to_string()), token);
        }

        Ok(())
    }

    #[test]
    fn test_hex_string() -> Result<(), Box<dyn error::Error>> {
        let cases = [
            ("<0>", vec![b'\x00']),
            ("<FFFFFFFF>", vec![b'\xFF', b'\xFF', b'\xFF', b'\xFF']),
            ("<ffffffff>", vec![b'\xFF', b'\xFF', b'\xFF', b'\xFF']),
            ("<ffffFFFF>", vec![b'\xFF', b'\xFF', b'\xFF', b'\xFF']),
            ("<901fa>", vec![b'\x90', b'\x1F', b'\xA0']),
            ("<901fa3>", vec![b'\x90', b'\x1F', b'\xA3']),
        ];

        for (input, expect) in cases {
            let mut scanner = Scanner::new(input);
            let token = scanner.read_token()?;

            let expect = str::from_utf8(&expect)?;

            assert_eq!(Token::String(expect.to_string()), token);
        }

        Ok(())
    }

    #[test]
    fn test_base85_string() -> Result<(), Box<dyn error::Error>> {
        let input = "<~FD,B0+DGm>F)Po,+EV1>F8~>";
        let mut scanner = Scanner::new(input);
        let token = scanner.read_token()?;

        assert_eq!(Token::String(String::from("this is some text")), token);

        Ok(())
    }

    #[test]
    fn test_multiple_string() -> Result<(), Box<dyn error::Error>> {
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
    fn test_name() -> Result<(), Box<dyn error::Error>> {
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
    fn test_all() -> Result<(), Box<dyn error::Error>> {
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
