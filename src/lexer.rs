use std::iter;

use crate::{
    encoding::{decode_ascii85, decode_hex},
    object::{Access, Composite, Container, Object},
    Error, ErrorKind,
};

const FORM_FEED: char = '\x0C';
const BACKSPACE: char = '\x08';

pub struct Lexer<I: Iterator<Item = char>> {
    input: iter::Peekable<I>,
}

impl<I> From<I> for Lexer<I>
where
    I: Iterator<Item = char>,
{
    fn from(value: I) -> Self {
        Self {
            input: value.peekable(),
        }
    }
}

impl<I> Lexer<I>
where
    I: Iterator<Item = char>,
{
    pub fn next_obj(
        &mut self,
        string_container: &mut Container<Composite<String>>,
    ) -> Option<crate::Result<Object>> {
        loop {
            if self.next_is_whitespace() {
                self.input.next()?;
                continue;
            }

            let ch = self.input.peek()?;

            match ch {
                '%' => {
                    if let Err(e) = self.lex_comment() {
                        return Some(Err(e));
                    }
                }
                '-' | '.' | '0'..='9' => {
                    return Some(self.lex_numeric());
                }
                '(' => return Some(self.lex_string_literal(string_container)),
                '<' => return Some(self.lex_gt(string_container)),
                _ => {
                    let name = String::from(self.input.next()?);
                    return Some(self.lex_name(name));
                }
            };
        }
    }

    fn lex_comment(&mut self) -> crate::Result<()> {
        self.expect_char('%')?;

        loop {
            match self.input.next() {
                None => break,
                Some(ch) => match ch {
                    '\n' | FORM_FEED => break,
                    _ => {}
                },
            }
        }

        Ok(())
    }

    fn lex_gt(
        &mut self,
        string_container: &mut Container<Composite<String>>,
    ) -> crate::Result<Object> {
        self.expect_char('<')?;

        let Some(ch) = self.input.peek() else {
            return Err(Error::new(ErrorKind::Syntax, "unterminated hex string"));
        };

        match ch {
            '<' => Ok(Object::Name("<<".to_string())),
            '~' => self.lex_string_base85(string_container),
            '0'..='9' | 'a'..='f' | 'A'..='F' => self.lex_string_hex(string_container),
            _ => self.lex_name("<".to_string()),
        }
    }

    fn lex_name(&mut self, mut name: String) -> crate::Result<Object> {
        loop {
            if self.next_is_whitespace() {
                break;
            }

            let is_lexing_delim = name.is_empty()
                || name == "<<"
                || name == ">>"
                || (name.len() == 1
                    && is_delimiter(
                        name.chars()
                            .nth(0)
                            .expect("failed to get first character of non-empty name"),
                    ));

            let should_deliminate = (!is_lexing_delim && self.next_is_delimiter())
                || (is_lexing_delim && !self.next_is_delimiter());

            if should_deliminate {
                break;
            }

            match self.input.next() {
                Some(ch) => name.push(ch),
                None => break,
            }
        }

        Ok(match name.as_str() {
            "true" => Object::Boolean(true),
            "false" => Object::Boolean(true),
            _ => Object::Name(name),
        })
    }

    fn lex_numeric(&mut self) -> crate::Result<Object> {
        let mut numeric = String::new();

        loop {
            if !self.next_is_regular() {
                break;
            }

            let Some(ch) = self.input.next() else {
                break;
            };

            match ch {
                'e' | 'E' => numeric.push('E'),
                _ => {
                    numeric.push(ch);
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
                    match (decimal.parse::<f64>(), exponent.parse::<i32>()) {
                        (Ok(decimal), Ok(exponent)) => {
                            let value = decimal * 10.0_f64.powi(exponent);
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
            Err(_) => match numeric.parse::<f64>() {
                Ok(r) => Ok(Object::Real(r)),
                Err(_) => self.lex_name(numeric),
            },
        }
    }

    fn lex_string_base85(
        &mut self,
        container: &mut Container<Composite<String>>,
    ) -> crate::Result<Object> {
        let mut string = String::new();

        loop {
            match self.input.next() {
                None => return Err(Error::new(ErrorKind::Syntax, "unterminated base85 string")),
                Some('~') => match self.input.peek() {
                    None => {
                        return Err(Error::new(ErrorKind::Syntax, "unterminated base85 string"))
                    }
                    Some('>') => break,
                    _ => continue,
                },
                Some(ch) => string.push(ch),
            }
        }

        let inner = decode_ascii85(&string)?;
        let len = inner.len();
        let composite = Composite {
            access: Access::default(),
            inner,
            len,
        };

        let idx = container.insert(composite);

        Ok(Object::String(idx))
    }

    fn lex_string_hex(
        &mut self,
        container: &mut Container<Composite<String>>,
    ) -> crate::Result<Object> {
        let mut string = String::new();

        loop {
            if self.next_is_whitespace() {
                let _ = self.input.next();
                continue;
            }

            let Some(ch) = self.input.next() else {
                return Err(Error::new(ErrorKind::Syntax, "unterminated string"));
            };

            match ch {
                '>' => break,
                '0'..='9' | 'a'..='z' | 'A'..='Z' => string.push(ch),
                _ => return Err(Error::new(ErrorKind::Syntax, "invalid hex string")),
            }
        }

        let inner = decode_hex(&string)?;
        let len = inner.len();
        let composite = Composite {
            access: Access::default(),
            inner,
            len,
        };

        let idx = container.insert(composite);

        Ok(Object::String(idx))
    }

    fn lex_string_literal(
        &mut self,
        container: &mut Container<Composite<String>>,
    ) -> crate::Result<Object> {
        self.expect_char('(')?;

        let mut string = String::new();
        let mut active_parenthesis = 0;

        loop {
            let Some(ch) = self.input.next() else {
                return Err(Error::new(ErrorKind::Syntax, "unterminated string"));
            };

            match ch {
                '(' => {
                    string.push(ch);
                    active_parenthesis += 1;
                }
                ')' => {
                    if active_parenthesis < 1 {
                        break;
                    }
                    string.push(ch);
                    active_parenthesis -= 1;
                }
                '\\' => {
                    let next_ch = match self.input.next() {
                        None => Err(Error::new(ErrorKind::IoError, "unexpected eof")),
                        Some(next_ch) => Ok(next_ch),
                    }?;
                    match next_ch {
                        '\n' => continue,
                        'r' => string.push('\r'),
                        'n' => string.push('\n'),
                        't' => string.push('\t'),
                        'b' => string.push(BACKSPACE),
                        'f' => string.push(FORM_FEED),
                        '\\' => string.push('\\'),
                        '(' => string.push('('),
                        ')' => string.push(')'),
                        '\r' => match self.input.peek() {
                            None => {
                                return Err(Error::new(ErrorKind::Syntax, "unterminated string"))
                            }
                            Some('\n') => {
                                let _ = self.input.next();
                            }
                            _ => {}
                        },
                        '0'..='9' => {
                            match (self.input.peek().cloned(), self.input.peek().cloned()) {
                                (Some(second_digit), Some(third_digit)) => {
                                    let octal =
                                        String::from_iter([next_ch, second_digit, third_digit]);

                                    match u8::from_str_radix(&octal, 8) {
                                        Err(_) => return Err(Error::from(ErrorKind::Syntax)),
                                        Ok(value) => {
                                            string.push(value.into());
                                            let _ = self.input.next();
                                            let _ = self.input.next();
                                        }
                                    }
                                }
                                _ => return Err(Error::from(ErrorKind::Syntax)),
                            }
                        }
                        _ => string.push(next_ch),
                    }
                }
                _ => string.push(ch),
            }
        }

        let len = string.len();
        let composite = Composite {
            access: Access::default(),
            inner: string,
            len,
        };

        let idx = container.insert(composite);

        Ok(Object::String(idx))
    }

    fn expect_char(&mut self, ch: char) -> crate::Result<()> {
        match self.input.next() {
            Some(received) if ch == received => Ok(()),
            _ => Err(Error::new(ErrorKind::Syntax, format!("expected {ch}"))),
        }
    }

    fn next_is_delimiter(&mut self) -> bool {
        self.input.peek().map_or(false, |ch| is_delimiter(*ch))
    }

    fn next_is_whitespace(&mut self) -> bool {
        self.input.peek().map_or(false, |ch| is_whitespace(*ch))
    }

    fn next_is_regular(&mut self) -> bool {
        self.input.peek().map_or(false, |ch| is_regular(*ch))
    }
}

fn is_delimiter(ch: char) -> bool {
    matches!(ch, '<' | '>' | '[' | ']' | '{' | '}' | '%')
}

fn is_whitespace(ch: char) -> bool {
    matches!(ch, '\0' | ' ' | '\t' | '\r' | '\n' | BACKSPACE | FORM_FEED)
}

fn is_regular(ch: char) -> bool {
    !is_delimiter(ch) && !is_whitespace(ch)
}
