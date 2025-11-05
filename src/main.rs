use std::io::{self, stdin, stdout, Error, ErrorKind};

use libps::Interpreter;

fn main() -> io::Result<()> {
    let mut interpreter = Interpreter::new(stdin().lock(), stdout().lock());

    interpreter
        .executive()
        .or(Err(Error::from(ErrorKind::BrokenPipe)))
}
