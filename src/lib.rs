pub use error::{Error, ErrorKind};
pub use lexer::Lexer;
pub use object::Object;

mod encoding;
mod error;
mod execution;
mod interpreter;
mod lexer;
mod object;
mod operators;
mod rand;
mod stack;
mod token;

pub type Result<T> = std::result::Result<T, crate::Error>;

pub fn eval(input: &str) -> crate::Result<()> {
    /*
    let scanner = Scanner::from(input.chars());
    let mut evaluator = Evaluator::default();
    evaluator.evaluate(scanner.filter_map(|o| o.ok()))?;
    */

    Ok(())
}
