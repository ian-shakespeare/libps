pub use error::{Error, ErrorKind};
use scanner::Scanner;

mod encoding;
mod error;
mod evaluator;
mod object;
mod scanner;
mod stack;
mod token;

pub type Result<T> = std::result::Result<T, crate::Error>;

pub fn scan(input: &str) {
    let scanner = Scanner::from(input.chars());
    for token in scanner {
        match token {
            Result::Ok(t) => println!("{:?}", t),
            Result::Err(e) if e.kind() == ErrorKind::UnexpectedEof => break,
            Result::Err(e) => println!("{:?}", e),
        }
    }
}
