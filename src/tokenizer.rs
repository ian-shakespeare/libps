use std::{collections, iter, rc};

use crate::{object::Object, token::Token, Error, ErrorKind};

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
        match self.input.next() {
            None => None,
            Some(token) => Some(match token {
                Token::Integer(i) => Ok(Object::Integer(i)),
                Token::Real(r) => Ok(Object::Real(r)),
                Token::String(s) => Ok(Object::String(s)),
                Token::Name(n) => Ok(match n.as_str() {
                    "true" => Object::Boolean(true),
                    "false" => Object::Boolean(false),
                    "[" => {
                        let mut arr = Vec::new();
                        loop {
                            match self.input.peek() {
                                None => {
                                    return Some(Err(Error::from(ErrorKind::UnterminatedArray)))
                                }
                                Some(token) => {
                                    if let Token::Name(n) = token {
                                        if n == "]" {
                                            let _ = self.input.next();
                                            break;
                                        }
                                    }
                                    match self.next() {
                                        Some(Ok(obj)) => arr.push(obj),
                                        _ => {
                                            return Some(Err(Error::from(
                                                ErrorKind::UnterminatedArray,
                                            )))
                                        }
                                    }
                                }
                            }
                        }
                        Object::Array(rc::Rc::new(arr))
                    }
                    "{" => {
                        let mut arr = Vec::new();
                        loop {
                            match self.input.peek() {
                                None => {
                                    return Some(Err(Error::from(ErrorKind::UnterminatedArray)))
                                }
                                Some(token) => {
                                    if let Token::Name(n) = token {
                                        if n == "}" {
                                            let _ = self.input.next();
                                            break;
                                        }
                                    }
                                    match self.next() {
                                        Some(Ok(obj)) => arr.push(obj),
                                        _ => {
                                            return Some(Err(Error::from(
                                                ErrorKind::UnterminatedArray,
                                            )))
                                        }
                                    }
                                }
                            }
                        }
                        Object::PackedArray(rc::Rc::new(arr))
                    }
                    "<<" => {
                        let mut dict = collections::HashMap::new();
                        loop {
                            match self.input.peek() {
                                None => {
                                    return Some(Err(Error::from(
                                        ErrorKind::UnterminatedDictionary,
                                    )))
                                }
                                Some(token) => {
                                    if let Token::Name(n) = token {
                                        if n == ">>" {
                                            let _ = self.input.next();
                                            break;
                                        }
                                    }
                                    match (self.next(), self.next()) {
                                        (Some(Ok(key_obj)), Some(Ok(value_obj))) => {
                                            if let Object::Name(ref n) = value_obj {
                                                if n == ">>" {
                                                    return Some(Err(Error::from(
                                                        ErrorKind::MissingValue,
                                                    )));
                                                }
                                            }
                                            let key = String::from(key_obj);
                                            dict.insert(key, value_obj);
                                        }
                                        _ => {
                                            return Some(Err(Error::from(
                                                ErrorKind::UnterminatedDictionary,
                                            )))
                                        }
                                    }
                                }
                            }
                        }

                        Object::Dictionary(rc::Rc::new(dict))
                    }
                    _ => Object::Name(n),
                }),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::error;

    use super::*;

    #[test]
    fn test_unterminated() -> Result<(), Box<dyn error::Error>> {
        let cases = [
            (
                vec![Token::Name("[".to_string())],
                ErrorKind::UnterminatedArray,
            ),
            (
                vec![Token::Name("[".to_string()), Token::Integer(1)],
                ErrorKind::UnterminatedArray,
            ),
            (
                vec![Token::Name("{".to_string())],
                ErrorKind::UnterminatedArray,
            ),
            (
                vec![Token::Name("{".to_string()), Token::Integer(1)],
                ErrorKind::UnterminatedArray,
            ),
            (
                vec![Token::Name("<<".to_string())],
                ErrorKind::UnterminatedDictionary,
            ),
            (
                vec![
                    Token::Name("<<".to_string()),
                    Token::Name("key".to_string()),
                ],
                ErrorKind::UnterminatedDictionary,
            ),
            (
                vec![
                    Token::Name("<<".to_string()),
                    Token::Name("key".to_string()),
                    Token::String("value".to_string()),
                ],
                ErrorKind::UnterminatedDictionary,
            ),
            (
                vec![
                    Token::Name("<<".to_string()),
                    Token::Name("key".to_string()),
                    Token::Name(">>".to_string()),
                ],
                ErrorKind::MissingValue,
            ),
        ];

        for (input, expect) in cases {
            let Some(obj) = Tokenizer::from(input.into_iter()).next() else {
                return Err("expected object".into());
            };
            assert!(obj.is_err());
            assert_eq!(expect, obj.unwrap_err().kind());
        }

        Ok(())
    }

    #[test]
    fn test_array() {
        let cases = [
            (
                vec![Token::Name("[".to_string()), Token::Name("]".to_string())],
                "[ ]",
            ),
            (
                vec![
                    Token::Name("[".to_string()),
                    Token::Integer(1),
                    Token::Name("]".to_string()),
                ],
                "[ 1 ]",
            ),
            (
                vec![
                    Token::Name("[".to_string()),
                    Token::Name("[".to_string()),
                    Token::Integer(1),
                    Token::Integer(2),
                    Token::Name("]".to_string()),
                    Token::Name("[".to_string()),
                    Token::Integer(3),
                    Token::Integer(4),
                    Token::Name("]".to_string()),
                    Token::Name("]".to_string()),
                ],
                "[ [ 1 2 ] [ 3 4 ] ]",
            ),
        ];
        for (input, expect) in cases {
            let objects: Vec<Object> = Tokenizer::from(input.into_iter())
                .filter_map(|obj| obj.ok())
                .collect();
            assert_eq!(expect, &String::from(objects[0].clone()));
        }
    }

    #[test]
    fn test_procedure() {
        let cases = [
            (
                vec![Token::Name("{".to_string()), Token::Name("}".to_string())],
                "{ }",
            ),
            (
                vec![
                    Token::Name("{".to_string()),
                    Token::Integer(1),
                    Token::Name("}".to_string()),
                ],
                "{ 1 }",
            ),
            (
                vec![
                    Token::Name("{".to_string()),
                    Token::Name("{".to_string()),
                    Token::Integer(1),
                    Token::Integer(2),
                    Token::Name("}".to_string()),
                    Token::Name("{".to_string()),
                    Token::Integer(3),
                    Token::Integer(4),
                    Token::Name("}".to_string()),
                    Token::Name("}".to_string()),
                ],
                "{ { 1 2 } { 3 4 } }",
            ),
        ];
        for (input, expect) in cases {
            let objects: Vec<Object> = Tokenizer::from(input.into_iter())
                .filter_map(|obj| obj.ok())
                .collect();
            assert_eq!(expect, &String::from(objects[0].clone()));
        }
    }

    #[test]
    fn test_dict() {
        let cases = [
            (
                vec![Token::Name("<<".to_string()), Token::Name(">>".to_string())],
                "<< >>",
            ),
            (
                vec![
                    Token::Name("<<".to_string()),
                    Token::Name("key".to_string()),
                    Token::String("value".to_string()),
                    Token::Name(">>".to_string()),
                ],
                "<< key value >>",
            ),
            (
                vec![
                    Token::Name("<<".to_string()),
                    Token::Name("outer".to_string()),
                    Token::Name("<<".to_string()),
                    Token::Name("inner".to_string()),
                    Token::Integer(1),
                    Token::Name(">>".to_string()),
                    Token::Name(">>".to_string()),
                ],
                "<< outer << inner 1 >> >>",
            ),
        ];

        for (input, expect) in cases {
            let objects: Vec<Object> = Tokenizer::from(input.into_iter())
                .filter_map(|obj| obj.ok())
                .collect();
            assert_eq!(expect, &String::from(objects[0].clone()));
        }
    }
}
