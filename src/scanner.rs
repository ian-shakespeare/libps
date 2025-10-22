use std::{
    io::{Read, Seek, SeekFrom},
    string,
};

use crate::{
    encoding::{decode_ascii85, decode_hex},
    file::FileObject,
    Error, ErrorKind,
};

const FORM_FEED: u8 = b'\x0C';
const BACKSPACE: u8 = b'\x08';

pub enum Token {
    Integer(i32),
    Real(f32),
    String(Vec<u8>),
    Name(String),
    LiteralName(String),
    Procedure(Vec<Token>),
}

pub(crate) struct Scanner {
    input: FileObject,
}

impl Iterator for Scanner {
    type Item = crate::Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.next_is_whitespace() {
                self.next_char()?;
                continue;
            }

            let ch = self.peek_char()?;
            return match ch {
                b'%' => {
                    match self.scan_comment() {
                        Ok(_) => continue,
                        Err(e) => Some(Err(e)),
                    }
                },
                b'-' | b'.' | b'0'..=b'9' => Some(self.scan_numeric()),
                b'(' => Some(self.scan_string_literal()),
                b'<' => Some(self.scan_gt()),
                b'{' => Some(self.scan_procedure()),
                _ => {
                    let name = String::new();
                    Some(self.scan_name(name))
                },
            };
        }
    }
}

impl Scanner {
    pub fn new<F: Into<FileObject>>(input: F) -> Self {
        Self {
            input: input.into(),
        }
    }

    fn scan_comment(&mut self) -> crate::Result<()> {
        self.expect_char(b'%')?;

        loop {
            match self.next_char() {
                None | Some(b'\n' | FORM_FEED) => break,
                _ => {},
            }
        }

        Ok(())
    }

    fn scan_gt(&mut self) -> crate::Result<Token> {
        self.expect_char(b'<')?;

        let Some(ch) = self.peek_char() else {
            return Err(Error::new(
                ErrorKind::SyntaxError,
                "unterminated hex string",
            ));
        };

        match ch {
            b'<' => {
                let _ = self.next_char();
                Ok(Token::Name(String::from("<<")))
            },
            b'~' => self.scan_string_base85(),
            b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F' => self.scan_string_hex(),
            _ => self.scan_name("<".to_string()),
        }
    }

    fn scan_name(&mut self, mut name: string::String) -> crate::Result<Token> {
        loop {
            if self.next_is_whitespace() {
                break;
            }

            if self.next_is_delimiter() && !name.is_empty() && name != "<" && name != ">" {
                break;
            }

            let first_ch = name.as_bytes().first().copied().unwrap_or(b'\0');

            let scaning_delim =
                name == "<<" || name == ">>" || (name.len() == 1 && is_delimiter(first_ch));

            let scaning_literal = first_ch == b'/';

            if self.next_is_regular() && scaning_delim && !scaning_literal {
                break;
            }

            match self.next_char() {
                Some(ch) => name.push(ch as char),
                None => break,
            }
        }

        if name.starts_with('/') {
            name.remove(0);
            Ok(Token::LiteralName(name))
        } else {
            Ok(Token::Name(name))
        }
    }

    fn scan_numeric(&mut self) -> crate::Result<Token> {
        let mut numeric = string::String::new();

        loop {
            if !self.next_is_regular() {
                break;
            }

            let Some(ch) = self.next_char() else {
                break;
            };

            match ch {
                b'e' | b'E' => numeric.push('E'),
                _ => {
                    numeric.push(ch as char);
                },
            }
        }

        let is_radix = numeric.contains('#');
        let is_scientific = numeric.contains('E');

        if is_radix {
            let mut parts = numeric.split('#');
            return match (parts.next(), parts.next()) {
                (Some(base), Some(digits)) => {
                    match base.parse::<u32>() {
                        Ok(base) => {
                            match i32::from_str_radix(digits, base) {
                                Ok(value) => Ok(Token::Integer(value)),
                                Err(_) => self.scan_name(numeric),
                            }
                        },
                        Err(_) => self.scan_name(numeric),
                    }
                },
                _ => self.scan_name(numeric),
            };
        }

        if is_scientific {
            let mut parts = numeric.split('E');
            return match (parts.next(), parts.next()) {
                (Some(decimal), Some(exponent)) => {
                    match (decimal.parse::<f32>(), exponent.parse::<i32>()) {
                        (Ok(decimal), Ok(exponent)) => {
                            let value = decimal * 10.0_f32.powi(exponent);
                            Ok(Token::Real(value))
                        },
                        _ => self.scan_name(numeric),
                    }
                },
                _ => self.scan_name(numeric),
            };
        }

        match numeric.parse::<i32>() {
            Ok(i) => Ok(Token::Integer(i)),
            Err(_) => {
                match numeric.parse::<f32>() {
                    Ok(r) => Ok(Token::Real(r)),
                    Err(_) => self.scan_name(numeric),
                }
            },
        }
    }

    fn scan_procedure(&mut self) -> crate::Result<Token> {
        self.expect_char(b'{')?;

        let mut objs = Vec::new();

        loop {
            let obj = self
                .next()
                .ok_or(Error::new(ErrorKind::SyntaxError, "unterminated procedure"))??;

            if let Token::Name(ref n) = obj {
                if n == "}" {
                    break;
                }
            }

            objs.push(obj);
        }

        Ok(Token::Procedure(objs))
    }

