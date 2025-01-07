use std::iter;

use crate::{
    encoding::{decode_ascii85, decode_hex},
    token::Token,
    Error, ErrorKind,
};

pub struct Scanner<I: Iterator<Item = char>> {
    input: iter::Peekable<I>,
}

impl<I> From<I> for Scanner<I>
where
    I: Iterator<Item = char>,
{
    fn from(value: I) -> Self {
        Scanner {
            input: value.peekable(),
        }
    }
}

impl<I> Iterator for Scanner<I>
where
    I: Iterator<Item = char>,
{
    type Item = crate::Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut word = String::new();
        loop {
            match self.input.next() {
                None => return None,
                Some('\0' | ' ' | '\t' | '\r' | '\n' | '\x08' | '\x0C') => {
                    continue;
                }
                Some('%') => {
                    if let Err(e) = self.read_comment() {
                        return Some(Err(e));
                    }
                }
                Some(ch) => {
                    return Some(match ch {
                        '-' | '0'..='9' => {
                            word.push(ch);
                            self.read_numeric(word)
                        }
                        '.' => {
                            word.push(ch);
                            self.read_real(word)
                        }
                        '(' => self.read_string_literal(),
                        '<' => match self.input.next() {
                            None => Err(Error::from(ErrorKind::UnterminatedString)),
                            Some('~') => self.read_string_base85(),
                            Some(next_ch) => {
                                word.push(next_ch);
                                self.read_string_hex(word)
                            }
                        },
                        _ => {
                            word.push(ch);
                            self.read_name(word)
                        }
                    });
                }
            };
        }
    }
}

