use std::iter;

use crate::{
    encoding::{decode_ascii85, decode_hex},
    object::Object,
    token::{Token, TokenKind},
    Error, ErrorKind,
};

pub struct Tokenizer<I: Iterator<Item = Token>> {
    input: iter::Peekable<I>,
}

impl<I> From<I> for Tokenizer<I>
where
    I: Iterator<Item = Token>,
{
    fn from(value: I) -> Self {
        Self {
            input: value.peekable(),
        }
    }
}

impl<I> Iterator for Tokenizer<I>
where
    I: Iterator<Item = Token>,
{
    type Item = crate::Result<Object>;

    fn next(&mut self) -> Option<Self::Item> {
        self.input.next().map(|token| match token.kind() {
            TokenKind::Integer => {
                let is_radix = token.value().contains('#');
                if is_radix {
                    let mut parts = token.value().split('#');
                    match (parts.next(), parts.next()) {
                        (Some(base), Some(digits)) => match base.parse::<u32>() {
                            Ok(base) => match i32::from_str_radix(digits, base) {
                                Ok(value) => Ok(Object::Integer(value)),
                                Err(_) => {
                                    Err(Error::new(ErrorKind::Syntax, "invalid radix digits"))
                                }
                            },
                            Err(_) => Err(Error::new(ErrorKind::Syntax, "invalid radix base")),
                        },
                        _ => Err(Error::new(ErrorKind::Syntax, "invalid radix")),
                    }
                } else {
                    match token.value().parse::<i32>() {
                        Ok(i) => Ok(Object::Integer(i)),
                        Err(_) => match token.value().parse::<f64>() {
                            Ok(r) => Ok(Object::Real(r)),
                            Err(_) => Err(Error::new(ErrorKind::Syntax, "invalid numeric")),
                        },
                    }
                }
            }
            TokenKind::Real => {
                let is_scientific = token.value().contains('e');
                if is_scientific {
                    let mut parts = token.value().split('e');
                    match (parts.next(), parts.next()) {
                        (Some(decimal), Some(exponent)) => {
                            match (decimal.parse::<f64>(), exponent.parse::<i32>()) {
                                (Ok(decimal), Ok(exponent)) => {
                                    let value = decimal * 10.0_f64.powi(exponent);
                                    Ok(Object::Real(value))
                                }
                                _ => Err(Error::new(ErrorKind::Syntax, "invalid scientific real")),
                            }
                        }
                        _ => Err(Error::new(ErrorKind::Syntax, "invalid scientific real")),
                    }
                } else {
                    match token.value().parse::<f64>() {
                        Ok(value) => Ok(Object::Real(value)),
                        Err(_) => Err(Error::new(ErrorKind::Syntax, "invalid real")),
                    }
                }
            }
            TokenKind::StringLiteral => Ok(Object::String(token.value().to_string())),
            TokenKind::StringHex => match decode_hex(token.value()) {
                Ok(decoded) => Ok(Object::String(decoded)),
                Err(e) => Err(e),
            },
            TokenKind::StringBase85 => match decode_ascii85(token.value()) {
                Ok(decoded) => Ok(Object::String(decoded)),
                Err(e) => Err(e),
            },
            TokenKind::Name => Ok(match token.value() {
                "true" => Object::Boolean(true),
                "false" => Object::Boolean(false),
                _ => Object::Name(token.value().to_string()),
            }),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::error;

    use super::*;

    #[test]
    fn test_tokenize() -> Result<(), Box<dyn error::Error>> {
        let cases = [
            (
                Token::new(TokenKind::Integer, "1".to_string()),
                Object::Integer(1),
            ),
            (
                Token::new(TokenKind::Integer, "16#FFFE".to_string()),
                Object::Integer(0xFFFE),
            ),
            (
                Token::new(TokenKind::Real, "1.0".to_string()),
                Object::Real(1.0),
            ),
            (
                Token::new(TokenKind::Real, "1.2e-7".to_string()),
                Object::Real(0.00000012),
            ),
            (
                Token::new(
                    TokenKind::StringLiteral,
                    "this is a literal string".to_string(),
                ),
                Object::String("this is a literal string".to_string()),
            ),
            (
                Token::new(
                    TokenKind::StringHex,
                    "7468697320697320612068657820737472696E67".to_string(),
                ),
                Object::String("this is a hex string".to_string()),
            ),
            (
                Token::new(
                    TokenKind::StringBase85,
                    "FD,B0+DGm>@3B#fF(I<g+EMXFBl7P".to_string(),
                ),
                Object::String("this is a base85 string".to_string()),
            ),
            (
                Token::new(TokenKind::Name, "true".to_string()),
                Object::Boolean(true),
            ),
            (
                Token::new(TokenKind::Name, "false".to_string()),
                Object::Boolean(false),
            ),
            (
                Token::new(TokenKind::Name, "name".to_string()),
                Object::Name("name".to_string()),
            ),
        ];

        for (input, expect) in cases {
            let Some(Ok(obj)) = Tokenizer::from([input].into_iter()).next() else {
                return Err("expected object".into());
            };
            assert_eq!(expect, obj);
        }

        Ok(())
    }
}
