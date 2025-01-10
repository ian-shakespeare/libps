pub use error::{Error, ErrorKind};
use object::Object;
use scanner::Scanner;
use tokenizer::Tokenizer;

mod encoding;
mod error;
mod evaluator;
mod object;
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
