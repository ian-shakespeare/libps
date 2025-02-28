use std::iter;

use crate::{
    access::Access,
    composite::{Composite, Mode},
    encoding::{decode_ascii85, decode_hex},
    memory::VirtualMemory,
    object::Object,
    value::Value,
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
    pub fn next_obj(&mut self, mem: &mut VirtualMemory<Value>) -> Option<crate::Result<Object>> {
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
                },
                '-' | '.' | '0'..='9' => {
                    return Some(self.lex_numeric());
                },
                '(' => return Some(self.lex_string_literal(mem)),
                '<' => return Some(self.lex_gt(mem)),
                '{' => return Some(self.lex_procedure(mem)),
                _ => {
                    let name = String::from(self.input.next()?);
                    return Some(self.lex_name(name));
                },
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
                    _ => {},
                },
            }
        }

        Ok(())
    }

    fn lex_gt(&mut self, mem: &mut VirtualMemory<Value>) -> crate::Result<Object> {
        self.expect_char('<')?;

        let Some(ch) = self.input.peek() else {
            return Err(Error::new(
                ErrorKind::SyntaxError,
                "unterminated hex string",
            ));
        };

        match ch {
            '<' => {
                let _ = self.input.next();
                Ok(Object::Name("<<".into()))
            },
            '~' => self.lex_string_base85(mem),
            '0'..='9' | 'a'..='f' | 'A'..='F' => self.lex_string_hex(mem),
            _ => self.lex_name("<".to_string()),
        }
    }

    fn lex_name(&mut self, mut name: String) -> crate::Result<Object> {
        loop {
            if self.next_is_whitespace() {
                break;
            }

            if self.next_is_delimiter() && !name.is_empty() && name != "<" && name != ">" {
                break;
            }

            let first_ch = name.chars().nth(0).unwrap_or('\0');

            let lexing_delim =
                name == "<<" || name == ">>" || (name.len() == 1 && is_delimiter(first_ch));

            let lexing_literal = first_ch == '/';

            if self.next_is_regular() && lexing_delim && !lexing_literal {
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
            name => Object::Name(name.into()),
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
                },
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
                        },
                        _ => self.lex_name(numeric),
                    }
                },
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

    fn lex_procedure(&mut self, mem: &mut VirtualMemory<Value>) -> crate::Result<Object> {
        self.expect_char('{')?;

        let mut objs = Vec::new();

        loop {
            let obj = self
                .next_obj(mem)
                .ok_or(Error::new(ErrorKind::SyntaxError, "unterminated procedure"))??;

            if let Object::Name(ref n) = obj {
                if "}" == n.value() {
                    break;
                }
            }

            objs.push(obj);
        }

        let key = mem.insert(objs);

        Ok(Object::Array(Composite {
            access: Access::ExecuteOnly,
            mode: Mode::Literal,
            key,
        }))
    }

    fn lex_string_base85(&mut self, mem: &mut VirtualMemory<Value>) -> crate::Result<Object> {
        let mut string = String::new();

        loop {
            match self.input.next() {
                None => {
                    return Err(Error::new(
                        ErrorKind::SyntaxError,
                        "unterminated base85 string",
                    ))
                },
                Some('~') => match self.input.peek() {
                    None => {
                        return Err(Error::new(
                            ErrorKind::SyntaxError,
                            "unterminated base85 string",
                        ))
                    },
                    Some('>') => break,
                    _ => continue,
                },
                Some(ch) => string.push(ch),
            }
        }

        let string = decode_ascii85(&string)?;
        let key = mem.insert(string);

        Ok(Object::String(Composite {
            access: Access::Unlimited,
            mode: Mode::Literal,
            key,
        }))
    }

    fn lex_string_hex(&mut self, mem: &mut VirtualMemory<Value>) -> crate::Result<Object> {
        let mut string = String::new();

        loop {
            if self.next_is_whitespace() {
                let _ = self.input.next();
                continue;
            }

            let Some(ch) = self.input.next() else {
                return Err(Error::new(ErrorKind::SyntaxError, "unterminated string"));
            };

            match ch {
                '>' => break,
                '0'..='9' | 'a'..='z' | 'A'..='Z' => string.push(ch),
                _ => return Err(Error::new(ErrorKind::SyntaxError, "invalid hex string")),
            }
        }

        let string = decode_hex(&string)?;
        let key = mem.insert(string);

        Ok(Object::String(Composite {
            access: Access::Unlimited,
            mode: Mode::Literal,
            key,
        }))
    }

    fn lex_string_literal(&mut self, mem: &mut VirtualMemory<Value>) -> crate::Result<Object> {
        self.expect_char('(')?;

        let mut string = String::new();
        let mut active_parenthesis = 0;

        loop {
            let Some(ch) = self.input.next() else {
                return Err(Error::new(ErrorKind::SyntaxError, "unterminated string"));
            };

            match ch {
                '(' => {
                    string.push(ch);
                    active_parenthesis += 1;
                },
                ')' => {
                    if active_parenthesis < 1 {
                        break;
                    }
                    string.push(ch);
                    active_parenthesis -= 1;
                },
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
                                return Err(Error::new(
                                    ErrorKind::SyntaxError,
                                    "unterminated string",
                                ))
                            },
                            Some('\n') => {
                                let _ = self.input.next();
                            },
                            _ => {},
                        },
                        '0'..='9' => {
                            match (self.input.peek().cloned(), self.input.peek().cloned()) {
                                (Some(second_digit), Some(third_digit)) => {
                                    let octal =
                                        String::from_iter([next_ch, second_digit, third_digit]);

                                    match u8::from_str_radix(&octal, 8) {
                                        Err(_) => return Err(Error::from(ErrorKind::SyntaxError)),
                                        Ok(value) => {
                                            string.push(value.into());
                                            let _ = self.input.next();
                                            let _ = self.input.next();
                                        },
                                    }
                                },
                                _ => return Err(Error::from(ErrorKind::SyntaxError)),
                            }
                        },
                        _ => string.push(next_ch),
                    }
                },
                _ => string.push(ch),
            }
        }

        let string = decode_hex(&string)?;
        let key = mem.insert(string);

        Ok(Object::String(Composite {
            access: Access::Unlimited,
            mode: Mode::Literal,
            key,
        }))
    }

    fn expect_char(&mut self, ch: char) -> crate::Result<()> {
        match self.input.next() {
            Some(received) if ch == received => Ok(()),
            _ => Err(Error::new(ErrorKind::SyntaxError, format!("expected {ch}"))),
        }
    }

    fn next_is_delimiter(&mut self) -> bool {
        self.input.peek().is_some_and(|ch| is_delimiter(*ch))
    }

    fn next_is_regular(&mut self) -> bool {
        self.input.peek().is_some_and(|ch| is_regular(*ch))
    }

    fn next_is_whitespace(&mut self) -> bool {
        self.input.peek().is_some_and(|ch| is_whitespace(*ch))
    }
}

