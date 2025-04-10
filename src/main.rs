use std::{
    io::{self, BufRead, Write},
    process,
};

use libps::{evaluate, write_stack, Context};

fn fatal(message: &str) -> ! {
    eprintln!("{}", message);

    process::exit(1)
}

fn main() -> io::Result<()> {
    let mut input = io::stdin().lock();
    let mut output = io::stdout().lock();
    let mut buf = String::new();
    let mut ctx = Context::default();

    output.write_all(b"libPS 0.0.0")?;

    loop {
        output.write_all(b"\n>>> ")?;
        output.flush()?;

        input.read_line(&mut buf)?;

        if let Err(e) = evaluate(&mut ctx, &buf) {
            fatal(&e.to_string());
        }

        output.write_all(b"|-")?;

        write_stack(&mut output, &ctx)?;

        buf.clear();
    }
}
