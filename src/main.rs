use std::io::{stdin, stdout};

use libps::Interpreter;

fn main() {
    let mut interpreter = Interpreter::new(stdin().lock(), stdout().lock());

    interpreter.executive()
}
