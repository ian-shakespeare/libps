use std::result;

pub use error::{Error, ErrorKind};
pub use file::FileObject;
pub use interpreter::Interpreter;
pub use object::{Mode, Object};

mod array;
mod composite;
mod dictionary;
mod encoding;
mod error;
mod file;
mod gstate;
mod interpreter;
mod name;
mod object;
mod operator;
mod packedarray;
mod save;
mod scanner;
mod string;

type Result<T> = result::Result<T, Error>;
