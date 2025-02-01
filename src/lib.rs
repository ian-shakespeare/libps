pub use error::{Error, ErrorKind};
pub use interpreter::Interpreter;
pub use lexer::Lexer;
pub use object::Object;

mod encoding;
mod error;
mod interpreter;
mod lexer;
mod object;
mod rand;

pub type Result<T> = std::result::Result<T, crate::Error>;

pub fn eval(input: &str) -> crate::Result<()> {
    let mut interpreter = Interpreter::new(input.chars());
    interpreter.evaluate()?;

    Ok(())
}
