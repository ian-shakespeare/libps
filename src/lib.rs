use std::collections;

pub use error::{Error, ErrorKind};
use evaluator::{Defined, Evaluator};
use object::Object;
use scanner::Scanner;
use stack::Stack;
use tokenizer::Tokenizer;

mod encoding;
mod error;
mod evaluator;
mod object;
mod operator;
mod scanner;
mod stack;
mod token;
mod tokenizer;

pub type Result<T> = std::result::Result<T, crate::Error>;

pub fn scan(input: &str) {
    let scanner = Scanner::from(input.chars());
    let tokens = scanner.filter_map(|t| t.ok());
    let objects: Vec<Object> = Tokenizer::from(tokens).filter_map(|obj| obj.ok()).collect();
    println!("{:?}", objects);
}

pub fn eval(input: &str) {
    let scanner = Scanner::from(input.chars());
    let tokenizer = Tokenizer::from(scanner.filter_map(|t| t.ok()));

    let mut system_dict = collections::HashMap::new();
    system_dict.insert("add".to_string(), Defined::Function(operator::add));

    let mut dicts = Stack::new();
    dicts.push(system_dict);

    let mut evaluator = Evaluator::new(dicts);
    evaluator.evaluate(tokenizer.filter_map(|o| o.ok()));
}