    fn scan_string_base85(&mut self) -> crate::Result<Token> {
        let mut string = string::String::new();

        loop {
            match self.next_char() {
                None => {
                    return Err(Error::new(
                        ErrorKind::SyntaxError,
                        "unterminated base85 string",
                    ))
                },
                Some(b'~') => {
                    match self.peek_char() {
                        None => {
                            return Err(Error::new(
                                ErrorKind::SyntaxError,
                                "unterminated base85 string",
                            ))
                        },
                        Some(b'>') => break,
                        _ => continue,
                    }
                },
                Some(ch) => string.push(ch as char),
            }
        }

        Ok(Token::String(decode_ascii85(&string)?.into()))
    }

    fn scan_string_hex(&mut self) -> crate::Result<Token> {
        let mut string = String::new();

        loop {
            if self.next_is_whitespace() {
                let _ = self.next_char();
                continue;
            }

            let Some(ch) = self.next_char() else {
                return Err(Error::new(ErrorKind::SyntaxError, "unterminated string"));
            };

            match ch {
                b'>' => break,
                b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z' => string.push(ch as char),
                _ => return Err(Error::new(ErrorKind::SyntaxError, "invalid hex string")),
            }
        }

        Ok(Token::String(decode_hex(&string)?.into()))
    }

    fn scan_string_literal(&mut self) -> crate::Result<Token> {
        self.expect_char(b'(')?;

        let mut string = String::new();
        let mut active_parenthesis = 0;

        loop {
            let Some(ch) = self.next_char() else {
                return Err(Error::new(ErrorKind::SyntaxError, "unterminated string"));
            };

            match ch {
                b'(' => {
                    string.push('(');
                    active_parenthesis += 1;
                },
                b')' => {
                    if active_parenthesis < 1 {
                        break;
                    }
                    string.push(')');
                    active_parenthesis -= 1;
                },
                b'\\' => {
                    let next_ch = match self.next_char() {
                        None => Err(Error::new(ErrorKind::IoError, "unexpected eof")),
                        Some(next_ch) => Ok(next_ch),
                    }?;
                    match next_ch {
                        b'\n' => continue,
                        b'r' => string.push('\r'),
                        b'n' => string.push('\n'),
                        b't' => string.push('\t'),
                        b'b' => string.push(BACKSPACE as char),
                        b'f' => string.push(FORM_FEED as char),
                        b'\\' => string.push('\\'),
                        b'(' => string.push('('),
                        b')' => string.push(')'),
                        b'\r' => {
                            match self.peek_char() {
                                None => {
                                    return Err(Error::new(
                                        ErrorKind::SyntaxError,
                                        "unterminated string",
                                    ))
                                },
                                Some(b'\n') => {
                                    let _ = self.next_char();
                                },
                                _ => {},
                            }
                        },
                        b'0'..=b'9' => {
                            let mut octal: u8 = 0;
                            octal |= next_ch << 6;

                            let ch1 = self.next_char();
                            let ch2 = self.next_char();
                            let _ = self.input.seek(SeekFrom::Current(-2));

                            if let (Some(ch1), Some(ch2)) = (ch1, ch2) {
                                octal |= ch1 << 3;
                                octal |= ch2;
                            } else {
                                return Err(Error::new(
                                    ErrorKind::SyntaxError,
                                    "invalid octal number",
                                ));
                            }

                            string.push(octal.into());
                        },
                        _ => string.push(next_ch as char),
                    }
                },
                _ => string.push(ch as char),
            }
        }

        Ok(Token::String(string.into()))
    }

    fn expect_char(&mut self, ch: u8) -> crate::Result<()> {
        match self.next_char() {
            Some(received) if ch == received => Ok(()),
            _ => Err(Error::new(ErrorKind::SyntaxError, format!("expected {ch}"))),
        }
    }

    fn next_char(&mut self) -> Option<u8> {
        let mut buf: [u8; 1] = [0];
        match self.input.read(&mut buf) {
            Ok(1) => Some(buf[0]),
            _ => None,
        }
    }

    fn peek_char(&mut self) -> Option<u8> {
        if let Some(ch) = self.next_char() {
            let _ = self.input.seek(SeekFrom::Current(-1));
            return Some(ch);
        }

        None
    }

    fn next_is_delimiter(&mut self) -> bool {
        self.peek_char().is_some_and(is_delimiter)
    }

    fn next_is_regular(&mut self) -> bool {
        self.peek_char().is_some_and(is_regular)
    }

    fn next_is_whitespace(&mut self) -> bool {
        self.peek_char().is_some_and(is_whitespace)
    }
}

fn is_delimiter(ch: u8) -> bool {
    matches!(
        ch,
        b'<' | b'>' | b'(' | b')' | b'[' | b']' | b'{' | b'}' | b'/' | b'%'
    )
}

fn is_regular(ch: u8) -> bool {
    !is_delimiter(ch) && !is_whitespace(ch)
}

fn is_whitespace(ch: u8) -> bool {
    matches!(
        ch,
        b'\0' | b' ' | b'\t' | b'\r' | b'\n' | BACKSPACE | FORM_FEED
    )
}
