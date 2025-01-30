use std::{collections, hash::Hash};

use crate::{
    encoding::{decode_ascii85, decode_hex},
    execution::Procedure,
    object::Object,
    rand::RandomNumberGenerator,
    stack::Stack,
    token::{Token, TokenKind},
    Error, ErrorKind,
};

pub struct Interpreter {
    operand_stack: Vec<Object>,
    execution_stack: Stack<Object>,
    dict_stack: Stack<collections::HashMap<String, Procedure>>,
    rng: RandomNumberGenerator,
}

impl Interpreter {
    pub fn dump_operand_stack(&mut self) -> Stack<Object> {
        let mut objs = Vec::with_capacity(self.operand_stack.len());
        while let Some(obj) = self.operand_stack.pop() {
            objs.push(obj);
        }

        Stack::from(objs)
    }

    pub fn evaluate<I>(&mut self, tokens: I) -> crate::Result<()>
    where
        I: Iterator<Item = Token>,
    {
        Ok(())
    }

    fn evaluate_object(&mut self, obj: Object) -> crate::Result<()> {
        Ok(())
    }
}

// Utils
impl Interpreter {
    fn push(&mut self, obj: Object) {
        self.operand_stack.push(obj);
    }

    fn pop_int(&mut self) -> crate::Result<i32> {
        match self.operand_stack.pop() {
            Some(Object::Integer(i)) => Ok(i),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected integer")),
        }
    }
}

#[derive(Default)]
struct Container<V> {
    data: collections::HashMap<usize, V>,
    counter: usize,
}

impl<V> Container<V> {
    fn insert(&mut self, v: V) -> Option<V> {
        self.counter += 1;

        self.data.insert(self.counter, v)
    }

    fn get(&mut self, k: usize) -> Option<&V> {
        self.data.get(&k)
    }

    fn get_mut(&mut self, k: usize) -> Option<&mut V> {
        self.data.get_mut(&k)
    }
}
