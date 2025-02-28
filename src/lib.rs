pub use error::{Error, ErrorKind};
pub use interpreter::Interpreter;
pub use lexer::Lexer;
pub use object::Object;

mod access;
mod composite;
mod encoding;
mod error;
mod interpreter;
mod lexer;
mod memory;
mod object;
mod operators;
mod rand;
mod value;

pub type Result<T> = std::result::Result<T, crate::Error>;