impl<I> Scanner<I>
where
    I: Iterator<Item = char>,
{
    fn read_comment(&mut self) -> crate::Result<()> {
        loop {
            match self.input.next() {
                None => break,
                Some(ch) => match ch {
                    '\n' | '\x0c' => break,
                    _ => {}
                },
            }
        }

        Ok(())
    }

    fn read_numeric(&mut self, mut word: String) -> crate::Result<Token> {
        loop {
            match self.input.next() {
                None => break,
                Some(ch) => match ch {
                    '\0' | ' ' | '\t' | '\r' | '\n' | '\x08' | '\x0C' => break,
                    '0'..='9' => word.push(ch),
                    '.' => {
                        word.push(ch);
                        return self.read_real(word);
                    }
                    '#' => {
                        if word.starts_with("-") {
                            return Err(Error::from(ErrorKind::Syntax));
                        }
                        word.push(ch);
                        return self.read_radix(word);
                    }
                    _ => {
                        word.push(ch);
                        return self.read_name(word);
                    }
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
        let mut is_scientific = false;
        loop {
            match self.input.next() {
                None => break,
                Some(ch) => match ch {
                    '\0' | ' ' | '\t' | '\r' | '\n' | '\x08' | '\x0C' => {
                        if word.to_lowercase().ends_with('e') {
                            return Err(Error::from(ErrorKind::Syntax));
                        }
                        break;
                    }
                    'e' | 'E' => {
                        if word.to_lowercase().contains('e') {
                            return self.read_name(word);
                        }
                        is_scientific = true;
                        word.push('e');
                    }
                    '-' => {
                        if !word.to_lowercase().ends_with('e') {
                            return self.read_name(word);
                        }
                        word.push(ch);
                    }
                    '0'..='9' => word.push(ch),
                    _ => {
                        word.push(ch);
                        return self.read_name(word);
                    }
                },
            }
        }

        if is_scientific {
            let mut parts = word.split('e');
            match (parts.next(), parts.next()) {
                (Some(decimal), Some(exponent)) => {
                    match (decimal.parse::<f64>(), exponent.parse::<i32>()) {
                        (Ok(decimal), Ok(exponent)) => {
                            let value = decimal * 10.0_f64.powi(exponent);
                            return Ok(Token::Real(value));
                        }
                        _ => Err(Error::from(ErrorKind::Syntax)),
                    }
                }
                _ => Err(Error::from(ErrorKind::Syntax)),
            }?;
        }

        if let Ok(value) = word.parse::<f64>() {
            Ok(Token::Real(value))
        } else {
            Err(Error::from(ErrorKind::Syntax))
        }
    }

    fn read_radix(&mut self, mut word: String) -> crate::Result<Token> {
        loop {
            match self.input.next() {
                None => break,
                Some(ch) => match ch {
                    '\0' | ' ' | '\t' | '\r' | '\n' | '\x08' | '\x0C' => {
                        if word.ends_with('#') {
                            return Err(Error::from(ErrorKind::Syntax));
                        }
                        break;
                    }
                    '0'..='9' | 'a'..='z' | 'A'..='Z' => word.push(ch),
                    _ => {
                        word.push(ch);
                        return self.read_name(word);
                    }
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
            match self.input.next() {
                None => return Err(Error::from(ErrorKind::UnterminatedString)),
                Some(ch) => match ch {
                    '(' => {
                        word.push(ch);
                        active_parenthesis += 1;
                    }
                    ')' => {
                        if active_parenthesis < 1 {
                            break;
                        }
                        word.push(ch);
                        active_parenthesis -= 1;
                    }
                    '\\' => {
                        let next_ch = match self.input.next() {
                            None => Err(Error::from(ErrorKind::UnexpectedEof)),
                            Some(next_ch) => Ok(next_ch),
                        }?;
                        match next_ch {
                            '\n' => continue,
                            'r' => word.push('\r'),
                            'n' => word.push('\n'),
                            't' => word.push('\t'),
                            'b' => word.push('\x08'),
                            'f' => word.push('\x0C'),
                            '\\' => word.push('\\'),
                            '(' => word.push('('),
                            ')' => word.push(')'),
                            '\r' => match self.input.peek() {
                                None => return Err(Error::from(ErrorKind::UnterminatedString)),
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
                                                word.push(value.into());
                                                let _ = self.input.next();
                                                let _ = self.input.next();
                                            }
                                        }
                                    }
                                    _ => return Err(Error::from(ErrorKind::Syntax)),
                                }
                            }
                            _ => word.push(next_ch),
                        }
                    }
                    _ => word.push(ch),
                },
            }
        }

        Ok(Token::String(word))
    }

    fn read_string_hex(&mut self, mut word: String) -> crate::Result<Token> {
        loop {
            match self.input.next() {
                None => return Err(Error::from(ErrorKind::UnterminatedString)),
                Some(ch) => match ch {
                    '>' => break,
                    '\0' | ' ' | '\t' | '\r' | '\n' | '\x08' | '\x0C' => continue,
                    '0'..='9' | 'a'..='z' | 'A'..='Z' => word.push(ch),
                    _ => return Err(Error::new(ErrorKind::Syntax, "invalid hex string")),
                },
            }
        }

        Ok(Token::String(decode_hex(&word)?))
    }

    fn read_string_base85(&mut self) -> crate::Result<Token> {
        let mut word = String::new();
        loop {
            match self.input.next() {
                None => return Err(Error::from(ErrorKind::UnterminatedString)),
                Some(ch) => match ch {
                    '~' => match self.input.peek() {
                        None => return Err(Error::from(ErrorKind::UnterminatedString)),
                        Some('>') => break,
                        _ => continue,
                    },
                    _ => word.push(ch),
                },
            }
        }

        Ok(Token::String(decode_ascii85(&word)?))
    }

    fn read_name(&mut self, mut word: String) -> crate::Result<Token> {
        loop {
            match self.input.next() {
                None => break,
                Some(ch) => match ch {
                    '\0' | ' ' | '\t' | '\r' | '\n' | '\x08' | '\x0C' => break,
                    _ => word.push(ch),
                },
            }
        }

        Ok(Token::Name(word))
    }
}

#[cfg(test)]
mod tests {
    use std::error;

    use super::*;

    #[test]
    fn test_comment() {
        let mut scanner = Scanner::from("% this is a comment".chars());
        let token = scanner.next();

        assert!(token.is_none());
    }

    #[test]
    fn test_bad_numeric() -> Result<(), Box<dyn error::Error>> {
        let inputs = ["1x0", "1.x0"];
        for input in inputs {
            let mut scanner = Scanner::from(input.chars());
            let Some(token) = scanner.next() else {
                return Err("expected token".into());
            };

            assert_eq!(Token::Name(String::from(input)), token?);
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
            let mut scanner = Scanner::from(input.chars());
            let Some(token) = scanner.next() else {
                return Err("expected token".into());
            };

            assert_eq!(expect, token?);
        }

        Ok(())
    }

    #[test]
    fn test_bad_string() -> Result<(), Box<dyn error::Error>> {
        let inputs = ["(this is a string", "(this is a string\\)"];
        for input in inputs {
            let mut scanner = Scanner::from(input.chars());
            let Some(token) = scanner.next() else {
                return Err("expected token".into());
            };

            assert!(token.is_err_and(|e| e.kind() == ErrorKind::UnterminatedString));
        }

        Ok(())
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
            let mut scanner = Scanner::from(input.chars());
            let Some(token) = scanner.next() else {
                return Err("expected token".into());
            };

            assert_eq!(Token::String(String::from(expect)), token?);
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
            let mut scanner = Scanner::from(input.chars());
            let Some(token) = scanner.next() else {
                return Err("expected token".into());
            };

            assert_eq!(Token::String(String::from(expect)), token?);
        }

        Ok(())
    }

    #[test]
    fn test_ignore_escaped_string() -> Result<(), Box<dyn error::Error>> {
        let mut scanner = Scanner::from("(\\ii)".chars());
        let Some(token) = scanner.next() else {
            return Err("expected token".into());
        };

        assert_eq!(Token::String(String::from("ii")), token?);
        Ok(())
    }

    #[test]
    fn test_octal_string() -> Result<(), Box<dyn error::Error>> {
        let cases = [("(\\000)", "\0"), ("(\\377)", "ÿ")];

        for (input, expect) in cases {
            let mut scanner = Scanner::from(input.chars());
            let Some(token) = scanner.next() else {
                return Err("expected token".into());
            };

            assert_eq!(Token::String(expect.to_string()), token?);
        }

        Ok(())
    }

    #[test]
    fn test_hex_string() -> Result<(), Box<dyn error::Error>> {
        let cases = [
            ("<736F6D65>", "some"),
            ("<736f6d65>", "some"),
            ("<736f6D65>", "some"),
            ("<73 6F 6D 65>", "some"),
            ("<70756D7>", "pump"),
            ("<70756D70>", "pump"),
        ];

        for (input, expect) in cases {
            let mut scanner = Scanner::from(input.chars());
            let Some(token) = scanner.next() else {
                return Err("expected token".into());
            };

            assert_eq!(Token::String(expect.to_string()), token?);
        }

        Ok(())
    }

    #[test]
    fn test_base85_string() -> Result<(), Box<dyn error::Error>> {
        let input = "<~FD,B0+DGm>F)Po,+EV1>F8~>";
        let mut scanner = Scanner::from(input.chars());
        let Some(token) = scanner.next() else {
            return Err("expected token".into());
        };

        assert_eq!(Token::String(String::from("this is some text")), token?);

        Ok(())
    }

    #[test]
    fn test_multiple_string() -> Result<(), Box<dyn error::Error>> {
        let input = "(this is a literal string) <7468697320697320612068657820737472696E67> <~FD,B0+DGm>@3B#fF(I<g+EMXFBl7P~>";
        let mut scanner = Scanner::from(input.chars());

        for expected in [
            "this is a literal string",
            "this is a hex string",
            "this is a base85 string",
        ] {
            let Some(token) = scanner.next() else {
                return Err("expected token".into());
            };
            assert_eq!(Token::String(expected.to_string()), token?);
        }

        Ok(())
    }

    #[test]
    fn test_name() -> Result<(), Box<dyn error::Error>> {
        let inputs = [
            "abc",
            "Offset",
            "$$",
            "23A",
            "13-456",
            "a.b",
            "$MyDict",
            "@pattern",
            "16#FFFF.LMAO",
        ];

        for input in inputs {
            let mut scanner = Scanner::from(input.chars());
            let Some(token) = scanner.next() else {
                return Err("expected token".into());
            };

            assert_eq!(Token::Name(input.to_string()), token?);
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

        let mut scanner = Scanner::from(input.chars());
        for expect in expected_tokens {
            let Some(received) = scanner.next() else {
                return Err("expected token".into());
            };

            assert_eq!(expect, received?);
        }

        Ok(())
    }
}
