use std::collections;

use crate::{object::Object, stack::Stack};

#[allow(dead_code)]
pub struct Evaluator {
    operand_stack: Stack<Object>,
    dict_stack: Stack<collections::HashMap<String, Object>>,
    execution_stack: Stack<Object>,
}

impl Evaluator {
    pub fn evaluate<I>(&mut self, objects: I)
    where
        I: Iterator<Item = Object>,
    {
        for obj in objects {
            match obj {
                Object::Integer(_)
                | Object::Real(_)
                | Object::Boolean(_)
                | Object::LiteralName(_) => self.operand_stack.push(obj),
                Object::Name(ref name) => {}
                _ => todo!(),
            }
        }
    }
}
