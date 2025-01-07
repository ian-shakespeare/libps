use std::collections;

use crate::{object::Object, stack::Stack, token::Token};

pub struct Evaluator<'a> {
    operand_stack: Stack<Object<'a>>,
    dictionary_stack: Stack<collections::HashMap<Object<'a>, Object<'a>>>,
    execution_stack: Stack<Object<'a>>,
}

impl<'a> Evaluator<'a> {
    pub fn new(dictionaries: Stack<collections::HashMap<Object<'a>, Object<'a>>>) -> Self {
        Self {
            dictionary_stack: dictionaries,
            operand_stack: Stack::new(),
            execution_stack: Stack::new(),
        }
    }

    fn evaluate<I>(&mut self, tokens: I)
    where
        I: Iterator<Item = crate::Result<Token>>,
    {
        for token in tokens {
            if let Ok(token) = token {
                match token {
                    // Literals
                    Token::Integer(i) => self.operand_stack.push(Object::Integer(i)),
                    Token::Real(r) => self.operand_stack.push(Object::Real(r)),
                    Token::String(s) => self.operand_stack.push(Object::String(s)),
                    Token::Name(n) => {
                        if n == "true" {
                            self.operand_stack.push(Object::Boolean(true));
                            continue;
                        }

                        if n == "false" {
                            self.operand_stack.push(Object::Boolean(false));
                            continue;
                        }

                        if n.starts_with('/') {
                            let mut parts = n.split("/");
                            let _ = parts.next();
                            let name = parts.next().expect("received a bad bad boy"); // TODO:
                                                                                      // remove the
                                                                                      // horny
                            self.operand_stack.push(Object::Name(String::from(name)));
                            continue;
                        }

                        let name = Object::Name(n);

                        self.dictionary_stack
                            .search(|dict| dict.contains_key(&name));
                    }
                    _ => todo!(),
                }
            } else {
                panic!("TODO: replace this");
            }
        }
    }
}
