use crate::stack::Stack;

pub enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

impl Operator {
    pub fn apply_to_stack(&self, value_stack: &mut Stack<i32>) -> Result<(), ()> {
        match self {
            Operator::Add => {
                if let Some((lhs, rhs)) = value_stack.pop_two() {
                    value_stack.push(lhs + rhs);
                } else {
                    return Err(());
                }
            }
            Operator::Subtract => {
                if let Some((lhs, rhs)) = value_stack.pop_two() {
                    value_stack.push(lhs - rhs);
                } else {
                    return Err(());
                }
            }
            Operator::Multiply => {
                if let Some((lhs, rhs)) = value_stack.pop_two() {
                    value_stack.push(lhs * rhs);
                } else {
                    return Err(());
                }
            }
            Operator::Divide => {
                if let Some((lhs, rhs)) = value_stack.pop_two() {
                    value_stack.push(lhs / rhs);
                } else {
                    return Err(());
                }
            }
        }

        Ok(())
    }
}
