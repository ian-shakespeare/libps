use crate::{stack::Stack, token::Token};

const UNEXPECTED_OPERATOR_ERR: &'static str = "expected integer, received operator";
const UNUSED_INTEGER_ERR: &'static str = "unused integer";

pub fn parse(tokens: Vec<Token>) -> Result<i32, &'static str> {
    let mut value_stack: Stack<i32> = Stack::new();

    for token in tokens {
        match token {
            Token::Integer(n) => value_stack.push(n),
            Token::Real(_r) => todo!(),
            Token::Operator(op) => match op.apply_to_stack(&mut value_stack) {
                Ok(_) => (),
                Err(_) => return Err(UNEXPECTED_OPERATOR_ERR),
            },
        }
    }

    match value_stack.pop() {
        Some(final_value) => {
            if let Some(_) = value_stack.top() {
                Err(UNUSED_INTEGER_ERR)
            } else {
                Ok(final_value)
            }
        }
        None => Err(UNEXPECTED_OPERATOR_ERR),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operator::Operator;

    #[test]
    fn test_parse_unexpected_operator() {
        let tokens = vec![Token::Operator(Operator::Add)];
        assert!(parse(tokens).is_err())
    }

    #[test]
    fn test_parse_unused_integer() {
        let tokens = vec![Token::Integer(1), Token::Integer(1)];
        assert!(parse(tokens).is_err())
    }

    #[test]
    fn test_parse_unexpected_operator_compound() {
        let tokens = vec![
            Token::Integer(2),
            Token::Integer(1),
            Token::Integer(1),
            Token::Operator(Operator::Add),
            Token::Operator(Operator::Add),
            Token::Operator(Operator::Add),
        ];
        assert!(parse(tokens).is_err())
    }

    #[test]
    fn test_parse_simple() {
        let tokens = vec![
            Token::Integer(1),
            Token::Integer(1),
            Token::Operator(Operator::Add),
        ];
        assert_eq!(2, parse(tokens).unwrap())
    }

    #[test]
    fn test_parse_compound() {
        let tokens = vec![
            Token::Integer(16),
            Token::Integer(8),
            Token::Integer(4),
            Token::Integer(2),
            Token::Integer(1),
            Token::Integer(1),
            Token::Operator(Operator::Add),
            Token::Operator(Operator::Add),
            Token::Operator(Operator::Add),
            Token::Operator(Operator::Add),
            Token::Operator(Operator::Add),
        ];
        assert_eq!(32, parse(tokens).unwrap())
    }
}
