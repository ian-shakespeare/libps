use crate::{stack::Stack, token::Token};

#[derive(Debug)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
}

impl Operator {
    pub fn apply_to_stack(&self, value_stack: &mut Stack<Token>) -> Result<(), ()> {
        match self {
            Operator::Add => {
                if let Some((lhs, rhs)) = value_stack.pop_two() {
                    value_stack.push(add_tokens(lhs, rhs)?);
                } else {
                    return Err(());
                }
            }
            Operator::Sub => {
                if let Some((lhs, rhs)) = value_stack.pop_two() {
                    value_stack.push(sub_tokens(lhs, rhs)?);
                } else {
                    return Err(());
                }
            }
            Operator::Mul => {
                if let Some((lhs, rhs)) = value_stack.pop_two() {
                    value_stack.push(mul_tokens(lhs, rhs)?);
                } else {
                    return Err(());
                }
            }
            Operator::Div => {
                if let Some((lhs, rhs)) = value_stack.pop_two() {
                    value_stack.push(div_tokens(lhs, rhs)?);
                } else {
                    return Err(());
                }
            }
        }

        Ok(())
    }
}

fn add_tokens(lhs: Token, rhs: Token) -> Result<Token, ()> {
    match lhs {
        Token::Integer(left_int) => match rhs {
            Token::Integer(right_int) => Ok(Token::Integer(left_int + right_int)),
            Token::Real(right_real) => Ok(Token::Real(f64::from(left_int) + right_real)),
            _ => Err(()),
        },
        Token::Real(left_real) => match rhs {
            Token::Integer(right_int) => Ok(Token::Real(left_real + f64::from(right_int))),
            Token::Real(right_real) => Ok(Token::Real(left_real + right_real)),
            _ => Err(()),
        },
        _ => Err(()),
    }
}

fn sub_tokens(lhs: Token, rhs: Token) -> Result<Token, ()> {
    match lhs {
        Token::Integer(left_int) => match rhs {
            Token::Integer(right_int) => Ok(Token::Integer(left_int - right_int)),
            Token::Real(right_real) => Ok(Token::Real(f64::from(left_int) - right_real)),
            _ => Err(()),
        },
        Token::Real(left_real) => match rhs {
            Token::Integer(right_int) => Ok(Token::Real(left_real - f64::from(right_int))),
            Token::Real(right_real) => Ok(Token::Real(left_real - right_real)),
            _ => Err(()),
        },
        _ => Err(()),
    }
}

fn mul_tokens(lhs: Token, rhs: Token) -> Result<Token, ()> {
    match lhs {
        Token::Integer(left_int) => match rhs {
            Token::Integer(right_int) => Ok(Token::Integer(left_int * right_int)),
            Token::Real(right_real) => Ok(Token::Real(f64::from(left_int) * right_real)),
            _ => Err(()),
        },
        Token::Real(left_real) => match rhs {
            Token::Integer(right_int) => Ok(Token::Real(left_real * f64::from(right_int))),
            Token::Real(right_real) => Ok(Token::Real(left_real * right_real)),
            _ => Err(()),
        },
        _ => Err(()),
    }
}

fn div_tokens(lhs: Token, rhs: Token) -> Result<Token, ()> {
    match rhs {
        Token::Integer(right_int) => {
            if right_int == 0 {
                Err(())
            } else {
                match lhs {
                    Token::Integer(left_int) => {
                        Ok(Token::Real(f64::from(left_int) / f64::from(right_int)))
                    }
                    Token::Real(left_real) => Ok(Token::Real(left_real / f64::from(right_int))),
                    _ => Err(()),
                }
            }
        }
        Token::Real(right_real) => {
            if right_real == 0.0 {
                Err(())
            } else {
                match lhs {
                    Token::Integer(left_int) => Ok(Token::Real(f64::from(left_int) / right_real)),
                    Token::Real(left_real) => Ok(Token::Real(left_real / right_real)),
                    _ => Err(()),
                }
            }
        }
        _ => Err(()),
    }
}
