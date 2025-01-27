use crate::{
    encoding::{decode_ascii85, decode_hex},
    execution::{ExecutionState, Procedure},
    object::Object,
    stack::Stack,
    token::{Token, TokenKind},
    Error, ErrorKind,
};

#[derive(Default)]
pub struct Evaluator {
    execution_state: ExecutionState,
}

impl Evaluator {
    pub fn state(&self) -> &ExecutionState {
        &self.execution_state
    }

    pub fn dump_operand_stack(&mut self) -> Stack<Object> {
        let mut objs = Vec::with_capacity(self.execution_state.operand_stack.count());
        while let Some(obj) = self.execution_state.operand_stack.pop() {
            objs.push(obj);
        }

        Stack::from(objs)
    }

    pub fn evaluate<I>(&mut self, tokens: I) -> crate::Result<()>
    where
        I: Iterator<Item = Token>,
    {
        for token in tokens {
            let obj = match token.kind() {
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
                                    _ => Err(Error::new(
                                        ErrorKind::Syntax,
                                        "invalid scientific real",
                                    )),
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
            }?;

            self.evaluate_object(obj)?;
        }

        Ok(())
    }

    fn evaluate_object(&mut self, obj: Object) -> crate::Result<()> {
        match obj {
            Object::Integer(_) | Object::Real(_) | Object::Boolean(_) | Object::LiteralName(_) => {
                self.execution_state.operand_stack.push(obj);
                Ok(())
            }
            Object::Name(ref name) => {
                match self
                    .execution_state
                    .dict_stack
                    .search(|dict| dict.contains_key(name))
                {
                    None => Err(Error::from(ErrorKind::Undefined)),
                    Some(dict) => match dict.get(name) {
                        None => Err(Error::from(ErrorKind::Undefined)),
                        Some(defined) => match defined {
                            Procedure::Defined(obj) => self.evaluate_object(obj.clone()),
                            Procedure::System(op) => op(&mut self.execution_state),
                        },
                    },
                }
            }
            _ => todo!(),
        }
    }
}