fn is_delimiter(ch: char) -> bool {
    matches!(ch, '<' | '>' | '[' | ']' | '{' | '}' | '/' | '%')
}

fn is_regular(ch: char) -> bool {
    !is_delimiter(ch) && !is_whitespace(ch)
}

fn is_whitespace(ch: char) -> bool {
    matches!(ch, '\0' | ' ' | '\t' | '\r' | '\n' | BACKSPACE | FORM_FEED)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::error;

    #[test]
    fn test_lex_comment() -> Result<(), Box<dyn error::Error>> {
        let mut lexer = Lexer::from("% this is a comment".chars());
        let mut mem = VirtualMemory::new();
        let obj = lexer.next_obj(&mut mem);

        assert!(obj.is_none());

        let cases = [
            ("10% this is a comment", Object::Integer(10)),
            ("16#FFFE% this is a comment", Object::Integer(0xFFFE)),
            ("1.0% this is a comment", Object::Real(1.0)),
            ("1.0e7% this is a comment", Object::Real(1.0e7)),
        ];

        for (input, expect) in cases {
            let mut lexer = Lexer::from(input.chars());

            let obj = lexer.next_obj(&mut mem).ok_or("expected object")??;

            assert_eq!(expect, obj);
        }

        Ok(())
    }

    #[test]
    fn test_lex_bad_numeric() -> Result<(), Box<dyn error::Error>> {
        let inputs = ["1x0", "1.x0"];

        for input in inputs {
            let mut lexer = Lexer::from(input.chars());
            let mut mem = VirtualMemory::new();

            let name = lexer.next_obj(&mut mem).ok_or("expected object")??;

            assert_eq!(Object::Name(input.into()), name);
        }

        Ok(())
    }

    #[test]
    fn test_lex_numeric() -> Result<(), Box<dyn error::Error>> {
        let cases = [
            ("1", Object::Integer(1)),
            ("-1", Object::Integer(-1)),
            ("1234567890", Object::Integer(1234567890)),
            ("2147483648", Object::Real(2147483648.0)),
            (".1", Object::Real(0.1)),
            ("-.1", Object::Real(-0.1)),
            ("1.234567890", Object::Real(1.234567890)),
            ("1.2E7", Object::Real(1.2e7)),
            ("1.2e7", Object::Real(1.2e7)),
            ("-1.2e7", Object::Real(-1.2e7)),
            ("1.2e-7", Object::Real(1.2e-7)),
            ("-1.2e-7", Object::Real(-1.2e-7)),
            ("2#1000", Object::Integer(0b1000)),
            ("8#1777", Object::Integer(0o1777)),
            ("16#fffe", Object::Integer(0xFFFE)),
            ("16#FFFE", Object::Integer(0xFFFE)),
            ("16#ffFE", Object::Integer(0xFFFE)),
        ];

        for (input, expect) in cases {
            let mut lexer = Lexer::from(input.chars());
            let mut mem = VirtualMemory::new();

            let obj = lexer.next_obj(&mut mem).ok_or("expected object")??;

            assert_eq!(expect, obj);
        }

        Ok(())
    }

    #[test]
    fn test_lex_bad_string() -> Result<(), Box<dyn error::Error>> {
        let inputs = ["(this is a string", "(this is a string\\)"];

        for input in inputs {
            let mut lexer = Lexer::from(input.chars());
            let mut mem = VirtualMemory::new();

            let result = lexer.next_obj(&mut mem).ok_or("expected object")?;

            assert!(result.is_err());
            assert_eq!(ErrorKind::SyntaxError, result.unwrap_err().kind());
        }

        Ok(())
    }

    #[test]
    fn test_lex_string() -> Result<(), Box<dyn error::Error>> {
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
            let mut lexer = Lexer::from(input.chars());
            let mut mem = VirtualMemory::new();

            let Some(Ok(Object::String(Composite { key, .. }))) = lexer.next_obj(&mut mem) else {
                return Err("expected string object".into());
            };

            let string: &str = mem.get(key)?.try_into()?;

            assert_eq!(expect, string);
        }

        Ok(())
    }

    #[test]
    fn test_lex_escaped_string() -> Result<(), Box<dyn error::Error>> {
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
            let mut lexer = Lexer::from(input.chars());
            let mut mem = VirtualMemory::new();

            let Some(Ok(Object::String(Composite { key, .. }))) = lexer.next_obj(&mut mem) else {
                return Err("expected string object".into());
            };

            let string: &str = mem.get(key)?.try_into()?;

            assert_eq!(expect, string);
        }

        Ok(())
    }

    #[test]
    fn test_lex_ignore_escaped_string() -> Result<(), Box<dyn error::Error>> {
        let input = "(\\ii)";
        let mut lexer = Lexer::from(input.chars());
        let mut mem = VirtualMemory::new();

        let Some(Ok(Object::String(Composite { key, .. }))) = lexer.next_obj(&mut mem) else {
            return Err("expected string object".into());
        };

        let string: &str = mem.get(key)?.try_into()?;

        assert_eq!("ii", string);

        Ok(())
    }

    #[test]
    fn test_lex_octal_string() -> Result<(), Box<dyn error::Error>> {
        let cases = [("(\\000)", "\0"), ("(\\377)", "ÿ")];

        for (input, expect) in cases {
            let mut lexer = Lexer::from(input.chars());
            let mut mem = VirtualMemory::new();

            let Some(Ok(Object::String(Composite { key, .. }))) = lexer.next_obj(&mut mem) else {
                return Err("expected string object".into());
            };

            let string: &str = mem.get(key)?.try_into()?;

            assert_eq!(expect, string);
        }

        Ok(())
    }

    #[test]
    fn test_lex_hex_string() -> Result<(), Box<dyn error::Error>> {
        let cases = [
            ("<736F6D65>", "some"),
            ("<736f6d65>", "some"),
            ("<736f6D65>", "some"),
            ("<73 6F 6D 65>", "some"),
            ("<70756D7>", "pump"),
            ("<70756D70>", "pump"),
        ];

        for (input, expect) in cases {
            let mut lexer = Lexer::from(input.chars());
            let mut mem = VirtualMemory::new();

            let Some(Ok(Object::String(Composite { key, .. }))) = lexer.next_obj(&mut mem) else {
                return Err("expected string object".into());
            };

            let string: &str = mem.get(key)?.try_into()?;

            assert_eq!(expect, string);
        }

        Ok(())
    }

    #[test]
    fn test_lex_base85_string() -> Result<(), Box<dyn error::Error>> {
        let input = "<~FD,B0+DGm>F)Po,+EV1>F8~>";
        let expect = "this is some text";
        let mut lexer = Lexer::from(input.chars());
        let mut mem = VirtualMemory::new();

        let Some(Ok(Object::String(Composite { key, .. }))) = lexer.next_obj(&mut mem) else {
            return Err("expected string object".into());
        };

        let string: &str = mem.get(key)?.try_into()?;

        assert_eq!(expect, string);

        Ok(())
    }

    #[test]
    fn test_lex_multiple_string() -> Result<(), Box<dyn error::Error>> {
        let input = "(this is a literal string) <7468697320697320612068657820737472696E67> <~FD,B0+DGm>@3B#fF(I<g+EMXFBl7P~>";
        let expect = [
            "this is a literal string",
            "this is a hex string",
            "this is a base85 string",
        ];

        let mut lexer = Lexer::from(input.chars());
        let mut mem = VirtualMemory::new();

        for e in expect {
            let Some(Ok(Object::String(Composite { key, .. }))) = lexer.next_obj(&mut mem) else {
                return Err("expected string object".into());
            };

            let string: &str = mem.get(key)?.try_into()?;

            assert_eq!(e, string);
        }

        Ok(())
    }

    #[test]
    fn test_lex_name() -> Result<(), Box<dyn error::Error>> {
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
            let mut lexer = Lexer::from(input.chars());
            let mut mem = VirtualMemory::new();

            let name = lexer.next_obj(&mut mem).ok_or("expected object")??;

            assert_eq!(Object::Name(input.into()), name);
        }

        Ok(())
    }

    #[test]
    fn test_lex_procedure() -> Result<(), Box<dyn error::Error>> {
        let inputs = ["{}", "{ }", "{ { } }"];

        for input in inputs {
            let mut lexer = Lexer::from(input.chars());
            let mut mem = VirtualMemory::new();

            let obj = lexer.next_obj(&mut mem).ok_or("expected object")??;
            assert!(obj.is_array());
        }

        Ok(())
    }

    #[test]
    fn test_lex_procedure_nested() -> Result<(), Box<dyn error::Error>> {
        let inputs = ["{ { 1 } }", "{{1}}"];

        for input in inputs {
            let mut lexer = Lexer::from(input.chars());
            let mut mem = VirtualMemory::new();

            let Some(Ok(Object::Array(Composite { access, key, .. }))) = lexer.next_obj(&mut mem)
            else {
                return Err("expected procedure object".into());
            };
            assert!(access.is_exec_only());

            let outer: &Vec<Object> = mem.get(key)?.try_into()?;
            assert_eq!(1, outer.len());

            let Some(Object::Array(Composite { access, key, .. })) = outer.first() else {
                return Err("expected procedure object".into());
            };
            assert!(access.is_exec_only());

            let inner: &Vec<Object> = mem.get(*key)?.try_into()?;
            assert_eq!(1, inner.len());
            assert_eq!(Some(Object::Integer(1)), inner.first().cloned());
        }

        Ok(())
    }

    #[test]
    fn test_lex_self_deliminating() -> Result<(), Box<dyn error::Error>> {
        let inputs = [
            ("mid[dle", "[".to_string()),
            ("mid]dle", "]".to_string()),
            ("mid<<dle", "<<".to_string()),
            ("mid>>dle", ">>".to_string()),
            ("mid/dle", "/dle".to_string()),
            ("1[2", "[".to_string()),
            ("1]2", "]".to_string()),
            ("1<<2", "<<".to_string()),
            ("1>>2", ">>".to_string()),
            ("1/2", "/2".to_string()),
            ("1.2[3", "[".to_string()),
            ("1.2]3", "]".to_string()),
            ("1.2<<3", "<<".to_string()),
            ("1.2>>3", ">>".to_string()),
            ("1.2/3", "/3".to_string()),
            ("16#FF[FF", "[".to_string()),
            ("16#FF]FF", "]".to_string()),
            ("16#FF<<FF", "<<".to_string()),
            ("16#FF>>FF", ">>".to_string()),
            ("16#FF/FF", "/FF".to_string()),
        ];

        for (input, expect) in inputs {
            let mut lexer = Lexer::from(input.chars());
            let mut mem = VirtualMemory::new();
            let _ = lexer.next_obj(&mut mem);

            let obj = lexer.next_obj(&mut mem).ok_or("expected object")??;

            assert_eq!(Object::Name(expect.into()), obj);
        }

        Ok(())
    }

    #[test]
    fn test_lex_self_deliminating_pair() -> Result<(), Box<dyn error::Error>> {
        let cases = [("[[", "["), ("]]", "]"), ("<<<<", "<<"), (">>>>", ">>")];

        for (input, expect) in cases {
            let mut lexer = Lexer::from(input.chars());
            let mut mem = VirtualMemory::new();

            while let Some(obj) = lexer.next_obj(&mut mem) {
                assert_eq!(Object::Name(expect.into()), obj?);
            }
        }

        Ok(())
    }

    #[test]
    fn test_lex_all() -> Result<(), Box<dyn error::Error>> {
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

        let mut lexer = Lexer::from(input.chars());
        let mut mem = VirtualMemory::new();

        let name = lexer.next_obj(&mut mem).ok_or("expected object")??;
        assert_eq!(Object::Name("myStr".into()), name);

        let Some(Ok(Object::String(Composite { key, .. }))) = lexer.next_obj(&mut mem) else {
            return Err("expected string object".into());
        };
        let string: &str = mem.get(key)?.try_into()?;
        assert_eq!("i have a string right here", string);

        let name = lexer.next_obj(&mut mem).ok_or("expected object")??;
        assert_eq!(Object::Name("myOtherStr".into()), name);

        let Some(Ok(Object::String(Composite { key, .. }))) = lexer.next_obj(&mut mem) else {
            return Err("expected string object".into());
        };
        let string: &str = mem.get(key)?.try_into()?;
        assert_eq!("and\nanother right here", string);

        let name = lexer.next_obj(&mut mem).ok_or("expected object")??;
        assert_eq!(Object::Name("myInt".into()), name);

        let integer = lexer.next_obj(&mut mem).ok_or("expected object")??;
        assert_eq!(Object::Integer(1234567890), integer);

        let name = lexer.next_obj(&mut mem).ok_or("expected object")??;
        assert_eq!(Object::Name("myNegativeInt".into()), name);

        let integer = lexer.next_obj(&mut mem).ok_or("expected object")??;
        assert_eq!(Object::Integer(-1234567890), integer);

        let name = lexer.next_obj(&mut mem).ok_or("expected object")??;
        assert_eq!(Object::Name("myReal".into()), name);

        let integer = lexer.next_obj(&mut mem).ok_or("expected object")??;
        assert_eq!(Object::Real(3.1456), integer);

        let name = lexer.next_obj(&mut mem).ok_or("expected object")??;
        assert_eq!(Object::Name("myNegativeReal".into()), name);

        let integer = lexer.next_obj(&mut mem).ok_or("expected object")??;
        assert_eq!(Object::Real(-3.1456), integer);

        Ok(())
    }
}
