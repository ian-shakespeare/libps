pub use error::{Error, ErrorKind};
pub use evaluator::Evaluator;
pub use object::Object;
pub use scanner::Scanner;

mod encoding;
mod error;
mod evaluator;
mod execution;
mod object;
mod operators;
mod rand;
mod scanner;
mod stack;
mod token;

pub type Result<T> = std::result::Result<T, crate::Error>;

pub fn eval(input: &str) -> crate::Result<()> {
    let scanner = Scanner::from(input.chars());
    let mut evaluator = Evaluator::default();
    evaluator.evaluate(scanner.filter_map(|o| o.ok()))?;

    Ok(())
}
