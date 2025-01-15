use std::collections;

use crate::{object::Object, stack::Stack};

pub enum Defined {
    Object(Object),
    Function(fn(&mut Stack<Object>, &mut Stack<Object>)),
}

#[allow(dead_code)]
pub struct Evaluator {
    operand_stack: Stack<Object>,
    dict_stack: Stack<collections::HashMap<String, Defined>>,
    execution_stack: Stack<Object>,
}

impl Evaluator {
    pub fn new(dict_stack: Stack<collections::HashMap<String, Defined>>) -> Self {
        Self {
            operand_stack: Stack::new(),
            execution_stack: Stack::new(),
            dict_stack,
        }
    }

    pub fn evaluate<I>(&mut self, objects: I)
    where
        I: Iterator<Item = Object>,
    {
        for obj in objects {
            self.evaluate_object(obj);
        }

        println!("{:?}", self.operand_stack);
    }

    fn evaluate_object(&mut self, obj: Object) {
        match obj {
            Object::Integer(_) | Object::Real(_) | Object::Boolean(_) | Object::LiteralName(_) => {
                self.operand_stack.push(obj)
            }
            Object::Name(ref name) => {
                match self.dict_stack.search(|dict| dict.contains_key(name)) {
                    None => todo!("missing name"),
                    Some(dict) => {
                        if let Some(defined) = dict.get(name) {
                            match defined {
                                Defined::Object(obj) => self.evaluate_object(obj.clone()),
                                Defined::Function(func) => {
                                    func(&mut self.operand_stack, &mut self.execution_stack)
                                }
                            }
                        }
                    }
                }
            }
            _ => todo!(),
        }
    }
}
