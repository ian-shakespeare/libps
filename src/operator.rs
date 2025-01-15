use crate::{object::Object, stack::Stack};

pub fn add(operand_stack: &mut Stack<Object>, _execution_stack: &mut Stack<Object>) {
    let rhs = operand_stack.pop();
    let lhs = operand_stack.pop();

    match (lhs, rhs) {
        (Some(lhs), Some(rhs)) => match (lhs, rhs) {
            (Object::Integer(lhs), Object::Integer(rhs)) => {
                operand_stack.push(Object::Integer(lhs + rhs));
            }
            (Object::Integer(lhs), Object::Real(rhs)) => {
                operand_stack.push(Object::Real(f64::from(lhs) + rhs));
            }
            (Object::Real(lhs), Object::Integer(rhs)) => {
                operand_stack.push(Object::Real(lhs + f64::from(rhs)));
            }
            (Object::Real(lhs), Object::Real(rhs)) => {
                operand_stack.push(Object::Real(lhs + rhs));
            }
            _ => todo!("error"),
        },
        _ => todo!("error"),
    }
}
