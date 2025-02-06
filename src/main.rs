use std::{
    io::{self, BufRead, Write},
    process,
};

use libps::Interpreter;

fn fatal(message: &str) -> ! {
    eprintln!("{}", message);

    process::exit(1)
}

fn main() -> io::Result<()> {
    let mut input = io::stdin().lock();
    let mut output = io::stdout().lock();
    let mut buf = String::new();
    let mut interpreter = Interpreter::default();

    output.write_all(b"libPS 0.0.0")?;

    loop {
        output.write_all(b"\n>>> ")?;
        output.flush()?;

        input.read_line(&mut buf)?;

        if buf.starts_with(".quit") {
            break;
        }

        if let Err(e) = interpreter.evaluate(buf.chars().into()) {
            fatal(&e.to_string());
        }

        output.write_all(b"|-")?;
        interpreter.write_stack(&mut output)?;

        buf.clear();
    }

    Ok(())
}
