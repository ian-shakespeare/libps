use crate::{stack::Stack, token::Token};

const UNEXPECTED_OPERATOR_ERR: &'static str = "expected integer, received operator";
const UNUSED_INTEGER_ERR: &'static str = "unused integer";

pub fn parse(tokens: Vec<Token>) -> Result<Token, &'static str> {
    let mut value_stack: Stack<Token> = Stack::new();

    for token in tokens {
        match token {
            Token::Operator(op) => match op.apply_to_stack(&mut value_stack) {
                Ok(_) => (),
                Err(_) => return Err(UNEXPECTED_OPERATOR_ERR),
            },
            _ => value_stack.push(token),
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
        // +
        let tokens = vec![Token::Operator(Operator::Add)];
        assert!(parse(tokens).is_err())
    }

    #[test]
    fn test_parse_unused_integer() {
        // 1 1
        let tokens = vec![Token::Integer(1), Token::Integer(1)];
        assert!(parse(tokens).is_err())
    }

    #[test]
    fn test_parse_unexpected_operator_compound() {
        // 2 1 1 +++
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
    fn test_parse_add_simple() {
        // 1 1 +
        let tokens = vec![
            Token::Integer(1),
            Token::Integer(1),
            Token::Operator(Operator::Add),
        ];
        assert_eq!(Token::Integer(2), parse(tokens).unwrap())
    }

    #[test]
    fn test_parse_add_compound() {
        // 16 8 4 2 1 1 +++++
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
        assert_eq!(Token::Integer(32), parse(tokens).unwrap())
    }

    #[test]
    fn test_parse_subtract_simple() {
        // 1 1 -
        let tokens = vec![
            Token::Integer(1),
            Token::Integer(1),
            Token::Operator(Operator::Sub),
        ];
        assert_eq!(Token::Integer(0), parse(tokens).unwrap())
    }

    #[test]
    fn test_parse_subtract_compound() {
        // 100 100 50 -- 150 - 0 100 --
        let tokens = vec![
            Token::Integer(100),
            Token::Integer(100),
            Token::Integer(50),
            Token::Operator(Operator::Sub),
            Token::Operator(Operator::Sub),
            Token::Integer(150),
            Token::Operator(Operator::Sub),
            Token::Integer(0),
            Token::Integer(100),
            Token::Operator(Operator::Sub),
            Token::Operator(Operator::Sub),
        ];
        assert_eq!(Token::Integer(0), parse(tokens).unwrap())
    }

    #[test]
    fn test_parse_multiply_simple() {
        // Zero
        // 2 0 *
        let tokens = vec![
            Token::Integer(2),
            Token::Integer(0),
            Token::Operator(Operator::Mul),
        ];
        assert_eq!(Token::Integer(0), parse(tokens).unwrap());

        // Identity
        // 2 1 *
        let tokens = vec![
            Token::Integer(2),
            Token::Integer(1),
            Token::Operator(Operator::Mul),
        ];
        assert_eq!(Token::Integer(2), parse(tokens).unwrap());

        // 2 2 *
        let tokens = vec![
            Token::Integer(2),
            Token::Integer(2),
            Token::Operator(Operator::Mul),
        ];
        assert_eq!(Token::Integer(4), parse(tokens).unwrap());
    }

    #[test]
    fn test_parse_multiply_compound() {
        // 5 10 5 * 2 * *
        let tokens = vec![
            Token::Integer(5),
            Token::Integer(10),
            Token::Integer(5),
            Token::Operator(Operator::Mul),
            Token::Integer(2),
            Token::Operator(Operator::Mul),
            Token::Operator(Operator::Mul),
        ];
        assert_eq!(Token::Integer(500), parse(tokens).unwrap());
    }

    #[test]
    fn test_parse_divide_simple() {
        // Zero
        // 2 0 /
        let tokens = vec![
            Token::Integer(10),
            Token::Integer(0),
            Token::Operator(Operator::Div),
        ];
        assert!(parse(tokens).is_err());

        // Identity
        // 2 1 /
        let tokens = vec![
            Token::Integer(2),
            Token::Integer(1),
            Token::Operator(Operator::Div),
        ];
        assert_eq!(Token::Integer(2), parse(tokens).unwrap());

        // 8 2 /
        let tokens = vec![
            Token::Integer(8),
            Token::Integer(2),
            Token::Operator(Operator::Div),
        ];
        assert_eq!(Token::Integer(4), parse(tokens).unwrap());
    }

    #[test]
    fn test_parse_divide_compound() {
        // 20 360 6 / 3 / /
        let tokens = vec![
            Token::Integer(20),
            Token::Integer(360),
            Token::Integer(6),
            Token::Operator(Operator::Div),
            Token::Integer(3),
            Token::Operator(Operator::Div),
            Token::Operator(Operator::Div),
        ];
        assert_eq!(Token::Integer(1), parse(tokens).unwrap());
    }

    #[test]
    fn test_parse_all_operators() {
        // Pythagorean's Alg set to zero
        // 1 - (((a * a) + (b * b)) / (c * c)) = 0
        let a = 3;
        let b = 4;
        let c = 5;

        // 1 a a * b b * + c c * / -
        let tokens = vec![
            Token::Integer(1),
            Token::Integer(a),
            Token::Integer(a),
            Token::Operator(Operator::Mul),
            Token::Integer(b),
            Token::Integer(b),
            Token::Operator(Operator::Mul),
            Token::Operator(Operator::Add),
            Token::Integer(c),
            Token::Integer(c),
            Token::Operator(Operator::Mul),
            Token::Operator(Operator::Div),
            Token::Operator(Operator::Sub),
        ];
        assert_eq!(Token::Integer(0), parse(tokens).unwrap())
    }
}
