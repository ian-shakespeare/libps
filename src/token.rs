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

#[derive(Debug)]
pub enum Token {
    Integer(i32),
    Real(f64),
    Operator(Operator),
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Token::Integer(n) => match other {
                Token::Integer(other_n) => n == other_n,
                Token::Real(other_r) => f64::from(*n) == *other_r,
                _ => false,
            },
            Token::Real(r) => match other {
                Token::Integer(other_n) => *r == f64::from(*other_n),
                Token::Real(other_r) => r == other_r,
                _ => false,
            },
            _ => false,
        }
    }
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
                    '+' => tokens.push(Token::Operator(Operator::Add)),
                    '-' => tokens.push(tokenize_dash(&chars, &mut i)?),
                    '*' => tokens.push(Token::Operator(Operator::Mul)),
                    '/' => tokens.push(Token::Operator(Operator::Div)),
                    '.' | '0'..='9' => tokens.push(tokenize_numeric(&chars, &mut i)?),
                    ' ' | '\t' | '\r' | '\n' => (), // Do nothing on white space
                    _ => return Err(()),
                },
            },
        }

        i += 1;
    }

    Ok(tokens)
}

fn tokenize_dash(chars: &Vec<char>, index: &mut usize) -> Result<Token, ()> {
    if let Some(next_char) = (*chars).get(*index + 1) {
        if is_numeric(next_char) {
            tokenize_numeric(chars, index)
        } else {
            Ok(Token::Operator(Operator::Sub))
        }
    } else {
        Err(())
    }
}

fn tokenize_numeric(chars: &Vec<char>, index: &mut usize) -> Result<Token, ()> {
    let mut word = String::from(chars[*index]);
    let mut j = index.clone() + 1;
    'word: while let Some(next_char) = (*chars).get(j) {
        if !is_numeric(next_char) {
            j -= 1;
            break 'word;
        }

        word.push(next_char.clone());
        j += 1;
    }

    *index = j;
    match word.parse::<i32>() {
        Ok(n) => Ok(Token::Integer(n)),
        Err(_) => match word.parse::<f64>() {
            Ok(r) => Ok(Token::Real(r)),
            Err(_) => Err(()),
        },
    }
}

fn is_numeric(c: &char) -> bool {
    *c == '.' || ('0'..='9').contains(c)
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
        let inputs = [(1, "1"), (10, "10"), (-1, "-1"), (-10, "-10")];

        for (input_val, input_str) in inputs {
            let tokens = tokenize(input_str).unwrap();
            assert_eq!(1, tokens.len());

            if let Token::Integer(n) = tokens[0] {
                assert_eq!(input_val, n);
            } else {
                assert!(false, "expected integer");
            }
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
            assert!(false, "expected integer");
        }
    }

    #[test]
    fn test_tokenize_real_single() {
        let inputs = vec![
            (0.1, "0.1"),
            (1.1, "1.1"),
            (0.1, ".1"),
            (-1.1, "-1.1"),
            (-0.1, "-.1"),
        ];

        for (input_val, input_str) in inputs {
            let tokens = tokenize(input_str).unwrap();
            assert_eq!(1, tokens.len());

            if let Token::Real(r) = tokens[0] {
                assert_eq!(input_val, r);
            } else {
                assert!(false, "expected real");
            }
        }
    }

    #[test]
    fn test_tokenize_real_multiple() {
        let tokens = tokenize("1.1 1.1 1.1").unwrap();
        assert_eq!(3, tokens.len());

        for token in tokens {
            if let Token::Real(r) = token {
                assert_eq!(1.1, r);
            } else {
                assert!(false, "expected real");
            }
        }
    }

    #[test]
    fn test_tokenize_real_multidigit() {
        let tokens = tokenize("3.1456").unwrap();
        assert_eq!(1, tokens.len());

        if let Token::Real(r) = tokens[0] {
            assert_eq!(3.1456, r);
        } else {
            assert!(false, "expected real");
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
                if let Operator::Sub = op {
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
                if let Operator::Mul = op {
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
                if let Operator::Div = op {
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
