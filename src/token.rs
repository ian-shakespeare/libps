/*
pub enum Token<'a> {
    Integer(i32),
    Real(f64),
    String(&'a str),
    Name,
    Array,
    Procedure,
    Dictionary,
}
*/

use crate::operator::Operator;

pub enum Token {
    Integer(i32),
    Real(f64),
    Operator(Operator),
}

pub fn tokenize<'a>(s: &'a str) -> Result<Vec<Token>, ()> {
    let chars: Vec<char> = s.chars().collect();
    let mut tokens: Vec<Token> = Vec::new();

    let mut i = 0;
    while i < chars.len() {
        match chars.get(i) {
            None => break,
            Some(char) => match char.is_ascii() {
                false => break,
                true => match char {
                    '0'..='9' => {
                        let mut word = String::new();
                        let mut j = i;
                        'word: while let Some(next_char) = chars.get(j) {
                            if *next_char != '.' && !('0'..='9').contains(next_char) {
                                j -= 1;
                                break 'word;
                            }

                            word.push(next_char.clone());
                            j += 1;
                        }
                        match word.parse::<i32>() {
                            Ok(n) => tokens.push(Token::Integer(n)),
                            Err(_) => match word.parse::<f64>() {
                                Ok(r) => tokens.push(Token::Real(r)),
                                Err(_) => return Err(()),
                            },
                        }
                        i = j;
                    }
                    '+' => tokens.push(Token::Operator(Operator::Add)),
                    '-' => tokens.push(Token::Operator(Operator::Subtract)),
                    '*' => tokens.push(Token::Operator(Operator::Multiply)),
                    '/' => tokens.push(Token::Operator(Operator::Divide)),
                    ' ' | '\t' | '\r' | '\n' => (),
                    _ => return Err(()),
                },
            },
        }

        i += 1;
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_empty() {
        let tokens = tokenize("").unwrap();
        assert_eq!(0, tokens.len());
    }

    #[test]
    fn test_tokenize_integer_single() {
        let tokens = tokenize("1").unwrap();
        assert_eq!(1, tokens.len());

        if let Token::Integer(n) = tokens[0] {
            assert_eq!(1, n);
        } else {
            assert!(false, "expected integer");
        }
    }

    #[test]
    fn test_tokenize_integer_multiple() {
        let tokens = tokenize("1 1 1").unwrap();
        assert_eq!(3, tokens.len());

        for token in tokens {
            if let Token::Integer(n) = token {
                assert_eq!(1, n);
            } else {
                assert!(false, "expected integer");
            }
        }
    }

    #[test]
    fn test_tokenize_integer_multidigit() {
        let tokens = tokenize("1234567890").unwrap();
        assert_eq!(1, tokens.len());

        if let Token::Integer(n) = tokens[0] {
            assert_eq!(1234567890, n);
        } else {
            assert!(false, "expected integer")
        }
    }

    #[test]
    fn test_tokenize_operator() {
        let tokens = tokenize("+").unwrap();
        assert_eq!(1, tokens.len());
        assert!(
            if let Token::Operator(_) = tokens[0] {
                true
            } else {
                false
            },
            "expected operator"
        );
    }

    #[test]
    fn test_tokenize_operator_all() {
        let tokens = tokenize("+-*/").unwrap();
        assert_eq!(4, tokens.len());

        assert!(
            if let Token::Operator(op) = &tokens[0] {
                if let Operator::Add = op {
                    true
                } else {
                    false
                }
            } else {
                false
            },
            "expected operator"
        );

        assert!(
            if let Token::Operator(op) = &tokens[1] {
                if let Operator::Subtract = op {
                    true
                } else {
                    false
                }
            } else {
                false
            },
            "expected operator"
        );

        assert!(
            if let Token::Operator(op) = &tokens[2] {
                if let Operator::Multiply = op {
                    true
                } else {
                    false
                }
            } else {
                false
            },
            "expected operator"
        );

        assert!(
            if let Token::Operator(op) = &tokens[3] {
                if let Operator::Divide = op {
                    true
                } else {
                    false
                }
            } else {
                false
            },
            "expected operator"
        );
    }

    #[test]
    fn test_tokenize_operator_multiple() {
        for input in vec!["+ +", "++"] {
            let tokens = tokenize(input).unwrap();
            assert_eq!(2, tokens.len());

            for token in tokens {
                assert!(
                    if let Token::Operator(_) = token {
                        true
                    } else {
                        false
                    },
                    "expected operator"
                );
            }
        }
    }
}
