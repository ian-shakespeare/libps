use crate::{object::Object, stack::Stack};

pub fn dup(operand_stack: &mut Stack<Object>, _: &mut Stack<Object>) {
    match operand_stack.top() {
        None => todo!(),
        Some(top) => operand_stack.push(top.clone()),
    }
}

pub fn exch(operand_stack: &mut Stack<Object>, _: &mut Stack<Object>) {
    let first = operand_stack.pop();
    let second = operand_stack.pop();

    match (first, second) {
        (Some(first), Some(second)) => {
            operand_stack.push(first);
            operand_stack.push(second);
        }
        _ => todo!(),
    }
}

pub fn pop(operand_stack: &mut Stack<Object>, _: &mut Stack<Object>) {
    if let None = operand_stack.pop() {
        todo!();
    }
}

pub fn copy(operand_stack: &mut Stack<Object>, _: &mut Stack<Object>) {
    todo!("what does do it do?");
}

pub fn roll(operand_stack: &mut Stack<Object>, _: &mut Stack<Object>) {
    todo!("what does do it do?");
}

pub fn index(operand_stack: &mut Stack<Object>, _: &mut Stack<Object>) {
    todo!("what direction does it index?");
}

pub fn mark(operand_stack: &mut Stack<Object>, _: &mut Stack<Object>) {
    operand_stack.push(Object::Mark);
}

pub fn clear(operand_stack: &mut Stack<Object>, _: &mut Stack<Object>) {
    operand_stack.clear();
}

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
