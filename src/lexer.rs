use std::{
    cell::RefCell,
    io::{Read, Seek, SeekFrom},
    rc::Rc,
    string,
};

use crate::{
    array::ArrayObject,
    encoding::{decode_ascii85, decode_hex},
    file::FileObject,
    name::NameObject,
    object::{Access, Mode, Object},
    string::StringObject,
    Error, ErrorKind,
};

const FORM_FEED: u8 = b'\x0C';
const BACKSPACE: u8 = b'\x08';

pub(crate) struct Lexer {
    input: FileObject,
}

impl Iterator for Lexer {
    type Item = crate::Result<Object>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.next_is_whitespace() {
                self.next_char()?;
                continue;
            }

            let ch = self.peek_char()?;
            return match ch {
                b'%' => match self.lex_comment() {
                    Ok(_) => continue,
                    Err(e) => Some(Err(e)),
                },
                b'-' | b'.' | b'0'..=b'9' => Some(self.lex_numeric()),
                b'(' => Some(self.lex_string_literal()),
                b'<' => Some(self.lex_gt()),
                b'{' => Some(self.lex_procedure()),
                _ => {
                    let name = String::new();
                    Some(self.lex_name(name))
                }
            };
        }
    }
}

impl Lexer {
    pub fn new<F: Into<FileObject>>(input: F) -> Self {
        Self {
            input: input.into(),
        }
    }

    fn lex_comment(&mut self) -> crate::Result<()> {
        self.expect_char(b'%')?;

        loop {
            match self.next_char() {
                None => break,
                Some(ch) => match ch {
                    b'\n' | FORM_FEED => break,
                    _ => {}
                },
            }
        }

        Ok(())
    }

    fn lex_gt(&mut self) -> crate::Result<Object> {
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
                Ok(Object::Name(NameObject::from("<<")))
            }
            b'~' => self.lex_string_base85(),
            b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F' => self.lex_string_hex(),
            _ => self.lex_name("<".to_string()),
        }
    }

    fn lex_name(&mut self, mut name: string::String) -> crate::Result<Object> {
        loop {
            if self.next_is_whitespace() {
                break;
            }

            if self.next_is_delimiter() && !name.is_empty() && name != "<" && name != ">" {
                break;
            }

            let first_ch = name.bytes().nth(0).unwrap_or(b'\0');

            let lexing_delim =
                name == "<<" || name == ">>" || (name.len() == 1 && is_delimiter(first_ch));

            let lexing_literal = first_ch == b'/';

            if self.next_is_regular() && lexing_delim && !lexing_literal {
                break;
            }

            match self.next_char() {
                Some(ch) => name.push(ch as char),
                None => break,
            }
        }

        if name.starts_with('/') {
            name.remove(0);
            Ok(Object::Name(NameObject::new(&name, Mode::Literal)))
        } else {
            Ok(Object::Name(NameObject::new(&name, Mode::Executable)))
        }
    }

    fn lex_numeric(&mut self) -> crate::Result<Object> {
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
                }
            }
        }

        let is_radix = numeric.contains('#');
        let is_scientific = numeric.contains('E');

        if is_radix {
            let mut parts = numeric.split('#');
            return match (parts.next(), parts.next()) {
                (Some(base), Some(digits)) => match base.parse::<u32>() {
                    Ok(base) => match i32::from_str_radix(digits, base) {
                        Ok(value) => Ok(Object::Integer(value)),
                        Err(_) => self.lex_name(numeric),
                    },
                    Err(_) => self.lex_name(numeric),
                },
                _ => self.lex_name(numeric),
            };
        }

        if is_scientific {
            let mut parts = numeric.split('E');
            return match (parts.next(), parts.next()) {
                (Some(decimal), Some(exponent)) => {
                    match (decimal.parse::<f32>(), exponent.parse::<i32>()) {
                        (Ok(decimal), Ok(exponent)) => {
                            let value = decimal * 10.0_f32.powi(exponent);
                            Ok(Object::Real(value))
                        }
                        _ => self.lex_name(numeric),
                    }
                }
                _ => self.lex_name(numeric),
            };
        }

        match numeric.parse::<i32>() {
            Ok(i) => Ok(Object::Integer(i)),
            Err(_) => match numeric.parse::<f32>() {
                Ok(r) => Ok(Object::Real(r)),
                Err(_) => self.lex_name(numeric),
            },
        }
    }

    fn lex_procedure(&mut self) -> crate::Result<Object> {
        self.expect_char(b'{')?;

        let mut objs = Vec::new();

        loop {
            let obj = self
                .next()
                .ok_or(Error::new(ErrorKind::SyntaxError, "unterminated procedure"))??;

            if let Object::Name(ref n) = obj {
                if n == "}" {
                    break;
                }
            }

            objs.push(obj);
        }

        let arr = ArrayObject::new(objs, Access::ExecuteOnly, Mode::Executable);

        Ok(Object::Array(Rc::new(RefCell::new(arr))))
    }

    fn lex_string_base85(&mut self) -> crate::Result<Object> {
        let mut string = string::String::new();

        loop {
            match self.next_char() {
                None => {
                    return Err(Error::new(
                        ErrorKind::SyntaxError,
                        "unterminated base85 string",
                    ))
                }
                Some(b'~') => match self.peek_char() {
                    None => {
                        return Err(Error::new(
                            ErrorKind::SyntaxError,
                            "unterminated base85 string",
                        ))
                    }
                    Some(b'>') => break,
                    _ => continue,
                },
                Some(ch) => string.push(ch as char),
            }
        }

        let string = StringObject::new(decode_ascii85(&string)?, Mode::Literal);

        Ok(Object::String(Rc::new(RefCell::new(string))))
    }

    fn lex_string_hex(&mut self) -> crate::Result<Object> {
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

        let string = StringObject::new(decode_hex(&string)?, Mode::Literal);

        Ok(Object::String(Rc::new(RefCell::new(string))))
    }

    fn lex_string_literal(&mut self) -> crate::Result<Object> {
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
                }
                b')' => {
                    if active_parenthesis < 1 {
                        break;
                    }
                    string.push(')');
                    active_parenthesis -= 1;
                }
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
                        b'\r' => match self.peek_char() {
                            None => {
                                return Err(Error::new(
                                    ErrorKind::SyntaxError,
                                    "unterminated string",
                                ))
                            }
                            Some(b'\n') => {
                                let _ = self.next_char();
                            }
                            _ => {}
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
                        }
                        _ => string.push(next_ch as char),
                    }
                }
                _ => string.push(ch as char),
            }
        }

        let string = StringObject::new(string, Mode::Literal);

        Ok(Object::String(Rc::new(RefCell::new(string))))
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
        self.peek_char().is_some_and(|ch| is_delimiter(ch))
    }

    fn next_is_regular(&mut self) -> bool {
        self.peek_char().is_some_and(|ch| is_regular(ch))
    }

    fn next_is_whitespace(&mut self) -> bool {
        self.peek_char().is_some_and(|ch| is_whitespace(ch))
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
